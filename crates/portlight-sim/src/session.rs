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
//! all met. A sea day calls `enrich_voyage_day` immediately after `advance_day`.
//! Arrival writes the port visit, the arrival text, and one port consequence
//! before the contract board refreshes. Sell and a sea day evaluate narrative
//! beats. An in-port day does not.
//!
//! [`Session::buy_infrastructure`], [`Session::take_credit`], and
//! [`Session::buy_insurance`] write warehouses, brokers, licenses, policies,
//! and credit onto the house books. A fulfilled contract is written by [`Session::sell`], which
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
//! `portlight duel` and `GameSession._resolve_pending_duel`. Ending an
//! encounter also clears it, matching `cli._clear_encounter`. The voyage event
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
use crate::custom_captain::{self, CustomCaptainSpec};
use crate::duel::{self, DuelOutcome};
use crate::economy::{self, recalculate_prices, TradeReceipt};
use crate::encounter::{self, BoardingOutcome, EncounterState};
use crate::error::SimError;
use crate::fleet;
use crate::infrastructure::{self, dry_dock_service};
use crate::injuries;
use crate::loot;
use crate::memory;
use crate::model::{
    ActiveContract, Armor, CargoItem, Consequence, Contract, ContractBoard, ContractOutcome,
    InfrastructureRecord, InstalledUpgrade, NarrativeState, Officer, PendingDuel,
    PirateEncounterRecord, VoyageStatus, Weapon, World,
};
use crate::naval::{self, NavalRound};
use crate::pyrand::PyRandom;
use crate::reputation::{self, record_trade_outcome};
use crate::save::{self, LoadedGame};
use crate::ship::{self, wage_bill};
use crate::skills;
use crate::training;
use crate::util::{py_round, py_trunc};
use crate::voyage::{self, sail_lanes, SailLane, VoyageEvent};
use crate::weapon_provenance;
use crate::weapon_quality;
use crate::world::{new_game, new_game_with_def};

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
    pub player_stamina: i64,
    pub player_stamina_max: i64,
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
    /// Personal-combat win is waiting on spare or take-all.
    pending_victory: bool,
    narrative: NarrativeState,
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
        Ok(Self::open(new_game(
            captain_name,
            captain_type,
            seed,
            starting_port,
        )?))
    }

    /// Start from a custom captain spec.
    ///
    /// Python validates, builds the template, registers it, then calls
    /// `GameSession.new(..., captain_type="custom")` (`app/cli.py` line 286).
    /// Validation failures use the same sentences, joined by newlines in the
    /// order `validate_spec` returns them. `starting_port` overrides the
    /// spec's home port the way `new_game` does.
    pub fn new_custom(
        spec: &CustomCaptainSpec,
        seed: i128,
        starting_port: Option<&str>,
    ) -> Result<Self, SimError> {
        let errors = custom_captain::validate_spec(spec);
        if !errors.is_empty() {
            return Err(SimError::Sentence(errors.join("\n")));
        }
        let template = custom_captain::build_custom_template(spec);
        let def = template.to_captain_def();
        let world = new_game_with_def(&spec.name, &def, seed, starting_port, true)?;
        Ok(Self::open(world))
    }

    fn open(world: World) -> Self {
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
            pending_victory: false,
            narrative: NarrativeState::default(),
        };
        session.refresh_new_game_board();
        session
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
                pending_victory: self.pending_victory,
            },
            &self.infra,
            &self.narrative,
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

    /// Save slots under `base_path/saves`, peeked the way `list_save_slots` does.
    ///
    /// Missing directories are an empty list. The files are not migrated and
    /// the version stays whatever is on disk.
    pub fn list_saves(base_path: impl AsRef<Path>) -> Vec<save::SaveSlotSummary> {
        save::list_save_slots(base_path.as_ref())
    }

    /// Catalog captains in Python roster order. Custom creation is
    /// [`Session::new_custom`], not a row in this list.
    pub fn starting_captains() -> Vec<custom_captain::StartingCaptain> {
        custom_captain::starting_captains()
    }

    /// Regions, ports, blocs, factions, and mentors a custom spec may name.
    pub fn custom_captain_options() -> custom_captain::CustomCaptainOptions {
        custom_captain::custom_captain_options()
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
            pending_victory: loaded.pending_victory,
            narrative: loaded.narrative,
        };
        reprice_all(&mut session.world);
        session.project_books();
        Ok(session)
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    /// Beats that have fired. Save writes this as `narrative`.
    pub fn narrative(&self) -> &NarrativeState {
        &self.narrative
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

    /// Spare/take-all is waiting on a personal-combat win.
    pub fn pending_victory(&self) -> bool {
        self.pending_victory
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
        let mods = pricing(&self.world).clone();
        if let Some(port) = self.world.port_mut(&port_id) {
            recalculate_prices(port, Some(&mods));
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
        let mods = pricing(&self.world).clone();
        if let Some(port) = self.world.port_mut(&port_id) {
            recalculate_prices(port, Some(&mods));
        }
        self.evaluate_narrative(&[]);
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
        let space = ship::resolve_crew_max(ship) - ship.crew;
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

    /// Hunt or forage. `GameSession.hunt` (`session.py` line 1335).
    ///
    /// At sea when the voyage is `at_sea`, otherwise in port. Applies the
    /// yield from [`crate::hunting::hunt`] and copies `captain.day` onto
    /// `world.day`. Markets, wages, and provisions do not tick.
    pub fn hunt(&mut self) -> Result<crate::hunting::HuntResult, SimError> {
        let at_sea = self.world.voyage.status == VoyageStatus::AtSea;
        let location = if at_sea { "sea" } else { "port" };
        if at_sea {
            if let Some(ship) = self.world.captain.ship.as_ref() {
                if ship.morale < 20 {
                    return Err(SimError::Sentence(
                        "Crew morale too low for hunting at sea (need 20+).".into(),
                    ));
                }
            }
        }
        let crew_count = self
            .world
            .captain
            .ship
            .as_ref()
            .map(|ship| ship.crew)
            .unwrap_or(1);
        let result =
            crate::hunting::hunt(&mut self.world.captain, location, crew_count, &mut self.rng);
        self.world.day = self.world.captain.day;
        if result.success {
            if result.provisions_gained > 0 {
                self.world.captain.provisions += result.provisions_gained;
            }
            if result.pelts_gained > 0 {
                if let Some(existing) = self
                    .world
                    .captain
                    .cargo
                    .iter_mut()
                    .find(|item| item.good_id == "pelts")
                {
                    existing.quantity += result.pelts_gained;
                } else {
                    let acquired_port = current_port_id(&self.world).unwrap_or("").to_string();
                    let acquired_day = self.world.captain.day;
                    self.world.captain.cargo.push(CargoItem {
                        good_id: "pelts".to_string(),
                        quantity: result.pelts_gained,
                        cost_basis: 0,
                        acquired_port,
                        acquired_region: String::new(),
                        acquired_day,
                    });
                }
            }
            if result.silver_gained > 0 {
                self.world.captain.silver += result.silver_gained;
            }
        }
        if result.crew_lost > 0 {
            if let Some(ship) = self.world.captain.ship.as_mut() {
                naval::apply_crew_casualties(ship, result.crew_lost, 1);
            }
        }
        if result.hull_damage > 0 {
            if let Some(ship) = self.world.captain.ship.as_mut() {
                ship.hull = 1.max(ship.hull - result.hull_damage);
            }
        }
        if result.morale_cost > 0 {
            if let Some(ship) = self.world.captain.ship.as_mut() {
                ship.morale = 0.max(ship.morale - result.morale_cost);
            }
        }
        Ok(result)
    }

    /// `generate_bounty_board` on the session RNG. Listing does not save the board.
    pub fn bounty_board(&mut self) -> Vec<crate::bounty::BountyTarget> {
        crate::bounty::generate_bounty_board(&self.world.captain_memories, &mut self.rng, 3)
    }

    /// `accept_bounty` (`engine/bounty.py` line 120). The CLI saves after a success.
    pub fn accept_bounty(&mut self, target_id: &str) -> Result<(), SimError> {
        crate::bounty::accept_bounty(&mut self.world.captain, target_id).map_err(SimError::Sentence)
    }

    /// `GameSession.hunt_bounty_cmd` (`session.py` line 1402).
    ///
    /// Spawns the locked encounter, writes `pending_duel`, and keeps the
    /// encounter on the session so a later `encounter` command does not re-roll.
    pub fn hunt_bounty(&mut self, target_id: &str) -> Result<EncounterState, SimError> {
        let enc = crate::bounty::hunt_bounty(&self.world, target_id, &mut self.rng)
            .map_err(SimError::Sentence)?;
        self.world.pending_duel = Some(PendingDuel {
            captain_id: enc.enemy_captain_id.clone(),
            captain_name: enc.enemy_captain_name.clone(),
            faction_id: enc.enemy_faction_id.clone(),
            personality: enc.enemy_personality.clone(),
            strength: enc.enemy_strength,
            region: enc.enemy_region.clone(),
        });
        self.encounter = Some(enc.clone());
        self.player_combat = None;
        self.opponent_combat = None;
        self.pending_victory = false;
        Ok(enc)
    }

    /// `claim_bounty` (`engine/bounty.py` line 165). Pays the table reward.
    pub fn claim_bounty(&mut self, target_id: &str) -> Result<i64, SimError> {
        crate::bounty::claim_bounty(
            &mut self.world.captain,
            &self.world.captain_memories,
            target_id,
        )
        .map_err(SimError::Sentence)
    }

    /// Assign `captain.wanted_level`.
    ///
    /// `GameSession.advance` and `abandon_contract_cmd` omit the captain, so
    /// play never writes this field. The voyage bounty-hunter check still
    /// reads it. The parity script uses this assignment to reach that check.
    pub fn set_wanted_level(&mut self, level: i64) {
        self.world.captain.wanted_level = level;
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
        let outcome_str = if outcome.player_won {
            "duel_win"
        } else if outcome.draw {
            "duel_draw"
        } else {
            "duel_loss"
        };
        self.world.captain.encounters.push(PirateEncounterRecord {
            captain_id: pending.captain_id.clone(),
            faction_id: pending.faction_id.clone(),
            day: self.world.day,
            outcome: outcome_str.to_string(),
            region: pending.region.clone(),
        });
        if outcome.player_won {
            self.world.captain.duels_won += 1;
        } else if !outcome.draw {
            self.world.captain.duels_lost += 1;
        }
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
            .map(ship::resolve_cannons)
            .unwrap_or(0);
        let valid = naval::valid_actions(cannons);
        if !valid.contains(&action.as_str()) {
            return Err(SimError::InvalidAction(valid.join(", ")));
        }
        if action == "flee" {
            return self.naval_flee();
        }
        let round = {
            let ship = self.combat_ship()?;
            let encounter = self.encounter.as_mut().ok_or(SimError::NotInNavalCombat)?;
            encounter::resolve_naval_turn(encounter, &action, &ship, &mut self.rng)
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
            self.finish_personal_fight(&step, player_won, draw);
            if !player_won {
                step.phase = "resolved".into();
            }
        }
        Ok(step)
    }

    /// Mercy after a personal-combat win. Less silver, more underworld standing.
    pub fn spare(&mut self) -> Result<(), SimError> {
        self.finalize_victory(true)
    }

    /// Take the defeated captain's silver and loot.
    pub fn take_all(&mut self) -> Result<(), SimError> {
        self.finalize_victory(false)
    }

    /// Mark fleet hulls at the current port as in transit. `depart` calls this.
    pub fn form_convoy(&mut self) -> Result<(), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?;
        fleet::form_convoy(&mut self.world.captain, &port_id);
        Ok(())
    }

    /// `GameSession.repair` with no amount: restore the flagship's missing hull.
    ///
    /// Python has no escort hull repair. `dry_dock` restores template `hull_max`
    /// and is a different call. Arrival only docks the convoy.
    pub fn repair_fleet(&mut self) -> Result<(i64, i64), SimError> {
        self.repair(None)
    }

    /// `GameSession.repair` (`session.py` line 996). Restores the flagship only.
    ///
    /// `amount` is hull points. `None` restores every missing point, which is
    /// what [`Session::repair_fleet`] calls. A non-positive amount is rejected
    /// only after the "already perfect" check, matching Python's order.
    /// Python's `repair` does not take a ship name. The named-ship yard job is
    /// `dry_dock` (line 1028), already reached through `buy_infrastructure`.
    pub fn repair(&mut self, amount: Option<i64>) -> Result<(i64, i64), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Sentence("Must be docked to repair".into()))?;
        let repair_cost = self
            .world
            .port(&port_id)
            .ok_or_else(|| SimError::Sentence("Must be docked to repair".into()))?
            .repair_cost;
        let ship = self.world.captain.ship.as_ref().ok_or(SimError::NoShip)?;
        let damage = ship.hull_max - ship.hull;
        if damage == 0 {
            return Err(SimError::Sentence(
                "Ship is already in perfect condition".into(),
            ));
        }
        let mut points = if let Some(asked) = amount {
            if asked <= 0 {
                return Err(SimError::QuantityMustBeAPositiveNumber);
            }
            asked.min(damage)
        } else {
            damage
        };
        let mult = reputation::service_modifier(&self.world.captain.standing, &port_id);
        let cost_per = 1.max(py_trunc(repair_cost as f64 * mult));
        let mut cost = points * cost_per;
        if cost > self.world.captain.silver {
            let affordable = if cost_per > 0 {
                self.world.captain.silver / cost_per
            } else {
                0
            };
            if affordable == 0 {
                return Err(SimError::Sentence("Can't afford any repairs".into()));
            }
            points = affordable;
            cost = points * cost_per;
        }
        self.world.captain.silver -= cost;
        let ship = self.world.captain.ship.as_mut().ok_or(SimError::NoShip)?;
        ship.hull += points;
        Ok((points, cost))
    }

    /// `GameSession.rename_ship`. The flagship when `ship_name` is omitted,
    /// otherwise the first fleet hull whose name or template id matches.
    /// The stored name is stripped and capped at 30 characters.
    pub fn rename_ship(&mut self, new_name: &str, ship_name: Option<&str>) -> Result<(), SimError> {
        let trimmed = new_name.trim();
        if trimmed.is_empty() {
            return Err(SimError::Sentence("Name cannot be empty".into()));
        }
        let new_name: String = trimmed.chars().take(30).collect();
        if let Some(ship_name) = ship_name {
            let needle = ship_name.to_lowercase();
            let found = self.world.captain.fleet.iter_mut().find(|owned| {
                owned.ship.name.to_lowercase() == needle
                    || owned.ship.template_id.to_lowercase() == needle
            });
            let Some(owned) = found else {
                return Err(SimError::Sentence(format!(
                    "No ship named '{ship_name}' in fleet"
                )));
            };
            owned.ship.name = new_name;
            return Ok(());
        }
        let ship = self.world.captain.ship.as_mut().ok_or(SimError::NoShip)?;
        ship.name = new_name;
        Ok(())
    }

    /// `GameSession.dock_current_ship`. The flagship joins the fleet at this
    /// port and the first hull already docked here becomes the flagship.
    pub fn dock_current_ship(&mut self) -> Result<(), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?;
        fleet::dock_flagship(&mut self.world.captain, &port_id).map_err(SimError::Sentence)
    }

    /// `GameSession.board_fleet_ship`. Swap the flagship with a hull docked
    /// here, matched by display name or template id.
    pub fn board_fleet_ship(&mut self, ship_name: &str) -> Result<(), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?;
        fleet::board_ship(&mut self.world.captain, ship_name, &port_id).map_err(SimError::Sentence)
    }

    /// `GameSession.sell_fleet_ship`. A shipyard buys a docked hull for
    /// `int(template_price * 0.3 * hull / hull_max)`. Cargo blocks the sale.
    pub fn sell_fleet_ship(&mut self, ship_name: &str) -> Result<(i64, String), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?;
        let port_name = self
            .world
            .port(&port_id)
            .map(|port| port.name.clone())
            .unwrap_or_else(|| port_id.clone());
        let shipyard = self
            .world
            .port(&port_id)
            .is_some_and(|port| port.has_feature("shipyard"));
        if !shipyard {
            return Err(SimError::Sentence(format!("{port_name} has no shipyard")));
        }
        fleet::sell_docked_ship(&mut self.world.captain, ship_name, &port_id)
            .map_err(SimError::Sentence)
    }

    /// `GameSession.fire_crew`. Specialists lose named officers from the end
    /// of the list. Sailors have no officer records.
    pub fn fire_crew(&mut self, count: i64, role: &str) -> Result<(), SimError> {
        if current_port_id(&self.world).is_none() {
            return Err(SimError::Sentence("Must be docked to fire crew".into()));
        }
        let ship = self.world.captain.ship.as_ref().ok_or(SimError::NoShip)?;
        let spec = role_spec(&role.to_lowercase())
            .ok_or_else(|| SimError::Sentence(format!("Unknown role: {role}")))?;
        let current = role_count(ship, spec.role);
        if current <= 0 {
            return Err(SimError::Sentence(format!("No {role}s to fire")));
        }
        let fired = count.min(current);
        let ship = self.world.captain.ship.as_mut().ok_or(SimError::NoShip)?;
        set_role_count(ship, spec.role, current - fired);
        ship.sync_crew();
        if spec.role != "sailor" && fired > 0 {
            let mut to_remove = fired;
            let mut kept = Vec::new();
            for officer in ship.officers.iter().rev() {
                if officer.role == spec.role && to_remove > 0 {
                    to_remove -= 1;
                } else {
                    kept.push(officer.clone());
                }
            }
            kept.reverse();
            ship.officers = kept;
        }
        Ok(())
    }

    /// `GameSession.abandon_contract_cmd`. The engine call does not receive
    /// the captain, so it does not write a breach or raise wanted level.
    /// Trust, standing, and heat stay on the outcome record.
    pub fn abandon_contract(&mut self, offer_id: &str) -> Result<ContractOutcome, SimError> {
        contracts::abandon_contract(&mut self.board, offer_id, self.world.day, None)
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
        self.pending_victory = false;
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
                let ship = self.combat_ship()?;
                let encounter = self.encounter.as_mut().ok_or(SimError::NoActiveEncounter)?;
                encounter::begin_fight(encounter, &ship)
            };
            message.push('\n');
            message.push_str(&fight);
            Ok(self.choice_step("negotiate", false, false, 0, message))
        }
    }

    fn choose_flee(&mut self) -> Result<EncounterStep, SimError> {
        let (escaped, damage, mut message) = {
            let ship = self.combat_ship()?;
            let encounter = self.encounter.as_mut().ok_or(SimError::NoActiveEncounter)?;
            encounter::resolve_flee(encounter, &ship, &mut self.rng)
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
                let ship = self.combat_ship()?;
                let encounter = self.encounter.as_mut().ok_or(SimError::NoActiveEncounter)?;
                encounter::begin_fight(encounter, &ship)
            };
            message.push('\n');
            message.push_str(&fight);
            Ok(self.choice_step("flee", false, false, damage, message))
        }
    }

    fn choose_fight(&mut self) -> Result<EncounterStep, SimError> {
        let message = {
            let ship = self.combat_ship()?;
            let encounter = self.encounter.as_mut().ok_or(SimError::NoActiveEncounter)?;
            encounter::begin_fight(encounter, &ship)
        };
        Ok(self.choice_step("fight", true, false, 0, message))
    }

    fn naval_flee(&mut self) -> Result<EncounterStep, SimError> {
        let (escaped, damage) = {
            let ship = self.combat_ship()?;
            let encounter = self.encounter.as_ref().ok_or(SimError::NotInNavalCombat)?;
            let enemy = encounter::enemy_ship(encounter);
            naval::attempt_flee(&ship, &enemy, &mut self.rng)
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

    /// `cli._clear_encounter` (`cli.py` 1903).
    ///
    /// `self.encounter` is the persisted `encounter_phase` and `encounter_state`
    /// blob. Dropping it leaves phase `""` and state `{}`. `pending_duel` goes
    /// with them, so a finished hunt does not leave a stance duel behind.
    fn clear_encounter(&mut self) {
        self.encounter = None;
        self.player_combat = None;
        self.opponent_combat = None;
        self.pending_victory = false;
        self.world.pending_duel = None;
    }

    fn combat_ship(&self) -> Result<crate::model::Ship, SimError> {
        self.world
            .captain
            .ship
            .as_ref()
            .map(ship::resolved_ship)
            .ok_or(SimError::NoShip)
    }

    fn finish_personal_fight(&mut self, step: &EncounterStep, player_won: bool, draw: bool) {
        if !step.injury.is_empty() {
            let day = self.world.day;
            self.world
                .captain
                .injuries
                .push(injuries::create_injury(&step.injury, day));
        }
        self.degrade_equipped();
        self.sync_combat_ammo();
        if player_won {
            self.pending_victory = true;
        } else if draw {
            self.world.captain.duels_won += 1;
            self.clear_encounter();
        } else {
            let loss = 15 + self.enemy_strength() * 3;
            self.world.captain.silver = 0.max(self.world.captain.silver - loss);
            self.world.captain.duels_lost += 1;
            self.clear_encounter();
        }
    }

    fn degrade_equipped(&mut self) {
        let melee = self
            .world
            .captain
            .melee
            .as_ref()
            .map(|weapon| weapon.id.clone());
        let armor = self
            .world
            .captain
            .armor
            .as_ref()
            .map(|armor| armor.id.clone());
        let bonus = skills::degrade_threshold_bonus(skills::skill_level(
            &self.world.captain.skills,
            "blacksmith",
        ));
        let captain = &mut self.world.captain;
        if let Some(id) = melee {
            let _ = weapon_quality::tick_weapon_degradation(
                &mut captain.weapon_quality,
                &mut captain.weapon_usage,
                &id,
                "melee",
                1,
                bonus,
            );
        }
        if let Some(id) = armor {
            let _ = weapon_quality::tick_weapon_degradation(
                &mut captain.weapon_quality,
                &mut captain.weapon_usage,
                &id,
                "armor",
                1,
                bonus,
            );
        }
    }

    fn sync_combat_ammo(&mut self) {
        let Some(player) = self.player_combat.as_ref() else {
            return;
        };
        let ammo = player.ammo;
        let mechanical = player.mechanical_ammo;
        let throwing_left = player.throwing_weapons;
        if let Some(weapon) = self.world.captain.firearm.as_mut() {
            weapon.ammo = ammo;
        }
        if let Some(weapon) = self.world.captain.mechanical.as_mut() {
            weapon.ammo = mechanical;
        }
        if !self.world.captain.throwing.is_empty() {
            let total: i64 = self
                .world
                .captain
                .throwing
                .iter()
                .map(|weapon| weapon.ammo.max(0))
                .sum();
            let mut spent = total - throwing_left;
            for weapon in &mut self.world.captain.throwing {
                if spent <= 0 {
                    break;
                }
                let take = spent.min(weapon.ammo);
                weapon.ammo -= take;
                spent -= take;
            }
            self.world.captain.throwing.retain(|weapon| weapon.ammo > 0);
        }
    }

    fn finalize_victory(&mut self, spared: bool) -> Result<(), SimError> {
        if !self.pending_victory || self.encounter.is_none() {
            if self.encounter.as_ref().map(|enc| enc.phase.as_str()) == Some("capture_available") {
                return Err(SimError::Sentence(
                    "Prize waiting. Use portlight capture <crew> (or 0 to decline).".into(),
                ));
            }
            return Err(SimError::Sentence(if spared {
                "No defeated opponent to spare. Win a duel first.".into()
            } else {
                "No defeated opponent. Win a duel first.".into()
            }));
        }
        let enc = self.encounter.clone().ok_or(SimError::NoActiveEncounter)?;
        let silver_gain = if spared {
            20 + enc.enemy_strength * 3
        } else {
            20 + enc.enemy_strength * 7
        };
        self.world.captain.silver += silver_gain;
        self.world.captain.duels_won += 1;
        let crew_killed = 0.max(enc.enemy_ship_crew_max - enc.enemy_ship_crew);
        let memory =
            memory::get_or_create_memory(&mut self.world.captain_memories, &enc.enemy_captain_id);
        memory::record_encounter(
            memory,
            self.world.day,
            &enc.enemy_region,
            "player_won",
            spared,
            false,
            crew_killed,
        );
        record_duel_standing(
            &mut self.world.captain.standing,
            &enc.enemy_faction_id,
            true,
            spared,
        );
        if let Some(weapon_id) = self
            .world
            .captain
            .melee
            .as_ref()
            .map(|weapon| weapon.id.clone())
        {
            if self.world.captain.provenance_mut(&weapon_id).is_none() {
                self.world.captain.weapon_provenance.push((
                    weapon_id.clone(),
                    weapon_provenance::create_provenance(&weapon_id, "", "", 0),
                ));
            }
            if let Some(prov) = self.world.captain.provenance_mut(&weapon_id) {
                weapon_provenance::record_kill(
                    prov,
                    Some(&enc.enemy_captain_id),
                    Some(&enc.enemy_captain_name),
                );
            }
        }
        if !spared {
            let drops = loot::roll_loot(
                enc.enemy_strength,
                Some(&enc.enemy_captain_id),
                &mut self.rng,
                2,
            );
            loot::apply_loot(&mut self.world.captain, &drops);
        }
        let trigger = if spared { "spared_enemy" } else { "took_all" };
        companion::apply_morale_trigger(&mut self.world.captain, trigger);
        companion::check_departures(&mut self.world.captain);
        self.clear_encounter();
        Ok(())
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
            player_stamina: 0,
            player_stamina_max: 0,
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
        if let Some(player) = self.player_combat.as_ref() {
            step.player_stamina = player.stamina;
            step.player_stamina_max = player.stamina_max;
        }
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
        self.heal_injuries();
        let sailed = self.world.voyage.status == VoyageStatus::AtSea;
        let mut turn = if !sailed {
            let shocks = economy::tick_markets(&mut self.world.ports, 1, &mut self.rng, 0);
            self.world.day += 1;
            self.world.captain.day += 1;
            if self.world.captain.provisions > 0 {
                self.world.captain.provisions -= 1;
            }
            if let Some(ship) = self.world.captain.ship.as_ref() {
                let wage = wage_bill(ship) + fleet::fleet_daily_wages(&self.world.captain);
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
            let breaches = self.board.breaches.len() as i64;
            let mut events = voyage::advance_day(&mut self.world, &mut self.rng, breaches)?;
            events = crate::sea_culture::enrich_voyage_day(
                &mut self.world,
                events,
                &mut self.rng,
                &self.receipts,
                self.books.total_sells,
                &self.board,
            );
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
                    crate::culture::record_port_visit(&port_id, &region, &mut self.world.culture);
                    if self
                        .world
                        .culture
                        .active_festivals
                        .iter()
                        .any(|fest| fest.port_id == port_id)
                    {
                        self.world.culture.festivals_visited += 1;
                    }
                    let arrival =
                        crate::port_arrival_engine::generate_arrival(&self.world, &port_id);
                    let lines = crate::port_arrival_engine::format_arrival_text(&arrival);
                    if !lines.is_empty() {
                        events.push(VoyageEvent::annotated(
                            voyage::EventType::Nothing,
                            lines.join("\n"),
                            "[arrival]",
                        ));
                    }
                    let found = crate::consequences::check_port_consequences(
                        &self.world,
                        &port_id,
                        &self.receipts,
                        self.books.total_sells,
                        &self.board,
                        &mut self.rng,
                    );
                    for consequence in &found {
                        crate::consequences::apply_consequence(&mut self.world, consequence);
                        events.push(VoyageEvent::annotated(
                            voyage::EventType::Nothing,
                            format!(
                                "{}{}",
                                consequence.text,
                                crate::consequences::port_effect_note(consequence)
                            ),
                            format!("[consequence:{}]", consequence.effect_type),
                        ));
                    }
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
        if sailed {
            let events = turn.events.clone();
            self.evaluate_narrative(&events);
        }
        Ok(turn)
    }

    /// Arrival prose for the docked port, or the voyage destination while at sea.
    ///
    /// Deterministic. Python builds this inside the arrival branch of `advance`
    /// and also exposes the same text through `generate_arrival`.
    pub fn arrival_narrative(&self) -> Vec<String> {
        let port_id =
            current_port_id(&self.world).unwrap_or(self.world.voyage.destination_id.as_str());
        let experience = crate::port_arrival_engine::generate_arrival(&self.world, port_id);
        crate::port_arrival_engine::format_arrival_text(&experience)
    }

    /// One history-gated consequence, applied immediately.
    ///
    /// At sea this is `check_sea_consequences`. In port it is
    /// `check_port_consequences`. `advance` already runs those checks at
    /// Python's call sites, so a second call draws the session RNG again.
    pub fn evaluate_consequences(&mut self) -> Vec<Consequence> {
        if self.world.voyage.status == VoyageStatus::AtSea {
            let found = crate::consequences::check_sea_consequences(
                &self.world,
                &self.receipts,
                self.books.total_sells,
                &self.board,
                &mut self.rng,
            );
            for consequence in &found {
                crate::consequences::apply_consequence(&mut self.world, consequence);
            }
            found
        } else {
            let port_id = current_port_id(&self.world)
                .unwrap_or(self.world.voyage.destination_id.as_str())
                .to_string();
            let found = crate::consequences::check_port_consequences(
                &self.world,
                &port_id,
                &self.receipts,
                self.books.total_sells,
                &self.board,
                &mut self.rng,
            );
            for consequence in &found {
                crate::consequences::apply_consequence(&mut self.world, consequence);
            }
            found
        }
    }

    fn evaluate_narrative(&mut self, events: &[VoyageEvent]) {
        let port_id = current_port_id(&self.world).map(str::to_string);
        crate::narrative::evaluate_narrative(
            &mut self.narrative,
            &self.world,
            &self.board,
            &self.infra,
            &self.books,
            &self.receipts,
            port_id.as_deref(),
            events,
        );
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
        let repair = self
            .world
            .port(&port_id)
            .map(|port| port.repair_cost)
            .unwrap_or(0);
        let service = dry_dock_service(&self.world.captain.standing, &port_id);
        if let Some(name) = ship_name {
            return self.dry_dock_fleet_ship(&port_id, name, repair, service);
        }
        infrastructure::dry_dock(&mut self.world.captain, repair, service)?;
        Ok(())
    }

    /// `GameSession.dry_dock` when a fleet hull is named.
    ///
    /// The ship must be docked at this port. The match is the display name or
    /// the template id, case-insensitive. Cost and the hull restore are
    /// `_do_dry_dock`: template `hull_max` at 5× the repair rate.
    fn dry_dock_fleet_ship(
        &mut self,
        port_id: &str,
        name: &str,
        repair_cost: i64,
        service: f64,
    ) -> Result<(), SimError> {
        let wanted = name.to_lowercase();
        let index = self.world.captain.fleet.iter().position(|owned| {
            owned.docked_port_id == port_id
                && (owned.ship.name.to_lowercase() == wanted
                    || owned.ship.template_id.to_lowercase() == wanted)
        });
        let Some(index) = index else {
            return Err(SimError::Rejected(format!(
                "No ship named '{name}' docked at this port"
            )));
        };
        let (template_id, hull_max, hull) = {
            let ship = &self.world.captain.fleet[index].ship;
            (ship.template_id.clone(), ship.hull_max, ship.hull)
        };
        let Some(template_hull) = content::content()
            .ship(&template_id)
            .map(|ship| ship.hull_max)
        else {
            return Err(SimError::Rejected("Unknown ship template".to_string()));
        };
        let degradation = template_hull - hull_max;
        if degradation <= 0 {
            return Err(SimError::Rejected("Ship hull is not degraded".to_string()));
        }
        let cost_per = 1.max(py_trunc(repair_cost as f64 * service * 5.0));
        let cost = degradation * cost_per;
        if cost > self.world.captain.silver {
            return Err(SimError::Rejected(format!(
                "Need {cost} silver for dry dock ({degradation} points at {cost_per}/point), have {}",
                self.world.captain.silver
            )));
        }
        self.world.captain.silver -= cost;
        let ship = &mut self.world.captain.fleet[index].ship;
        ship.hull_max = template_hull;
        ship.hull = (hull + degradation).min(ship.hull_max);
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

    fn heal_injuries(&mut self) {
        if self.world.captain.injuries.is_empty() {
            return;
        }
        let in_port = self.world.voyage.status != VoyageStatus::AtSea;
        let bay = self.world.captain.ship.as_ref().is_some_and(|ship| {
            ship.upgrades
                .iter()
                .any(|upgrade| upgrade.upgrade_id == "surgeons_bay")
        });
        if !(in_port || bay) {
            return;
        }
        let medicines = self
            .world
            .captain
            .cargo
            .iter()
            .any(|item| item.good_id == "medicines");
        self.world.captain.injuries =
            injuries::heal_injury_tick(&self.world.captain.injuries, 1, true, medicines);
    }

    /// Buy a hull at a shipyard. The old hull joins the fleet until
    /// [`naval::max_fleet_size`] is full, then it sells for 40%.
    pub fn buy_ship(&mut self, ship_id: &str) -> Result<(), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?;
        let port_name = self
            .world
            .port(&port_id)
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?
            .name
            .clone();
        let shipyard = self
            .world
            .port(&port_id)
            .is_some_and(|port| port.has_feature("shipyard"));
        if !shipyard {
            return Err(SimError::Sentence(format!("{port_name} has no shipyard")));
        }
        let template = content::content()
            .ship(ship_id)
            .cloned()
            .ok_or_else(|| SimError::Sentence(format!("Unknown ship: {ship_id}")))?;
        let current_id = self
            .world
            .captain
            .ship
            .as_ref()
            .map(|ship| ship.template_id.clone())
            .ok_or(SimError::NoShip)?;
        if template.id == current_id {
            return Err(SimError::Sentence("You already have this ship".into()));
        }
        if template.price > self.world.captain.silver {
            return Err(SimError::NeedSilver {
                need: template.price,
                have: self.world.captain.silver,
            });
        }
        let trust = self.world.captain.standing.commercial_trust;
        let fleet_limit = naval::max_fleet_size(trust);
        let fleet_count = self.world.captain.fleet.len() as i64 + 1;
        if fleet_count < fleet_limit {
            let old = self.world.captain.ship.take().ok_or(SimError::NoShip)?;
            self.world.captain.fleet.push(crate::model::FleetShip {
                ship: old,
                docked_port_id: port_id,
                cargo: Vec::new(),
            });
        } else if let Some(old) = self.world.captain.ship.as_ref() {
            let price = content::content()
                .ship(&old.template_id)
                .map(|ship| ship.price)
                .unwrap_or(0);
            self.world.captain.silver += py_trunc(price as f64 * 0.4);
        }
        self.world.captain.silver -= template.price;
        self.world.captain.ship = Some(crate::model::Ship::from_template(&template));
        trim_cargo(&mut self.world.captain.cargo, template.cargo_capacity);
        Ok(())
    }

    pub fn install_upgrade(&mut self, upgrade_id: &str) -> Result<(), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?;
        let port_name = self
            .world
            .port(&port_id)
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?
            .name
            .clone();
        if !self
            .world
            .port(&port_id)
            .is_some_and(|port| port.has_feature("shipyard"))
        {
            return Err(SimError::Sentence(format!("{port_name} has no shipyard")));
        }
        let template = content::content()
            .upgrade(upgrade_id)
            .cloned()
            .ok_or_else(|| SimError::Sentence(format!("Unknown upgrade: {upgrade_id}")))?;
        let ship = self.world.captain.ship.as_ref().ok_or(SimError::NoShip)?;
        if ship.upgrades.len() as i64 >= ship.upgrade_slots {
            return Err(SimError::Sentence(format!(
                "No upgrade slots remaining ({}/{} used)",
                ship.upgrade_slots, ship.upgrade_slots
            )));
        }
        if template.price > self.world.captain.silver {
            return Err(SimError::NeedSilver {
                need: template.price,
                have: self.world.captain.silver,
            });
        }
        self.world.captain.silver -= template.price;
        let day = self.world.day;
        self.world
            .captain
            .ship
            .as_mut()
            .ok_or(SimError::NoShip)?
            .upgrades
            .push(InstalledUpgrade {
                upgrade_id: upgrade_id.to_string(),
                installed_day: day,
            });
        Ok(())
    }

    pub fn remove_upgrade(&mut self, upgrade_id: &str) -> Result<(), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?;
        let port_name = self
            .world
            .port(&port_id)
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?
            .name
            .clone();
        if !self
            .world
            .port(&port_id)
            .is_some_and(|port| port.has_feature("shipyard"))
        {
            return Err(SimError::Sentence(format!("{port_name} has no shipyard")));
        }
        let ship = self.world.captain.ship.as_mut().ok_or(SimError::NoShip)?;
        if let Some(index) = ship
            .upgrades
            .iter()
            .position(|upgrade| upgrade.upgrade_id == upgrade_id)
        {
            ship.upgrades.remove(index);
            return Ok(());
        }
        Err(SimError::Sentence(format!(
            "Upgrade not installed: {upgrade_id}"
        )))
    }

    pub fn transfer_cargo(
        &mut self,
        good_id: &str,
        qty: i64,
        from_ship: &str,
        to_ship: &str,
    ) -> Result<(), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?;
        fleet::transfer_cargo(
            &mut self.world.captain,
            good_id,
            qty,
            from_ship,
            to_ship,
            &port_id,
        )
        .map_err(SimError::Sentence)
    }

    /// Buy one item from a merchant at the current port (`buy_from_merchant`).
    ///
    /// The price is catalog cost times that merchant's markup. Purchase does
    /// not write provenance or a quality tier. Python creates provenance on a
    /// named kill, and combat treats a missing tier as standard.
    pub fn buy_gear(&mut self, gear_id: &str) -> Result<(), SimError> {
        let port_id = current_port_id(&self.world)
            .map(str::to_string)
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?;
        let region = self
            .world
            .port(&port_id)
            .map(|port| port.region.clone())
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?;
        let merchants = content::content().merchants_at(&port_id);
        if merchants.is_empty() {
            return Err(SimError::Sentence("Unknown merchant".into()));
        }
        let stocked = merchants.iter().find(|merchant| {
            merchant_inventory(merchant, &region)
                .iter()
                .any(|item| item.item_id == gear_id)
        });
        let merchant = stocked.copied().unwrap_or(merchants[0]);
        let inventory = merchant_inventory(merchant, &region);
        let Some(entry) = inventory.into_iter().find(|item| item.item_id == gear_id) else {
            return Err(SimError::Sentence(format!(
                "{} doesn't sell {gear_id}",
                merchant.name
            )));
        };
        let total = entry.silver_cost;
        if total > self.world.captain.silver {
            return Err(SimError::NeedSilver {
                need: total,
                have: self.world.captain.silver,
            });
        }
        self.world.captain.silver -= total;
        apply_purchased_item(&mut self.world.captain, &entry.item_type, gear_id, 1);
        Ok(())
    }

    /// `portlight maintain`. The blacksmith discount is applied here, as in
    /// the CLI, not inside `weapon_quality.maintain_weapon`.
    pub fn maintain_weapon(&mut self, weapon_id: &str) -> Result<(), SimError> {
        let _port = current_port_id(&self.world)
            .ok_or_else(|| SimError::Sentence("Must be docked".into()))?;
        let base = weapon_quality::maintenance_cost(&self.world.captain.weapon_quality, weapon_id);
        let level = skills::skill_level(&self.world.captain.skills, "blacksmith");
        let cost = skills::apply_maintenance_discount(base, level);
        if self.world.captain.silver < cost {
            return Err(SimError::Sentence(format!(
                "Maintenance costs {cost} silver. You have {}.",
                self.world.captain.silver
            )));
        }
        self.world.captain.silver -= cost;
        self.world.captain.set_usage(weapon_id, 0);
        Ok(())
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

struct ShopEntry {
    item_type: String,
    item_id: String,
    silver_cost: i64,
}

/// `merchant._markup`: `max(1, round(base * markup))`.
fn merchant_price(base: i64, markup: f64) -> i64 {
    1.max(py_round(base as f64 * markup))
}

fn merchant_sells(types: &[String], kind: &str) -> bool {
    types.iter().any(|item| item == kind)
}

fn in_region(regions: &[String], region: &str) -> bool {
    regions.iter().any(|item| item == region)
}

/// `get_merchant_inventory` for the port's region.
fn merchant_inventory(merchant: &content::MerchantDef, region: &str) -> Vec<ShopEntry> {
    let catalog = content::content();
    let mut items = Vec::new();
    if merchant_sells(&merchant.inventory_types, "melee") {
        for weapon in catalog
            .melee_weapons
            .iter()
            .filter(|weapon| in_region(&weapon.available_regions, region))
        {
            items.push(ShopEntry {
                item_type: "melee".into(),
                item_id: weapon.id.clone(),
                silver_cost: merchant_price(weapon.silver_cost, merchant.price_markup),
            });
        }
    }
    if merchant_sells(&merchant.inventory_types, "armor") {
        for armor in catalog
            .armor
            .iter()
            .filter(|armor| in_region(&armor.available_regions, region))
        {
            items.push(ShopEntry {
                item_type: "armor".into(),
                item_id: armor.id.clone(),
                silver_cost: merchant_price(armor.silver_cost, merchant.price_markup),
            });
        }
    }
    if merchant_sells(&merchant.inventory_types, "ranged") {
        for weapon in catalog
            .ranged_weapons
            .iter()
            .filter(|weapon| in_region(&weapon.available_regions, region))
        {
            items.push(ShopEntry {
                item_type: "ranged".into(),
                item_id: weapon.id.clone(),
                silver_cost: merchant_price(weapon.silver_cost, merchant.price_markup),
            });
        }
    }
    if merchant_sells(&merchant.inventory_types, "ammo") {
        for ammo in catalog
            .ammo
            .iter()
            .filter(|ammo| in_region(&ammo.available_regions, region))
        {
            items.push(ShopEntry {
                item_type: "ammo".into(),
                item_id: ammo.id.clone(),
                silver_cost: merchant_price(ammo.silver_cost, merchant.price_markup),
            });
        }
    }
    items
}

/// `merchant._apply_item`. Does not stamp quality or provenance.
fn apply_purchased_item(
    captain: &mut crate::model::Captain,
    item_type: &str,
    item_id: &str,
    qty: i64,
) {
    let catalog = content::content();
    match item_type {
        "melee" => {
            let Some(weapon) = catalog.melee_weapon(item_id) else {
                return;
            };
            captain.melee = Some(Weapon {
                id: weapon.id.clone(),
                name: weapon.name.clone(),
                kind: "melee".into(),
                quality: "standard".into(),
                ammo: 0,
            });
        }
        "armor" => {
            let Some(armor) = catalog.armor(item_id) else {
                return;
            };
            captain.armor = Some(Armor {
                id: armor.id.clone(),
                name: armor.name.clone(),
                armor_type: armor.armor_type.clone(),
                damage_reduction: armor.damage_reduction,
                dodge_penalty: armor.dodge_penalty,
                stamina_penalty: armor.stamina_penalty,
                quality: "standard".into(),
            });
        }
        "ranged" => {
            let Some(weapon) = catalog.ranged_weapon(item_id) else {
                return;
            };
            let carried = Weapon {
                id: weapon.id.clone(),
                name: weapon.name.clone(),
                kind: weapon.weapon_type.clone(),
                quality: "standard".into(),
                ammo: 0,
            };
            match weapon.weapon_type.as_str() {
                "firearm" => captain.firearm = Some(carried),
                "mechanical" => captain.mechanical = Some(carried),
                "thrown" => {
                    if let Some(existing) =
                        captain.throwing.iter_mut().find(|item| item.id == item_id)
                    {
                        existing.ammo += qty;
                    } else {
                        let mut carried = carried;
                        carried.ammo = qty;
                        captain.throwing.push(carried);
                    }
                }
                _ => {}
            }
        }
        "ammo" => {
            let Some(ammo) = catalog.ammo(item_id) else {
                return;
            };
            let gained = ammo.quantity * qty;
            match ammo.weapon_type.as_str() {
                "firearm" => {
                    if let Some(weapon) = captain.firearm.as_mut() {
                        weapon.ammo += gained;
                    }
                }
                "mechanical" => {
                    if let Some(weapon) = captain.mechanical.as_mut() {
                        weapon.ammo += gained;
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn record_duel_standing(
    standing: &mut crate::model::Standing,
    faction_id: &str,
    player_won: bool,
    spared: bool,
) -> i64 {
    let delta = if player_won {
        if spared {
            5
        } else {
            2
        }
    } else {
        -2
    };
    if let Some(slot) = standing
        .underworld
        .iter_mut()
        .find(|(id, _)| id == faction_id)
    {
        slot.1 = (slot.1 + delta).clamp(0, 100);
    } else {
        standing
            .underworld
            .push((faction_id.to_string(), delta.clamp(0, 100)));
    }
    delta
}

fn trim_cargo(cargo: &mut Vec<crate::model::CargoItem>, mut capacity: i64) {
    if capacity < 0 {
        capacity = 0;
    }
    while !cargo.is_empty() && cargo.iter().map(|item| item.quantity).sum::<i64>() > capacity {
        let last_qty = cargo.last().map(|item| item.quantity).unwrap_or(0);
        if last_qty <= 0 {
            cargo.pop();
            continue;
        }
        let overflow = cargo.iter().map(|item| item.quantity).sum::<i64>() - capacity;
        let drop = last_qty.min(overflow);
        let last = cargo.last_mut().unwrap();
        if drop <= 0 {
            break;
        }
        let original_qty = last.quantity;
        last.quantity -= drop;
        if original_qty > 0 && last.cost_basis != 0 {
            last.cost_basis =
                py_trunc(last.cost_basis as f64 * last.quantity as f64 / original_qty as f64);
        }
        if last.quantity <= 0 {
            cargo.pop();
        }
    }
}

fn record_receipt(session: &mut Session, receipt: &TradeReceipt) {
    session.trade_seq += 1;
    session
        .books
        .note_receipt(receipt.action, receipt.total_price);
    session.receipts.push(receipt.clone());
}

/// `GameSession._pricing` (`app/session.py` lines 540–543).
///
/// Reads [`custom_captain::captain_template`], so a missing custom template
/// and an unknown captain type both price as the merchant.
fn pricing(world: &World) -> &PricingDef {
    &custom_captain::captain_template(world).pricing
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
    let mods = pricing(world).clone();
    for port in &mut world.ports {
        recalculate_prices(port, Some(&mods));
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

    fn price_rows(port: &crate::model::Port) -> Vec<(String, i64, i64)> {
        port.market
            .iter()
            .map(|slot| (slot.good_id.clone(), slot.buy_price, slot.sell_price))
            .collect()
    }

    fn assert_market(port: &crate::model::Port, mods: Option<&PricingDef>) {
        let mut expected = port.clone();
        recalculate_prices(&mut expected, mods);
        assert_eq!(price_rows(port), price_rows(&expected));
    }

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

    /// `GameSession.captain_template` (`app/session.py` lines 488–496) falls
    /// back to `CAPTAIN_TEMPLATES[MERCHANT]` on `KeyError` or `ValueError`.
    /// `_pricing` (lines 540–543) is what `buy`, `sell`, and `_recalc` read,
    /// including the reprice in `load` (line 536). A custom captain after a
    /// fresh load has no registered template, so those three trade at merchant
    /// prices. Voyage lookups do not.
    #[test]
    fn loaded_custom_captain_buys_and_sells_at_merchant_prices() {
        let spec = CustomCaptainSpec {
            trade_points: 7,
            sailing_points: 1,
            shadow_points: 1,
            reputation_points: 1,
            ..CustomCaptainSpec::default()
        };
        let mut session = Session::new_custom(&spec, 7, None).unwrap();
        let custom = custom_captain::build_custom_template(&spec);
        let merchant = content::content()
            .captain("merchant")
            .unwrap()
            .pricing
            .clone();
        assert_eq!(
            pricing(&session.world).buy_price_mult,
            custom.pricing.buy_price_mult
        );
        assert_ne!(custom.pricing.buy_price_mult, merchant.buy_price_mult);

        let port_id = session.world.voyage.destination_id.clone();
        session.buy("grain", 1).unwrap();
        let dock = session.world.port(&port_id).unwrap();
        assert_market(dock, Some(&custom.pricing));
        let custom_rows = price_rows(dock);
        let custom_quote = dock.slot("grain").unwrap().buy_price;
        let mut merchant_view = dock.clone();
        recalculate_prices(&mut merchant_view, Some(&merchant));
        assert_ne!(custom_rows, price_rows(&merchant_view));
        let second = session.buy("grain", 1).unwrap();
        assert_eq!(second.unit_price, custom_quote);

        let dir =
            std::env::temp_dir().join(format!("portlight-custom-pricing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        session.save(&dir, "default").unwrap();
        let mut loaded = Session::load(&dir, "default").unwrap().unwrap();
        let _ = std::fs::remove_dir_all(&dir);

        assert_eq!(loaded.world.captain.captain_type, "custom");
        assert!(loaded.world.custom_captain.is_none());
        assert!(custom_captain::active_captain(&loaded.world).is_none());
        assert_eq!(
            custom_captain::captain_template(&loaded.world).id,
            "merchant"
        );
        assert_eq!(
            pricing(&loaded.world).buy_price_mult,
            merchant.buy_price_mult
        );
        assert_eq!(
            pricing(&loaded.world).sell_price_mult,
            merchant.sell_price_mult
        );
        for port in &loaded.world.ports {
            assert_market(port, Some(&merchant));
        }
        let home = loaded.world.port(&port_id).unwrap().clone();
        let mut still_custom = home.clone();
        recalculate_prices(&mut still_custom, Some(&custom.pricing));
        assert_ne!(price_rows(&home), price_rows(&still_custom));

        let grain_buy = home.slot("grain").unwrap().buy_price;
        let bought = loaded.buy("grain", 1).unwrap();
        assert_eq!(bought.unit_price, grain_buy);
        assert_market(loaded.world.port(&port_id).unwrap(), Some(&merchant));

        let grain_sell = loaded
            .world
            .port(&port_id)
            .unwrap()
            .slot("grain")
            .unwrap()
            .sell_price;
        let sold = loaded.sell("grain", 1).unwrap();
        assert_eq!(sold.receipt.unit_price, grain_sell);
        assert_market(loaded.world.port(&port_id).unwrap(), Some(&merchant));

        loaded.world.captain.captain_type = "not_a_captain".to_string();
        loaded.world.custom_captain = Some(custom.to_captain_def());
        assert_eq!(
            pricing(&loaded.world).buy_price_mult,
            merchant.buy_price_mult
        );
        assert!(custom_captain::active_captain(&loaded.world).is_none());
        if let Some(port) = loaded.world.port_mut(&port_id) {
            for slot in &mut port.market {
                slot.buy_price = 7;
                slot.sell_price = 7;
            }
        }
        loaded.buy("grain", 1).unwrap();
        assert_market(loaded.world.port(&port_id).unwrap(), Some(&merchant));
        let mut unmodified = loaded.world.port(&port_id).unwrap().clone();
        recalculate_prices(&mut unmodified, None);
        assert_ne!(
            price_rows(loaded.world.port(&port_id).unwrap()),
            price_rows(&unmodified)
        );
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
        // The fourth sea day is when the ship arrives, and the third day damages
        // 3 grain. Buy the loss up front so 23 still reaches Corsairs Rest.
        session.buy("grain", 26).unwrap();
        session.depart("corsairs_rest").unwrap();
        // Sea-culture draws sit between voyage days, so the third day is a storm
        // and arrival is the fourth sea day. Python seed 1 matches that.
        for _ in 0..4 {
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
    fn sea_culture_shifts_the_butcher_duel_onto_the_second_day() {
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
        let first = session.advance().unwrap();
        assert_eq!(session.world.day, 2);
        assert_eq!(session.world.captain.silver, 544);
        assert!(first
            .events
            .iter()
            .any(|event| event.event_type == voyage::EventType::Storm));
        let (encounter, ambush, _) = session.tick_sea_captain_agency();
        assert!(encounter.is_none() && !ambush);
        let second = session.advance().unwrap();
        assert_eq!(session.world.day, 3);
        assert_eq!(session.world.captain.silver, 541);
        assert!(second.events.iter().any(|event| {
            event.event_type == voyage::EventType::Pirates && event.message.contains("The Butcher")
        }));
        let (encounter, ambush, notices) = session.tick_sea_captain_agency();
        assert!(encounter.is_none() && !ambush && notices.is_empty());
        let pending = session.world.pending_duel.as_ref().unwrap();
        assert_eq!(pending.captain_id, "the_butcher");
        assert_eq!(pending.faction_id, "crimson_tide");
        assert_eq!(pending.personality, "aggressive");
        assert_eq!(pending.strength, 8);
        assert_eq!(pending.region, "Mediterranean");
        let progress = session.world.voyage.progress;
        let _frozen = session.advance().unwrap();
        assert_eq!(session.world.day, 3);
        assert_eq!(session.world.voyage.progress, progress);
        assert!(session.encounter.is_none());
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

    #[test]
    fn named_dry_dock_restores_only_the_docked_fleet_ship() {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        session.buy_ship("swift_cutter").unwrap();
        {
            let escort = &mut session.world.captain.fleet[0].ship;
            escort.hull_max -= 3;
            escort.hull = escort.hull.min(escort.hull_max);
        }
        let flag_before = session.world.captain.ship.as_ref().unwrap().hull_max;
        let before = session.world.captain.silver;
        session
            .buy_infrastructure("dry_dock", &["Coastal Sloop"])
            .unwrap();
        let escort = &session.world.captain.fleet[0].ship;
        assert_eq!(escort.template_id, "coastal_sloop");
        assert_eq!(escort.hull_max, 60);
        assert_eq!(escort.hull, 60);
        let repair = session.world.port("porto_novo").unwrap().repair_cost;
        let cost_per = 1.max(py_trunc(repair as f64 * 5.0));
        assert_eq!(session.world.captain.silver, before - 3 * cost_per);
        assert_eq!(
            session.world.captain.ship.as_ref().unwrap().hull_max,
            flag_before
        );
        session.world.captain.fleet[0].docked_port_id = "silva_bay".to_string();
        let err = session
            .buy_infrastructure("dry_dock", &["coastal_sloop"])
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "No ship named 'coastal_sloop' docked at this port"
        );
    }

    #[test]
    fn blacksmith_maintenance_uses_the_discounted_cost() {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        session.world.captain.skills.push(crate::model::Skill {
            id: "blacksmith".to_string(),
            level: 1,
        });
        session.buy_gear("cutlass").unwrap();
        session.world.captain.set_usage("cutlass", 4);
        let before = session.world.captain.silver;
        session.maintain_weapon("cutlass").unwrap();
        // Level 1 discount is 0.25. int(15 * 0.75) = 11.
        assert_eq!(session.world.captain.silver, before - 11);
        assert_eq!(session.world.captain.usage_of("cutlass"), 0);
    }

    #[test]
    fn rename_strips_caps_and_finds_a_fleet_hull() {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        session
            .rename_ship("  The Very Long Name Of This Fine Ship Indeed  ", None)
            .unwrap();
        assert_eq!(
            session.world.captain.ship.as_ref().unwrap().name,
            "The Very Long Name Of This Fin"
        );
        assert_eq!(
            session.rename_ship("   ", None).unwrap_err().to_string(),
            "Name cannot be empty"
        );
        session.buy_ship("swift_cutter").unwrap();
        session
            .rename_ship("Holdfast", Some("COASTAL_SLOOP"))
            .unwrap();
        assert_eq!(session.world.captain.fleet[0].ship.name, "Holdfast");
        assert_eq!(
            session
                .rename_ship("X", Some("Ghost"))
                .unwrap_err()
                .to_string(),
            "No ship named 'Ghost' in fleet"
        );
    }

    #[test]
    fn dock_board_and_sell_match_python_sentences() {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        assert_eq!(
            session.dock_current_ship().unwrap_err().to_string(),
            "No other ship at this port to switch to"
        );
        session.buy_ship("swift_cutter").unwrap();
        session.world.voyage.destination_id = "al_manar".into();
        assert_eq!(
            session
                .sell_fleet_ship("Coastal Sloop")
                .unwrap_err()
                .to_string(),
            "Al-Manar has no shipyard"
        );
        session.world.voyage.destination_id = "porto_novo".into();
        session.board_fleet_ship("coastal_sloop").unwrap();
        assert_eq!(
            session.world.captain.ship.as_ref().unwrap().name,
            "Coastal Sloop"
        );
        assert_eq!(
            session.board_fleet_ship("ghost").unwrap_err().to_string(),
            "No ship named 'ghost' docked at this port"
        );
        session.dock_current_ship().unwrap();
        session.world.captain.cargo.push(crate::model::CargoItem {
            good_id: "grain".into(),
            quantity: 1,
            cost_basis: 8,
            acquired_port: "porto_novo".into(),
            acquired_region: "Mediterranean".into(),
            acquired_day: 1,
        });
        session.dock_current_ship().unwrap();
        assert_eq!(
            session
                .sell_fleet_ship("Swift Cutter")
                .unwrap_err()
                .to_string(),
            "Ship has cargo — transfer it first"
        );
        session.world.captain.fleet[0].cargo.clear();
        session.world.captain.fleet[0].ship.hull = 10;
        let before = session.world.captain.silver;
        let (silver, name) = session.sell_fleet_ship("swift cutter").unwrap();
        assert_eq!(name, "Swift Cutter");
        // int(450 * 0.3 * (10 / 70)) == 19
        assert_eq!(silver, 19);
        assert_eq!(session.world.captain.silver, before + 19);
        assert_eq!(
            session
                .sell_fleet_ship("Swift Cutter")
                .unwrap_err()
                .to_string(),
            "No ship named 'Swift Cutter' docked at this port"
        );
        session.depart("silva_bay").unwrap();
        assert_eq!(
            session.dock_current_ship().unwrap_err().to_string(),
            "Must be docked"
        );
    }

    #[test]
    fn fire_drops_the_last_specialist_and_refuses_unknown_roles() {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        session.hire_crew(2, "gunner").unwrap();
        let names: Vec<_> = session
            .world
            .captain
            .ship
            .as_ref()
            .unwrap()
            .officers
            .iter()
            .map(|officer| officer.name.clone())
            .collect();
        session.fire_crew(1, "gunner").unwrap();
        let ship = session.world.captain.ship.as_ref().unwrap();
        assert_eq!(ship.gunners, 1);
        assert_eq!(ship.officers.len(), 1);
        assert_eq!(ship.officers[0].name, names[0]);
        assert_eq!(
            session.fire_crew(1, "Wizard").unwrap_err().to_string(),
            "Unknown role: Wizard"
        );
        session.fire_crew(1, "gunner").unwrap();
        assert_eq!(
            session.fire_crew(1, "gunner").unwrap_err().to_string(),
            "No gunners to fire"
        );
        session.depart("silva_bay").unwrap();
        assert_eq!(
            session.fire_crew(1, "sailor").unwrap_err().to_string(),
            "Must be docked to fire crew"
        );
    }

    #[test]
    fn abandon_records_the_outcome_and_does_not_breach() {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let trust = session.world.captain.standing.commercial_trust;
        session.accept_contract("71773aae754b").unwrap();
        let outcome = session.abandon_contract("71773aae754b").unwrap();
        assert_eq!(outcome.outcome_type, "abandoned");
        assert_eq!(outcome.trust_delta, -2);
        assert_eq!(outcome.standing_delta, -1);
        assert_eq!(outcome.heat_delta, 1);
        assert_eq!(outcome.silver_delta, 0);
        assert_eq!(
            outcome.summary,
            "Abandoned contract: Famine relief: grain to Corsair's Rest"
        );
        assert!(session.board.active.is_empty());
        assert_eq!(session.board.completed.len(), 1);
        assert_eq!(session.world.captain.standing.commercial_trust, trust);
        assert_eq!(session.world.captain.wanted_level, 0);
        assert!(session.books().completed_contracts.is_empty());
        assert_eq!(
            session
                .abandon_contract("71773aae754b")
                .unwrap_err()
                .to_string(),
            "No active contract with that ID"
        );
    }

    #[test]
    fn repair_amount_follows_python_order_and_silver() {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        assert_eq!(
            session.repair(Some(1)).unwrap_err().to_string(),
            "Ship is already in perfect condition"
        );
        let ship = session.world.captain.ship.as_mut().unwrap();
        ship.hull -= 5;
        assert_eq!(
            session.repair(Some(0)).unwrap_err().to_string(),
            "Quantity must be a positive number."
        );
        session.world.captain.silver = 0;
        assert_eq!(
            session.repair(None).unwrap_err().to_string(),
            "Can't afford any repairs"
        );
        // Porto Novo repair_cost is 2 and standing is below 5, so 2 silver per point.
        session.world.captain.silver = 2;
        assert_eq!(session.repair(None).unwrap(), (1, 2));
        assert_eq!(session.world.captain.ship.as_ref().unwrap().hull, 56);
        session.world.captain.silver = 20;
        assert_eq!(session.repair(Some(100)).unwrap(), (4, 8));
        assert_eq!(session.world.captain.ship.as_ref().unwrap().hull, 60);
        session.depart("silva_bay").unwrap();
        session.world.captain.ship.as_mut().unwrap().hull -= 1;
        assert_eq!(
            session.repair(Some(1)).unwrap_err().to_string(),
            "Must be docked to repair"
        );
    }

    #[test]
    fn finished_bounty_fight_clears_pending_duel_without_a_stance_duel() {
        let mut session = Session::new("Ada", "privateer", 4, None).unwrap();
        let start_silver = session.world.captain.silver;
        session.accept_bounty("raj_the_quiet").unwrap();
        session.hunt_bounty("raj_the_quiet").unwrap();
        assert_eq!(
            session.world.pending_duel.as_ref().unwrap().captain_id,
            "raj_the_quiet"
        );
        assert_eq!(session.world.captain.silver, start_silver);

        session.encounter_choice("fight").unwrap();
        for action in ["broadside", "broadside", "rake", "evade", "close", "close"] {
            session.naval_round(action).unwrap();
        }
        session.resolve_boarding().unwrap();
        let mut won = false;
        for action in [
            "thrust", "slash", "parry", "dodge", "thrust", "thrust", "thrust", "thrust",
        ] {
            won = session.fight(action).unwrap().player_won;
        }
        assert!(won);
        assert!(session.pending_victory);
        assert!(session.world.pending_duel.is_some());
        let before_outcome = session.world.captain.silver;
        assert_eq!(before_outcome, start_silver);

        session.take_all().unwrap();
        assert!(session.world.pending_duel.is_none());
        assert!(session.encounter.is_none());
        assert!(!session.pending_victory);
        assert_eq!(session.world.captain.duels_won, 1);
        assert_eq!(session.world.captain.duels_lost, 0);
        assert!(session
            .world
            .captain
            .encounters
            .iter()
            .all(|record| !record.outcome.starts_with("duel")));
        let victory = 20 + 5 * 7;
        let after_take = session.world.captain.silver;
        assert!(after_take >= before_outcome + victory);
        assert_eq!(session.claim_bounty("raj_the_quiet").unwrap(), 120);
        assert_eq!(session.world.captain.silver, after_take + 120);

        let dir =
            std::env::temp_dir().join(format!("portlight-bounty-clear-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        session.save(&dir, "bounty").unwrap();
        let saved: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("saves").join("bounty.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(saved["pirate_state"]["encounter_phase"], "");
        assert_eq!(
            saved["pirate_state"]["encounter_state"],
            serde_json::json!({})
        );
        assert!(saved["pirate_state"].get("pending_duel").is_none());
        let _ = std::fs::remove_dir_all(&dir);

        let claimed = session.world.captain.silver;
        let crew_cost = session.world.port("stormwall").unwrap().crew_cost;
        session.hire_crew(4, "sailor").unwrap();
        assert_eq!(session.world.captain.silver, claimed - crew_cost * 4);
        let port_fee = session.world.port("stormwall").unwrap().port_fee;
        let fee_mult = content::content()
            .captain("privateer")
            .unwrap()
            .pricing
            .port_fee_mult;
        let fee = 1.max(py_trunc(port_fee as f64 * fee_mult));
        session.depart("thornport").unwrap();
        assert_eq!(session.world.voyage.status, VoyageStatus::AtSea);
        assert_eq!(session.world.captain.silver, claimed - crew_cost * 4 - fee);
        assert!(session.world.pending_duel.is_none());
        assert!(session.encounter.is_none());
        let sailed = session.world.captain.silver;
        let err = session
            .resolve_pending_duel()
            .err()
            .expect("no stance duel remains")
            .to_string();
        assert!(err.contains("No pirate has challenged"));
        assert_eq!(session.world.captain.silver, sailed);
        assert_eq!(session.world.captain.duels_won, 1);
        assert!(session
            .world
            .captain
            .encounters
            .iter()
            .all(|record| !record.outcome.starts_with("duel")));
    }

    #[test]
    fn list_saves_reads_a_v12_slot_without_rewriting_it() {
        let dir = std::env::temp_dir().join(format!("portlight-list-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        let day = session.world.day;
        let silver = session.world.captain.silver;
        session.save(&dir, "voyage").unwrap();
        let path = dir.join("saves").join("voyage.json");
        let before = std::fs::read(&path).unwrap();
        assert!(std::str::from_utf8(&before)
            .unwrap()
            .contains("\"version\": 12"));
        let slots = Session::list_saves(&dir);
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].slot, "voyage");
        assert_eq!(slots[0].captain, "Ada");
        assert_eq!(slots[0].day, day);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let loaded = Session::load(&dir, "voyage").unwrap().unwrap();
        assert_eq!(loaded.world.captain.name, "Ada");
        assert_eq!(loaded.world.captain.silver, silver);
        assert_eq!(loaded.world.day, day);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
