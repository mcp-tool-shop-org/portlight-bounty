//! Python JSON save, version 12, from `engine/save.py`.
//!
//! The file is that JSON object. This module writes the state the sim already
//! has — world, ledger, house books, and the contract board — and runs the
//! v1–v12 migrations. `trade_seq` is the number of ledger receipts. The
//! MT19937 state is not stored; [`crate::session::Session::load`] reseeds with
//! `seed + day`. Breach rows live on the board in memory and are written as
//! `captain.breach_records`, which is where Python stores them.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::campaign::{
    ActiveLicense, BrokerSite, CompletedContract, CreditBook, HouseBooks, VictoryRecord,
    WarehouseSite,
};
use crate::combat::CombatantState;
use crate::content::{self, REGIONS};
use crate::economy::TradeReceipt;
use crate::encounter::{self, EncounterState};
use crate::error::SimError;
use crate::model::{
    ActiveContract, BreachRecord, Captain, CargoItem, Contract, ContractBoard, ContractOutcome,
    DeferredFee, Incident, MarketSlot, Officer, PendingDuel, PirateEncounterRecord, Port, Route,
    Ship, Standing, Voyage, VoyageStatus, World,
};

pub const SAVE_DIR: &str = "saves";
pub const SAVE_FILE: &str = "portlight_save.json";
pub const DEFAULT_SLOT: &str = "default";
pub const CURRENT_SAVE_VERSION: i64 = 12;

const PORT_FEATURES: [&str; 3] = ["shipyard", "black_market", "safe_harbor"];
const CREW_ROLES: [&str; 6] = [
    "sailor",
    "gunner",
    "navigator",
    "surgeon",
    "marine",
    "quartermaster",
];
const CONTRACT_FAMILIES: [&str; 7] = [
    "procurement",
    "shortage",
    "luxury_discreet",
    "return_freight",
    "circuit",
    "reputation_charter",
    "smuggling",
];
const BROKER_TIERS: [&str; 3] = ["none", "local", "established"];
const POLICY_FAMILIES: [&str; 3] = ["hull", "premium_cargo", "contract_guarantee"];
const POLICY_SCOPES: [&str; 3] = ["next_voyage", "active_cargo", "named_contract"];

#[derive(Debug)]
enum MigrateError {
    Version(SimError),
    Bad,
}

/// Active encounter passed into [`write_save`]. Absent fields are an idle sea.
pub(crate) struct LiveEncounter<'a> {
    pub encounter: Option<&'a EncounterState>,
    pub player: Option<&'a CombatantState>,
    pub opponent: Option<&'a CombatantState>,
}

/// A slot `load_game` would turn back into a session.
#[derive(Debug)]
pub(crate) struct LoadedGame {
    pub world: World,
    pub receipts: Vec<TradeReceipt>,
    pub run_id: String,
    pub books: HouseBooks,
    pub board: ContractBoard,
    pub encounter: Option<EncounterState>,
    pub player_combat: Option<CombatantState>,
    pub opponent_combat: Option<CombatantState>,
}

/// Filename for a slot. Characters outside letters, digits, `-`, and `_` are
/// dropped. An empty result is `default.json`.
pub fn save_filename(slot: &str) -> String {
    let safe: String = slot
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    let safe = if safe.is_empty() { DEFAULT_SLOT } else { &safe };
    format!("{safe}.json")
}

/// Bring a save object up to version 12. Already-current files are unchanged.
pub fn migrate_save(data: &mut Value) -> Result<(), SimError> {
    migrate_inner(data).map_err(|err| match err {
        MigrateError::Version(err) => err,
        MigrateError::Bad => SimError::SaveCorrupt,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn write_save(
    base: &Path,
    slot: &str,
    world: &World,
    receipts: &[TradeReceipt],
    run_id: &str,
    books: &HouseBooks,
    board: &ContractBoard,
    live: LiveEncounter<'_>,
) -> Result<PathBuf, SimError> {
    let dir = base.join(SAVE_DIR);
    fs::create_dir_all(&dir).map_err(io_err)?;
    let path = dir.join(save_filename(slot));
    let value = encode(world, receipts, run_id, books, board, live);
    let text =
        serde_json::to_string_pretty(&value).map_err(|err| SimError::SaveIo(err.to_string()))?;
    fs::write(&path, text).map_err(io_err)?;
    Ok(path)
}

pub(crate) fn read_save(base: &Path, slot: &str) -> Result<Option<LoadedGame>, SimError> {
    let dir = base.join(SAVE_DIR);
    let path = dir.join(save_filename(slot));
    if !path.exists() && slot == DEFAULT_SLOT {
        let legacy = dir.join(SAVE_FILE);
        if legacy.exists() {
            fs::rename(&legacy, &path).map_err(io_err)?;
        }
    }
    if !path.exists() {
        return Ok(None);
    }
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => return Err(io_err(err)),
    };
    let mut data: Value = match serde_json::from_str(&text) {
        Ok(data) => data,
        Err(_) => return Ok(None),
    };
    match migrate_inner(&mut data) {
        Ok(()) => {}
        Err(MigrateError::Version(err)) => return Err(err),
        Err(MigrateError::Bad) => return Ok(None),
    }
    Ok(decode(&data))
}

fn io_err(err: std::io::Error) -> SimError {
    SimError::SaveIo(err.to_string())
}

fn migrate_inner(data: &mut Value) -> Result<(), MigrateError> {
    let version = read_version(data)?;
    if version == CURRENT_SAVE_VERSION {
        return Ok(());
    }
    if version > CURRENT_SAVE_VERSION {
        return Err(MigrateError::Version(SimError::SaveVersion {
            found: version,
            supported: CURRENT_SAVE_VERSION,
        }));
    }
    let mut version = version;
    while version < CURRENT_SAVE_VERSION {
        let map = data.as_object_mut().ok_or(MigrateError::Bad)?;
        version = match version {
            1 => {
                migrate_v1_to_v2(map);
                2
            }
            2 => migrate_v2_to_v3(map)?,
            3 => {
                migrate_v3_to_v4(map);
                4
            }
            4 => migrate_v4_to_v5(map)?,
            5 => migrate_v5_to_v6(map)?,
            6 => migrate_v6_to_v7(map)?,
            7 => migrate_v7_to_v8(map)?,
            8 => migrate_v8_to_v9(map)?,
            9 => migrate_v9_to_v10(map)?,
            10 => migrate_v10_to_v11(map)?,
            11 => migrate_v11_to_v12(map)?,
            other => {
                return Err(MigrateError::Version(SimError::SaveMigration {
                    found: other,
                    supported: CURRENT_SAVE_VERSION,
                }));
            }
        };
    }
    if version != CURRENT_SAVE_VERSION {
        return Err(MigrateError::Version(SimError::SaveMigration {
            found: version,
            supported: CURRENT_SAVE_VERSION,
        }));
    }
    Ok(())
}

fn read_version(data: &Value) -> Result<i64, MigrateError> {
    let Some(map) = data.as_object() else {
        return Err(MigrateError::Bad);
    };
    match map.get("version") {
        None => Ok(1),
        Some(value) => json_i64(value).ok_or(MigrateError::Bad),
    }
}

fn migrate_v1_to_v2(data: &mut Map<String, Value>) {
    setdefault(
        data,
        "campaign",
        json_obj(&[
            ("completed", Value::Array(Vec::new())),
            ("completed_paths", Value::Array(Vec::new())),
        ]),
    );
    setdefault(
        data,
        "infrastructure",
        json_obj(&[
            ("warehouses", Value::Array(Vec::new())),
            ("brokers", Value::Array(Vec::new())),
            ("licenses", Value::Array(Vec::new())),
            ("policies", Value::Array(Vec::new())),
            ("claims", Value::Array(Vec::new())),
        ]),
    );
    setdefault(
        data,
        "contract_board",
        json_obj(&[
            ("offers", Value::Array(Vec::new())),
            ("active", Value::Array(Vec::new())),
            ("completed", Value::Array(Vec::new())),
            ("last_refresh_day", Value::from(0)),
            ("max_offers", Value::from(5)),
        ]),
    );
    setdefault(
        data,
        "ledger",
        json_obj(&[
            ("run_id", Value::from("")),
            ("receipts", Value::Array(Vec::new())),
            ("total_buys", Value::from(0)),
            ("total_sells", Value::from(0)),
            ("net_profit", Value::from(0)),
        ]),
    );
    data.insert("version".to_string(), Value::from(2));
}

fn migrate_v2_to_v3(data: &mut Map<String, Value>) -> Result<i64, MigrateError> {
    let mut captain = object_clone(data.get("captain"))?;
    let mut standing = object_clone(captain.get("standing"))?;
    let mut regional = object_clone(standing.get("regional_standing"))?;
    setdefault(&mut regional, "North Atlantic", Value::from(0));
    setdefault(&mut regional, "South Seas", Value::from(0));
    standing.insert("regional_standing".to_string(), Value::Object(regional));
    let mut heat = object_clone(standing.get("customs_heat"))?;
    setdefault(&mut heat, "North Atlantic", Value::from(0));
    setdefault(&mut heat, "South Seas", Value::from(0));
    standing.insert("customs_heat".to_string(), Value::Object(heat));
    captain.insert("standing".to_string(), Value::Object(standing));
    data.insert("captain".to_string(), Value::Object(captain));
    data.insert("version".to_string(), Value::from(3));
    Ok(3)
}

fn migrate_v3_to_v4(data: &mut Map<String, Value>) {
    setdefault(
        data,
        "cultural_state",
        json_obj(&[
            ("active_festivals", Value::Array(Vec::new())),
            ("regions_entered", Value::Array(Vec::new())),
            ("cultural_encounters", Value::from(0)),
            ("port_visits", Value::Object(Map::new())),
        ]),
    );
    data.insert("version".to_string(), Value::from(4));
}

fn migrate_v4_to_v5(data: &mut Map<String, Value>) -> Result<i64, MigrateError> {
    let mut captain = object_clone(data.get("captain"))?;
    let mut standing = object_clone(captain.get("standing"))?;
    setdefault(
        &mut standing,
        "underworld_standing",
        Value::Object(Map::new()),
    );
    setdefault(&mut standing, "underworld_heat", Value::from(0));
    captain.insert("standing".to_string(), Value::Object(standing));
    data.insert("captain".to_string(), Value::Object(captain));
    setdefault(
        data,
        "pirate_state",
        json_obj(&[
            ("encounters", Value::Array(Vec::new())),
            ("nemesis_id", Value::Null),
            ("duels_won", Value::from(0)),
            ("duels_lost", Value::from(0)),
        ]),
    );
    data.insert("version".to_string(), Value::from(5));
    Ok(5)
}

fn migrate_v5_to_v6(data: &mut Map<String, Value>) -> Result<i64, MigrateError> {
    let mut captain = object_clone(data.get("captain"))?;
    let mut gear = object_clone(captain.get("combat_gear"))?;
    setdefault(&mut gear, "armor", Value::Null);
    setdefault(&mut gear, "melee_weapon", Value::Null);
    setdefault(&mut gear, "weapon_upgrades", Value::Object(Map::new()));
    captain.insert("combat_gear".to_string(), Value::Object(gear));
    if let Some(mut ship) = truthy_object(captain.get("ship"))? {
        setdefault(&mut ship, "upgrades", Value::Array(Vec::new()));
        captain.insert("ship".to_string(), Value::Object(ship));
    }
    data.insert("captain".to_string(), Value::Object(captain));
    data.insert("version".to_string(), Value::from(6));
    Ok(6)
}

fn migrate_v6_to_v7(data: &mut Map<String, Value>) -> Result<i64, MigrateError> {
    let mut captain = object_clone(data.get("captain"))?;
    if let Some(mut ship) = truthy_object(captain.get("ship"))? {
        let old = ship
            .get("upgrades")
            .cloned()
            .unwrap_or(Value::Array(Vec::new()));
        if let Value::Array(items) = &old {
            if matches!(items.first(), Some(Value::String(_))) {
                let converted = items
                    .iter()
                    .map(|uid| {
                        json_obj(&[
                            ("upgrade_id", uid.clone()),
                            ("installed_day", Value::from(0)),
                        ])
                    })
                    .collect();
                ship.insert("upgrades".to_string(), Value::Array(converted));
            }
        }
        setdefault(&mut ship, "upgrade_slots", Value::from(2));
        captain.insert("ship".to_string(), Value::Object(ship));
    }
    data.insert("captain".to_string(), Value::Object(captain));
    data.insert("version".to_string(), Value::from(7));
    Ok(7)
}

fn migrate_v7_to_v8(data: &mut Map<String, Value>) -> Result<i64, MigrateError> {
    let mut captain = object_clone(data.get("captain"))?;
    setdefault(&mut captain, "fleet", Value::Array(Vec::new()));
    data.insert("captain".to_string(), Value::Object(captain));
    data.insert("version".to_string(), Value::from(8));
    Ok(8)
}

fn migrate_v8_to_v9(data: &mut Map<String, Value>) -> Result<i64, MigrateError> {
    let mut captain = object_clone(data.get("captain"))?;
    if let Some(mut ship) = truthy_object(captain.get("ship"))? {
        add_roster(&mut ship);
        captain.insert("ship".to_string(), Value::Object(ship));
    }
    if let Some(Value::Array(fleet)) = captain.get_mut("fleet") {
        for owned in fleet.iter_mut() {
            let Some(owned_map) = owned.as_object_mut() else {
                continue;
            };
            if let Some(mut fleet_ship) = truthy_object(owned_map.get("ship"))? {
                add_roster(&mut fleet_ship);
                owned_map.insert("ship".to_string(), Value::Object(fleet_ship));
            }
        }
    }
    data.insert("captain".to_string(), Value::Object(captain));
    data.insert("version".to_string(), Value::from(9));
    Ok(9)
}

fn migrate_v9_to_v10(data: &mut Map<String, Value>) -> Result<i64, MigrateError> {
    let mut captain = object_clone(data.get("captain"))?;
    if let Some(mut ship) = truthy_object(captain.get("ship"))? {
        setdefault(&mut ship, "morale", Value::from(50));
        captain.insert("ship".to_string(), Value::Object(ship));
    }
    if let Some(Value::Array(fleet)) = captain.get_mut("fleet") {
        for owned in fleet.iter_mut() {
            let Some(owned_map) = owned.as_object_mut() else {
                continue;
            };
            if let Some(mut fleet_ship) = truthy_object(owned_map.get("ship"))? {
                setdefault(&mut fleet_ship, "morale", Value::from(50));
                owned_map.insert("ship".to_string(), Value::Object(fleet_ship));
            }
        }
    }
    data.insert("captain".to_string(), Value::Object(captain));
    data.insert("version".to_string(), Value::from(10));
    Ok(10)
}

fn migrate_v10_to_v11(data: &mut Map<String, Value>) -> Result<i64, MigrateError> {
    let mut captain = object_clone(data.get("captain"))?;
    if let Some(mut ship) = truthy_object(captain.get("ship"))? {
        setdefault(&mut ship, "officers", Value::Array(Vec::new()));
        captain.insert("ship".to_string(), Value::Object(ship));
    }
    if let Some(Value::Array(fleet)) = captain.get_mut("fleet") {
        for owned in fleet.iter_mut() {
            let Some(owned_map) = owned.as_object_mut() else {
                continue;
            };
            if let Some(mut fleet_ship) = truthy_object(owned_map.get("ship"))? {
                setdefault(&mut fleet_ship, "officers", Value::Array(Vec::new()));
                owned_map.insert("ship".to_string(), Value::Object(fleet_ship));
            }
        }
    }
    data.insert("captain".to_string(), Value::Object(captain));
    data.insert("version".to_string(), Value::from(11));
    Ok(11)
}

fn migrate_v11_to_v12(data: &mut Map<String, Value>) -> Result<i64, MigrateError> {
    if let Some(Value::Object(voyage)) = data.get_mut("voyage") {
        setdefault(voyage, "recent_events", Value::Array(Vec::new()));
    }
    let mut captain = object_clone(data.get("captain"))?;
    setdefault(&mut captain, "breach_records", Value::Array(Vec::new()));
    setdefault(&mut captain, "wanted_level", Value::from(0));
    setdefault(&mut captain, "deferred_fees", Value::Array(Vec::new()));
    setdefault(&mut captain, "active_bounties", Value::Array(Vec::new()));
    setdefault(&mut captain, "claimed_bounties", Value::Array(Vec::new()));
    data.insert("captain".to_string(), Value::Object(captain));
    let mut pirates = match data.get("pirate_state") {
        Some(Value::Object(map)) => map.clone(),
        _ => Map::new(),
    };
    setdefault(&mut pirates, "bounty_board", Value::Array(Vec::new()));
    data.insert("pirate_state".to_string(), Value::Object(pirates));
    data.insert("version".to_string(), Value::from(12));
    Ok(12)
}

fn add_roster(ship: &mut Map<String, Value>) {
    if ship.contains_key("roster") {
        return;
    }
    let crew = ship.get("crew").and_then(json_i64).unwrap_or(0);
    ship.insert(
        "roster".to_string(),
        json_obj(&[
            ("sailors", Value::from(crew)),
            ("gunners", Value::from(0)),
            ("navigators", Value::from(0)),
            ("surgeons", Value::from(0)),
            ("marines", Value::from(0)),
            ("quartermasters", Value::from(0)),
        ]),
    );
}

fn object_clone(value: Option<&Value>) -> Result<Map<String, Value>, MigrateError> {
    match value {
        None => Ok(Map::new()),
        Some(Value::Object(map)) => Ok(map.clone()),
        Some(_) => Err(MigrateError::Bad),
    }
}

/// Python `if ship:` — missing, null, and empty objects are skipped.
fn truthy_object(value: Option<&Value>) -> Result<Option<Map<String, Value>>, MigrateError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Object(map)) if map.is_empty() => Ok(None),
        Some(Value::Object(map)) => Ok(Some(map.clone())),
        Some(_) => Err(MigrateError::Bad),
    }
}

fn setdefault(map: &mut Map<String, Value>, key: &str, value: Value) {
    if !map.contains_key(key) {
        map.insert(key.to_string(), value);
    }
}

#[allow(clippy::too_many_arguments)]
fn encode(
    world: &World,
    receipts: &[TradeReceipt],
    run_id: &str,
    books: &HouseBooks,
    board: &ContractBoard,
    live: LiveEncounter<'_>,
) -> Value {
    let mut ports = Map::new();
    for port in &world.ports {
        ports.insert(port.id.clone(), port_value(port));
    }
    json_obj(&[
        ("version", Value::from(CURRENT_SAVE_VERSION)),
        ("captain", captain_value(&world.captain, &board.breaches)),
        ("ports", Value::Object(ports)),
        (
            "routes",
            Value::Array(world.routes.iter().map(route_value).collect()),
        ),
        ("voyage", voyage_value(&world.voyage)),
        ("day", Value::from(world.day)),
        ("seed", seed_value(world.seed)),
        (
            "pirate_state",
            pirate_value(world, live.encounter, live.player, live.opponent),
        ),
        ("ledger", ledger_value(receipts, run_id, books)),
        ("contract_board", board_value(board, books)),
        ("infrastructure", infra_value(books)),
        ("campaign", campaign_value(books)),
    ])
}

fn decode(data: &Value) -> Option<LoadedGame> {
    let data = data.as_object()?;
    let captain = captain_from(data.get("captain")?)?;
    let ports_value = data.get("ports")?.as_object()?;
    let mut ports = Vec::with_capacity(ports_value.len());
    for (_, port) in ports_value {
        ports.push(port_from(port)?);
    }
    let routes_value = data.get("routes")?.as_array()?;
    let mut routes = Vec::with_capacity(routes_value.len());
    for route in routes_value {
        routes.push(route_from(route)?);
    }
    let voyage = match truthy(data.get("voyage")) {
        Some(value) => voyage_from(value)?,
        None => return None,
    };
    let day = json_i64(data.get("day")?)?;
    let seed = match data.get("seed") {
        None => 0,
        Some(value) => json_i128(value)?,
    };
    let pirate = match truthy(data.get("pirate_state")) {
        Some(value) => pirate_from(value, &captain)?,
        None => PirateLoaded::default(),
    };
    let mut captain = captain;
    captain.duels_won = pirate.duels_won;
    captain.duels_lost = pirate.duels_lost;
    captain.naval_victories = pirate.naval_victories;
    captain.naval_defeats = pirate.naval_defeats;
    captain.encounters = pirate.encounters;
    let pending_duel = pirate.pending_duel;
    let ledger = match truthy(data.get("ledger")) {
        Some(value) => ledger_from(value)?,
        None => LedgerParts {
            run_id: String::new(),
            receipts: Vec::new(),
            total_buys: 0,
            total_sells: 0,
            net_profit: 0,
        },
    };
    let mut books = HouseBooks {
        total_buys: ledger.total_buys,
        total_sells: ledger.total_sells,
        net_profit: ledger.net_profit,
        trade_count: i64::try_from(ledger.receipts.len()).unwrap_or(i64::MAX),
        ..HouseBooks::default()
    };
    let mut board = match truthy(data.get("contract_board")) {
        Some(value) => board_from(value)?,
        None => ContractBoard::default(),
    };
    board.breaches = breaches_from(data.get("captain")?)?;
    books.completed_contracts = board
        .completed
        .iter()
        .map(|outcome| CompletedContract {
            outcome_type: outcome.outcome_type.clone(),
            family: if outcome.family.is_empty() {
                None
            } else {
                Some(outcome.family.clone())
            },
            summary: outcome.summary.clone(),
        })
        .collect();
    if let Some(infra) = truthy(data.get("infrastructure")) {
        apply_infra(&mut books, infra)?;
    }
    if let Some(campaign) = truthy(data.get("campaign")) {
        books.completed_paths = paths_from(campaign)?;
    }
    Some(LoadedGame {
        world: World {
            captain,
            ports,
            routes,
            voyage,
            day,
            seed,
            pending_duel,
        },
        receipts: ledger.receipts,
        run_id: ledger.run_id,
        books,
        board,
        encounter: pirate.encounter,
        player_combat: pirate.player,
        opponent_combat: pirate.opponent,
    })
}

fn captain_value(captain: &Captain, breaches: &[BreachRecord]) -> Value {
    json_obj(&[
        ("name", Value::from(captain.name.as_str())),
        ("captain_type", Value::from(captain.captain_type.as_str())),
        ("silver", Value::from(captain.silver)),
        (
            "ship",
            captain.ship.as_ref().map(ship_value).unwrap_or(Value::Null),
        ),
        (
            "cargo",
            Value::Array(captain.cargo.iter().map(cargo_value).collect()),
        ),
        ("provisions", Value::from(captain.provisions)),
        ("day", Value::from(captain.day)),
        ("standing", standing_value(&captain.standing)),
        (
            "active_style",
            captain
                .active_style
                .as_ref()
                .map(|id| Value::from(id.as_str()))
                .unwrap_or(Value::Null),
        ),
        (
            "deferred_fees",
            Value::Array(captain.deferred_fees.iter().map(fee_value).collect()),
        ),
        (
            "breach_records",
            Value::Array(breaches.iter().map(breach_value).collect()),
        ),
        ("wanted_level", Value::from(captain.wanted_level)),
        (
            "active_bounties",
            Value::Array(
                captain
                    .active_bounties
                    .iter()
                    .map(|id| Value::from(id.as_str()))
                    .collect(),
            ),
        ),
    ])
}

fn captain_from(value: &Value) -> Option<Captain> {
    let map = value.as_object()?;
    let standing = match map.get("standing") {
        Some(value) => standing_from(value)?,
        None => Standing {
            regional: [0; 5],
            heat: [0; 5],
            commercial_trust: 0,
            port_standing: Vec::new(),
            underworld: Vec::new(),
            incidents: Vec::new(),
        },
    };
    Some(Captain {
        name: req_str(map, "name")?,
        captain_type: opt_str(map, "captain_type").unwrap_or_else(|| "merchant".to_string()),
        silver: req_i64(map, "silver")?,
        ship: match truthy(map.get("ship")) {
            Some(ship) => Some(ship_from(ship)?),
            None => None,
        },
        cargo: cargo_list(map.get("cargo"))?,
        provisions: req_i64(map, "provisions")?,
        day: req_i64(map, "day")?,
        standing,
        wanted_level: opt_i64(map, "wanted_level").unwrap_or(0),
        active_bounties: string_list(map.get("active_bounties"))?,
        deferred_fees: fee_list(map.get("deferred_fees"))?,
        melee: None,
        firearm: None,
        mechanical: None,
        throwing: Vec::new(),
        armor: None,
        styles: Vec::new(),
        active_style: opt_str(map, "active_style"),
        duels_won: 0,
        duels_lost: 0,
        encounters: Vec::new(),
        fleet: Vec::new(),
        naval_victories: 0,
        naval_defeats: 0,
    })
}

fn ship_value(ship: &Ship) -> Value {
    json_obj(&[
        ("template_id", Value::from(ship.template_id.as_str())),
        ("name", Value::from(ship.name.as_str())),
        ("hull", Value::from(ship.hull)),
        ("hull_max", Value::from(ship.hull_max)),
        ("cargo_capacity", Value::from(ship.cargo_capacity)),
        ("speed", f64_value(ship.speed)),
        ("crew", Value::from(ship.crew)),
        ("crew_max", Value::from(ship.crew_max)),
        ("cannons", Value::from(ship.cannons)),
        ("maneuver", f64_value(ship.maneuver)),
        ("upgrades", Value::Array(Vec::new())),
        ("upgrade_slots", Value::from(ship.upgrade_slots)),
        (
            "roster",
            json_obj(&[
                ("sailors", Value::from(ship.sailors)),
                ("gunners", Value::from(ship.gunners)),
                ("navigators", Value::from(ship.navigators)),
                ("surgeons", Value::from(ship.surgeons)),
                ("marines", Value::from(ship.marines)),
                ("quartermasters", Value::from(ship.quartermasters)),
            ]),
        ),
        ("morale", Value::from(ship.morale)),
        (
            "officers",
            Value::Array(ship.officers.iter().map(officer_value).collect()),
        ),
    ])
}

fn ship_from(value: &Value) -> Option<Ship> {
    let map = value.as_object()?;
    let template_id = req_str(map, "template_id")?;
    let template = content::content().ship(&template_id);
    let cannons = match map.get("cannons").and_then(json_i64) {
        Some(cannons) if map.contains_key("cannons") => cannons,
        _ => template.map(|ship| ship.cannons).unwrap_or(0),
    };
    let maneuver = match map.get("maneuver").and_then(json_f64) {
        Some(maneuver) if map.contains_key("maneuver") => maneuver,
        _ => template.map(|ship| ship.maneuver).unwrap_or(0.5),
    };
    let crew = req_i64(map, "crew")?;
    let (sailors, gunners, navigators, surgeons, marines, quartermasters) = match map.get("roster")
    {
        Some(Value::Object(roster)) => (
            opt_i64(roster, "sailors").unwrap_or(0),
            opt_i64(roster, "gunners").unwrap_or(0),
            opt_i64(roster, "navigators").unwrap_or(0),
            opt_i64(roster, "surgeons").unwrap_or(0),
            opt_i64(roster, "marines").unwrap_or(0),
            opt_i64(roster, "quartermasters").unwrap_or(0),
        ),
        Some(_) => return None,
        None => (crew, 0, 0, 0, 0, 0),
    };
    Some(Ship {
        template_id,
        name: req_str(map, "name")?,
        hull: req_i64(map, "hull")?,
        hull_max: req_i64(map, "hull_max")?,
        cargo_capacity: req_i64(map, "cargo_capacity")?,
        speed: req_f64(map, "speed")?,
        crew,
        crew_max: req_i64(map, "crew_max")?,
        cannons,
        maneuver,
        upgrade_slots: opt_i64(map, "upgrade_slots").unwrap_or(2),
        morale: opt_i64(map, "morale").unwrap_or(50),
        sailors,
        gunners,
        navigators,
        surgeons,
        marines,
        quartermasters,
        officers: officers_from(map.get("officers"))?,
    })
}

fn officer_value(officer: &Officer) -> Value {
    json_obj(&[
        ("name", Value::from(officer.name.as_str())),
        ("role", Value::from(officer.role.as_str())),
        ("experience", Value::from(0)),
        ("origin_port", Value::from(officer.origin_port.as_str())),
        ("trait", Value::from(officer.trait_name.as_str())),
    ])
}

fn officers_from(value: Option<&Value>) -> Option<Vec<Officer>> {
    let Some(value) = value else {
        return Some(Vec::new());
    };
    let items = value.as_array()?;
    let mut officers = Vec::with_capacity(items.len());
    for item in items {
        let map = item.as_object()?;
        let role = map
            .get("role")
            .and_then(Value::as_str)
            .filter(|role| CREW_ROLES.contains(role))
            .unwrap_or("sailor");
        officers.push(Officer {
            name: opt_str(map, "name").unwrap_or_else(|| "Unknown".to_string()),
            role: role.to_string(),
            origin_port: opt_str(map, "origin_port").unwrap_or_default(),
            trait_name: opt_str(map, "trait").unwrap_or_default(),
        });
    }
    Some(officers)
}

fn cargo_value(cargo: &CargoItem) -> Value {
    json_obj(&[
        ("good_id", Value::from(cargo.good_id.as_str())),
        ("quantity", Value::from(cargo.quantity)),
        ("cost_basis", Value::from(cargo.cost_basis)),
        ("acquired_port", Value::from(cargo.acquired_port.as_str())),
        (
            "acquired_region",
            Value::from(cargo.acquired_region.as_str()),
        ),
        ("acquired_day", Value::from(cargo.acquired_day)),
    ])
}

fn cargo_list(value: Option<&Value>) -> Option<Vec<CargoItem>> {
    let Some(value) = value else {
        return Some(Vec::new());
    };
    let mut cargo = Vec::new();
    for item in value.as_array()? {
        let map = item.as_object()?;
        cargo.push(CargoItem {
            good_id: req_str(map, "good_id")?,
            quantity: req_i64(map, "quantity")?,
            cost_basis: opt_i64(map, "cost_basis").unwrap_or(0),
            acquired_port: opt_str(map, "acquired_port").unwrap_or_default(),
            acquired_region: opt_str(map, "acquired_region").unwrap_or_default(),
            acquired_day: opt_i64(map, "acquired_day").unwrap_or(0),
        });
    }
    Some(cargo)
}

fn fee_value(fee: &DeferredFee) -> Value {
    json_obj(&[
        ("type", Value::from(fee.fee_type.as_str())),
        ("amount", Value::from(fee.amount)),
        ("day", Value::from(fee.day)),
    ])
}

fn fee_list(value: Option<&Value>) -> Option<Vec<DeferredFee>> {
    let Some(value) = value else {
        return Some(Vec::new());
    };
    let mut fees = Vec::new();
    for item in value.as_array()? {
        let Some(map) = item.as_object() else {
            continue;
        };
        fees.push(DeferredFee {
            fee_type: opt_str(map, "type").unwrap_or_default(),
            amount: opt_i64(map, "amount").unwrap_or(0),
            day: opt_i64(map, "day").unwrap_or(0),
        });
    }
    Some(fees)
}

fn standing_value(standing: &Standing) -> Value {
    let mut regional = Map::new();
    let mut heat = Map::new();
    for (index, region) in REGIONS.iter().enumerate() {
        regional.insert((*region).to_string(), Value::from(standing.regional[index]));
        heat.insert((*region).to_string(), Value::from(standing.heat[index]));
    }
    let mut ports = Map::new();
    for (id, value) in &standing.port_standing {
        ports.insert(id.clone(), Value::from(*value));
    }
    let mut underworld = Map::new();
    for (id, value) in &standing.underworld {
        underworld.insert(id.clone(), Value::from(*value));
    }
    json_obj(&[
        ("regional_standing", Value::Object(regional)),
        ("port_standing", Value::Object(ports)),
        ("customs_heat", Value::Object(heat)),
        ("commercial_trust", Value::from(standing.commercial_trust)),
        (
            "recent_incidents",
            Value::Array(standing.incidents.iter().map(incident_value).collect()),
        ),
        ("underworld_standing", Value::Object(underworld)),
    ])
}

fn standing_from(value: &Value) -> Option<Standing> {
    let map = value.as_object()?;
    let mut regional = [0; 5];
    let mut heat = [0; 5];
    if let Some(saved) = map.get("regional_standing") {
        let saved = saved.as_object()?;
        for (index, region) in REGIONS.iter().enumerate() {
            if let Some(number) = saved.get(*region).and_then(json_i64) {
                regional[index] = number;
            }
        }
    }
    if let Some(saved) = map.get("customs_heat") {
        let saved = saved.as_object()?;
        for (index, region) in REGIONS.iter().enumerate() {
            if let Some(number) = saved.get(*region).and_then(json_i64) {
                heat[index] = number;
            }
        }
    }
    let mut port_standing = Vec::new();
    if let Some(saved) = map.get("port_standing") {
        for (id, number) in saved.as_object()? {
            port_standing.push((id.clone(), json_i64(number)?));
        }
    }
    let mut underworld = Vec::new();
    if let Some(saved) = map.get("underworld_standing") {
        for (id, number) in saved.as_object()? {
            underworld.push((id.clone(), json_i64(number)?));
        }
    }
    let mut incidents = Vec::new();
    if let Some(saved) = map.get("recent_incidents") {
        for item in saved.as_array()? {
            incidents.push(incident_from(item)?);
        }
    }
    Some(Standing {
        regional,
        heat,
        commercial_trust: opt_i64(map, "commercial_trust").unwrap_or(0),
        port_standing,
        underworld,
        incidents,
    })
}

fn incident_value(incident: &Incident) -> Value {
    json_obj(&[
        ("day", Value::from(incident.day)),
        ("port_id", Value::from(incident.port_id.as_str())),
        ("region", Value::from(incident.region.as_str())),
        (
            "incident_type",
            Value::from(incident.incident_type.as_str()),
        ),
        ("description", Value::from(incident.description.as_str())),
        ("heat_delta", Value::from(incident.heat_delta)),
        ("standing_delta", Value::from(incident.standing_delta)),
        ("trust_delta", Value::from(incident.trust_delta)),
    ])
}

fn incident_from(value: &Value) -> Option<Incident> {
    let map = value.as_object()?;
    Some(Incident {
        day: req_i64(map, "day")?,
        port_id: req_str(map, "port_id")?,
        region: req_str(map, "region")?,
        incident_type: req_str(map, "incident_type")?,
        description: req_str(map, "description")?,
        heat_delta: opt_i64(map, "heat_delta").unwrap_or(0),
        standing_delta: opt_i64(map, "standing_delta").unwrap_or(0),
        trust_delta: opt_i64(map, "trust_delta").unwrap_or(0),
    })
}

fn port_value(port: &Port) -> Value {
    json_obj(&[
        ("id", Value::from(port.id.as_str())),
        ("name", Value::from(port.name.as_str())),
        ("description", Value::from(port.description.as_str())),
        ("region", Value::from(port.region.as_str())),
        (
            "features",
            Value::Array(
                port.features
                    .iter()
                    .map(|feature| Value::from(feature.as_str()))
                    .collect(),
            ),
        ),
        (
            "market",
            Value::Array(port.market.iter().map(slot_value).collect()),
        ),
        ("port_fee", Value::from(port.port_fee)),
        ("provision_cost", Value::from(port.provision_cost)),
        ("repair_cost", Value::from(port.repair_cost)),
        ("crew_cost", Value::from(port.crew_cost)),
        ("map_x", Value::from(port.map_x)),
        ("map_y", Value::from(port.map_y)),
    ])
}

fn port_from(value: &Value) -> Option<Port> {
    let map = value.as_object()?;
    let mut features = Vec::new();
    if let Some(saved) = map.get("features") {
        for feature in saved.as_array()? {
            let Some(feature) = feature.as_str() else {
                continue;
            };
            if PORT_FEATURES.contains(&feature) {
                features.push(feature.to_string());
            }
        }
    }
    let mut market = Vec::new();
    if let Some(saved) = map.get("market") {
        for slot in saved.as_array()? {
            market.push(slot_from(slot)?);
        }
    }
    Some(Port {
        id: req_str(map, "id")?,
        name: req_str(map, "name")?,
        description: req_str(map, "description")?,
        region: req_str(map, "region")?,
        features,
        market,
        port_fee: opt_i64(map, "port_fee").unwrap_or(5),
        provision_cost: opt_i64(map, "provision_cost").unwrap_or(2),
        repair_cost: opt_i64(map, "repair_cost").unwrap_or(3),
        crew_cost: opt_i64(map, "crew_cost").unwrap_or(5),
        map_x: opt_i64(map, "map_x").unwrap_or(0),
        map_y: opt_i64(map, "map_y").unwrap_or(0),
    })
}

fn slot_value(slot: &MarketSlot) -> Value {
    json_obj(&[
        ("good_id", Value::from(slot.good_id.as_str())),
        ("stock_current", Value::from(slot.stock_current)),
        ("stock_target", Value::from(slot.stock_target)),
        ("restock_rate", f64_value(slot.restock_rate)),
        ("local_affinity", f64_value(slot.local_affinity)),
        ("spread", f64_value(slot.spread)),
        ("buy_price", Value::from(slot.buy_price)),
        ("sell_price", Value::from(slot.sell_price)),
        ("flood_penalty", f64_value(slot.flood_penalty)),
    ])
}

fn slot_from(value: &Value) -> Option<MarketSlot> {
    let map = value.as_object()?;
    Some(MarketSlot {
        good_id: req_str(map, "good_id")?,
        stock_current: req_i64(map, "stock_current")?,
        stock_target: req_i64(map, "stock_target")?,
        restock_rate: req_f64(map, "restock_rate")?,
        local_affinity: opt_f64(map, "local_affinity").unwrap_or(1.0),
        spread: opt_f64(map, "spread").unwrap_or(0.15),
        buy_price: opt_i64(map, "buy_price").unwrap_or(0),
        sell_price: opt_i64(map, "sell_price").unwrap_or(0),
        flood_penalty: opt_f64(map, "flood_penalty").unwrap_or(0.0),
    })
}

fn route_value(route: &Route) -> Value {
    json_obj(&[
        ("port_a", Value::from(route.port_a.as_str())),
        ("port_b", Value::from(route.port_b.as_str())),
        ("distance", Value::from(route.distance)),
        ("danger", f64_value(route.danger)),
        ("min_ship_class", Value::from(route.min_ship_class.as_str())),
    ])
}

fn route_from(value: &Value) -> Option<Route> {
    let map = value.as_object()?;
    Some(Route {
        port_a: req_str(map, "port_a")?,
        port_b: req_str(map, "port_b")?,
        distance: req_i64(map, "distance")?,
        danger: opt_f64(map, "danger").unwrap_or(0.1),
        min_ship_class: opt_str(map, "min_ship_class").unwrap_or_else(|| "sloop".to_string()),
        lore_name: String::new(),
        lore: String::new(),
    })
}

fn voyage_value(voyage: &Voyage) -> Value {
    json_obj(&[
        ("origin_id", Value::from(voyage.origin_id.as_str())),
        (
            "destination_id",
            Value::from(voyage.destination_id.as_str()),
        ),
        ("distance", Value::from(voyage.distance)),
        ("progress", Value::from(voyage.progress)),
        ("days_elapsed", Value::from(voyage.days_elapsed)),
        ("status", Value::from(voyage.status.as_str())),
        (
            "recent_events",
            Value::Array(
                voyage
                    .recent_events
                    .iter()
                    .map(|event| Value::from(event.as_str()))
                    .collect(),
            ),
        ),
    ])
}

fn voyage_from(value: &Value) -> Option<Voyage> {
    let map = value.as_object()?;
    let status = match map.get("status")?.as_str() {
        Some("in_port") => VoyageStatus::InPort,
        Some("arrived") => VoyageStatus::Arrived,
        Some(_) | None => VoyageStatus::AtSea,
    };
    let mut recent_events = Vec::new();
    if let Some(saved) = map.get("recent_events") {
        for event in saved.as_array()? {
            if let Some(event) = event.as_str() {
                recent_events.push(event.to_string());
            }
        }
    }
    Some(Voyage {
        origin_id: req_str(map, "origin_id")?,
        destination_id: req_str(map, "destination_id")?,
        distance: req_i64(map, "distance")?,
        progress: opt_i64(map, "progress").unwrap_or(0),
        days_elapsed: opt_i64(map, "days_elapsed").unwrap_or(0),
        status,
        recent_events,
    })
}

fn pirate_value(
    world: &World,
    encounter: Option<&EncounterState>,
    player: Option<&CombatantState>,
    opponent: Option<&CombatantState>,
) -> Value {
    let captain = &world.captain;
    let mut map = Map::new();
    map.insert(
        "encounters".to_string(),
        Value::Array(
            captain
                .encounters
                .iter()
                .map(encounter_record_value)
                .collect(),
        ),
    );
    map.insert("nemesis_id".to_string(), Value::Null);
    map.insert("duels_won".to_string(), Value::from(captain.duels_won));
    map.insert("duels_lost".to_string(), Value::from(captain.duels_lost));
    map.insert(
        "naval_victories".to_string(),
        Value::from(captain.naval_victories),
    );
    map.insert(
        "naval_defeats".to_string(),
        Value::from(captain.naval_defeats),
    );
    map.insert(
        "encounter_phase".to_string(),
        Value::from(encounter.map(|enc| enc.phase.as_str()).unwrap_or("")),
    );
    map.insert(
        "encounter_state".to_string(),
        encounter_blob(encounter, player, opponent),
    );
    map.insert("bounty_board".to_string(), Value::Array(Vec::new()));
    if let Some(duel) = &world.pending_duel {
        map.insert(
            "pending_duel".to_string(),
            json_obj(&[
                ("captain_id", Value::from(duel.captain_id.as_str())),
                ("captain_name", Value::from(duel.captain_name.as_str())),
                ("faction_id", Value::from(duel.faction_id.as_str())),
                ("personality", Value::from(duel.personality.as_str())),
                ("strength", Value::from(duel.strength)),
                ("region", Value::from(duel.region.as_str())),
            ]),
        );
    }
    Value::Object(map)
}

fn encounter_record_value(record: &PirateEncounterRecord) -> Value {
    json_obj(&[
        ("captain_id", Value::from(record.captain_id.as_str())),
        ("faction_id", Value::from(record.faction_id.as_str())),
        ("day", Value::from(record.day)),
        ("outcome", Value::from(record.outcome.as_str())),
        ("region", Value::from(record.region.as_str())),
    ])
}

/// `app.session.encounter_persist_blob`. `pending_victory` stays false: the
/// spare/take-all choice is not on this session.
fn encounter_blob(
    encounter: Option<&EncounterState>,
    player: Option<&CombatantState>,
    opponent: Option<&CombatantState>,
) -> Value {
    let Some(enc) = encounter else {
        return Value::Object(Map::new());
    };
    let mut map = Map::new();
    map.insert(
        "enemy_captain_id".to_string(),
        Value::from(enc.enemy_captain_id.as_str()),
    );
    map.insert(
        "enemy_captain_name".to_string(),
        Value::from(enc.enemy_captain_name.as_str()),
    );
    map.insert(
        "enemy_faction_id".to_string(),
        Value::from(enc.enemy_faction_id.as_str()),
    );
    map.insert(
        "enemy_personality".to_string(),
        Value::from(enc.enemy_personality.as_str()),
    );
    map.insert(
        "enemy_strength".to_string(),
        Value::from(enc.enemy_strength),
    );
    map.insert(
        "enemy_region".to_string(),
        Value::from(enc.enemy_region.as_str()),
    );
    map.insert(
        "enemy_ship_hull".to_string(),
        Value::from(enc.enemy_ship_hull),
    );
    map.insert(
        "enemy_ship_hull_max".to_string(),
        Value::from(enc.enemy_ship_hull_max),
    );
    map.insert(
        "enemy_ship_cannons".to_string(),
        Value::from(enc.enemy_ship_cannons),
    );
    map.insert(
        "enemy_ship_maneuver".to_string(),
        f64_value(enc.enemy_ship_maneuver),
    );
    map.insert(
        "enemy_ship_speed".to_string(),
        f64_value(enc.enemy_ship_speed),
    );
    map.insert(
        "enemy_ship_crew".to_string(),
        Value::from(enc.enemy_ship_crew),
    );
    map.insert(
        "enemy_ship_crew_max".to_string(),
        Value::from(enc.enemy_ship_crew_max),
    );
    map.insert(
        "boarding_progress".to_string(),
        Value::from(enc.boarding_progress),
    );
    map.insert(
        "boarding_threshold".to_string(),
        Value::from(enc.boarding_threshold),
    );
    map.insert("naval_turns".to_string(), Value::from(enc.naval_turns));
    map.insert("duel_turns".to_string(), Value::from(enc.duel_turns));
    map.insert("pending_victory".to_string(), Value::from(false));
    if let Some(player) = player {
        map.insert("player_hp".to_string(), Value::from(player.hp));
        map.insert("player_stamina".to_string(), Value::from(player.stamina));
    }
    if let Some(opponent) = opponent {
        map.insert("opponent_hp".to_string(), Value::from(opponent.hp));
        map.insert(
            "opponent_stamina".to_string(),
            Value::from(opponent.stamina),
        );
    }
    Value::Object(map)
}

#[derive(Default)]
struct PirateLoaded {
    encounters: Vec<PirateEncounterRecord>,
    duels_won: i64,
    duels_lost: i64,
    naval_victories: i64,
    naval_defeats: i64,
    pending_duel: Option<PendingDuel>,
    encounter: Option<EncounterState>,
    player: Option<CombatantState>,
    opponent: Option<CombatantState>,
}

fn pirate_from(value: &Value, captain: &Captain) -> Option<PirateLoaded> {
    let map = value.as_object()?;
    let pending_duel = pending_from(value)?;
    let mut encounters = Vec::new();
    if let Some(saved) = map.get("encounters").and_then(Value::as_array) {
        for record in saved {
            let record = record.as_object()?;
            encounters.push(PirateEncounterRecord {
                captain_id: req_str(record, "captain_id")?,
                faction_id: req_str(record, "faction_id")?,
                day: req_i64(record, "day")?,
                outcome: req_str(record, "outcome")?,
                region: opt_str(record, "region").unwrap_or_default(),
            });
        }
    }
    let phase = opt_str(map, "encounter_phase").unwrap_or_default();
    let empty_estate = Map::new();
    let estate = match map.get("encounter_state") {
        Some(Value::Object(estate)) => estate,
        _ => &empty_estate,
    };
    let encounter = encounter_from(&phase, estate, pending_duel.as_ref());
    let (player, opponent) = match &encounter {
        Some(enc) => combatants_from(captain, enc, estate),
        None => (None, None),
    };
    Some(PirateLoaded {
        encounters,
        duels_won: opt_i64(map, "duels_won").unwrap_or(0),
        duels_lost: opt_i64(map, "duels_lost").unwrap_or(0),
        naval_victories: opt_i64(map, "naval_victories").unwrap_or(0),
        naval_defeats: opt_i64(map, "naval_defeats").unwrap_or(0),
        pending_duel,
        encounter,
        player,
        opponent,
    })
}

fn encounter_from(
    phase: &str,
    estate: &Map<String, Value>,
    pending: Option<&PendingDuel>,
) -> Option<EncounterState> {
    if phase.is_empty() && estate.is_empty() {
        return None;
    }
    let hull = opt_i64(estate, "enemy_ship_hull").unwrap_or(0);
    let crew = opt_i64(estate, "enemy_ship_crew").unwrap_or(0);
    let mut enc = EncounterState {
        enemy_captain_id: opt_str(estate, "enemy_captain_id")
            .or_else(|| pending.map(|duel| duel.captain_id.clone()))
            .unwrap_or_default(),
        enemy_captain_name: opt_str(estate, "enemy_captain_name")
            .or_else(|| pending.map(|duel| duel.captain_name.clone()))
            .unwrap_or_default(),
        enemy_faction_id: opt_str(estate, "enemy_faction_id")
            .or_else(|| pending.map(|duel| duel.faction_id.clone()))
            .unwrap_or_default(),
        enemy_personality: opt_str(estate, "enemy_personality")
            .or_else(|| pending.map(|duel| duel.personality.clone()))
            .unwrap_or_default(),
        enemy_strength: opt_i64(estate, "enemy_strength")
            .or_else(|| pending.map(|duel| duel.strength))
            .unwrap_or(0),
        enemy_region: opt_str(estate, "enemy_region")
            .or_else(|| pending.map(|duel| duel.region.clone()))
            .unwrap_or_default(),
        enemy_ship_hull: hull,
        enemy_ship_hull_max: opt_i64(estate, "enemy_ship_hull_max").unwrap_or(hull),
        enemy_ship_cannons: opt_i64(estate, "enemy_ship_cannons").unwrap_or(0),
        enemy_ship_maneuver: opt_f64(estate, "enemy_ship_maneuver").unwrap_or(0.5),
        enemy_ship_speed: opt_f64(estate, "enemy_ship_speed").unwrap_or(6.0),
        enemy_ship_crew: crew,
        enemy_ship_crew_max: opt_i64(estate, "enemy_ship_crew_max").unwrap_or(crew),
        phase: if phase.is_empty() {
            "approach".to_string()
        } else {
            phase.to_string()
        },
        boarding_progress: opt_i64(estate, "boarding_progress").unwrap_or(0),
        boarding_threshold: opt_i64(estate, "boarding_threshold").unwrap_or(3),
        naval_turns: opt_i64(estate, "naval_turns").unwrap_or(0),
        duel_turns: opt_i64(estate, "duel_turns").unwrap_or(0),
    };
    if opt_bool(estate, "pending_victory").unwrap_or(false) && enc.phase != "capture_available" {
        enc.phase = "resolved".to_string();
    }
    Some(enc)
}

fn combatants_from(
    captain: &Captain,
    encounter: &EncounterState,
    estate: &Map<String, Value>,
) -> (Option<CombatantState>, Option<CombatantState>) {
    if !estate.contains_key("player_hp") && !estate.contains_key("opponent_hp") {
        return (None, None);
    }
    let (mut player, mut opponent) = encounter::create_duel_combatants(encounter, captain);
    if let Some(hp) = opt_i64(estate, "player_hp") {
        player.hp = hp;
    }
    if let Some(stamina) = opt_i64(estate, "player_stamina") {
        player.stamina = stamina;
    }
    if let Some(hp) = opt_i64(estate, "opponent_hp") {
        opponent.hp = hp;
    }
    if let Some(stamina) = opt_i64(estate, "opponent_stamina") {
        opponent.stamina = stamina;
    }
    (Some(player), Some(opponent))
}

fn pending_from(value: &Value) -> Option<Option<PendingDuel>> {
    let map = value.as_object()?;
    let Some(duel) = truthy(map.get("pending_duel")) else {
        return Some(None);
    };
    let duel = duel.as_object()?;
    Some(Some(PendingDuel {
        captain_id: req_str(duel, "captain_id")?,
        captain_name: req_str(duel, "captain_name")?,
        faction_id: req_str(duel, "faction_id")?,
        personality: req_str(duel, "personality")?,
        strength: req_i64(duel, "strength")?,
        region: opt_str(duel, "region").unwrap_or_default(),
    }))
}

fn ledger_value(receipts: &[TradeReceipt], run_id: &str, books: &HouseBooks) -> Value {
    json_obj(&[
        ("run_id", Value::from(run_id)),
        (
            "receipts",
            Value::Array(receipts.iter().map(receipt_value).collect()),
        ),
        ("total_buys", Value::from(books.total_buys)),
        ("total_sells", Value::from(books.total_sells)),
        ("net_profit", Value::from(books.net_profit)),
    ])
}

struct LedgerParts {
    run_id: String,
    receipts: Vec<TradeReceipt>,
    total_buys: i64,
    total_sells: i64,
    net_profit: i64,
}

fn ledger_from(value: &Value) -> Option<LedgerParts> {
    let map = value.as_object()?;
    let mut receipts = Vec::new();
    if let Some(saved) = map.get("receipts") {
        for item in saved.as_array()? {
            if let Some(receipt) = receipt_from(item)? {
                receipts.push(receipt);
            }
        }
    }
    Some(LedgerParts {
        run_id: opt_str(map, "run_id").unwrap_or_default(),
        receipts,
        total_buys: opt_i64(map, "total_buys").unwrap_or(0),
        total_sells: opt_i64(map, "total_sells").unwrap_or(0),
        net_profit: opt_i64(map, "net_profit").unwrap_or(0),
    })
}

fn receipt_value(receipt: &TradeReceipt) -> Value {
    json_obj(&[
        ("receipt_id", Value::from(receipt.receipt_id.as_str())),
        ("captain_name", Value::from(receipt.captain_name.as_str())),
        ("port_id", Value::from(receipt.port_id.as_str())),
        ("good_id", Value::from(receipt.good_id.as_str())),
        ("action", Value::from(receipt.action)),
        ("quantity", Value::from(receipt.quantity)),
        ("unit_price", Value::from(receipt.unit_price)),
        ("total_price", Value::from(receipt.total_price)),
        ("day", Value::from(receipt.day)),
        ("stock_before", Value::from(receipt.stock_before)),
        ("stock_after", Value::from(receipt.stock_after)),
    ])
}

fn receipt_from(value: &Value) -> Option<Option<TradeReceipt>> {
    let map = value.as_object()?;
    let action = match map.get("action").and_then(Value::as_str) {
        Some("buy") => "buy",
        Some("sell") => "sell",
        _ => return Some(None),
    };
    Some(Some(TradeReceipt {
        receipt_id: req_str(map, "receipt_id")?,
        captain_name: req_str(map, "captain_name")?,
        port_id: req_str(map, "port_id")?,
        good_id: req_str(map, "good_id")?,
        action,
        quantity: req_i64(map, "quantity")?,
        unit_price: req_i64(map, "unit_price")?,
        total_price: req_i64(map, "total_price")?,
        day: req_i64(map, "day")?,
        stock_before: opt_i64(map, "stock_before").unwrap_or(0),
        stock_after: opt_i64(map, "stock_after").unwrap_or(0),
    }))
}

fn board_value(board: &ContractBoard, books: &HouseBooks) -> Value {
    // Real resolutions live on the board. A books-only row (the victory seam
    // from before the board was ported) still uses the Python outcome keys
    // when the board has nothing completed, so that slot round-trips.
    let completed = if board.completed.is_empty() {
        books
            .completed_contracts
            .iter()
            .enumerate()
            .map(|(index, contract)| books_outcome_value(index, contract))
            .collect()
    } else {
        board.completed.iter().map(outcome_value).collect()
    };
    json_obj(&[
        (
            "offers",
            Value::Array(board.offers.iter().map(offer_value).collect()),
        ),
        (
            "active",
            Value::Array(board.active.iter().map(active_value).collect()),
        ),
        ("completed", Value::Array(completed)),
        ("last_refresh_day", Value::from(board.last_refresh_day)),
        ("max_offers", Value::from(board.max_offers)),
    ])
}

fn board_from(value: &Value) -> Option<ContractBoard> {
    let map = value.as_object()?;
    Some(ContractBoard {
        offers: offers_from(map.get("offers"))?,
        active: active_from(map.get("active"))?,
        completed: outcomes_from(map.get("completed"))?,
        last_refresh_day: opt_i64(map, "last_refresh_day").unwrap_or(0),
        max_offers: opt_i64(map, "max_offers").unwrap_or(5),
        breaches: Vec::new(),
    })
}

fn offer_value(offer: &Contract) -> Value {
    json_obj(&[
        ("id", Value::from(offer.id.as_str())),
        ("template_id", Value::from(offer.template_id.as_str())),
        ("family", Value::from(offer.family.as_str())),
        ("title", Value::from(offer.title.as_str())),
        ("description", Value::from(offer.description.as_str())),
        ("issuer_port_id", Value::from(offer.issuer_port_id.as_str())),
        (
            "destination_port_id",
            Value::from(offer.destination_port_id.as_str()),
        ),
        ("good_id", Value::from(offer.good_id.as_str())),
        ("quantity", Value::from(offer.quantity)),
        ("created_day", Value::from(offer.created_day)),
        ("deadline_day", Value::from(offer.deadline_day)),
        ("reward_silver", Value::from(offer.reward_silver)),
        ("bonus_reward", Value::from(offer.bonus_reward)),
        (
            "required_trust_tier",
            Value::from(offer.required_trust_tier.as_str()),
        ),
        ("required_standing", Value::from(offer.required_standing)),
        ("heat_ceiling", opt_i64_value(offer.heat_ceiling)),
        ("inspection_modifier", f64_value(offer.inspection_modifier)),
        ("source_region", opt_string_value(&offer.source_region)),
        ("source_port", opt_string_value(&offer.source_port)),
        ("offer_reason", Value::from(offer.offer_reason.as_str())),
        (
            "tags",
            Value::Array(
                offer
                    .tags
                    .iter()
                    .map(|tag| Value::from(tag.as_str()))
                    .collect(),
            ),
        ),
        ("acceptance_window", Value::from(offer.acceptance_window)),
    ])
}

fn offers_from(value: Option<&Value>) -> Option<Vec<Contract>> {
    let Some(value) = value else {
        return Some(Vec::new());
    };
    let mut offers = Vec::new();
    for item in value.as_array()? {
        if let Some(offer) = offer_from(item)? {
            offers.push(offer);
        }
    }
    Some(offers)
}

/// `None` means the offer object is corrupt. `Some(None)` drops an unknown family,
/// matching `_offer_from_dict`.
fn offer_from(value: &Value) -> Option<Option<Contract>> {
    let map = value.as_object()?;
    let family = req_str(map, "family")?;
    if !CONTRACT_FAMILIES.contains(&family.as_str()) {
        return Some(None);
    }
    Some(Some(Contract {
        id: req_str(map, "id")?,
        template_id: req_str(map, "template_id")?,
        family,
        title: req_str(map, "title")?,
        description: req_str(map, "description")?,
        issuer_port_id: req_str(map, "issuer_port_id")?,
        destination_port_id: req_str(map, "destination_port_id")?,
        good_id: req_str(map, "good_id")?,
        quantity: req_i64(map, "quantity")?,
        created_day: req_i64(map, "created_day")?,
        deadline_day: req_i64(map, "deadline_day")?,
        reward_silver: req_i64(map, "reward_silver")?,
        bonus_reward: opt_i64(map, "bonus_reward").unwrap_or(0),
        required_trust_tier: opt_str(map, "required_trust_tier")
            .unwrap_or_else(|| "unproven".to_string()),
        required_standing: opt_i64(map, "required_standing").unwrap_or(0),
        heat_ceiling: map.get("heat_ceiling").and_then(json_i64),
        inspection_modifier: opt_f64(map, "inspection_modifier").unwrap_or(0.0),
        source_region: opt_str(map, "source_region"),
        source_port: opt_str(map, "source_port"),
        offer_reason: opt_str(map, "offer_reason").unwrap_or_default(),
        tags: string_list(map.get("tags"))?,
        acceptance_window: opt_i64(map, "acceptance_window").unwrap_or(10),
    }))
}

fn active_value(contract: &ActiveContract) -> Value {
    json_obj(&[
        ("offer_id", Value::from(contract.offer_id.as_str())),
        ("template_id", Value::from(contract.template_id.as_str())),
        ("family", Value::from(contract.family.as_str())),
        ("title", Value::from(contract.title.as_str())),
        ("accepted_day", Value::from(contract.accepted_day)),
        ("deadline_day", Value::from(contract.deadline_day)),
        (
            "destination_port_id",
            Value::from(contract.destination_port_id.as_str()),
        ),
        ("good_id", Value::from(contract.good_id.as_str())),
        ("required_quantity", Value::from(contract.required_quantity)),
        (
            "delivered_quantity",
            Value::from(contract.delivered_quantity),
        ),
        ("reward_silver", Value::from(contract.reward_silver)),
        ("bonus_reward", Value::from(contract.bonus_reward)),
        ("source_region", opt_string_value(&contract.source_region)),
        ("source_port", opt_string_value(&contract.source_port)),
        (
            "inspection_modifier",
            f64_value(contract.inspection_modifier),
        ),
        ("status", Value::from(contract.status.as_str())),
    ])
}

fn active_from(value: Option<&Value>) -> Option<Vec<ActiveContract>> {
    let Some(value) = value else {
        return Some(Vec::new());
    };
    let mut contracts = Vec::new();
    for item in value.as_array()? {
        if let Some(contract) = active_one(item)? {
            contracts.push(contract);
        }
    }
    Some(contracts)
}

fn active_one(value: &Value) -> Option<Option<ActiveContract>> {
    let map = value.as_object()?;
    let family = map
        .get("family")
        .and_then(Value::as_str)
        .filter(|family| CONTRACT_FAMILIES.contains(family))
        .unwrap_or("procurement");
    Some(Some(ActiveContract {
        offer_id: req_str(map, "offer_id")?,
        template_id: req_str(map, "template_id")?,
        family: family.to_string(),
        title: req_str(map, "title")?,
        accepted_day: req_i64(map, "accepted_day")?,
        deadline_day: req_i64(map, "deadline_day")?,
        destination_port_id: req_str(map, "destination_port_id")?,
        good_id: req_str(map, "good_id")?,
        required_quantity: req_i64(map, "required_quantity")?,
        delivered_quantity: opt_i64(map, "delivered_quantity").unwrap_or(0),
        reward_silver: opt_i64(map, "reward_silver").unwrap_or(0),
        bonus_reward: opt_i64(map, "bonus_reward").unwrap_or(0),
        source_region: opt_str(map, "source_region"),
        source_port: opt_str(map, "source_port"),
        inspection_modifier: opt_f64(map, "inspection_modifier").unwrap_or(0.0),
        status: opt_str(map, "status").unwrap_or_else(|| "accepted".to_string()),
    }))
}

fn outcome_value(outcome: &ContractOutcome) -> Value {
    let mut map = Map::new();
    map.insert(
        "contract_id".to_string(),
        Value::from(outcome.contract_id.as_str()),
    );
    map.insert(
        "outcome_type".to_string(),
        Value::from(outcome.outcome_type.as_str()),
    );
    map.insert(
        "silver_delta".to_string(),
        Value::from(outcome.silver_delta),
    );
    map.insert("trust_delta".to_string(), Value::from(outcome.trust_delta));
    map.insert(
        "standing_delta".to_string(),
        Value::from(outcome.standing_delta),
    );
    map.insert("heat_delta".to_string(), Value::from(outcome.heat_delta));
    map.insert(
        "completion_day".to_string(),
        Value::from(outcome.completion_day),
    );
    map.insert("summary".to_string(), Value::from(outcome.summary.as_str()));
    if !outcome.family.is_empty() && CONTRACT_FAMILIES.contains(&outcome.family.as_str()) {
        map.insert("family".to_string(), Value::from(outcome.family.as_str()));
    }
    Value::Object(map)
}

fn books_outcome_value(index: usize, contract: &CompletedContract) -> Value {
    let mut map = Map::new();
    map.insert(
        "contract_id".to_string(),
        Value::from(format!("contract-{index}")),
    );
    map.insert(
        "outcome_type".to_string(),
        Value::from(contract.outcome_type.as_str()),
    );
    map.insert("silver_delta".to_string(), Value::from(0));
    map.insert("trust_delta".to_string(), Value::from(0));
    map.insert("standing_delta".to_string(), Value::from(0));
    map.insert("heat_delta".to_string(), Value::from(0));
    map.insert("completion_day".to_string(), Value::from(0));
    map.insert(
        "summary".to_string(),
        Value::from(contract.summary.as_str()),
    );
    if let Some(family) = &contract.family {
        map.insert("family".to_string(), Value::from(family.as_str()));
    }
    Value::Object(map)
}

fn outcomes_from(value: Option<&Value>) -> Option<Vec<ContractOutcome>> {
    let Some(value) = value else {
        return Some(Vec::new());
    };
    let mut outcomes = Vec::new();
    for item in value.as_array()? {
        outcomes.push(outcome_from(item)?);
    }
    Some(outcomes)
}

fn outcome_from(value: &Value) -> Option<ContractOutcome> {
    let map = value.as_object()?;
    let family = map
        .get("family")
        .and_then(Value::as_str)
        .filter(|family| CONTRACT_FAMILIES.contains(family))
        .unwrap_or("")
        .to_string();
    Some(ContractOutcome {
        contract_id: req_str(map, "contract_id")?,
        outcome_type: req_str(map, "outcome_type")?,
        silver_delta: opt_i64(map, "silver_delta").unwrap_or(0),
        trust_delta: opt_i64(map, "trust_delta").unwrap_or(0),
        standing_delta: opt_i64(map, "standing_delta").unwrap_or(0),
        heat_delta: opt_i64(map, "heat_delta").unwrap_or(0),
        completion_day: opt_i64(map, "completion_day").unwrap_or(0),
        summary: req_str(map, "summary")?,
        family,
        good_id: String::new(),
        required_quantity: 0,
        delivered_quantity: 0,
        destination_port_id: String::new(),
        deadline_day: 0,
        reward_silver: 0,
    })
}

fn breach_value(breach: &BreachRecord) -> Value {
    json_obj(&[
        ("contract_id", Value::from(breach.contract_id.as_str())),
        ("day", Value::from(breach.day)),
        ("port_id", Value::from(breach.port_id.as_str())),
        ("family", Value::from(breach.family.as_str())),
    ])
}

fn breaches_from(captain: &Value) -> Option<Vec<BreachRecord>> {
    let map = captain.as_object()?;
    let Some(saved) = map.get("breach_records") else {
        return Some(Vec::new());
    };
    let mut breaches = Vec::new();
    for item in saved.as_array()? {
        let item = item.as_object()?;
        breaches.push(BreachRecord {
            contract_id: req_str(item, "contract_id")?,
            day: req_i64(item, "day")?,
            port_id: req_str(item, "port_id")?,
            family: req_str(item, "family")?,
        });
    }
    Some(breaches)
}

fn opt_i64_value(value: Option<i64>) -> Value {
    match value {
        Some(value) => Value::from(value),
        None => Value::Null,
    }
}

fn opt_string_value(value: &Option<String>) -> Value {
    match value {
        Some(value) => Value::from(value.as_str()),
        None => Value::Null,
    }
}

fn infra_value(books: &HouseBooks) -> Value {
    let mut map = Map::new();
    map.insert(
        "warehouses".to_string(),
        Value::Array(
            books
                .warehouses
                .iter()
                .enumerate()
                .map(|(index, site)| {
                    json_obj(&[
                        ("id", Value::from(format!("warehouse-{index}"))),
                        ("port_id", Value::from(site.port_id.as_str())),
                        ("tier", Value::from("depot")),
                        ("capacity", Value::from(0)),
                        ("lease_cost", Value::from(0)),
                        ("upkeep_per_day", Value::from(1)),
                        ("inventory", Value::Array(Vec::new())),
                        ("opened_day", Value::from(0)),
                        ("upkeep_paid_through", Value::from(0)),
                        ("active", Value::from(site.active)),
                    ])
                })
                .collect(),
        ),
    );
    map.insert(
        "brokers".to_string(),
        Value::Array(
            books
                .brokers
                .iter()
                .map(|broker| {
                    json_obj(&[
                        ("region", Value::from(broker.region.as_str())),
                        ("tier", Value::from(broker.tier.as_str())),
                        ("opened_day", Value::from(0)),
                        ("upkeep_paid_through", Value::from(0)),
                        ("active", Value::from(broker.active)),
                    ])
                })
                .collect(),
        ),
    );
    map.insert(
        "licenses".to_string(),
        Value::Array(
            books
                .licenses
                .iter()
                .map(|license| {
                    json_obj(&[
                        ("license_id", Value::from(license.license_id.as_str())),
                        ("purchased_day", Value::from(0)),
                        ("upkeep_paid_through", Value::from(0)),
                        ("active", Value::from(license.active)),
                    ])
                })
                .collect(),
        ),
    );
    let policies = (0..books.policies.max(0))
        .map(|index| {
            json_obj(&[
                ("id", Value::from(format!("policy-{index}"))),
                ("spec_id", Value::from(format!("policy-{index}"))),
                ("family", Value::from("hull")),
                ("scope", Value::from("next_voyage")),
                ("purchased_day", Value::from(0)),
                ("coverage_pct", f64_value(0.5)),
                ("coverage_cap", Value::from(100)),
                ("premium_paid", Value::from(0)),
                ("target_id", Value::from("")),
                ("claims_made", Value::from(0)),
                ("total_paid_out", Value::from(0)),
                ("active", Value::from(true)),
                ("voyage_origin", Value::from("")),
                ("voyage_destination", Value::from("")),
            ])
        })
        .collect();
    map.insert("policies".to_string(), Value::Array(policies));
    map.insert("claims".to_string(), Value::Array(Vec::new()));
    if let Some(credit) = &books.credit {
        map.insert(
            "credit".to_string(),
            json_obj(&[
                ("tier", Value::from("none")),
                ("credit_limit", Value::from(0)),
                ("outstanding", Value::from(0)),
                ("interest_accrued", Value::from(0)),
                ("last_interest_day", Value::from(0)),
                ("next_due_day", Value::from(0)),
                ("defaults", Value::from(credit.defaults)),
                ("total_borrowed", Value::from(credit.total_borrowed)),
                ("total_repaid", Value::from(0)),
                ("active", Value::from(true)),
            ]),
        );
    }
    Value::Object(map)
}

fn apply_infra(books: &mut HouseBooks, value: &Value) -> Option<()> {
    let map = value.as_object()?;
    if let Some(saved) = map.get("warehouses") {
        for item in saved.as_array()? {
            let item = item.as_object()?;
            books.warehouses.push(WarehouseSite {
                port_id: req_str(item, "port_id")?,
                active: opt_bool(item, "active").unwrap_or(true),
            });
        }
    }
    if let Some(saved) = map.get("brokers") {
        for item in saved.as_array()? {
            let item = item.as_object()?;
            let tier = item
                .get("tier")
                .and_then(Value::as_str)
                .filter(|tier| BROKER_TIERS.contains(tier))
                .unwrap_or("none");
            books.brokers.push(BrokerSite {
                region: req_str(item, "region")?,
                tier: tier.to_string(),
                active: opt_bool(item, "active").unwrap_or(true),
            });
        }
    }
    if let Some(saved) = map.get("licenses") {
        for item in saved.as_array()? {
            let item = item.as_object()?;
            books.licenses.push(ActiveLicense {
                license_id: req_str(item, "license_id")?,
                active: opt_bool(item, "active").unwrap_or(true),
            });
        }
    }
    if let Some(saved) = map.get("policies") {
        let mut count = 0i64;
        for item in saved.as_array()? {
            let item = item.as_object()?;
            let family = item.get("family").and_then(Value::as_str);
            let scope = item.get("scope").and_then(Value::as_str);
            if family.is_some_and(|family| POLICY_FAMILIES.contains(&family))
                && scope.is_some_and(|scope| POLICY_SCOPES.contains(&scope))
            {
                count += 1;
            }
        }
        books.policies = count;
    }
    if let Some(credit) = truthy(map.get("credit")) {
        let credit = credit.as_object()?;
        books.credit = Some(CreditBook {
            total_borrowed: opt_i64(credit, "total_borrowed").unwrap_or(0),
            defaults: opt_i64(credit, "defaults").unwrap_or(0),
        });
    }
    Some(())
}

fn campaign_value(books: &HouseBooks) -> Value {
    json_obj(&[
        ("completed", Value::Array(Vec::new())),
        (
            "completed_paths",
            Value::Array(
                books
                    .completed_paths
                    .iter()
                    .map(|record| {
                        json_obj(&[
                            ("path_id", Value::from(record.path_id.as_str())),
                            ("completion_day", Value::from(record.completion_day)),
                            ("summary", Value::from(record.summary.as_str())),
                            ("is_first", Value::from(record.is_first)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn paths_from(value: &Value) -> Option<Vec<VictoryRecord>> {
    let map = value.as_object()?;
    let Some(saved) = map.get("completed_paths") else {
        return Some(Vec::new());
    };
    let mut paths = Vec::new();
    for item in saved.as_array()? {
        let item = item.as_object()?;
        paths.push(VictoryRecord {
            path_id: req_str(item, "path_id")?,
            completion_day: opt_i64(item, "completion_day").unwrap_or(0),
            summary: opt_str(item, "summary").unwrap_or_default(),
            is_first: opt_bool(item, "is_first").unwrap_or(false),
        });
    }
    Some(paths)
}

fn truthy(value: Option<&Value>) -> Option<&Value> {
    value.filter(|value| match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(|number| number != 0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(map) => !map.is_empty(),
    })
}

fn string_list(value: Option<&Value>) -> Option<Vec<String>> {
    let Some(value) = value else {
        return Some(Vec::new());
    };
    let mut ids = Vec::new();
    for item in value.as_array()? {
        if let Some(item) = item.as_str() {
            ids.push(item.to_string());
        }
    }
    Some(ids)
}

fn json_obj(pairs: &[(&str, Value)]) -> Value {
    let mut map = Map::new();
    for (key, value) in pairs {
        map.insert((*key).to_string(), value.clone());
    }
    Value::Object(map)
}

fn f64_value(number: f64) -> Value {
    serde_json::Number::from_f64(number)
        .map(Value::Number)
        .unwrap_or(Value::from(0))
}

fn seed_value(seed: i128) -> Value {
    if let Ok(seed) = i64::try_from(seed) {
        return Value::from(seed);
    }
    if let Ok(seed) = u64::try_from(seed) {
        return Value::from(seed);
    }
    Value::from(seed.to_string())
}

fn json_i64(value: &Value) -> Option<i64> {
    if let Some(number) = value.as_i64() {
        return Some(number);
    }
    if let Some(number) = value.as_u64() {
        return i64::try_from(number).ok();
    }
    let number = value.as_f64()?;
    if number.is_finite()
        && number.fract() == 0.0
        && (i64::MIN as f64..=i64::MAX as f64).contains(&number)
    {
        Some(number as i64)
    } else {
        None
    }
}

fn json_i128(value: &Value) -> Option<i128> {
    if let Some(number) = value.as_i64() {
        return Some(i128::from(number));
    }
    if let Some(number) = value.as_u64() {
        return Some(i128::from(number));
    }
    if let Some(text) = value.as_str() {
        return text.parse().ok();
    }
    let number = value.as_f64()?;
    if number.is_finite()
        && number.fract() == 0.0
        && (i64::MIN as f64..=i64::MAX as f64).contains(&number)
    {
        Some(number as i128)
    } else {
        None
    }
}

fn json_f64(value: &Value) -> Option<f64> {
    value.as_f64().filter(|number| number.is_finite())
}

fn req_str(map: &Map<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(Value::as_str).map(str::to_string)
}

fn opt_str(map: &Map<String, Value>, key: &str) -> Option<String> {
    match map.get(key) {
        None => None,
        Some(value) => value.as_str().map(str::to_string),
    }
}

fn req_i64(map: &Map<String, Value>, key: &str) -> Option<i64> {
    map.get(key).and_then(json_i64)
}

fn opt_i64(map: &Map<String, Value>, key: &str) -> Option<i64> {
    match map.get(key) {
        None => None,
        Some(value) => json_i64(value),
    }
}

fn req_f64(map: &Map<String, Value>, key: &str) -> Option<f64> {
    map.get(key).and_then(json_f64)
}

fn opt_f64(map: &Map<String, Value>, key: &str) -> Option<f64> {
    match map.get(key) {
        None => None,
        Some(value) => json_f64(value),
    }
}

fn opt_bool(map: &Map<String, Value>, key: &str) -> Option<bool> {
    match map.get(key) {
        None => None,
        Some(Value::Bool(flag)) => Some(*flag),
        Some(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    fn migrate(text: &str) -> Value {
        let mut data: Value = serde_json::from_str(text).unwrap();
        migrate_save(&mut data).unwrap();
        data
    }

    #[test]
    fn filename_keeps_python_sanitizer() {
        assert_eq!(save_filename("default"), "default.json");
        assert_eq!(save_filename("a b"), "ab.json");
        assert_eq!(save_filename("@@@"), "default.json");
        assert_eq!(save_filename("west-india_1"), "west-india_1.json");
    }

    #[test]
    fn newer_version_uses_the_python_sentence() {
        let mut data = serde_json::json!({"version": 13});
        let err = migrate_save(&mut data).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Save file version 13 is newer than supported version 12. Update Portlight to load this save."
        );
    }

    #[test]
    fn gap_in_the_chain_uses_the_python_sentence() {
        let mut data = serde_json::json!({"version": 0});
        let err = migrate_save(&mut data).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Migration chain broken: reached version 0, expected 12"
        );
    }

    #[test]
    fn current_version_is_left_alone() {
        let mut data = serde_json::json!({"version": 12, "day": 3});
        migrate_save(&mut data).unwrap();
        assert_eq!(data["day"], 3);
        assert_eq!(data.as_object().unwrap().len(), 2);
    }

    #[test]
    fn each_old_version_gains_its_python_fields() {
        let v1 = migrate(
            r#"{"captain":{"standing":{"regional_standing":{"Mediterranean":4},"customs_heat":{"West Africa":1}}},"ship_note":1}"#,
        );
        assert_eq!(v1["version"], 12);
        assert!(v1.get("campaign").is_some());
        assert!(v1.get("ledger").is_some());
        assert!(v1.get("contract_board").is_some());
        assert!(v1.get("infrastructure").is_some());
        assert_eq!(
            v1["captain"]["standing"]["regional_standing"]["Mediterranean"],
            4
        );
        assert_eq!(
            v1["captain"]["standing"]["regional_standing"]["North Atlantic"],
            0
        );
        assert_eq!(
            v1["captain"]["standing"]["regional_standing"]["South Seas"],
            0
        );
        assert_eq!(v1["captain"]["standing"]["customs_heat"]["West Africa"], 1);
        assert_eq!(
            v1["captain"]["standing"]["customs_heat"]["North Atlantic"],
            0
        );
        assert!(v1.get("cultural_state").is_some());
        assert_eq!(v1["captain"]["standing"]["underworld_heat"], 0);
        assert!(v1["pirate_state"].get("bounty_board").is_some());
        assert!(v1["captain"].get("breach_records").is_some());
        assert_eq!(v1["captain"]["wanted_level"], 0);
        assert_eq!(v1["ship_note"], 1);

        let v6 = migrate(
            r#"{"version":6,"captain":{"ship":{"upgrades":["lateen","copper"],"crew":4}}}"#,
        );
        assert_eq!(v6["captain"]["ship"]["upgrades"][0]["upgrade_id"], "lateen");
        assert_eq!(v6["captain"]["ship"]["upgrades"][0]["installed_day"], 0);
        assert_eq!(v6["captain"]["ship"]["upgrades"][1]["upgrade_id"], "copper");
        assert_eq!(v6["captain"]["ship"]["upgrade_slots"], 2);
        assert_eq!(v6["captain"]["ship"]["roster"]["sailors"], 4);
        assert_eq!(v6["captain"]["ship"]["roster"]["gunners"], 0);
        assert_eq!(v6["captain"]["ship"]["morale"], 50);
        assert_eq!(v6["captain"]["ship"]["officers"], serde_json::json!([]));
        assert_eq!(v6["captain"]["fleet"], serde_json::json!([]));

        let dicts = migrate(
            r#"{"version":6,"captain":{"ship":{"upgrades":[{"upgrade_id":"lateen","installed_day":2}]}}}"#,
        );
        assert_eq!(dicts["captain"]["ship"]["upgrades"][0]["installed_day"], 2);

        let v11 = migrate(
            r#"{"version":11,"voyage":{"origin_id":"a"},"captain":{},"pirate_state":{"duels_won":2}}"#,
        );
        assert_eq!(v11["voyage"]["recent_events"], serde_json::json!([]));
        assert_eq!(v11["voyage"]["origin_id"], "a");
        assert_eq!(v11["pirate_state"]["duels_won"], 2);
        assert_eq!(v11["pirate_state"]["bounty_board"], serde_json::json!([]));
        assert_eq!(v11["captain"]["claimed_bounties"], serde_json::json!([]));
    }

    #[test]
    fn v1_migration_matches_the_python_oracle() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity/saves");
        let mut data: Value =
            serde_json::from_str(&fs::read_to_string(root.join("v1.json")).unwrap()).unwrap();
        migrate_save(&mut data).unwrap();
        let expected: Value =
            serde_json::from_str(&fs::read_to_string(root.join("v1.migrated.json")).unwrap())
                .unwrap();
        assert_eq!(data, expected);
    }
}
