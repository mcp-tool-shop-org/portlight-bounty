//! Type balance sweep: a deterministic Rust bot plays every starting
//! captain type with three strategies over a fixed seed set to day 60, and
//! reports day-60 net worth per type against the field median.
//!
//! Off by default (`#[ignore]`). Run it with:
//!
//! ```text
//! cargo test --release -p portlight-sim --test balance_bot -- --ignored
//! ```
//!
//! Output goes to `$PT_S0R_OUT` (default `target/pt-s0r`): `pt-s0r.csv` and
//! `pt-s0r-summary.md`. The committed copies live in `docs/playtest/balance/`.
//! The bot only calls public `Session` verbs; it never
//! edits game state directly. Every choice is a pure function of visible
//! state, ties broken by the lowest id; the only randomness is the session's
//! own seeded RNG.

use std::fmt::Write as _;
use std::path::PathBuf;

use portlight_sim::content;
use portlight_sim::model::VoyageStatus;
use portlight_sim::session::Session;
use portlight_sim::voyage::LaneSuitability;

/// Fixed seed set: 1001 to 1030 inclusive.
const SEEDS: [i128; 30] = [
    1001, 1002, 1003, 1004, 1005, 1006, 1007, 1008, 1009, 1010, 1011, 1012, 1013, 1014, 1015, 1016,
    1017, 1018, 1019, 1020, 1021, 1022, 1023, 1024, 1025, 1026, 1027, 1028, 1029, 1030,
];
const NAME: &str = "Balance";
const END_DAY: i64 = 60;
/// Silver an unarmed ship must hold to hunt: a quarter of a 4000 silver stake,
/// so a thin purse trades instead.
const HUNT_RESERVE: i64 = 1000;
/// A run is a bust when its final net worth is under this many silver.
const BUST_BELOW: i64 = 200;
/// Band for the gate: +/-15 percent of the field median.
const BAND_PCT: i64 = 15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Strategy {
    /// S1 cautious trader.
    Cautious,
    /// S2 contract runner.
    Contracts,
    /// S3 opportunist.
    Opportunist,
}

impl Strategy {
    const ALL: [Strategy; 3] = [
        Strategy::Cautious,
        Strategy::Contracts,
        Strategy::Opportunist,
    ];

    fn id(self) -> &'static str {
        match self {
            Strategy::Cautious => "s1_cautious",
            Strategy::Contracts => "s2_contracts",
            Strategy::Opportunist => "s3_opportunist",
        }
    }
}

/// Net worth in silver, integers only, split into its five parts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Worth {
    silver: i64,
    banked: i64,
    cargo: i64,
    ships: i64,
    debt: i64,
}

impl Worth {
    fn total(&self) -> i64 {
        self.silver + self.banked + self.cargo + self.ships - self.debt
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Run {
    captain_type: String,
    strategy: Strategy,
    seed: i128,
    net_worth: i64,
    duels_won: i64,
    days_at_sea: i64,
    contracts_completed: i64,
    bounties_claimed: i64,
    game_over: bool,
    worth: Worth,
}

fn ship_value(ship: &portlight_sim::model::Ship) -> i64 {
    // Same formula as a docked-ship sale (fleet.rs): trunc(price x 0.3 x hull / max(1, hull_max)).
    let price = content::content()
        .ship(&ship.template_id)
        .map(|def| def.price)
        .unwrap_or(0);
    let hull_max = ship.hull_max.max(1);
    (price as f64 * 0.3 * (ship.hull as f64 / hull_max as f64)).trunc() as i64
}

fn worth(session: &Session) -> Worth {
    let captain = &session.world().captain;
    let infra = session.infrastructure();
    // cost_basis is per stack (sales cut it in proportion to the quantity
    // taken), so it is summed once per lot, not multiplied by quantity.
    let cargo = captain.cargo.iter().map(|lot| lot.cost_basis).sum();
    let mut ships = captain.ship.as_ref().map(ship_value).unwrap_or(0);
    ships += captain
        .fleet
        .iter()
        .map(|f| ship_value(&f.ship))
        .sum::<i64>();
    let debt = infra
        .credit
        .as_ref()
        .map(|c| c.outstanding + c.interest_accrued)
        .unwrap_or(0);
    Worth {
        silver: captain.silver,
        banked: infra.banked_total(),
        cargo,
        ships,
        debt,
    }
}

fn docked_port(session: &Session) -> Option<String> {
    let world = session.world();
    (world.voyage.status != VoyageStatus::AtSea).then(|| world.voyage.destination_id.clone())
}

fn held(session: &Session) -> i64 {
    session
        .world()
        .captain
        .cargo
        .iter()
        .map(|c| c.quantity)
        .sum()
}

fn capacity(session: &Session) -> i64 {
    session
        .world()
        .captain
        .ship
        .as_ref()
        .map(|s| s.cargo_capacity)
        .unwrap_or(0)
}

/// Own strength as crew plus guns.
fn own_strength(session: &Session) -> i64 {
    session
        .world()
        .captain
        .ship
        .as_ref()
        .map(|s| s.crew + s.cannons)
        .unwrap_or(0)
}

fn slot<'a>(
    session: &'a Session,
    port: &str,
    good: &str,
) -> Option<&'a portlight_sim::model::MarketSlot> {
    session
        .world()
        .port(port)
        .and_then(|p| p.market.iter().find(|s| s.good_id == good))
}

/// Open lanes (not WARNING/BLOCKED) from the docked port, sorted by id.
fn open_lanes(session: &Session) -> Vec<portlight_sim::voyage::SailLane> {
    let mut lanes: Vec<_> = session
        .sail_lanes()
        .into_iter()
        .filter(|lane| lane.suitability == LaneSuitability::Ok)
        .collect();
    lanes.sort_by(|a, b| a.destination_id.cmp(&b.destination_id));
    lanes
}

/// Best trade from `here`: (margin, good, destination), ties to the lowest ids.
fn best_trade(session: &Session, here: &str) -> Option<(i64, String, String)> {
    let port = session.world().port(here)?;
    let mut goods: Vec<_> = port
        .market
        .iter()
        .filter(|s| s.buy_price > 0 && s.stock_current > 0)
        .collect();
    goods.sort_by(|a, b| a.good_id.cmp(&b.good_id));
    let mut best: Option<(i64, String, String)> = None;
    for lane in open_lanes(session) {
        for good in &goods {
            let Some(there) = slot(session, &lane.destination_id, &good.good_id) else {
                continue;
            };
            let margin = there.sell_price - good.buy_price;
            if best.as_ref().is_none_or(|(m, _, _)| margin > *m) {
                best = Some((margin, good.good_id.clone(), lane.destination_id.clone()));
            }
        }
    }
    best
}

/// Deliver: at a contract's destination, sell the held contract good (the
/// sale settles the contract), whatever the margin.
fn deliver_contracts(session: &mut Session, here: &str, stats: &mut Stats) {
    let mut goods: Vec<String> = session
        .board()
        .active
        .iter()
        .filter(|a| a.destination_port_id == here)
        .map(|a| a.good_id.clone())
        .collect();
    goods.sort();
    goods.dedup();
    for good in goods {
        let qty: i64 = session
            .world()
            .captain
            .cargo
            .iter()
            .filter(|c| c.good_id == good)
            .map(|c| c.quantity)
            .sum();
        if qty > 0 {
            if let Ok(sale) = session.sell(&good, qty) {
                stats.contracts += count_completed(&sale.contracts);
            }
        }
    }
}

/// Goods reserved for active contracts are not sold for margin.
fn reserved(session: &Session) -> Vec<String> {
    session
        .board()
        .active
        .iter()
        .map(|a| a.good_id.clone())
        .collect()
}

/// Sell every lot whose sale beats its cost basis.
fn sell_winners(session: &mut Session, here: &str, stats: &mut Stats) {
    let keep = reserved(session);
    let mut lots: Vec<(String, i64, i64)> = session
        .world()
        .captain
        .cargo
        .iter()
        .map(|c| (c.good_id.clone(), c.quantity, c.cost_basis))
        .collect();
    lots.sort();
    for (good, qty, basis) in lots {
        let Some(price) = slot(session, here, &good).map(|s| s.sell_price) else {
            continue;
        };
        if qty > 0 && price * qty > basis && !keep.contains(&good) {
            if let Ok(sale) = session.sell(&good, qty) {
                stats.contracts += count_completed(&sale.contracts);
            }
        }
    }
}

fn count_completed(outcomes: &[portlight_sim::model::ContractOutcome]) -> i64 {
    outcomes
        .iter()
        .filter(|o| o.outcome_type.starts_with("completed"))
        .count() as i64
}

fn buy_for(session: &mut Session, here: &str, good: &str, cap_qty: i64) {
    let Some(s) = slot(session, here, good) else {
        return;
    };
    let price = s.buy_price.max(1);
    let stock = s.stock_current;
    let budget = session.world().captain.silver * 6 / 10;
    let space = (capacity(session) - held(session)).max(0);
    let qty = (budget / price).min(space).min(stock).min(cap_qty);
    if qty > 0 {
        let _ = session.buy(good, qty);
    }
}

fn prepare_and_sail(session: &mut Session, dest: &str) -> bool {
    let need = session
        .sail_lanes()
        .into_iter()
        .find(|l| l.destination_id == dest)
        .map(|l| l.provisions_needed)
        .unwrap_or(5);
    let have = session.world().captain.provisions;
    if have < need {
        let _ = session.provision(need - have);
    }
    session.depart(dest).is_ok()
}

fn maybe_repair(session: &mut Session) {
    let low = session
        .world()
        .captain
        .ship
        .as_ref()
        .is_some_and(|s| s.hull * 10 < s.hull_max * 7);
    if low {
        let _ = session.repair(None);
    }
}

#[derive(Default)]
struct Stats {
    days_at_sea: i64,
    contracts: i64,
}

/// Next hop toward `target` over all routes (BFS by distance, ties by id),
/// restricted to open lanes for the first hop.
fn next_hop(session: &Session, here: &str, target: &str) -> Option<(String, i64)> {
    let world = session.world();
    let mut ids: Vec<String> = world.ports.iter().map(|p| p.id.clone()).collect();
    ids.sort();
    let idx = |id: &str| ids.iter().position(|x| x == id);
    let n = ids.len();
    let (src, dst) = (idx(here)?, idx(target)?);
    let mut dist = vec![i64::MAX; n];
    let mut prev = vec![usize::MAX; n];
    let mut done = vec![false; n];
    dist[dst] = 0;
    // Dijkstra from the target, so prev points one hop closer to it.
    for _ in 0..n {
        let mut u = usize::MAX;
        for i in 0..n {
            if !done[i] && dist[i] != i64::MAX && (u == usize::MAX || dist[i] < dist[u]) {
                u = i;
            }
        }
        if u == usize::MAX {
            break;
        }
        done[u] = true;
        for route in &world.routes {
            let (a, b) = (idx(&route.port_a), idx(&route.port_b));
            let (Some(a), Some(b)) = (a, b) else { continue };
            let v = if a == u {
                b
            } else if b == u {
                a
            } else {
                continue;
            };
            let d = dist[u] + route.distance;
            if d < dist[v] {
                dist[v] = d;
                prev[v] = u;
            }
        }
    }
    if dist[src] == i64::MAX || src == dst {
        return None;
    }
    let hop = ids[prev[src]].clone();
    open_lanes(session)
        .iter()
        .any(|l| l.destination_id == hop)
        .then_some((hop, dist[src]))
}

/// S2: take the best-paying feasible offer (reward per travel day).
fn take_contract(session: &mut Session, here: &str) {
    if session.board().active.len() >= 2 {
        return;
    }
    let offers = session.available_contracts();
    let speed = session
        .world()
        .captain
        .ship
        .as_ref()
        .map(|s| s.speed)
        .unwrap_or(5.0)
        .max(0.1);
    let day = session.world().day;
    let space = (capacity(session) - held(session)).max(0);
    let silver = session.world().captain.silver;
    let mut best: Option<(i64, String)> = None;
    let mut sorted = offers.clone();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    for offer in &sorted {
        let Some((_, total)) = next_hop(session, here, &offer.destination_port_id) else {
            continue;
        };
        let days = ((total as f64) / speed).ceil().max(1.0) as i64;
        if offer.deadline_day < day + days || offer.quantity > space {
            continue;
        }
        let Some(s) = slot(session, here, &offer.good_id) else {
            continue;
        };
        if s.stock_current < offer.quantity || s.buy_price * offer.quantity > silver {
            continue;
        }
        let per_day = (offer.reward_silver + offer.bonus_reward) / days;
        if best.as_ref().is_none_or(|(b, _)| per_day > *b) {
            best = Some((per_day, offer.id.clone()));
        }
    }
    if let Some((_, id)) = best {
        if let Ok(active) = session.accept_contract(&id) {
            let qty = active.required_quantity - active.delivered_quantity;
            let have: i64 = session
                .world()
                .captain
                .cargo
                .iter()
                .filter(|c| c.good_id == active.good_id)
                .map(|c| c.quantity)
                .sum();
            if qty > have {
                let _ = session.buy(&active.good_id, qty - have);
            }
        }
    }
}

/// S3: the bounty to hunt now: the held one (retried while unclaimed), else
/// accept the weakest target whose strength is at most the board's median
/// pirate strength (ties by id). The caller already required hull >= 70%.
fn try_bounty(session: &mut Session) -> Option<String> {
    let captain = &session.world().captain;
    if let Some(held) = captain
        .active_bounties
        .iter()
        .find(|id| !captain.claimed_bounties.contains(id))
    {
        return Some(held.clone());
    }
    // Take only a target no stronger than the board's median pirate, on
    // the content strength scale (the caller already required hull >= 70%).
    let mut board = session.bounty_board();
    let strength = |id: &str| content::content().pirate(id).map(|p| p.strength);
    let mut all: Vec<i64> = board
        .iter()
        .filter_map(|t| strength(&t.captain_id))
        .collect();
    if all.is_empty() {
        return None;
    }
    all.sort();
    let cap = all[(all.len() - 1) / 2];
    board.retain(|t| strength(&t.captain_id).is_some_and(|s| s <= cap));
    board.sort_by_key(|t| {
        (
            content::content().pirate(&t.captain_id).map(|p| p.strength),
            t.captain_id.clone(),
        )
    });
    let target = board.into_iter().next()?;
    session.accept_bounty(&target.captain_id).ok()?;
    Some(target.captain_id)
}

const CORE: [&str; 3] = ["thrust", "slash", "parry"];

/// Opponent stance weights for the core moves, by personality, as the
/// sim's opponent picks them (combat.rs), with the balanced counter bump.
fn opponent_weights(personality: &str, my_last: Option<&str>) -> [f64; 3] {
    let mut w = match personality {
        "aggressive" => [0.40, 0.30, 0.10],
        "defensive" => [0.15, 0.15, 0.35],
        "wild" => [0.20, 0.20, 0.15],
        _ => [0.25, 0.25, 0.20],
    };
    if personality != "aggressive" && personality != "defensive" && personality != "wild" {
        match my_last {
            Some("thrust") => w[2] += 0.25,
            Some("slash") => w[0] += 0.25,
            Some("parry") => w[1] += 0.25,
            _ => {}
        }
    }
    w
}

/// The core stance with the best expected outcome (+1 win, -1 loss)
/// against those weights; ties go to the earlier stance in CORE.
fn best_stance(personality: &str, my_last: Option<&str>) -> &'static str {
    let w = opponent_weights(personality, my_last);
    // thrust beats slash, slash beats parry, parry beats thrust.
    let score = |i: usize| -> f64 {
        let beaten = (i + 1) % 3;
        let beats_me = (i + 2) % 3;
        w[beaten] - w[beats_me]
    };
    let mut best = 0;
    for i in 1..3 {
        if score(i) > score(best) {
            best = i;
        }
    }
    CORE[best]
}

/// Drive one encounter to its end. `fight` picks fight over flee.
fn run_encounter(session: &mut Session, start_phase: &str, captain_id: Option<&str>, fight: bool) {
    let mut phase = start_phase.to_string();
    let personality = captain_id
        .and_then(|id| content::content().pirate(id))
        .map(|p| p.personality.clone())
        .unwrap_or_default();
    let mut last: Option<&'static str> = None;
    for _ in 0..300 {
        let step = match phase.as_str() {
            "approach" => session.encounter_choice_with(
                if fight { "fight" } else { "flee" },
                captain_id,
                None,
            ),
            "naval" => session.naval_round(if fight { "broadside" } else { "flee" }),
            "boarding" => session.resolve_boarding(),
            "duel" => {
                let pick = best_stance(&personality, last);
                let mut result = session.fight(pick);
                last = Some(pick);
                for alt in CORE {
                    if result.is_ok() {
                        break;
                    }
                    result = session.fight(alt);
                    last = Some(alt);
                }
                result
            }
            _ => break,
        };
        match step {
            Ok(step) => {
                if step.phase.is_empty() || step.phase == "resolved" || step.escaped {
                    if step.player_won || step.enemy_sunk {
                        let _ = session.take_all();
                    }
                    break;
                }
                phase = step.phase;
            }
            Err(_) => break,
        }
    }
    // A won fight waits for the prize call; without a win this is refused.
    if fight {
        let _ = session.take_all();
    }
    if session.world().pending_duel.is_some() {
        let _ = session.resolve_pending_duel();
    }
}

fn fight_choice(session: &Session, strategy: Strategy, enemy: i64) -> bool {
    strategy == Strategy::Opportunist && own_strength(session) * 10 >= enemy * 12
}

fn play(captain_type: &str, strategy: Strategy, seed: i128) -> Run {
    let mut session = Session::new(NAME, captain_type, seed, None).expect("new session");
    let mut stats = Stats::default();
    let mut game_over = false;
    let mut guard = 0;
    // S3 hunts at most 6 times per run, so a hard target cannot eat the run.
    let mut hunted = 0;
    while session.world().day < END_DAY && guard < 2000 {
        guard += 1;
        if session.world().captain.ship.is_none() {
            game_over = true;
            break;
        }
        // A route duel or a held encounter is settled before anything else.
        if let Some(duel) = session.world().pending_duel.clone() {
            let fight = fight_choice(&session, strategy, duel.strength);
            run_encounter(&mut session, "approach", Some(&duel.captain_id), fight);
            if session.world().pending_duel.is_some() {
                game_over = true;
                break;
            }
            continue;
        }
        if let Some(here) = docked_port(&session) {
            maybe_repair(&mut session);
            deliver_contracts(&mut session, &here, &mut stats);
            sell_winners(&mut session, &here, &mut stats);
            let healthy = session
                .world()
                .captain
                .ship
                .as_ref()
                .is_some_and(|ship| ship.hull * 10 >= ship.hull_max * 7);
            // Hunt guard: hunt only with cannons aboard, or while silver is at
            // least HUNT_RESERVE (a 25% reserve against a 4000 stake), so
            // sloop types trade instead of busting.
            let armed = session
                .world()
                .captain
                .ship
                .as_ref()
                .is_some_and(|ship| ship.cannons > 0);
            let reserve = session.world().captain.silver >= HUNT_RESERVE;
            if strategy == Strategy::Opportunist && healthy && (armed || reserve) && hunted < 6 {
                if let Some(target) = try_bounty(&mut session) {
                    hunted += 1;
                    if let Ok(state) = session.hunt_bounty(&target) {
                        // Hull was >= 70% and the target's strength is at most
                        // the board's median (accept rule), so the hunt is
                        // fought, not fled.
                        run_encounter(&mut session, &state.phase, Some(&target), true);
                        let _ = session.claim_bounty(&target);
                    }
                    continue;
                }
            }
            let mut dest = None;
            if strategy == Strategy::Contracts {
                take_contract(&mut session, &here);
                let target = session
                    .board()
                    .active
                    .first()
                    .map(|a| a.destination_port_id.clone());
                if let Some(target) = target {
                    dest = next_hop(&session, &here, &target).map(|(hop, _)| hop);
                }
            }
            if dest.is_none() {
                match best_trade(&session, &here) {
                    Some((margin, good, to)) if margin > 0 => {
                        let need = session
                            .sail_lanes()
                            .into_iter()
                            .find(|l| l.destination_id == to)
                            .map(|l| l.provisions_needed)
                            .unwrap_or(5);
                        let have = session.world().captain.provisions;
                        if have < need {
                            let _ = session.provision(need - have);
                        }
                        buy_for(&mut session, &here, &good, i64::MAX);
                        dest = Some(to);
                    }
                    _ => {
                        dest = open_lanes(&session)
                            .into_iter()
                            .min_by(|a, b| {
                                a.distance
                                    .cmp(&b.distance)
                                    .then(a.destination_id.cmp(&b.destination_id))
                            })
                            .map(|l| l.destination_id);
                    }
                }
            }
            if let Some(dest) = dest {
                if prepare_and_sail(&mut session, &dest) {
                    continue;
                }
            }
            // Could not sail: wait a day in port.
            let before = session.world().day;
            if session.advance().is_err() || session.world().day == before {
                game_over = session.world().captain.ship.is_none();
                break;
            }
            continue;
        }
        // At sea: one day, then the sea-captain agency as the game asks it.
        let before = session.world().day;
        stats.days_at_sea += 1;
        match session.advance() {
            Ok(turn) => {
                for outcome in &turn.contracts {
                    if outcome.outcome_type.starts_with("completed") {
                        stats.contracts += 1;
                    }
                }
            }
            Err(_) => {
                game_over = session.world().captain.ship.is_none();
                break;
            }
        }
        if session.world().day != before && session.world().voyage.status == VoyageStatus::AtSea {
            let (encounter, _ambush, _notes) = session.tick_sea_captain_agency();
            if let Some(state) = encounter {
                let fight = fight_choice(&session, strategy, state.enemy_strength);
                run_encounter(&mut session, &state.phase, None, fight);
            }
        }
    }
    let w = worth(&session);
    let captain = &session.world().captain;
    Run {
        captain_type: captain_type.to_string(),
        strategy,
        seed,
        net_worth: w.total().max(0),
        duels_won: captain.duels_won,
        days_at_sea: stats.days_at_sea,
        contracts_completed: stats.contracts,
        bounties_claimed: captain.claimed_bounties.len() as i64,
        game_over: game_over || captain.ship.is_none(),
        worth: w,
    }
}

fn types() -> Vec<String> {
    Session::starting_captains()
        .into_iter()
        .map(|c| c.id)
        .collect()
}

fn sweep(seeds: &[i128]) -> Vec<Run> {
    let mut runs = Vec::new();
    for t in types() {
        for strategy in Strategy::ALL {
            for &seed in seeds {
                runs.push(play(&t, strategy, seed));
            }
        }
    }
    runs
}

/// Integer median (mean of the two middle values, rounded down, when even).
fn median(mut v: Vec<i64>) -> i64 {
    if v.is_empty() {
        return 0;
    }
    v.sort();
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]).div_euclid(2)
    }
}

fn csv(runs: &[Run]) -> String {
    let mut out = String::from(
        "type,strategy,seed,net_worth,duels_won,days_at_sea,contracts_completed,bounties_claimed,game_over,bust,silver,banked,cargo,ships,debt\n",
    );
    for r in runs {
        let _ = writeln!(
            out,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.captain_type,
            r.strategy.id(),
            r.seed,
            r.net_worth,
            r.duels_won,
            r.days_at_sea,
            r.contracts_completed,
            r.bounties_claimed,
            if r.game_over { "y" } else { "n" },
            if r.net_worth < BUST_BELOW { "y" } else { "n" },
            r.worth.silver,
            r.worth.banked,
            r.worth.cargo,
            r.worth.ships,
            r.worth.debt
        );
    }
    out
}

fn pct(value: i64, base: i64) -> String {
    if base == 0 {
        return "n/a".into();
    }
    let tenths = (value - base) * 1000 / base;
    format!("{:+}.{}%", tenths / 10, (tenths % 10).abs())
}

fn summary(runs: &[Run], seeds: &[i128]) -> String {
    let types = types();
    let mut scores = Vec::new();
    let mut per_strategy: Vec<Vec<i64>> = Vec::new();
    let mut winners: Vec<String> = Vec::new();
    for t in &types {
        let mut best_of = Vec::new();
        let mut wins = [0usize; 3];
        for &seed in seeds {
            let mut best = (i64::MIN, 0usize);
            for (i, s) in Strategy::ALL.iter().enumerate() {
                let nw = runs
                    .iter()
                    .find(|r| &r.captain_type == t && r.strategy == *s && r.seed == seed)
                    .map(|r| r.net_worth)
                    .unwrap_or(0);
                if nw > best.0 {
                    best = (nw, i);
                }
            }
            best_of.push(best.0);
            wins[best.1] += 1;
        }
        scores.push(median(best_of));
        let top = (0..3)
            .max_by(|a, b| wins[*a].cmp(&wins[*b]).then(b.cmp(a)))
            .unwrap();
        winners.push(Strategy::ALL[top].id().to_string());
        per_strategy.push(
            Strategy::ALL
                .iter()
                .map(|s| {
                    median(
                        runs.iter()
                            .filter(|r| &r.captain_type == t && r.strategy == *s)
                            .map(|r| r.net_worth)
                            .collect(),
                    )
                })
                .collect(),
        );
    }
    let field = median(scores.clone());
    let mut md = String::new();
    let _ = writeln!(md, "# Type balance sweep (day 60)\n");
    let _ = writeln!(
        md,
        "Seeds {}..{} ({} seeds), captain \"{NAME}\", 9 types x 3 strategies. Score = median over seeds of the best of the 3 strategies. Gate: within +/-{BAND_PCT}% of the field median.\n",
        seeds.first().unwrap(),
        seeds.last().unwrap(),
        seeds.len()
    );
    let _ = writeln!(md, "Field median: **{field}**\n");
    let _ = writeln!(
        md,
        "| Type | Score | vs field | Gate | Winning strategy | Spread |"
    );
    let _ = writeln!(md, "|---|---:|---:|---|---|---:|");
    for (i, t) in types.iter().enumerate() {
        let s = scores[i];
        let pass = field > 0 && (s - field).abs() * 100 <= field * BAND_PCT;
        let ps = &per_strategy[i];
        let spread = ps.iter().max().unwrap() - ps.iter().min().unwrap();
        let _ = writeln!(
            md,
            "| {t} | {s} | {} | {} | {} | {} |",
            pct(s, field),
            if pass { "PASS" } else { "FLAG" },
            winners[i],
            if field > 0 {
                format!("{}%", spread * 100 / field)
            } else {
                "n/a".into()
            }
        );
    }
    let _ = writeln!(md, "\n## Per-strategy medians\n");
    let _ = writeln!(md, "| Type | s1_cautious | s2_contracts | s3_opportunist |");
    let _ = writeln!(md, "|---|---:|---:|---:|");
    for (i, t) in types.iter().enumerate() {
        let ps = &per_strategy[i];
        let _ = writeln!(md, "| {t} | {} | {} | {} |", ps[0], ps[1], ps[2]);
    }
    let rate = |s: Strategy, f: fn(&Run) -> bool| {
        let all: Vec<_> = runs.iter().filter(|r| r.strategy == s).collect();
        (all.iter().filter(|r| f(r)).count(), all.len())
    };
    let (c, cn) = rate(Strategy::Contracts, |r| r.contracts_completed > 0);
    let (b, bn) = rate(Strategy::Opportunist, |r| r.bounties_claimed > 0);
    let _ = writeln!(md, "\n## Completion rates\n");
    let _ = writeln!(
        md,
        "- s2_contracts: a contract completed on {c}/{cn} runs (gate: at least 98%)."
    );
    let _ = writeln!(
        md,
        "- s3_opportunist: a bounty claimed on {b}/{bn} runs (reported, not gated). The opportunist hunts only if armed or holding at least {HUNT_RESERVE} silver."
    );
    let _ = writeln!(md, "\n## Game over by strategy\n");
    let _ = writeln!(
        md,
        "| Strategy | Game over | Busts (net worth < {BUST_BELOW}) | Runs | Broken bot (>20%) |"
    );
    let _ = writeln!(md, "|---|---:|---:|---:|---|");
    for s in Strategy::ALL {
        let all: Vec<_> = runs.iter().filter(|r| r.strategy == s).collect();
        let over = all.iter().filter(|r| r.game_over).count();
        let busts = all.iter().filter(|r| r.net_worth < BUST_BELOW).count();
        let _ = writeln!(
            md,
            "| {} | {over} | {busts} | {} | {} |",
            s.id(),
            all.len(),
            if over * 5 > all.len() { "yes" } else { "no" }
        );
    }
    md
}

fn out_dir() -> PathBuf {
    std::env::var_os("PT_S0R_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/pt-s0r"))
}

#[test]
#[ignore = "balance sweep; run with --ignored in its own CI step"]
fn balance_sweep_day_60() {
    let runs = sweep(&SEEDS);
    assert_eq!(runs.len(), 9 * 3 * SEEDS.len());
    // Gate: S2 completes a contract on at least 98% of its runs.
    let s2: Vec<_> = runs
        .iter()
        .filter(|r| r.strategy == Strategy::Contracts)
        .collect();
    let done = s2.iter().filter(|r| r.contracts_completed > 0).count();
    assert!(
        done * 100 >= s2.len() * 98,
        "contracts completed on {done}/{} S2 runs (< 98%)",
        s2.len()
    );
    let dir = out_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("pt-s0r.csv"), csv(&runs)).unwrap();
    let md = summary(&runs, &SEEDS);
    std::fs::write(dir.join("pt-s0r-summary.md"), &md).unwrap();
    println!("{md}");
}

#[test]
#[ignore = "balance sweep determinism; run with --ignored"]
fn balance_sweep_is_deterministic() {
    let seeds = &SEEDS[..3];
    assert_eq!(csv(&sweep(seeds)), csv(&sweep(seeds)));
}
