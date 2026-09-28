//! Stepwise game session for a front end.
//!
//! A Godot/gdext node should keep one [`Session`] and call [`Session::buy`],
//! [`Session::sell`], [`Session::depart`], and [`Session::advance`] one turn
//! at a time. Read [`Session::world`] for ports (including `map_x` / `map_y`),
//! the ship, and the voyage. Read [`Session::sail_lanes`] for the picker and
//! [`Session::victory`] for the four victory paths.
//!
//! `run_script` is a thin wrapper over these methods, so the parity harness
//! covers the same path. [`Session::save`] and [`Session::load`] speak the
//! Python version-12 JSON slot. The CLI does not grow a second format.
//!
//! This is the slice of `GameSession` the port compares with Python: new game,
//! buy, sell (including trade reputation and the receipt ledger), depart,
//! advance, and dock work. Advance ticks reputation, ticks contract deadlines,
//! ticks markets while in port (without seasonal `current_day`), sails one day
//! at sea, records inspection and arrival reputation, reprices with the
//! captain's modifiers, and records a victory path when its requirements are
//! all met.
//!
//! [`Session::buy_infrastructure`], [`Session::take_credit`], and
//! [`Session::buy_insurance`] write warehouses, brokers, licenses, policies,
//! and credit onto the house books. Sea-culture enrichment and narrative are
//! not on this type. A fulfilled contract is written by [`Session::sell`], which
//! also calls [`Session::complete_contract`]'s settlement. A second
//! `complete_contract` does not pay again. Callers do not use
//! [`Session::books_mut`] for that. The new-game board is drawn from
//! `Random(seed + 7919)` and then the session RNG is restored. Arrival and
//! later in-port refreshes draw the session stream. Milestone evaluation runs
//! at the end of [`Session::advance`], in the same place as
//! `GameSession._evaluate_campaign`, before victory closure.
//!
//! A pending pirate duel still freezes [`Session::advance`] until
//! [`Session::duel`] or [`Session::resolve_pending_duel`] clears it. That is
//! `portlight duel` and `GameSession._resolve_pending_duel`. The voyage event
//! still goes straight to that duel. [`Session::encounter_choice`],
//! [`Session::naval_round`], and [`Session::resolve_boarding`] are the separate
//! approach machine: negotiate, flee, or fight, then naval combat and boarding.
//! [`Session::board`] is the contract board. The script command for the deck
//! melee is still `board`.
//! `advance` does not auto-resolve, matching `auto_resolve_duels = False`.

use std::path::{Path, PathBuf};

use crate::campaign::{
    self, ActiveLicense, BrokerSite, CompletedContract, CreditBook, HouseBooks, VictoryPathStatus,
    WarehouseSite,
};
use crate::combat::{self, CombatRound, CombatantState};
use crate::companion;
use crate::content::{self, PricingDef};
use crate::contracts;
use crate::duel::{self, DuelOutcome};
use crate::economy::{self, recalculate_prices, TradeReceipt};
use crate::encounter::{self, BoardingOutcome, EncounterState};
use crate::error::SimError;
use crate::infrastructure::{self, dry_dock_service};
use crate::memory;
use crate::model::{
    ActiveContract, Contract, ContractBoard, ContractOutcome, InfrastructureRecord, Officer,
    PendingDuel, VoyageStatus, World,
};
use crate::naval::{self, NavalRound};
use crate::pyrand::PyRandom;
use crate::reputation::{self, record_trade_outcome};
use crate::save::{self, LoadedGame};
use crate::ship::wage_bill;
use crate::skills;
use crate::training;
use crate::util::py_trunc;
use crate::voyage::{self, sail_lanes, SailLane, VoyageEvent};
use crate::world::new_game;

/// One turn of [`Session::advance`].
#[derive(Debug, Clone)]
pub struct Turn {
    pub events: Vec<VoyageEvent>,
    pub shocks: Vec<String>,
    /// Contracts that expired on this day. Empty when nothing lapsed.
    pub contracts: Vec<ContractOutcome>,
    /// Infrastructure and credit messages from this day.
    pub notes: Vec<String>,
}

/// A sale, plus any contracts that sale completed.
///
/// Python `GameSession.sell` credits delivery and resolves fulfilled
/// contracts in the same call. The silver is already on the captain.
#[derive(Debug, Clone)]
pub struct Sale {
    pub receipt: TradeReceipt,
    pub contracts: Vec<ContractOutcome>,
}

/// One step of the approach, naval, boarding, or personal-fight machine.
#[derive(Debug, Clone)]
pub struct EncounterStep {
    pub kind: String,
    pub phase: String,
    pub message: String,
    pub choice: String,
    pub success: bool,
    pub escaped: bool,
    pub hull_damage: i64,
    pub enemy_captain_id: String,
    pub enemy_captain_name: String,
    pub enemy_strength: i64,
    pub turn: i64,
    pub player_action: String,
    pub enemy_action: String,
    pub player_hull_delta: i64,
    pub enemy_hull_delta: i64,
    pub player_crew_delta: i64,
    pub enemy_crew_delta: i64,
    pub boarding_progress: i64,
    pub boarding_threshold: i64,
    pub enemy_sunk: bool,
    pub player_sunk: bool,
    pub boarding_triggered: bool,
    pub flavor: String,
    pub player_hull: i64,
    pub enemy_hull: i64,
    pub player_crew: i64,
    pub enemy_crew: i64,
    pub player_crew_lost: i64,
    pub enemy_crew_lost: i64,
    pub player_advantage: bool,
    pub damage_to_opponent: i64,
    pub damage_to_player: i64,
    pub player_hp: i64,
    pub opponent_hp: i64,
    pub player_stamina_delta: i64,
    pub opponent_stamina_delta: i64,
    pub player_won: bool,
    pub draw: bool,
    pub injury: String,
    pub opponent_injury: String,
    pub style_effect: String,
    pub prize_ok: bool,
    pub prize_reason: String,
}

/// Encounter returned by [`Session::tick_sea_captain_agency`].
/// One playable game.
#[derive(Debug, Clone)]
pub struct Session {
    world: World,
    rng: PyRandom,
    trade_seq: u64,
    books: HouseBooks,
    receipts: Vec<TradeReceipt>,
    run_id: String,
    board: ContractBoard,
    encounter: Option<EncounterState>,
    player_combat: Option<CombatantState>,
    opponent_combat: Option<CombatantState>,
    infra: InfrastructureRecord,
}

impl Session {
    /// Start a game. `seed` matches `random.Random(seed)` for every `i128`.
    ///
    /// `starting_port` overrides the captain's home port. `None` uses the
    /// archetype home.
    pub fn new(
        captain_name: &str,
        captain_type: &str,
        seed: i128,
        starting_port: Option<&str>,
    ) -> Result<Self, SimError> {
        let world = new_game(captain_name, captain_type, seed, starting_port)?;
        let rng = PyRandom::from_seed(world.seed);
        let run_id = format!("run-{}", world.seed);
        let mut session = Self {
            world,
            rng,
            trade_seq: 0,
            books: HouseBooks::default(),
            receipts: Vec::new(),
            run_id,
            board: ContractBoard::default(),
            encounter: None,
            player_combat: None,
            opponent_combat: None,
            infra: InfrastructureRecord::default(),
        };
        session.refresh_new_game_board();
        Ok(session)
    }

    /// Write this game to a Python version-12 JSON slot under `base_path/saves`.
    ///
    /// Silver is clamped at zero first, matching `GameSession._save`. The
    /// MT19937 state is not written. `trade_seq` is the length of the ledger.
    pub fn save(&mut self, base_path: impl AsRef<Path>, slot: &str) -> Result<PathBuf, SimError> {
        if self.world.captain.silver < 0 {
            self.world.captain.silver = 0;
        }
        save::write_save(
            base_path.as_ref(),
            slot,
            &self.world,
            &self.receipts,
            &self.run_id,
            &self.books,
            &self.board,
            save::LiveEncounter {
                encounter: self.encounter.as_ref(),
                player: self.player_combat.as_ref(),
                opponent: self.opponent_combat.as_ref(),
            },
            &self.infra,
        )
    }

    /// Load a Python JSON slot.
    ///
    /// `Ok(None)` means the file is missing or corrupt, which is `load_game`
    /// returning `None`. A newer version or a broken migration chain returns
    /// the Python `SaveVersionError` text. The RNG is reseeded with
    /// `Random(seed + day)`. Prices are recalculated with the captain's
    /// modifiers, matching `GameSession.load`.
    pub fn load(base_path: impl AsRef<Path>, slot: &str) -> Result<Option<Self>, SimError> {
        let Some(loaded) = save::read_save(base_path.as_ref(), slot)? else {
            return Ok(None);
        };
        Ok(Some(Self::from_loaded(loaded)?))
    }

    fn from_loaded(loaded: LoadedGame) -> Result<Self, SimError> {
        let rng_seed = loaded
            .world
            .seed
            .checked_add(i128::from(loaded.world.day))
            .ok_or(SimError::SaveCorrupt)?;
        let trade_seq = loaded.receipts.len() as u64;
        let mut session = Self {
            world: loaded.world,
            rng: PyRandom::from_seed(rng_seed),
            trade_seq,
            books: loaded.books,
            receipts: loaded.receipts,
            run_id: loaded.run_id,
            board: loaded.board,
            encounter: loaded.encounter,
            player_combat: loaded.player_combat,
            opponent_combat: loaded.opponent_combat,
            infra: loaded.infra,
        };
        reprice_all(&mut session.world);
        session.project_books();
        Ok(session)
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    /// Warehouses, brokers, licenses, policies, claims, and credit.
    pub fn infrastructure(&self) -> &InfrastructureRecord {
        &self.infra
    }

    /// Install a record and project it onto the house books.
    ///
    /// Purchases go through [`Session::buy_infrastructure`],
    /// [`Session::take_credit`], and [`Session::buy_insurance`]. Load uses
    /// this after reading a version-12 `infrastructure` object.
    pub fn adopt_infrastructure(&mut self, infra: InfrastructureRecord) {
        self.infra = infra;
        self.project_books();
    }

    /// Offers, accepted work, and resolved outcomes. Save writes this board.
    /// Breaches are stored here and serialized as `captain.breach_records`.
    pub fn board(&self) -> &ContractBoard {
        &self.board
    }

    pub fn trade_seq(&self) -> u64 {
        self.trade_seq
    }

    /// Ledger and the contract/infrastructure records victory reads.
    pub fn books(&self) -> &HouseBooks {
        &self.books
    }

    /// Mutable books for systems that grant licenses, warehouses, brokers,
    /// policies, or credit. Trade receipts update the ledger. A fulfilled
    /// contract is written by the settlement shared by [`Session::sell`] and
    /// [`Session::complete_contract`].
    pub fn books_mut(&mut self) -> &mut HouseBooks {
        &mut self.books
    }

    pub fn sail_lanes(&self) -> Vec<SailLane> {
        sail_lanes(&self.world)
    }

    /// The four victory paths, highest candidate strength first.
    pub fn victory(&self) -> Vec<VictoryPathStatus> {
        campaign::compute_victory_progress(&self.world, &self.books)
    }

    pub fn buy(&mut self, good_id: &str, qty: i64) -> Result<TradeReceipt, SimError> {
        let Some(port_id) = current_port_id(&self.world).map(str::to_string) else {
            return Err(SimError::NotDocked);
        };
        let seq = self.trade_seq;
        let receipt = {
            let world = &mut self.world;
            let port = world
                .ports
                .iter_mut()
                .find(|port| port.id == port_id)
                .ok_or(SimError::NotDocked)?;
            economy::execute_buy(&mut world.captain, port, good_id, qty, seq)?
        };
        record_receipt(self, &receipt);
        let pricing = pricing(&self.world).cloned();
        if let Some(port) = self.world.port_mut(&port_id) {
            recalculate_prices(port, pricing.as_ref());
        }
        Ok(receipt)
    }

    pub fn sell(&mut self, good_id: &str, qty: i64) -> Result<Sale, SimError> {
        let Some(port_id) = current_port_id(&self.world).map(str::to_string) else {
            return Err(SimError::NotDocked);
        };
        let (flood_before, stock_target, region) = {
            let port = self.world.port(&port_id).ok_or(SimError::NotDocked)?;
            let slot = port.slot(good_id);
            (
                slot.map(|slot| slot.flood_penalty).unwrap_or(0.0),
                slot.map(|slot| slot.stock_target).unwrap_or(50),
                port.region.clone(),
            )
        };
        let (source_port, source_region) = self
            .world
            .captain
            .cargo
            .iter()
            .find(|item| item.good_id == good_id)
            .map(|item| (item.acquired_port.clone(), item.acquired_region.clone()))
            .unwrap_or_default();
        let seq = self.trade_seq;
        let receipt = {
            let world = &mut self.world;
            let port = world
                .ports
                .iter_mut()
                .find(|port| port.id == port_id)
                .ok_or(SimError::NotDocked)?;
            economy::execute_sell(&mut world.captain, port, good_id, qty, seq)?
        };
        record_receipt(self, &receipt);
        let cost_basis =
            economy::estimate_cost_basis(&self.world.captain, good_id, receipt.quantity);
        let margin_pct = if cost_basis > 0 {
            (receipt.total_price - cost_basis) as f64 / cost_basis.max(1) as f64 * 100.0
        } else {
            50.0
        };
        let category = content::content()
            .good(good_id)
            .map(|good| good.category.as_str())
            .unwrap_or("commodity");
        record_trade_outcome(
            &mut self.world.captain.standing,
            &self.world.captain.captain_type,
            self.world.day,
            &port_id,
            &region,
            good_id,
            category,
            receipt.quantity,
            margin_pct,
            stock_target,
            flood_before,
        );
        let credited = contracts::check_delivery(
            &mut self.board,
            &port_id,
            good_id,
            receipt.quantity,
            &source_port,
            &source_region,
        );
        let contracts = if credited.is_empty() {
            Vec::new()
        } else {
            self.settle_fulfilled()
        };
        let pricing = pricing(&self.world).cloned();
        if let Some(port) = self.world.port_mut(&port_id) {
            recalculate_prices(port, pricing.as_ref());
        }
        Ok(Sale { receipt, contracts })
    }

    pub fn depart(&mut self, destination_id: &str) -> Result<(), SimError> {
        voyage::depart(&mut self.world, destination_id, false)
    }

    /// Hire crew at the current port. `role` defaults to `"sailor"` when empty.
    ///
    /// Sailors cost `port.crew_cost`. Specialists cost `wage * 10` and receive
    /// a region-flavored name and trait, matching `GameSession.hire_crew`.
    pub fn hire_crew(&mut self, count: i64, role: &str) -> Result<(), SimError> {
        let role = if role.is_empty() { "sailor" } else { role };
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or(SimError::MustBeDockedToHire)?;
        let (crew_cost, region) = {
            let port = self
                .world
                .port(&port_id)
                .ok_or(SimError::MustBeDockedToHire)?;
            (port.crew_cost, port.region.clone())
        };
        let ship = self.world.captain.ship.as_ref().ok_or(SimError::NoShip)?;
        let space = ship.crew_max - ship.crew;
        if space <= 0 {
            return Err(SimError::CrewFull);
        }
        let spec = role_spec(&role.to_lowercase())
            .ok_or_else(|| SimError::UnknownRole(role.to_string()))?;
        let mut count = count;
        if let Some(max) = spec.max_per_ship {
            let current = role_count(ship, spec.role);
            let avail = max - current;
            if avail <= 0 {
                return Err(SimError::RoleMaximum {
                    name: spec.name.to_string(),
                    max,
                });
            }
            count = count.min(avail);
        }
        count = count.min(space);
        let cost_per = if spec.role == "sailor" {
            crew_cost
        } else {
            spec.wage * 10
        };
        let cost = count * cost_per;
        if cost > self.world.captain.silver {
            return Err(SimError::NeedCrewSilver {
                cost,
                count,
                name: spec.name.to_string(),
                each: cost_per,
                have: self.world.captain.silver,
            });
        }
        self.world.captain.silver -= cost;
        let ship = self.world.captain.ship.as_mut().ok_or(SimError::NoShip)?;
        let current = role_count(ship, spec.role);
        set_role_count(ship, spec.role, current + count);
        ship.sync_crew();
        if spec.role != "sailor" {
            for _ in 0..count {
                ship.officers.push(Officer {
                    name: officer_name(&region, &mut self.rng),
                    role: spec.role.to_string(),
                    origin_port: port_id.clone(),
                    trait_name: officer_trait(&mut self.rng),
                });
            }
        }
        Ok(())
    }

    /// Buy `days` of provisions at the current port.
    ///
    /// Cost per day is `max(1, int(provision_cost * service_modifier))`.
    pub fn provision(&mut self, days: i64) -> Result<(), SimError> {
        if days <= 0 {
            return Err(SimError::QuantityMustBeAPositiveNumber);
        }
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or(SimError::MustBeDockedToProvision)?;
        let provision_cost = self
            .world
            .port(&port_id)
            .ok_or(SimError::MustBeDockedToProvision)?
            .provision_cost;
        let mult = reputation::service_modifier(&self.world.captain.standing, &port_id);
        let per_day = 1.max(py_trunc(provision_cost as f64 * mult));
        let cost = days * per_day;
        if cost > self.world.captain.silver {
            return Err(SimError::NeedProvisions {
                cost,
                days,
                per_day,
                have: self.world.captain.silver,
            });
        }
        self.world.captain.silver -= cost;
        self.world.captain.provisions += days;
        Ok(())
    }

    /// Work the docks for a day. Returns silver earned (3 to 5).
    ///
    /// This is `GameSession.work`: one `randint(3, 5)` on the session RNG,
    /// then `captain.day` is copied onto `world.day`. Markets, provisions,
    /// wages, and reputation do not tick.
    pub fn work(&mut self) -> Result<i64, SimError> {
        if current_port_id(&self.world).is_none() {
            return Err(SimError::MustBeDockedToWork);
        }
        let earned = economy::work_docks(&mut self.world.captain, &mut self.rng);
        self.world.day = self.world.captain.day;
        Ok(earned)
    }

    /// Fight the pending pirate with the given stances (`portlight duel`).
    ///
    /// Clears the challenge and applies `silver_delta`. `standing_delta` is
    /// returned and not written onto reputation, matching the Python CLI.
    pub fn duel(&mut self, stances: &[String]) -> Result<DuelOutcome, SimError> {
        if self.world.pending_duel.is_none() {
            return Err(SimError::NoPendingDuel);
        }
        let mut parsed = Vec::with_capacity(stances.len());
        for stance in stances {
            let stance = stance.trim().to_lowercase();
            if !matches!(stance.as_str(), "thrust" | "slash" | "parry") {
                return Err(SimError::InvalidStance(stance));
            }
            parsed.push(stance);
        }
        if parsed.len() < 3 {
            return Err(SimError::TooFewStances);
        }
        self.finish_duel(parsed)
    }

    /// `GameSession._resolve_pending_duel`: five random stances, then the same
    /// silver and clear as [`Session::duel`].
    pub fn resolve_pending_duel(&mut self) -> Result<DuelOutcome, SimError> {
        if self.world.pending_duel.is_none() {
            return Err(SimError::NoPendingDuel);
        }
        let stances = duel::auto_stances(&mut self.rng);
        self.finish_duel(stances)
    }

    fn finish_duel(&mut self, stances: Vec<String>) -> Result<DuelOutcome, SimError> {
        let pending = self
            .world
            .pending_duel
            .clone()
            .ok_or(SimError::NoPendingDuel)?;
        let crew = self
            .world
            .captain
            .ship
            .as_ref()
            .map(|ship| ship.crew)
            .unwrap_or(5);
        let outcome = duel::resolve_duel(
            &stances,
            &pending.captain_id,
            &pending.captain_name,
            &pending.personality,
            pending.strength,
            &mut self.rng,
            crew,
        );
        self.world.captain.silver = 0.max(self.world.captain.silver + outcome.silver_delta);
        self.world.pending_duel = None;
        Ok(outcome)
    }

    /// Respond to an approach: `negotiate`, `flee`, or `fight`.
    ///
    /// Opens an encounter for the voyage destination when none is active.
    /// `captain_id` locks a catalog pirate. `band_strength` builds a ship with
    /// [`naval::generate_enemy_ship`] for that strength, including the 1–3 band
    /// that has no named captain. A failed flee applies the broadside and
    /// opens naval combat. It does not enter [`crate::duel`].
    pub fn encounter_choice(&mut self, choice: &str) -> Result<EncounterStep, SimError> {
        self.encounter_choice_with(choice, None, None)
    }

    pub fn encounter_choice_with(
        &mut self,
        choice: &str,
        captain_id: Option<&str>,
        band_strength: Option<i64>,
    ) -> Result<EncounterStep, SimError> {
        let choice = choice.trim().to_lowercase();
        if !matches!(choice.as_str(), "negotiate" | "flee" | "fight") {
            return Err(SimError::ChooseApproach);
        }
        self.ensure_encounter(captain_id, band_strength)?;
        if self.encounter.as_ref().map(|enc| enc.phase.as_str()) != Some("approach") {
            return Err(SimError::NoActiveEncounter);
        }
        if self.world.captain.ship.is_none() {
            return Err(SimError::NoShip);
        }
        match choice.as_str() {
            "negotiate" => self.choose_negotiate(),
            "flee" => self.choose_flee(),
            "fight" => self.choose_fight(),
            _ => Err(SimError::ChooseApproach),
        }
    }

    /// One naval action: `broadside`, `close`, `evade`, `rake`, or `flee`.
    ///
    /// Stops when the enemy sinks or the boarding threshold is met.
    /// [`Session::resolve_boarding`] resolves the deck melee. Cannon math uses the
    /// session RNG.
    pub fn naval_round(&mut self, action: &str) -> Result<EncounterStep, SimError> {
        let action = action.trim().to_lowercase();
        let phase = self.encounter.as_ref().map(|enc| enc.phase.clone());
        if phase.as_deref() != Some("naval") {
            return Err(SimError::NotInNavalCombat);
        }
        let cannons = self
            .world
            .captain
            .ship
            .as_ref()
            .map(|ship| ship.cannons)
            .unwrap_or(0);
        let valid = naval::valid_actions(cannons);
        if !valid.contains(&action.as_str()) {
            return Err(SimError::InvalidAction(valid.join(", ")));
        }
        if action == "flee" {
            return self.naval_flee();
        }
        let round = {
            let ship = self.world.captain.ship.as_ref().ok_or(SimError::NoShip)?;
            let encounter = self.encounter.as_mut().ok_or(SimError::NotInNavalCombat)?;
            encounter::resolve_naval_turn(encounter, &action, ship, &mut self.rng)
        };
        let crew_lost = 0.max(-round.player_crew_delta);
        if let Some(ship) = self.world.captain.ship.as_mut() {
            ship.hull = 0.max(ship.hull + round.player_hull_delta);
            naval::apply_crew_loss(ship, crew_lost);
        }
        let enemy_sunk = self
            .encounter
            .as_ref()
            .is_some_and(|enc| enc.enemy_ship_hull <= 0);
        let boarding_triggered = self
            .encounter
            .as_ref()
            .is_some_and(|enc| enc.boarding_progress >= enc.boarding_threshold);
        let player_sunk = self.player_lost_ship();
        let crew_gone = self
            .world
            .captain
            .ship
            .as_ref()
            .is_some_and(|ship| ship.crew <= 0);
        let mut prize_ok = false;
        let mut prize_reason = String::new();
        if enemy_sunk {
            let (ok, reason) = naval::can_capture_prize(&self.world.captain, self.enemy_strength());
            prize_ok = ok;
            prize_reason = reason;
        }
        let mut step = self.naval_step(
            round,
            enemy_sunk,
            player_sunk,
            boarding_triggered,
            prize_ok,
            prize_reason.clone(),
        );
        if enemy_sunk {
            self.world.captain.naval_victories += 1;
            if prize_ok {
                if let Some(enc) = self.encounter.as_mut() {
                    enc.phase = "capture_available".to_string();
                }
                step.phase = "capture_available".into();
            } else if let Some(enc) = self.encounter.clone() {
                encounter::remember(&mut self.world.captain, &enc, self.world.day, "attack");
                self.clear_encounter();
                step.phase = "resolved".into();
            }
        } else if player_sunk || (crew_gone && !boarding_triggered) {
            if crew_gone && !player_sunk {
                self.plunder();
            }
            self.world.captain.naval_defeats += 1;
            self.clear_encounter();
            step.phase = "resolved".into();
            step.player_sunk = player_sunk;
        }
        Ok(step)
    }

    /// Resolve the boarding melee and open the personal fight.
    ///
    /// The script command is still `board`. The method is not `board` because
    /// that name is the contract-board getter.
    pub fn resolve_boarding(&mut self) -> Result<EncounterStep, SimError> {
        if self.encounter.as_ref().map(|enc| enc.phase.as_str()) != Some("boarding") {
            return Err(SimError::NotBoarding);
        }
        let crew = self
            .world
            .captain
            .ship
            .as_ref()
            .map(|ship| ship.crew)
            .unwrap_or(0);
        let outcome = {
            let encounter = self.encounter.as_mut().ok_or(SimError::NotBoarding)?;
            encounter::resolve_boarding_phase(encounter, crew, &mut self.rng)
        };
        if let Some(ship) = self.world.captain.ship.as_mut() {
            naval::apply_crew_loss(ship, outcome.player_crew_lost);
        }
        self.player_combat = None;
        self.opponent_combat = None;
        Ok(self.board_step(outcome))
    }

    /// One action of the personal fight (`engine/combat.py`).
    pub fn fight(&mut self, action: &str) -> Result<EncounterStep, SimError> {
        let action = action.trim().to_lowercase();
        if self.encounter.as_ref().map(|enc| enc.phase.as_str()) != Some("duel") {
            return Err(SimError::NotInPersonalCombat);
        }
        self.ensure_combatants();
        let valid = self
            .player_combat
            .as_ref()
            .map(combat::available_actions)
            .unwrap_or_default();
        if !valid.iter().any(|item| item == &action) {
            return Err(SimError::InvalidAction(valid.join(", ")));
        }
        let round = {
            let encounter = self
                .encounter
                .as_mut()
                .ok_or(SimError::NotInPersonalCombat)?;
            let player = self
                .player_combat
                .as_mut()
                .ok_or(SimError::NotInPersonalCombat)?;
            let opponent = self
                .opponent_combat
                .as_mut()
                .ok_or(SimError::NotInPersonalCombat)?;
            encounter::resolve_duel_turn(encounter, &action, player, opponent, &mut self.rng)
        };
        let player_hp = self.player_combat.as_ref().map(|c| c.hp).unwrap_or(0);
        let opponent_hp = self.opponent_combat.as_ref().map(|c| c.hp).unwrap_or(0);
        let finished = self.encounter.as_ref().map(|enc| enc.phase.as_str()) == Some("resolved");
        let mut player_won = false;
        let mut draw = false;
        if finished {
            player_won = opponent_hp <= 0 && player_hp > 0;
            draw = player_hp <= 0 && opponent_hp <= 0;
        }
        let mut step = self.fight_step(round, player_hp, opponent_hp, player_won, draw);
        if finished {
            if player_won || draw {
                self.world.captain.duels_won += 1;
            } else {
                self.world.captain.duels_lost += 1;
                let loss = 15 + self.enemy_strength() * 3;
                self.world.captain.silver = 0.max(self.world.captain.silver - loss);
            }
            let outcome = if player_won {
                "duel_win"
            } else if draw {
                "duel_draw"
            } else {
                "duel_loss"
            };
            if let Some(enc) = self.encounter.clone() {
                encounter::remember(&mut self.world.captain, &enc, self.world.day, outcome);
            }
            self.clear_encounter();
            step.phase = "resolved".into();
        }
        Ok(step)
    }

    /// Take the sunk enemy as a prize, or pass `0` to let it go under.
    pub fn capture(&mut self, crew_to_prize: i64) -> Result<EncounterStep, SimError> {
        if self.encounter.as_ref().map(|enc| enc.phase.as_str()) != Some("capture_available") {
            return Err(SimError::CannotCapture(
                "No ship available to capture.".into(),
            ));
        }
        if crew_to_prize <= 0 {
            let mut step = self.blank_step("capture");
            step.message = "You let the prize go under.".into();
            step.phase = "resolved".into();
            self.clear_encounter();
            return Ok(step);
        }
        let enc = self
            .encounter
            .clone()
            .ok_or_else(|| SimError::CannotCapture("No ship available to capture.".into()))?;
        let (ok, reason) = naval::can_capture_prize(&self.world.captain, enc.enemy_strength);
        if !ok {
            return Err(SimError::CannotCapture(reason));
        }
        let catalog = content::content();
        let prize_min = catalog
            .ship(naval::prize_template_id(enc.enemy_strength))
            .map(|ship| ship.crew_min)
            .unwrap_or(3);
        let current_min = self
            .world
            .captain
            .ship
            .as_ref()
            .and_then(|ship| catalog.ship(&ship.template_id))
            .map(|ship| ship.crew_min)
            .unwrap_or(3);
        if crew_to_prize < prize_min {
            return Err(SimError::CannotCapture(format!(
                "Need at least {prize_min} crew for the prize ship."
            )));
        }
        let have = self
            .world
            .captain
            .ship
            .as_ref()
            .map(|ship| ship.crew)
            .unwrap_or(0);
        let leftover = have - crew_to_prize;
        if leftover < current_min {
            return Err(SimError::CannotCapture(format!(
                "Would leave your flagship with {leftover} crew (need {current_min})."
            )));
        }
        let dock = self.world.voyage.destination_id.clone();
        let owned = naval::capture_prize(
            &mut self.world.captain,
            &enc.enemy_captain_name,
            enc.enemy_strength,
            enc.enemy_ship_hull,
            crew_to_prize,
            &dock,
            &mut self.rng,
        );
        let name = owned.ship.name.clone();
        self.world.captain.fleet.push(owned);
        encounter::remember(&mut self.world.captain, &enc, self.world.day, "attack");
        self.clear_encounter();
        let mut step = self.blank_step("capture");
        step.message = format!("Prize captured! {name} added to your fleet.");
        step.phase = "resolved".into();
        step.prize_ok = true;
        step.prize_reason = name;
        step.player_crew = self
            .world
            .captain
            .ship
            .as_ref()
            .map(|ship| ship.crew)
            .unwrap_or(0);
        Ok(step)
    }

    fn ensure_encounter(
        &mut self,
        captain_id: Option<&str>,
        band_strength: Option<i64>,
    ) -> Result<(), SimError> {
        let active = self
            .encounter
            .as_ref()
            .is_some_and(|enc| !enc.phase.is_empty() && enc.phase != "resolved");
        if active {
            return Ok(());
        }
        if let Some(id) = captain_id {
            if content::content().pirate(id).is_none() {
                return Err(SimError::UnknownPirate(id.to_string()));
            }
        }
        let created = if let Some(strength) = band_strength {
            Some(encounter::create_band_encounter(
                &self.world,
                strength,
                &mut self.rng,
            ))
        } else {
            encounter::create_encounter(&self.world, &mut self.rng, captain_id)
        };
        let Some(created) = created else {
            return Err(SimError::NoPirateCaptain);
        };
        self.encounter = Some(created);
        self.player_combat = None;
        self.opponent_combat = None;
        Ok(())
    }

    fn choose_negotiate(&mut self) -> Result<EncounterStep, SimError> {
        let (success, mut message) = {
            let standing_type = self.world.captain.captain_type.clone();
            let encounter = self.encounter.as_mut().ok_or(SimError::NoActiveEncounter)?;
            encounter::resolve_negotiate(
                encounter,
                &self.world.captain.standing,
                &standing_type,
                &mut self.rng,
            )
        };
        if success {
            let allied = message.contains("ally");
            let outcome = if allied { "alliance" } else { "trade" };
            if let Some(enc) = self.encounter.clone() {
                encounter::remember(&mut self.world.captain, &enc, self.world.day, outcome);
            }
            let step = self.choice_step("negotiate", true, false, 0, message);
            self.clear_encounter();
            Ok(step)
        } else {
            let fight = {
                let ship = self.world.captain.ship.as_ref().ok_or(SimError::NoShip)?;
                let encounter = self.encounter.as_mut().ok_or(SimError::NoActiveEncounter)?;
                encounter::begin_fight(encounter, ship)
            };
            message.push('\n');
            message.push_str(&fight);
            Ok(self.choice_step("negotiate", false, false, 0, message))
        }
    }

    fn choose_flee(&mut self) -> Result<EncounterStep, SimError> {
        let (escaped, damage, mut message) = {
            let ship = self.world.captain.ship.as_ref().ok_or(SimError::NoShip)?;
            let encounter = self.encounter.as_mut().ok_or(SimError::NoActiveEncounter)?;
            encounter::resolve_flee(encounter, ship, &mut self.rng)
        };
        if damage > 0 {
            if let Some(ship) = self.world.captain.ship.as_mut() {
                ship.hull = 0.max(ship.hull - damage);
            }
        }
        if escaped {
            if let Some(enc) = self.encounter.clone() {
                encounter::remember(&mut self.world.captain, &enc, self.world.day, "fled");
            }
            let step = self.choice_step("flee", true, true, damage, message);
            self.clear_encounter();
            Ok(step)
        } else {
            let fight = {
                let ship = self.world.captain.ship.as_ref().ok_or(SimError::NoShip)?;
                let encounter = self.encounter.as_mut().ok_or(SimError::NoActiveEncounter)?;
                encounter::begin_fight(encounter, ship)
            };
            message.push('\n');
            message.push_str(&fight);
            Ok(self.choice_step("flee", false, false, damage, message))
        }
    }

    fn choose_fight(&mut self) -> Result<EncounterStep, SimError> {
        let message = {
            let ship = self.world.captain.ship.as_ref().ok_or(SimError::NoShip)?;
            let encounter = self.encounter.as_mut().ok_or(SimError::NoActiveEncounter)?;
            encounter::begin_fight(encounter, ship)
        };
        Ok(self.choice_step("fight", true, false, 0, message))
    }

    fn naval_flee(&mut self) -> Result<EncounterStep, SimError> {
        let (escaped, damage) = {
            let ship = self.world.captain.ship.as_ref().ok_or(SimError::NoShip)?;
            let encounter = self.encounter.as_ref().ok_or(SimError::NotInNavalCombat)?;
            let enemy = encounter::enemy_ship(encounter);
            naval::attempt_flee(ship, &enemy, &mut self.rng)
        };
        if let Some(ship) = self.world.captain.ship.as_mut() {
            ship.hull = 0.max(ship.hull - damage);
        }
        if let Some(enc) = self.encounter.as_mut() {
            enc.naval_turns += 1;
        }
        let mut message = if escaped {
            let mut msg = "You break away!".to_string();
            if damage > 0 {
                msg.push_str(&format!(
                    " A parting shot catches your hull for {damage} damage."
                ));
            }
            msg
        } else {
            format!("Flee failed! Their broadside rakes you for {damage} hull damage.")
        };
        let player_sunk = self.player_lost_ship();
        let crew_gone = self
            .world
            .captain
            .ship
            .as_ref()
            .is_some_and(|ship| ship.crew <= 0);
        let mut step = self.choice_step("flee", escaped, escaped, damage, message.clone());
        step.kind = "naval".into();
        step.turn = self
            .encounter
            .as_ref()
            .map(|enc| enc.naval_turns)
            .unwrap_or(0);
        step.player_sunk = player_sunk;
        if escaped {
            if let Some(enc) = self.encounter.clone() {
                encounter::remember(&mut self.world.captain, &enc, self.world.day, "fled");
            }
            self.clear_encounter();
            step.phase = "resolved".into();
        } else if player_sunk || crew_gone {
            if crew_gone && !player_sunk {
                self.plunder();
                message.push_str(" No crew left to sail.");
                step.message = message;
            }
            self.world.captain.naval_defeats += 1;
            self.clear_encounter();
            step.phase = "resolved".into();
        }
        Ok(step)
    }

    fn plunder(&mut self) {
        for item in &mut self.world.captain.cargo {
            item.quantity = 0.max(item.quantity - item.quantity / 2);
        }
        self.world.captain.cargo.retain(|item| item.quantity > 0);
        let silver_loss = self.world.captain.silver / 4;
        self.world.captain.silver -= silver_loss;
    }

    fn player_lost_ship(&self) -> bool {
        self.world
            .captain
            .ship
            .as_ref()
            .is_some_and(|ship| ship.hull <= 0)
    }

    fn enemy_strength(&self) -> i64 {
        self.encounter
            .as_ref()
            .map(|enc| enc.enemy_strength)
            .unwrap_or(0)
    }

    fn ensure_combatants(&mut self) {
        if self.player_combat.is_some() && self.opponent_combat.is_some() {
            return;
        }
        let Some(enc) = self.encounter.clone() else {
            return;
        };
        let (player, opponent) = encounter::create_duel_combatants(&enc, &self.world.captain);
        self.player_combat = Some(player);
        self.opponent_combat = Some(opponent);
    }

    fn clear_encounter(&mut self) {
        self.encounter = None;
        self.player_combat = None;
        self.opponent_combat = None;
    }

    fn blank_step(&self, kind: &str) -> EncounterStep {
        let enc = self.encounter.as_ref();
        let ship = self.world.captain.ship.as_ref();
        EncounterStep {
            kind: kind.to_string(),
            phase: enc.map(|e| e.phase.clone()).unwrap_or_default(),
            message: String::new(),
            choice: String::new(),
            success: false,
            escaped: false,
            hull_damage: 0,
            enemy_captain_id: enc.map(|e| e.enemy_captain_id.clone()).unwrap_or_default(),
            enemy_captain_name: enc
                .map(|e| e.enemy_captain_name.clone())
                .unwrap_or_default(),
            enemy_strength: enc.map(|e| e.enemy_strength).unwrap_or(0),
            turn: 0,
            player_action: String::new(),
            enemy_action: String::new(),
            player_hull_delta: 0,
            enemy_hull_delta: 0,
            player_crew_delta: 0,
            enemy_crew_delta: 0,
            boarding_progress: enc.map(|e| e.boarding_progress).unwrap_or(0),
            boarding_threshold: enc.map(|e| e.boarding_threshold).unwrap_or(0),
            enemy_sunk: false,
            player_sunk: false,
            boarding_triggered: false,
            flavor: String::new(),
            player_hull: ship.map(|s| s.hull).unwrap_or(0),
            enemy_hull: enc.map(|e| e.enemy_ship_hull).unwrap_or(0),
            player_crew: ship.map(|s| s.crew).unwrap_or(0),
            enemy_crew: enc.map(|e| e.enemy_ship_crew).unwrap_or(0),
            player_crew_lost: 0,
            enemy_crew_lost: 0,
            player_advantage: false,
            damage_to_opponent: 0,
            damage_to_player: 0,
            player_hp: self.player_combat.as_ref().map(|c| c.hp).unwrap_or(0),
            opponent_hp: self.opponent_combat.as_ref().map(|c| c.hp).unwrap_or(0),
            player_stamina_delta: 0,
            opponent_stamina_delta: 0,
            player_won: false,
            draw: false,
            injury: String::new(),
            opponent_injury: String::new(),
            style_effect: String::new(),
            prize_ok: false,
            prize_reason: String::new(),
        }
    }

    fn choice_step(
        &self,
        choice: &str,
        success: bool,
        escaped: bool,
        hull_damage: i64,
        message: String,
    ) -> EncounterStep {
        let mut step = self.blank_step("choice");
        step.choice = choice.to_string();
        step.success = success;
        step.escaped = escaped;
        step.hull_damage = hull_damage;
        step.message = message;
        step
    }

    fn naval_step(
        &self,
        round: NavalRound,
        enemy_sunk: bool,
        player_sunk: bool,
        boarding_triggered: bool,
        prize_ok: bool,
        prize_reason: String,
    ) -> EncounterStep {
        let mut step = self.blank_step("naval");
        step.turn = round.turn;
        step.player_action = round.player_action;
        step.enemy_action = round.enemy_action;
        step.player_hull_delta = round.player_hull_delta;
        step.enemy_hull_delta = round.enemy_hull_delta;
        step.player_crew_delta = round.player_crew_delta;
        step.enemy_crew_delta = round.enemy_crew_delta;
        step.boarding_progress = round.boarding_progress;
        step.flavor = round.flavor;
        step.enemy_sunk = enemy_sunk;
        step.player_sunk = player_sunk;
        step.boarding_triggered = boarding_triggered;
        step.prize_ok = prize_ok;
        step.prize_reason = prize_reason;
        if (enemy_sunk || player_sunk) && self.encounter.is_none() {
            step.phase = "resolved".into();
        }
        step
    }

    fn board_step(&self, outcome: BoardingOutcome) -> EncounterStep {
        let mut step = self.blank_step("board");
        step.player_crew_lost = outcome.player_crew_lost;
        step.enemy_crew_lost = outcome.enemy_crew_lost;
        step.player_advantage = outcome.player_advantage;
        step.flavor = outcome.flavor.clone();
        step.message = outcome.flavor;
        step
    }

    fn fight_step(
        &self,
        round: CombatRound,
        player_hp: i64,
        opponent_hp: i64,
        player_won: bool,
        draw: bool,
    ) -> EncounterStep {
        let mut step = self.blank_step("fight");
        if self.encounter.is_none() {
            step.phase = "resolved".into();
        }
        step.turn = round.turn;
        step.player_action = round.player_action;
        step.enemy_action = round.opponent_action;
        step.damage_to_opponent = round.damage_to_opponent;
        step.damage_to_player = round.damage_to_player;
        step.player_stamina_delta = round.player_stamina_delta;
        step.opponent_stamina_delta = round.opponent_stamina_delta;
        step.flavor = round.flavor.clone();
        step.message = round.flavor;
        step.player_hp = player_hp;
        step.opponent_hp = opponent_hp;
        step.player_won = player_won;
        step.draw = draw;
        step.injury = round.injury_inflicted.unwrap_or_default();
        step.opponent_injury = round.opponent_injury.unwrap_or_default();
        step.style_effect = round.style_effect.unwrap_or_default();
        step
    }

    /// Buy a warehouse, broker office, license, or dry-dock the current ship.
    ///
    /// `kind` is `warehouse`, `broker`, `license`, or `dry_dock`.
    /// Warehouse args are `[tier]`. Broker args are `[region, tier]`.
    /// License args are `[id]`. Dry dock takes an optional fleet ship name;
    /// a name searches the fleet, which this slice does not own.
    pub fn buy_infrastructure(&mut self, kind: &str, args: &[&str]) -> Result<(), SimError> {
        match kind {
            "warehouse" => {
                let tier = args.first().copied().unwrap_or("");
                if tier.is_empty() {
                    return Err(SimError::UsageBuyInfrastructure);
                }
                self.lease_warehouse(tier)
            }
            "broker" => {
                if args.len() < 2 {
                    return Err(SimError::UsageBuyInfrastructure);
                }
                self.open_broker(args[0], args[1])
            }
            "license" => {
                let id = args.first().copied().unwrap_or("");
                if id.is_empty() {
                    return Err(SimError::UsageBuyInfrastructure);
                }
                self.purchase_license(id)
            }
            "dry_dock" => self.dry_dock(args.first().copied()),
            _ => Err(SimError::UsageBuyInfrastructure),
        }
    }

    /// Open `tier` when the player does not already hold it, then draw `amount`.
    ///
    /// `tier` `emergency` is `emergency_loan` and does not open a line.
    /// `amount` of 0 only opens the line.
    pub fn take_credit(&mut self, tier: &str, amount: i64) -> Result<i64, SimError> {
        if tier == "emergency" {
            let received = infrastructure::emergency_loan(&mut self.world.captain, amount)?;
            return Ok(received);
        }
        let Some(spec) = content::content().credit_tier(tier).cloned() else {
            return Err(SimError::Rejected(format!("Unknown credit tier: {tier}")));
        };
        let (current_rank, active, current_name) = self
            .infra
            .credit
            .as_ref()
            .map(|credit| {
                (
                    credit_rank(&credit.tier),
                    credit.active,
                    credit.tier.clone(),
                )
            })
            .unwrap_or((0, false, "none".to_string()));
        let requested = credit_rank(tier);
        if active && current_rank > requested {
            return Err(SimError::Rejected(format!(
                "Already have {current_name} or better"
            )));
        }
        if !active || current_rank < requested {
            let day = self.world.day;
            infrastructure::open_credit_line(
                &mut self.infra,
                &spec,
                &self.world.captain.standing,
                day,
            )?;
        } else if amount == 0 {
            return Err(SimError::Rejected(format!(
                "Already have {current_name} or better"
            )));
        }
        if amount > 0 {
            infrastructure::draw_credit(&mut self.infra, &mut self.world.captain, amount)?;
        }
        self.project_books();
        Ok(amount)
    }

    /// `repay_credit`. Interest is paid before principal.
    pub fn repay_credit(&mut self, amount: i64) -> Result<(), SimError> {
        infrastructure::repay_credit(&mut self.infra, &mut self.world.captain, amount)?;
        self.project_books();
        Ok(())
    }

    /// Buy `policy_id`. Heat is the customs heat of the current port, or of
    /// the voyage destination while at sea.
    pub fn buy_insurance(
        &mut self,
        policy_id: &str,
        target_id: &str,
        voyage_origin: &str,
        voyage_destination: &str,
    ) -> Result<(), SimError> {
        let Some(spec) = content::content().policy(policy_id).cloned() else {
            return Err(SimError::Rejected(format!("Unknown policy: {policy_id}")));
        };
        let region = self.insurance_region();
        let heat = self.world.captain.standing.heat_of(&region);
        let day = self.world.day;
        infrastructure::purchase_policy(
            &mut self.infra,
            &mut self.world.captain,
            &spec,
            day,
            heat,
            target_id,
            voyage_origin,
            voyage_destination,
        )?;
        self.project_books();
        Ok(())
    }

    /// Move cargo from the ship into the warehouse at the current port.
    pub fn deposit_cargo(&mut self, good_id: &str, quantity: i64) -> Result<i64, SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Rejected("Must be docked to deposit".to_string()))?;
        let day = self.world.day;
        infrastructure::deposit_cargo(
            &mut self.infra,
            &port_id,
            &mut self.world.captain,
            good_id,
            quantity,
            day,
        )
    }

    /// Move cargo from the warehouse at the current port onto the ship.
    pub fn withdraw_cargo(
        &mut self,
        good_id: &str,
        quantity: i64,
        source_port: Option<&str>,
    ) -> Result<i64, SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Rejected("Must be docked to withdraw".to_string()))?;
        infrastructure::withdraw_cargo(
            &mut self.infra,
            &port_id,
            &mut self.world.captain,
            good_id,
            quantity,
            source_port,
        )
    }

    /// One session day. In port this ticks markets. At sea this sails.
    ///
    /// Order matches `GameSession.advance`: reputation, contract expiry and
    /// the contract-failure claim, infrastructure upkeep, credit, then the
    /// in-port day or the sea day. A pending duel returns no events and does
    /// not spend the day. Call [`Session::duel`] or
    /// [`Session::resolve_pending_duel`] first.
    pub fn advance(&mut self) -> Result<Turn, SimError> {
        reputation::tick_reputation(&mut self.world.captain.standing);
        // Same position as GameSession.advance: after the reputation tick,
        // before infrastructure upkeep. The captain is omitted, so a default
        // does not record a breach.
        let contracts = self.expire_contracts();
        self.file_contract_claims(&contracts);
        let mut notes = self.tick_upkeep();
        let mut turn = if self.world.voyage.status != VoyageStatus::AtSea {
            let shocks = economy::tick_markets(&mut self.world.ports, 1, &mut self.rng, 0);
            self.world.day += 1;
            self.world.captain.day += 1;
            if self.world.captain.provisions > 0 {
                self.world.captain.provisions -= 1;
            }
            if let Some(ship) = self.world.captain.ship.as_ref() {
                let wage = wage_bill(ship);
                if wage > 0 && self.world.captain.silver >= wage {
                    self.world.captain.silver -= wage;
                }
            }
            Turn {
                events: Vec::new(),
                shocks,
                contracts: Vec::new(),
                notes,
            }
        } else {
            let events = voyage::advance_day(&mut self.world, &mut self.rng)?;
            let destination = self.world.voyage.destination_id.clone();
            record_sea_consequences(&mut self.world, &mut self.infra, &events, &destination);
            if self.world.voyage.status == VoyageStatus::Arrived {
                let _ = voyage::arrive(&mut self.world);
                notes.extend(infrastructure::expire_voyage_policies(&mut self.infra));
                if let Some(port) = self.world.port(&self.world.voyage.destination_id) {
                    let port_id = port.id.clone();
                    let region = port.region.clone();
                    reputation::record_port_arrival(
                        &mut self.world.captain.standing,
                        &port_id,
                        &region,
                    );
                }
                self.refresh_contract_board();
            }
            Turn {
                events,
                shocks: Vec::new(),
                contracts: Vec::new(),
                notes,
            }
        };
        turn.contracts = contracts;
        self.project_books();
        reprice_all(&mut self.world);
        let milestones = campaign::evaluate_milestones(&self.world, &self.books);
        self.books.completed_milestones.extend(milestones);
        let newly = campaign::evaluate_victory_closure(&self.world, &self.books);
        self.books.completed_paths.extend(newly);
        Ok(turn)
    }

    /// Offers on the board at the current port.
    ///
    /// A later day redraws through [`Session::refresh_contract_board`], which
    /// consumes the session RNG. The isolated `Random(seed + 7919)` draw is
    /// only the new-game refresh.
    pub fn available_contracts(&mut self) -> Vec<Contract> {
        self.refresh_contract_board();
        self.board.offers.clone()
    }

    /// Accept an offer. The offer leaves the board.
    pub fn accept_contract(&mut self, offer_id: &str) -> Result<ActiveContract, SimError> {
        contracts::accept_offer(&mut self.board, offer_id, self.world.day)
    }

    /// Resolve a fulfilled contract and write it onto the house books.
    ///
    /// This is the same settlement [`Session::sell`] runs after `check_delivery`.
    /// A second call returns the recorded outcome and does not pay again.
    pub fn complete_contract(&mut self, offer_id: &str) -> Result<ContractOutcome, SimError> {
        if let Some(contract) = self
            .board
            .active
            .iter()
            .find(|contract| contract.offer_id == offer_id)
        {
            if contract.status != "accepted" {
                return Err(SimError::NoActiveContract);
            }
            if contract.delivered_quantity < contract.required_quantity {
                return Err(SimError::ContractNotFulfilled);
            }
            return self
                .settle_fulfilled()
                .into_iter()
                .find(|outcome| outcome.contract_id == offer_id)
                .ok_or(SimError::ContractNotFulfilled);
        }
        self.board
            .completed
            .iter()
            .find(|outcome| outcome.contract_id == offer_id)
            .cloned()
            .ok_or(SimError::NoActiveContract)
    }

    /// `resolve_completed`: pay every fulfilled obligation and record it.
    fn settle_fulfilled(&mut self) -> Vec<ContractOutcome> {
        let outcomes = contracts::resolve_completed(&mut self.board, self.world.day);
        for outcome in &outcomes {
            self.note_contract_on_books(outcome);
        }
        outcomes
    }

    fn expire_contracts(&mut self) -> Vec<ContractOutcome> {
        let outcomes = contracts::tick_contracts(&mut self.board, self.world.day, None);
        for outcome in &outcomes {
            self.note_contract_on_books(outcome);
        }
        outcomes
    }

    /// `GameSession.advance` files a `contract_failure` claim for expiry and
    /// abandonment before infrastructure upkeep.
    fn file_contract_claims(&mut self, outcomes: &[ContractOutcome]) {
        let day = self.world.day;
        for outcome in outcomes {
            if outcome.outcome_type != "expired" && outcome.outcome_type != "abandoned" {
                continue;
            }
            let loss = outcome.trust_delta.abs() * 50 + outcome.standing_delta.abs() * 30;
            infrastructure::resolve_claim(
                &mut self.infra,
                &mut self.world.captain,
                "contract_failure",
                loss,
                day,
                "",
                &outcome.contract_id,
                "",
            );
        }
    }

    fn tick_upkeep(&mut self) -> Vec<String> {
        let day = self.world.day;
        let mut notes =
            infrastructure::tick_infrastructure(&mut self.infra, &mut self.world.captain, day);
        let seized = notes
            .iter()
            .any(|message| message.to_lowercase().contains("seized"));
        if seized {
            infrastructure::resolve_claim(
                &mut self.infra,
                &mut self.world.captain,
                "cargo_damage",
                100,
                day,
                "",
                "",
                "",
            );
        }
        let credit_notes =
            infrastructure::tick_credit(&mut self.infra, &mut self.world.captain, day);
        for message in &credit_notes {
            if message.contains("DEFAULT") {
                let trust = self.world.captain.standing.commercial_trust;
                self.world.captain.standing.commercial_trust = 0.max(trust - 15);
            }
        }
        notes.extend(credit_notes);
        notes
    }

    fn project_books(&mut self) {
        self.books.warehouses = self
            .infra
            .warehouses
            .iter()
            .map(|lease| WarehouseSite {
                port_id: lease.port_id.clone(),
                active: lease.active,
            })
            .collect();
        self.books.brokers = self
            .infra
            .brokers
            .iter()
            .map(|broker| BrokerSite {
                region: broker.region.clone(),
                active: broker.active,
                tier: broker.tier.clone(),
            })
            .collect();
        self.books.licenses = self
            .infra
            .licenses
            .iter()
            .map(|license| ActiveLicense {
                license_id: license.license_id.clone(),
                active: license.active,
            })
            .collect();
        self.books.policies = self.infra.policies.len() as i64;
        self.books.claims_paid = self
            .infra
            .claims
            .iter()
            .filter(|claim| !claim.denied && claim.payout > 0)
            .count() as i64;
        self.books.credit = self.infra.credit.as_ref().and_then(|credit| {
            let visible = credit.active
                || credit.outstanding != 0
                || credit.interest_accrued != 0
                || credit.defaults != 0
                || credit.total_borrowed != 0
                || credit.total_repaid != 0;
            if !visible {
                return None;
            }
            Some(CreditBook {
                total_borrowed: credit.total_borrowed,
                defaults: credit.defaults,
                active: credit.active,
                total_repaid: credit.total_repaid,
            })
        });
    }

    fn lease_warehouse(&mut self, tier: &str) -> Result<(), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Rejected("Must be docked to lease a warehouse".to_string()))?;
        let catalog = content::content();
        let Some(spec) = catalog.warehouse_tier(tier).cloned() else {
            return Err(SimError::Rejected(format!(
                "Unknown warehouse tier: {tier}"
            )));
        };
        if !catalog
            .port_warehouse_tiers(&port_id)
            .iter()
            .any(|offered| offered == tier)
        {
            let name = self
                .world
                .port(&port_id)
                .map(|port| port.name.clone())
                .unwrap_or(port_id);
            return Err(SimError::Rejected(format!(
                "{tier} warehouse is not available at {name}"
            )));
        }
        let day = self.world.day;
        infrastructure::lease_warehouse(
            &mut self.infra,
            &mut self.world.captain,
            &port_id,
            &spec,
            day,
        )?;
        self.project_books();
        Ok(())
    }

    fn open_broker(&mut self, region: &str, tier: &str) -> Result<(), SimError> {
        let Some(spec) = content::content().broker(region, tier).cloned() else {
            return Err(SimError::Rejected(format!(
                "Unknown broker: {region} {tier}"
            )));
        };
        let day = self.world.day;
        infrastructure::open_broker_office(
            &mut self.infra,
            &mut self.world.captain,
            region,
            &spec,
            day,
        )?;
        self.project_books();
        Ok(())
    }

    fn purchase_license(&mut self, license_id: &str) -> Result<(), SimError> {
        let Some(spec) = content::content().license(license_id).cloned() else {
            return Err(SimError::Rejected(format!("Unknown license: {license_id}")));
        };
        let day = self.world.day;
        let standing = self.world.captain.standing.clone();
        infrastructure::purchase_license(
            &mut self.infra,
            &mut self.world.captain,
            &spec,
            &standing,
            day,
        )?;
        self.project_books();
        Ok(())
    }

    fn dry_dock(&mut self, ship_name: Option<&str>) -> Result<(), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Rejected("Must be docked".to_string()))?;
        let port_name = self
            .world
            .port(&port_id)
            .map(|port| port.name.clone())
            .unwrap_or_else(|| port_id.clone());
        let has_yard = self
            .world
            .port(&port_id)
            .is_some_and(|port| port.has_feature("shipyard"));
        if !has_yard {
            return Err(SimError::Rejected(format!("{port_name} has no shipyard")));
        }
        if let Some(name) = ship_name {
            return Err(SimError::Rejected(format!(
                "No ship named '{name}' docked at this port"
            )));
        }
        let repair = self
            .world
            .port(&port_id)
            .map(|port| port.repair_cost)
            .unwrap_or(0);
        let service = dry_dock_service(&self.world.captain.standing, &port_id);
        infrastructure::dry_dock(&mut self.world.captain, repair, service)?;
        Ok(())
    }

    fn insurance_region(&self) -> String {
        if self.world.voyage.status == VoyageStatus::AtSea {
            return self
                .world
                .port(&self.world.voyage.destination_id)
                .map(|port| port.region.clone())
                .unwrap_or_else(|| "Mediterranean".to_string());
        }
        current_port_id(&self.world)
            .and_then(|port_id| self.world.port(port_id))
            .map(|port| port.region.clone())
            .unwrap_or_else(|| "Mediterranean".to_string())
    }

    fn note_contract_on_books(&mut self, outcome: &ContractOutcome) {
        self.world.captain.silver += outcome.silver_delta;
        self.books.completed_contracts.push(CompletedContract {
            outcome_type: outcome.outcome_type.clone(),
            family: Some(outcome.family.clone()),
            summary: outcome.summary.clone(),
        });
    }

    /// `GameSession._refresh_board`. Draws `self.rng`, the session stream.
    ///
    /// Arrival and a later in-port view both use this. Only
    /// [`Session::refresh_new_game_board`] substitutes `Random(seed + 7919)`.
    fn refresh_contract_board(&mut self) {
        let Some(port_id) = current_port_id(&self.world).map(str::to_string) else {
            return;
        };
        if self.board.last_refresh_day == self.world.day {
            return;
        }
        let captain_type = self.world.captain.captain_type.clone();
        let rank = self
            .world
            .captain
            .ship
            .as_ref()
            .map(|ship| content::ship_class_rank(&ship.template_id))
            .unwrap_or(0);
        let max_offers = self.board.max_offers;
        let offers = {
            let Session { world, rng, .. } = self;
            contracts::generate_offers(world, &port_id, &captain_type, Some(rank), max_offers, rng)
        };
        self.board.offers = offers;
        self.board.last_refresh_day = self.world.day;
    }

    /// New-game board only: save the session RNG, draw `Random(seed + 7919)`, restore.
    fn refresh_new_game_board(&mut self) {
        let seed = self.world.seed;
        let saved = std::mem::replace(&mut self.rng, PyRandom::from_seed(seed + 7919));
        self.refresh_contract_board();
        self.rng = saved;
    }

    /// Learn a fighting style at the current port (`portlight train`).
    ///
    /// On success the style's silver is spent, then the clock advances
    /// `training_days` times through [`Session::advance`].
    pub fn train_crew(&mut self, style_id: &str) -> Result<(), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or(SimError::MustBeDockedToTrain)?;
        if let Some(error) = training::can_learn_style(
            &self.world.captain.learned_styles,
            &[],
            self.world.captain.silver,
            &port_id,
            style_id,
        ) {
            return Err(SimError::Rejected(error));
        }
        let days = training::learn_style(
            &mut self.world.captain.learned_styles,
            &mut self.world.captain.silver,
            style_id,
        );
        for _ in 0..days {
            self.advance()?;
        }
        Ok(())
    }

    /// Recruit a companion at the current port (`portlight recruit`).
    ///
    /// Hire cost is subtracted here, then [`companion::recruit`] records the
    /// companion. That is the CLI order.
    pub fn recruit_companion(&mut self, companion_id: &str) -> Result<(), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or(SimError::MustBeDockedToRecruit)?;
        if let Some(error) = companion::can_recruit(&self.world.captain, companion_id, &port_id) {
            return Err(SimError::Rejected(error));
        }
        let cost = content::content()
            .companion(companion_id)
            .expect("can_recruit accepted this companion")
            .hire_cost;
        self.world.captain.silver -= cost;
        companion::recruit(&mut self.world.captain, companion_id, self.world.day);
        Ok(())
    }

    /// Learn the next level of a skill (`portlight learn-skill`).
    ///
    /// Python spends silver and training days, not a separate point currency.
    /// This method is that spend: pay the level cost, then advance one session
    /// day per training day.
    pub fn spend_skill_point(&mut self, skill_id: &str) -> Result<(), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or(SimError::MustBeDockedToLearnSkill)?;
        let skill_id = skills::resolve_skill_id(skill_id);
        if let Some(error) = skills::can_learn_skill(
            &self.world.captain.skills,
            self.world.captain.silver,
            &port_id,
            &skill_id,
        ) {
            return Err(SimError::Rejected(error));
        }
        let (silver, days) = skills::learn_skill(
            &mut self.world.captain.skills,
            self.world.captain.silver,
            &skill_id,
        );
        self.world.captain.silver = silver;
        for _ in 0..days {
            self.advance()?;
        }
        Ok(())
    }

    /// `GameSession.tick_sea_captain_agency`.
    ///
    /// The CLI and the TUI call this after each sea day. It is not part of
    /// [`Session::advance`]. In port, or with no world at sea, the result is
    /// `(None, false, [])`.
    ///
    /// Silver gifts are applied immediately. An ambush or challenge consumes
    /// the same RNG draws as `create_encounter` (faction, captain, then one
    /// `random()` for ship speed), writes [`PendingDuel`], and returns the
    /// encounter. `ambush` is true only when the verb is `ambush`; that
    /// encounter's phase is `naval`.
    pub fn tick_sea_captain_agency(
        &mut self,
    ) -> (Option<EncounterState>, bool, Vec<(String, String)>) {
        if self.world.voyage.status != VoyageStatus::AtSea {
            return (None, false, Vec::new());
        }
        let region = encounter::voyage_region(&self.world);
        let actions = memory::tick_captain_agency(
            &self.world.captain_memories,
            &region,
            self.world.captain.silver,
            self.world.day,
            &mut self.rng,
        );
        let mut notices = Vec::new();
        let mut encounter = None;
        let mut ambush = false;
        for action in actions {
            notices.push((action.effect_type.clone(), action.message));
            if action.effect_type == "silver" {
                self.world.captain.silver += action.effect_value;
            } else if action.effect_type == "encounter" && encounter.is_none() {
                let Some(mut rolled) =
                    encounter::create_encounter(&self.world, &mut self.rng, None)
                else {
                    continue;
                };
                rolled.enemy_captain_id = action.captain_id.clone();
                rolled.enemy_captain_name = action.captain_name.clone();
                if let Some(captain) = content::content().pirate(&action.captain_id) {
                    rolled.enemy_faction_id = captain.faction_id.clone();
                    rolled.enemy_personality = captain.personality.clone();
                    rolled.enemy_strength = captain.strength;
                }
                rolled.enemy_region = region.clone();
                if action.verb == "ambush" {
                    rolled.phase = "naval".to_string();
                    ambush = true;
                }
                self.world.pending_duel = Some(PendingDuel {
                    captain_id: rolled.enemy_captain_id.clone(),
                    captain_name: rolled.enemy_captain_name.clone(),
                    faction_id: rolled.enemy_faction_id.clone(),
                    personality: rolled.enemy_personality.clone(),
                    strength: rolled.enemy_strength,
                    region: rolled.enemy_region.clone(),
                });
                self.player_combat = None;
                self.opponent_combat = None;
                self.encounter = Some(rolled.clone());
                encounter = Some(rolled);
                break;
            }
        }
        (encounter, ambush, notices)
    }

    /// Seed one captain memory. Encounter resolution will call the same
    /// record; the `remember` script command uses this so a sea-day golden
    /// can fire agency without the encounter machine.
    pub fn remember_captain(&mut self, captain_id: &str, outcome: &str) -> Result<(), SimError> {
        if content::content().pirate(captain_id).is_none() {
            return Err(SimError::UnknownPirate(captain_id.to_string()));
        }
        let region = encounter::voyage_region(&self.world);
        let day = self.world.day;
        let memory = memory::get_or_create_memory(&mut self.world.captain_memories, captain_id);
        memory::record_encounter(memory, day, &region, outcome, false, false, 0);
        Ok(())
    }
}

fn record_receipt(session: &mut Session, receipt: &TradeReceipt) {
    session.trade_seq += 1;
    session
        .books
        .note_receipt(receipt.action, receipt.total_price);
    session.receipts.push(receipt.clone());
}

fn pricing(world: &World) -> Option<&PricingDef> {
    content::content()
        .captain(&world.captain.captain_type)
        .map(|captain| &captain.pricing)
}

fn current_port_id(world: &World) -> Option<&str> {
    if world.voyage.status == VoyageStatus::InPort {
        Some(world.voyage.destination_id.as_str())
    } else {
        None
    }
}

fn credit_rank(tier: &str) -> i64 {
    match tier {
        "merchant_line" => 1,
        "house_credit" => 2,
        "premier_commercial" => 3,
        _ => 0,
    }
}

fn record_sea_consequences(
    world: &mut World,
    infra: &mut InfrastructureRecord,
    events: &[VoyageEvent],
    destination: &str,
) {
    let region = world
        .port(&world.voyage.destination_id)
        .map(|port| port.region.clone())
        .unwrap_or_else(|| "Mediterranean".to_string());
    let port_id = world.voyage.origin_id.clone();
    let day = world.day;
    for event in events {
        if event.event_type == voyage::EventType::Inspection {
            let seized = !event.cargo_lost.is_empty();
            reputation::record_inspection_outcome(
                &mut world.captain.standing,
                day,
                &port_id,
                &region,
                event.silver_delta.abs(),
                seized,
            );
        }
        settle_event_insurance(world, infra, event, destination, day);
    }
}

fn settle_event_insurance(
    world: &mut World,
    infra: &mut InfrastructureRecord,
    event: &VoyageEvent,
    destination: &str,
    day: i64,
) {
    let incident = event.event_type.as_str();
    if event.hull_delta < 0 {
        infrastructure::resolve_claim(
            infra,
            &mut world.captain,
            incident,
            event.hull_delta.abs() * 3,
            day,
            "",
            "",
            destination,
        );
    }
    for (good_id, qty) in &event.cargo_lost {
        let Some(good) = content::content().good(good_id) else {
            continue;
        };
        let category = good.category.clone();
        let value = good.base_price * qty;
        infrastructure::resolve_claim(
            infra,
            &mut world.captain,
            incident,
            value,
            day,
            &category,
            "",
            destination,
        );
    }
}

fn reprice_all(world: &mut World) {
    let pricing = pricing(world).cloned();
    for port in &mut world.ports {
        recalculate_prices(port, pricing.as_ref());
    }
}

struct RoleSpec {
    role: &'static str,
    name: &'static str,
    wage: i64,
    max_per_ship: Option<i64>,
}

fn role_spec(role: &str) -> Option<RoleSpec> {
    Some(match role {
        "sailor" => RoleSpec {
            role: "sailor",
            name: "Sailor",
            wage: 1,
            max_per_ship: None,
        },
        "gunner" => RoleSpec {
            role: "gunner",
            name: "Gunner",
            wage: 2,
            max_per_ship: Some(3),
        },
        "navigator" => RoleSpec {
            role: "navigator",
            name: "Navigator",
            wage: 3,
            max_per_ship: Some(1),
        },
        "surgeon" => RoleSpec {
            role: "surgeon",
            name: "Surgeon",
            wage: 3,
            max_per_ship: Some(1),
        },
        "marine" => RoleSpec {
            role: "marine",
            name: "Marine",
            wage: 2,
            max_per_ship: Some(4),
        },
        "quartermaster" => RoleSpec {
            role: "quartermaster",
            name: "Quartermaster",
            wage: 2,
            max_per_ship: Some(1),
        },
        _ => return None,
    })
}

fn role_count(ship: &crate::model::Ship, role: &str) -> i64 {
    match role {
        "sailor" => ship.sailors,
        "gunner" => ship.gunners,
        "navigator" => ship.navigators,
        "surgeon" => ship.surgeons,
        "marine" => ship.marines,
        "quartermaster" => ship.quartermasters,
        _ => 0,
    }
}

fn set_role_count(ship: &mut crate::model::Ship, role: &str, count: i64) {
    let count = count.max(0);
    match role {
        "sailor" => ship.sailors = count,
        "gunner" => ship.gunners = count,
        "navigator" => ship.navigators = count,
        "surgeon" => ship.surgeons = count,
        "marine" => ship.marines = count,
        "quartermaster" => ship.quartermasters = count,
        _ => {}
    }
}

fn officer_name(region: &str, rng: &mut PyRandom) -> String {
    let pool = content::content().officer_pool(region);
    pool[rng.choice_index(pool.len())].clone()
}

fn officer_trait(rng: &mut PyRandom) -> String {
    let traits = &content::content().officer_names.traits;
    traits[rng.choice_index(traits.len())].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provision_uses_the_service_modifier() {
        let mut session = Session::new("Ada", "merchant", 1, Some("silva_bay")).unwrap();
        session.provision(4).unwrap();
        assert_eq!(session.world.captain.silver, 550 - 8);
        assert_eq!(session.world.captain.provisions, 34);

        session.world.captain.standing.set_port("silva_bay", 30);
        let before = session.world.captain.silver;
        session.provision(4).unwrap();
        assert_eq!(session.world.captain.silver, before - 4);

        assert!(
            (reputation::service_modifier(&session.world.captain.standing, "silva_bay") - 0.8)
                .abs()
                < 1e-9
        );
        session.world.captain.standing.set_port("silva_bay", 15);
        assert!(
            (reputation::service_modifier(&session.world.captain.standing, "silva_bay") - 0.9)
                .abs()
                < 1e-9
        );
        session.world.captain.standing.set_port("silva_bay", 5);
        assert!(
            (reputation::service_modifier(&session.world.captain.standing, "silva_bay") - 0.95)
                .abs()
                < 1e-9
        );
        session.world.captain.standing.set_port("silva_bay", 4);
        assert!(
            (reputation::service_modifier(&session.world.captain.standing, "silva_bay") - 1.0)
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn save_clamps_negative_silver() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        session.world.captain.silver = -3;
        let dir = std::env::temp_dir().join(format!("portlight-clamp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        session.save(&dir, "default").unwrap();
        assert_eq!(session.world().captain.silver, 0);
        let loaded = Session::load(&dir, "default").unwrap().unwrap();
        assert_eq!(loaded.world().captain.silver, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn new_game_board_restores_the_session_rng_and_a_later_refresh_draws_it() {
        let mut session = Session::new("Ada", "merchant", 42, None).unwrap();
        let mut expected = PyRandom::from_seed(42);
        assert_eq!(session.rng.random(), expected.random());
        session.available_contracts();
        assert_eq!(session.rng.random(), expected.random());

        session.world.day += 1;
        session.board.last_refresh_day = 0;
        let mut replay = session.rng.clone();
        let port_id = session.world.voyage.destination_id.clone();
        let captain_type = session.world.captain.captain_type.clone();
        let rank =
            content::ship_class_rank(&session.world.captain.ship.as_ref().unwrap().template_id);
        session.available_contracts();
        let _ = contracts::generate_offers(
            &session.world,
            &port_id,
            &captain_type,
            Some(rank),
            session.board.max_offers,
            &mut replay,
        );
        assert_eq!(session.board.offers.len(), 5);
        assert_eq!(session.rng.random(), replay.random());
    }

    #[test]
    fn completing_the_grain_famine_writes_the_books() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        let offer = "71773aae754b";
        let accepted = session.accept_contract(offer).unwrap();
        assert_eq!(accepted.good_id, "grain");
        assert_eq!(accepted.required_quantity, 23);
        session.buy("grain", 23).unwrap();
        session.depart("corsairs_rest").unwrap();
        for _ in 0..3 {
            session.advance().unwrap();
        }
        assert_eq!(session.world.voyage.status, VoyageStatus::InPort);
        assert_eq!(session.world.voyage.destination_id, "corsairs_rest");
        let before_sell = session.world.captain.silver;
        let sale = session.sell("grain", 23).unwrap();
        assert_eq!(sale.contracts.len(), 1);
        assert_eq!(sale.contracts[0].outcome_type, "completed_bonus");
        assert_eq!(sale.contracts[0].silver_delta, 612);
        assert_eq!(
            sale.contracts[0].summary,
            "Delivered 23 grain to corsairs_rest (early bonus: +60 silver)"
        );
        assert_eq!(
            session.world.captain.silver,
            before_sell + sale.receipt.total_price + 612
        );
        assert_eq!(session.books.completed_contracts.len(), 1);
        let after_sell = session.world.captain.silver;
        let outcome = session.complete_contract(offer).unwrap();
        assert_eq!(outcome.silver_delta, 612);
        assert_eq!(session.world.captain.silver, after_sell);
        assert_eq!(session.books.completed_contracts.len(), 1);
        assert_eq!(
            session.books.completed_contracts[0].family.as_deref(),
            Some("shortage")
        );
        let detail = session
            .victory()
            .into_iter()
            .find(|path| path.path_id == "commercial_empire")
            .unwrap()
            .requirements
            .into_iter()
            .find(|req| req.description.contains("contracts"))
            .unwrap();
        assert_eq!(detail.detail, "Completed: 1");
    }

    #[test]
    fn an_accepted_contract_defaults_on_the_deadline() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        session.accept_contract("71773aae754b").unwrap();
        let mut expired = None;
        for _ in 0..20 {
            let turn = session.advance().unwrap();
            if !turn.contracts.is_empty() {
                expired = Some(turn.contracts);
                break;
            }
        }
        let outcomes = expired.expect("deadline");
        assert_eq!(outcomes[0].outcome_type, "expired");
        assert_eq!(outcomes[0].silver_delta, 0);
        assert_eq!(session.world.captain.wanted_level, 0);
        assert_eq!(session.books.completed_contracts[0].outcome_type, "expired");
        assert!(session.board.active.is_empty());
    }

    #[test]
    fn save_load_restores_active_contracts_and_breaches() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        let offer_id = session.board.offers[0].id.clone();
        session.accept_contract(&offer_id).unwrap();
        session.board.breaches.push(crate::model::BreachRecord {
            contract_id: offer_id.clone(),
            day: 2,
            port_id: "porto_novo".to_string(),
            family: session.board.active[0].family.clone(),
        });
        session.world.captain.wanted_level = 1;
        session.board.completed.push(ContractOutcome {
            contract_id: "done-1".to_string(),
            outcome_type: "completed".to_string(),
            silver_delta: 40,
            trust_delta: 1,
            standing_delta: 1,
            heat_delta: -1,
            completion_day: 3,
            summary: "Delivered 2 silk to porto_novo".to_string(),
            family: "luxury_discreet".to_string(),
            good_id: "silk".to_string(),
            required_quantity: 2,
            delivered_quantity: 2,
            destination_port_id: "porto_novo".to_string(),
            deadline_day: 20,
            reward_silver: 40,
        });
        let before = session.board.clone();
        let dir = std::env::temp_dir().join(format!("portlight-board-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        session.save(&dir, "contracts").unwrap();
        let text = std::fs::read_to_string(dir.join("saves").join("contracts.json")).unwrap();
        let saved: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert!(saved["contract_board"].get("breaches").is_none());
        assert_eq!(
            saved["captain"]["breach_records"][0]["contract_id"],
            offer_id
        );
        assert_eq!(saved["captain"]["wanted_level"], 1);
        assert_eq!(saved["contract_board"]["active"][0]["offer_id"], offer_id);
        assert_eq!(saved["contract_board"]["active"][0]["status"], "accepted");
        assert!(saved["contract_board"]["completed"][0]
            .get("good_id")
            .is_none());
        assert_eq!(
            saved["contract_board"]["completed"][0]["family"],
            "luxury_discreet"
        );

        let loaded = Session::load(&dir, "contracts").unwrap().unwrap();
        assert_eq!(loaded.board.offers, before.offers);
        assert_eq!(loaded.board.active, before.active);
        assert_eq!(loaded.board.last_refresh_day, before.last_refresh_day);
        assert_eq!(loaded.board.max_offers, before.max_offers);
        assert_eq!(loaded.board.breaches, before.breaches);
        assert_eq!(loaded.world().captain.wanted_level, 1);
        let completed = &loaded.board.completed[0];
        assert_eq!(completed.contract_id, "done-1");
        assert_eq!(completed.outcome_type, "completed");
        assert_eq!(completed.silver_delta, 40);
        assert_eq!(completed.trust_delta, 1);
        assert_eq!(completed.standing_delta, 1);
        assert_eq!(completed.heat_delta, -1);
        assert_eq!(completed.completion_day, 3);
        assert_eq!(completed.summary, "Delivered 2 silk to porto_novo");
        assert_eq!(completed.family, "luxury_discreet");
        assert!(completed.good_id.is_empty());
        assert_eq!(completed.required_quantity, 0);
        assert_eq!(
            loaded.books().completed_contracts[0].family.as_deref(),
            Some("luxury_discreet")
        );
        assert_eq!(
            loaded.world().captain.silver,
            session.world().captain.silver
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn saved_board_matches_the_python_v12_shape() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        session.accept_contract("63fc3f8be22a").unwrap();
        session.board.breaches.push(crate::model::BreachRecord {
            contract_id: "63fc3f8be22a".to_string(),
            day: 2,
            port_id: "porto_novo".to_string(),
            family: "smuggling".to_string(),
        });
        session.world.captain.wanted_level = 1;
        let dir = std::env::temp_dir().join(format!("portlight-shape-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        session.save(&dir, "contracts").unwrap();
        let rust: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("saves").join("contracts.json")).unwrap(),
        )
        .unwrap();
        let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../parity/saves/contract_v12.json");
        let python: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(fixture).unwrap()).unwrap();
        assert_eq!(rust["contract_board"], python["contract_board"]);
        assert_eq!(
            rust["captain"]["breach_records"],
            python["captain"]["breach_records"]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn python_v12_encounter_round_trips() {
        let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../parity/saves/encounter_v12.json");
        let python: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&fixture).unwrap()).unwrap();
        let root = fixture.parent().unwrap().parent().unwrap();
        let loaded = Session::load(root, "encounter_v12").unwrap().unwrap();
        assert_eq!(loaded.world.captain.name, "Ada");
        assert_eq!(loaded.world.captain.captain_type, "corsair");
        assert_eq!(loaded.world.seed, 1);
        assert_eq!(loaded.world.captain.duels_won, 2);
        assert_eq!(loaded.world.captain.duels_lost, 1);
        assert_eq!(loaded.world.captain.naval_victories, 3);
        assert_eq!(loaded.world.captain.naval_defeats, 4);
        assert_eq!(loaded.world.captain.encounters.len(), 1);
        assert_eq!(loaded.world.captain.encounters[0].captain_id, "old_coral");
        assert_eq!(loaded.world.captain.encounters[0].outcome, "trade");
        assert_eq!(loaded.world.captain.encounters[0].region, "Mediterranean");
        let enc = loaded.encounter.as_ref().expect("active encounter");
        assert_eq!(enc.phase, "naval");
        assert_eq!(enc.enemy_captain_id, "scarlet_ana");
        assert_eq!(enc.naval_turns, 1);
        assert_eq!(enc.enemy_ship_hull, 120);
        let player = loaded.player_combat.as_ref().expect("player hp");
        let opponent = loaded.opponent_combat.as_ref().expect("opponent hp");
        assert_eq!((player.hp, player.stamina), (9, 6));
        assert_eq!((opponent.hp, opponent.stamina), (4, 5));

        let dir = std::env::temp_dir().join(format!("portlight-enc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut loaded = loaded;
        loaded.save(&dir, "encounter_v12").unwrap();
        let again = Session::load(&dir, "encounter_v12").unwrap().unwrap();
        assert_eq!(again.world.captain.duels_won, 2);
        assert_eq!(again.encounter.as_ref().unwrap().enemy_ship_hull, 120);
        assert_eq!(again.player_combat.as_ref().unwrap().hp, 9);
        let rust: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("saves").join("encounter_v12.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(rust["pirate_state"], python["pirate_state"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sea_agency_ambush_matches_create_encounter_rng() {
        let mut session = Session::new("Ada", "merchant", 9, None).unwrap();
        let (encounter, ambush, notices) = session.tick_sea_captain_agency();
        assert!(encounter.is_none());
        assert!(!ambush);
        assert!(notices.is_empty());

        for _ in 0..4 {
            session
                .remember_captain("the_butcher", "ship_sunk")
                .unwrap();
        }
        session.depart("silva_bay").unwrap();
        session.advance().unwrap();
        let (encounter, ambush, _) = session.tick_sea_captain_agency();
        assert!(encounter.is_none() && !ambush);
        session.advance().unwrap();
        session.tick_sea_captain_agency();
        session.advance().unwrap();
        assert_eq!(session.world.day, 4);
        assert_eq!(session.world.captain.silver, 538);
        let (encounter, ambush, notices) = session.tick_sea_captain_agency();
        assert!(ambush);
        assert_eq!(notices.len(), 1);
        assert_eq!(notices[0].0, "encounter");
        assert!(notices[0].1.contains("The Butcher"));
        let encounter = encounter.expect("ambush");
        assert_eq!(encounter.phase, "naval");
        assert_eq!(encounter.enemy_captain_id, "the_butcher");
        assert_eq!(encounter.enemy_strength, 8);
        assert_eq!(encounter.enemy_ship_hull, 170);
        assert_eq!(encounter.enemy_ship_cannons, 12);
        assert_eq!(encounter.enemy_ship_crew, 28);
        assert_eq!(encounter.enemy_ship_crew_max, 36);
        assert!((encounter.enemy_ship_maneuver - 0.4).abs() < 1e-12);
        assert!((encounter.enemy_ship_speed - 5.251391040026104).abs() < 1e-12);
        let pending = session.world.pending_duel.as_ref().unwrap();
        assert_eq!(pending.captain_id, "the_butcher");
        assert_eq!(pending.faction_id, "crimson_tide");
        assert_eq!(pending.personality, "aggressive");
        assert_eq!(pending.strength, 8);
        assert_eq!(pending.region, "Mediterranean");
        let progress = session.world.voyage.progress;
        let turn = session.advance().unwrap();
        assert!(turn.events.is_empty());
        assert_eq!(session.world.day, 4);
        assert_eq!(session.world.voyage.progress, progress);
        assert_eq!(
            session
                .encounter
                .as_ref()
                .map(|enc| enc.enemy_captain_id.as_str()),
            Some("the_butcher")
        );
    }

    #[test]
    fn purchases_write_the_house_books() {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        session.buy_infrastructure("warehouse", &["depot"]).unwrap();
        session
            .buy_infrastructure("broker", &["Mediterranean", "local"])
            .unwrap();
        session.buy_insurance("hull_basic", "", "", "").unwrap();
        session.take_credit("merchant_line", 80).unwrap();
        let books = session.books();
        assert_eq!(books.warehouses.len(), 1);
        assert!(books.warehouses[0].active);
        assert_eq!(books.warehouses[0].port_id, "porto_novo");
        assert_eq!(books.brokers[0].region, "Mediterranean");
        assert_eq!(books.brokers[0].tier, "local");
        assert_eq!(books.policies, 1);
        assert_eq!(books.claims_paid, 0);
        let credit = books.credit.as_ref().unwrap();
        assert_eq!(credit.total_borrowed, 80);
        assert_eq!(credit.defaults, 0);
        assert!(credit.active);
        assert_eq!(credit.total_repaid, 0);
        let oceanic = session
            .victory()
            .into_iter()
            .find(|path| path.path_id == "oceanic_reach")
            .unwrap();
        let foothold = oceanic
            .requirements
            .iter()
            .find(|req| req.description.contains("foothold"))
            .unwrap();
        assert_eq!(foothold.status, "missing");
        assert!(foothold.detail.contains("EI warehouse: no"));
    }

    #[test]
    fn claims_paid_counts_only_paid_claims() {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        session.buy_insurance("hull_basic", "", "", "").unwrap();
        infrastructure::resolve_claim(
            &mut session.infra,
            &mut session.world.captain,
            "storm",
            80,
            session.world.day,
            "",
            "",
            "",
        );
        session.project_books();
        assert_eq!(session.books().claims_paid, 1);
        session.buy_insurance("cargo_standard", "", "", "").unwrap();
        infrastructure::resolve_claim(
            &mut session.infra,
            &mut session.world.captain,
            "inspection",
            100,
            session.world.day,
            "contraband",
            "",
            "",
        );
        session.project_books();
        assert_eq!(session.books().claims_paid, 1);
        assert_eq!(session.infrastructure().claims.len(), 2);
        session.advance().unwrap();
        let ids: Vec<_> = session
            .books()
            .completed_milestones
            .iter()
            .map(|row| row.milestone_id.as_str())
            .collect();
        assert!(ids.contains(&"finance_first_insurance"));
    }

    #[test]
    fn credit_draw_opens_the_finance_milestone() {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        session.take_credit("merchant_line", 80).unwrap();
        session.advance().unwrap();
        let ids: Vec<_> = session
            .books()
            .completed_milestones
            .iter()
            .map(|row| row.milestone_id.as_str())
            .collect();
        assert!(ids.contains(&"finance_credit_opened"));
        let credit = session.books().credit.as_ref().unwrap();
        assert!(credit.active);
        assert_eq!(credit.total_repaid, 0);
    }
}
