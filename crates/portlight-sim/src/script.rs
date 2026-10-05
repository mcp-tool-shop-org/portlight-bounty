//! Scripted session over [`crate::session::Session`].
//!
//! The commands are a thin wrapper: each one calls the same public method a
//! front end would call. Parity goldens therefore cover the stepwise API.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::custom_captain;
use crate::error::SimError;
use crate::session::Session;
use crate::snapshot::{self, LogEntry, Snapshot};

pub fn run_script(script: &str) -> Snapshot {
    let (session, log) = execute(script);
    finish(session.as_ref(), log)
}

/// Run `script`, then write a version-12 slot when a game is active.
///
/// The snapshot is the state after `Session::save`, which clamps silver at
/// zero. The log is the script log and does not gain a save row.
pub fn run_and_save(script: &str, base: &Path, slot: &str) -> Result<Snapshot, SimError> {
    let (mut session, log) = execute(script);
    if let Some(session) = session.as_mut() {
        session.save(base, slot)?;
    }
    Ok(finish(session.as_ref(), log))
}

/// Snapshot a slot the way `Session::load` restores it. `Ok(None)` means the
/// file is missing or corrupt.
pub fn load_snapshot(base: &Path, slot: &str) -> Result<Option<Snapshot>, SimError> {
    let Some(session) = Session::load(base, slot)? else {
        return Ok(None);
    };
    Ok(Some(finish(Some(&session), Vec::new())))
}

fn execute(script: &str) -> (Option<Session>, Vec<LogEntry>) {
    // The `save` verb's default directory is removed when this run ends.
    // `PORTLIGHT_SAVE_ROOT` belongs to the caller and is left in place.
    let _save_dir = SaveScope::begin();
    let mut session: Option<Session> = None;
    let mut log = Vec::new();
    for line in script.lines() {
        let raw = line.trim();
        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }
        let tokens = tokenize(raw);
        let mut entry = LogEntry::new(raw);
        match dispatch(&mut session, &tokens, &mut entry) {
            Ok(()) => log.push(entry),
            Err(err) => {
                entry.error = Some(err.to_string());
                log.push(entry);
                break;
            }
        }
    }
    (session, log)
}

fn finish(session: Option<&Session>, log: Vec<LogEntry>) -> Snapshot {
    match session {
        Some(session) => snapshot::capture(
            session.world(),
            session.trade_seq(),
            session.books(),
            session.infrastructure(),
            session.narrative(),
            session.board(),
            session.receipts(),
            session.run_id(),
            log,
        ),
        None => snapshot::empty(log),
    }
}

fn dispatch(
    session: &mut Option<Session>,
    tokens: &[String],
    entry: &mut LogEntry,
) -> Result<(), SimError> {
    let cmd = tokens.first().map(String::as_str).unwrap_or("");
    match cmd {
        "new" => {
            if tokens.len() < 4 {
                return Err(SimError::UsageNew);
            }
            let captain_type = &tokens[1];
            let name = &tokens[2];
            let seed: i128 = tokens[3]
                .parse()
                .map_err(|_| SimError::InvalidNumber(tokens[3].clone()))?;
            let port = tokens.get(4).map(String::as_str);
            *session = Some(Session::new(name, captain_type, seed, port)?);
            Ok(())
        }
        "custom" => {
            let start = custom_captain::parse_script_spec(tokens)?;
            let port = start.starting_port.as_deref();
            *session = Some(Session::new_custom(&start.spec, start.seed, port)?);
            Ok(())
        }
        "buy" => {
            let session = active(session)?;
            if tokens.len() != 3 {
                return Err(SimError::UsageBuy);
            }
            let qty = parse_qty(&tokens[2])?;
            let receipt = session.buy(&tokens[1], qty)?;
            entry.receipt = Some(snapshot::from_receipt(&receipt));
            Ok(())
        }
        "sell" => {
            let session = active(session)?;
            if tokens.len() != 3 {
                return Err(SimError::UsageSell);
            }
            let qty = parse_qty(&tokens[2])?;
            let sale = session.sell(&tokens[1], qty)?;
            entry.receipt = Some(snapshot::from_receipt(&sale.receipt));
            entry.contracts = sale
                .contracts
                .iter()
                .map(snapshot::from_contract_outcome)
                .collect();
            Ok(())
        }
        "depart" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::UsageDepart);
            }
            session.depart(&tokens[1])
        }
        "advance" => {
            let session = active(session)?;
            let turn = session.advance()?;
            entry.events = turn.events.iter().map(snapshot::from_event).collect();
            entry.shocks = turn.shocks;
            entry.contracts = turn
                .contracts
                .iter()
                .map(snapshot::from_contract_outcome)
                .collect();
            entry.notes = turn.notes;
            Ok(())
        }
        "arrival_narrative" => {
            let session = active(session)?;
            entry.notes = session.arrival_narrative();
            Ok(())
        }
        "evaluate_consequences" => {
            let session = active(session)?;
            entry.notes = session
                .evaluate_consequences()
                .iter()
                .map(|row| format!("{}: {}", row.id, row.text))
                .collect();
            Ok(())
        }
        "accept_contract" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::UsageAcceptContract);
            }
            let contract = session.accept_contract(&tokens[1])?;
            entry.contracts = vec![snapshot::from_accepted(&contract)];
            Ok(())
        }
        "complete_contract" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::UsageCompleteContract);
            }
            let outcome = session.complete_contract(&tokens[1])?;
            entry.contracts = vec![snapshot::from_contract_outcome(&outcome)];
            Ok(())
        }
        "buy_infrastructure" => {
            let session = active(session)?;
            if tokens.len() < 2 {
                return Err(SimError::UsageBuyInfrastructure);
            }
            let args: Vec<&str> = tokens[2..].iter().map(String::as_str).collect();
            session.buy_infrastructure(&tokens[1], &args)
        }
        "take_credit" => {
            let session = active(session)?;
            if tokens.len() != 3 {
                return Err(SimError::UsageTakeCredit);
            }
            let amount = parse_qty(&tokens[2])?;
            session.take_credit(&tokens[1], amount)?;
            Ok(())
        }
        "buy_insurance" => {
            let session = active(session)?;
            if tokens.len() < 2 || tokens.len() > 5 {
                return Err(SimError::UsageBuyInsurance);
            }
            let target = tokens.get(2).map(String::as_str).unwrap_or("");
            let origin = tokens.get(3).map(String::as_str).unwrap_or("");
            let destination = tokens.get(4).map(String::as_str).unwrap_or("");
            session.buy_insurance(&tokens[1], target, origin, destination)
        }
        "deposit" => {
            let session = active(session)?;
            if tokens.len() != 3 {
                return Err(SimError::UsageDeposit);
            }
            let qty = parse_qty(&tokens[2])?;
            session.deposit_cargo(&tokens[1], qty)?;
            Ok(())
        }
        "withdraw" => {
            let session = active(session)?;
            if tokens.len() < 3 || tokens.len() > 4 {
                return Err(SimError::UsageWithdraw);
            }
            let qty = parse_qty(&tokens[2])?;
            let source = tokens.get(3).map(String::as_str);
            session.withdraw_cargo(&tokens[1], qty, source)?;
            Ok(())
        }
        "repay_credit" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::UsageRepayCredit);
            }
            let amount = parse_qty(&tokens[1])?;
            session.repay_credit(amount)
        }
        "save" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::Sentence("Usage: save <slot>".into()));
            }
            session.save(save_root(), &tokens[1])?;
            Ok(())
        }
        "load" => {
            if tokens.len() != 2 {
                return Err(SimError::Sentence("Usage: load <slot>".into()));
            }
            let Some(loaded) = Session::load(save_root(), &tokens[1])? else {
                return Err(SimError::Sentence(format!("No save in slot {}", tokens[1])));
            };
            *session = Some(loaded);
            Ok(())
        }
        "hire" => {
            let session = active(session)?;
            if tokens.len() < 2 || tokens.len() > 3 {
                return Err(SimError::UsageHire);
            }
            let count = parse_qty(&tokens[1])?;
            let role = tokens.get(2).map(String::as_str).unwrap_or("sailor");
            session.hire_crew(count, role)
        }
        "provision" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::UsageProvision);
            }
            let days = parse_qty(&tokens[1])?;
            session.provision(days)
        }
        "work" => {
            let session = active(session)?;
            if tokens.len() != 1 {
                return Err(SimError::UsageWork);
            }
            let earned = session.work()?;
            entry.earned = Some(earned);
            Ok(())
        }
        "duel" => {
            let session = active(session)?;
            if tokens.len() < 2 {
                return Err(SimError::UsageDuel);
            }
            let stances = split_stances(&tokens[1..]);
            let outcome = session.duel(&stances)?;
            entry.duel = Some(snapshot::from_duel(&outcome));
            Ok(())
        }
        "resolve_duel" => {
            let session = active(session)?;
            let outcome = session.resolve_pending_duel()?;
            entry.duel = Some(snapshot::from_duel(&outcome));
            Ok(())
        }
        "encounter" => {
            let session = active(session)?;
            if tokens.len() < 2 || tokens.len() > 3 {
                return Err(SimError::UsageEncounter);
            }
            let (captain_id, band) = parse_encounter_target(tokens.get(2).map(String::as_str))?;
            let step = session.encounter_choice_with(&tokens[1], captain_id.as_deref(), band)?;
            entry.encounter = Some(snapshot::from_encounter(&step));
            Ok(())
        }
        "naval" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::UsageNaval);
            }
            let step = session.naval_round(&tokens[1])?;
            entry.encounter = Some(snapshot::from_encounter(&step));
            Ok(())
        }
        "board" => {
            let session = active(session)?;
            if tokens.len() != 1 {
                return Err(SimError::UsageBoard);
            }
            let step = session.resolve_boarding()?;
            entry.encounter = Some(snapshot::from_encounter(&step));
            Ok(())
        }
        "fight" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::UsageFight);
            }
            let step = session.fight(&tokens[1])?;
            entry.encounter = Some(snapshot::from_encounter(&step));
            Ok(())
        }
        "capture" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::UsageCapture);
            }
            let crew = parse_qty(&tokens[1])?;
            let step = session.capture(crew)?;
            entry.encounter = Some(snapshot::from_encounter(&step));
            Ok(())
        }
        "train" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::UsageTrain);
            }
            session.train_crew(&tokens[1])
        }
        "recruit" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::UsageRecruit);
            }
            session.recruit_companion(&tokens[1])
        }
        "skill" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::UsageSkill);
            }
            session.spend_skill_point(&tokens[1])
        }
        "remember" => {
            let session = active(session)?;
            if tokens.len() != 3 {
                return Err(SimError::UsageRemember);
            }
            session.remember_captain(&tokens[1], &tokens[2])
        }
        "agency" => {
            let session = active(session)?;
            if tokens.len() != 1 {
                return Err(SimError::UsageAgency);
            }
            let (encounter, ambush, notices) = session.tick_sea_captain_agency();
            entry.agency = Some(snapshot::agency_log(ambush, encounter.as_ref(), &notices));
            Ok(())
        }
        "spare" => {
            let session = active(session)?;
            session.spare()?;
            Ok(())
        }
        "take_all" => {
            let session = active(session)?;
            session.take_all()?;
            Ok(())
        }
        "gear" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::Sentence("Usage: gear <id>".into()));
            }
            session.buy_gear(&tokens[1])
        }
        "buy_ship" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::Sentence("Usage: buy_ship <template_id>".into()));
            }
            session.buy_ship(&tokens[1])
        }
        "upgrade" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::Sentence("Usage: upgrade <upgrade_id>".into()));
            }
            session.install_upgrade(&tokens[1])
        }
        "form_convoy" => {
            let session = active(session)?;
            session.form_convoy()
        }
        "repair_fleet" => {
            let session = active(session)?;
            session.repair_fleet().map(|_| ())
        }
        "repair" => {
            let session = active(session)?;
            if tokens.len() > 2 {
                return Err(SimError::Sentence("Usage: repair [points]".into()));
            }
            let amount = if tokens.len() == 2 {
                Some(parse_qty(&tokens[1])?)
            } else {
                None
            };
            session.repair(amount).map(|_| ())
        }
        "rename_ship" => {
            let session = active(session)?;
            if tokens.len() < 2 || tokens.len() > 3 {
                return Err(SimError::Sentence(
                    "Usage: rename_ship <new_name> [ship]".into(),
                ));
            }
            let fleet_ship = tokens.get(2).map(String::as_str);
            session.rename_ship(&tokens[1], fleet_ship)
        }
        "dock_current_ship" => {
            let session = active(session)?;
            if tokens.len() != 1 {
                return Err(SimError::Sentence("Usage: dock_current_ship".into()));
            }
            session.dock_current_ship()
        }
        "board_fleet_ship" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::Sentence("Usage: board_fleet_ship <ship>".into()));
            }
            session.board_fleet_ship(&tokens[1])
        }
        "sell_fleet_ship" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::Sentence("Usage: sell_fleet_ship <ship>".into()));
            }
            session.sell_fleet_ship(&tokens[1]).map(|_| ())
        }
        "fire" => {
            let session = active(session)?;
            if tokens.len() < 2 || tokens.len() > 3 {
                return Err(SimError::Sentence("Usage: fire <count> [role]".into()));
            }
            let count = parse_qty(&tokens[1])?;
            let role = tokens.get(2).map(String::as_str).unwrap_or("sailor");
            session.fire_crew(count, role)
        }
        "abandon_contract" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::Sentence(
                    "Usage: abandon_contract <offer_id>".into(),
                ));
            }
            let outcome = session.abandon_contract(&tokens[1])?;
            entry.contracts = vec![snapshot::from_contract_outcome(&outcome)];
            Ok(())
        }
        "transfer" => {
            let session = active(session)?;
            if tokens.len() != 5 {
                return Err(SimError::Sentence(
                    "Usage: transfer <good> <qty> <from> <to>".into(),
                ));
            }
            let qty = parse_qty(&tokens[2])?;
            session.transfer_cargo(&tokens[1], qty, &tokens[3], &tokens[4])
        }
        "maintain" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::Sentence("Usage: maintain <weapon_id>".into()));
            }
            session.maintain_weapon(&tokens[1])
        }
        "hunt" => {
            let session = active(session)?;
            if tokens.len() != 1 {
                return Err(SimError::Sentence("Usage: hunt".into()));
            }
            let result = session.hunt()?;
            let location = if session.world().voyage.status == crate::model::VoyageStatus::AtSea {
                "sea"
            } else {
                "port"
            };
            entry.hunt = Some(snapshot::HuntLog {
                success: result.success,
                location: location.to_string(),
                provisions_gained: result.provisions_gained,
                pelts_gained: result.pelts_gained,
                silver_gained: result.silver_gained,
                morale_cost: result.morale_cost,
                crew_lost: result.crew_lost,
                hull_damage: result.hull_damage,
                flavor: result.flavor,
                danger_text: result.danger_text,
            });
            Ok(())
        }
        "bounty" => dispatch_bounty(session, tokens, entry),
        "wanted" => {
            let session = active(session)?;
            if tokens.len() != 2 {
                return Err(SimError::Sentence("Usage: wanted <level>".into()));
            }
            let level = parse_qty(&tokens[1])?;
            session.set_wanted_level(level);
            Ok(())
        }
        other => Err(SimError::UnknownCommand(other.to_string())),
    }
}

fn dispatch_bounty(
    session: &mut Option<Session>,
    tokens: &[String],
    entry: &mut LogEntry,
) -> Result<(), SimError> {
    let session = active(session)?;
    let action = tokens.get(1).map(String::as_str).unwrap_or("list");
    if action == "list" {
        if tokens.len() > 2 {
            return Err(SimError::Sentence(
                "Usage: bounty [list|accept <id>|hunt <id>|claim <id>]".into(),
            ));
        }
        let targets = session.bounty_board();
        entry.bounty = Some(snapshot::BountyLog {
            action: "list".to_string(),
            target_id: String::new(),
            reward: 0,
            targets: targets
                .into_iter()
                .map(|target| snapshot::BountyTargetSnap {
                    captain_id: target.captain_id,
                    captain_name: target.captain_name,
                    faction_id: target.faction_id,
                    region: target.region,
                    reward: target.reward,
                    difficulty: target.difficulty,
                    description: target.description,
                })
                .collect(),
        });
        return Ok(());
    }
    if tokens.len() != 3 {
        return Err(SimError::Sentence(match action {
            "accept" => "Usage: bounty accept <captain_id>".into(),
            "hunt" => "Usage: bounty hunt <captain_id>".into(),
            "claim" => "Usage: bounty claim <captain_id>".into(),
            _ => format!("Unknown bounty action: {action}. Use: list, accept, hunt, claim"),
        }));
    }
    let target_id = tokens[2].as_str();
    match action {
        "accept" => {
            session.accept_bounty(target_id)?;
            entry.bounty = Some(bounty_log("accept", target_id, 0));
            Ok(())
        }
        "hunt" => {
            session.hunt_bounty(target_id)?;
            entry.bounty = Some(bounty_log("hunt", target_id, 0));
            Ok(())
        }
        "claim" => {
            let reward = session.claim_bounty(target_id)?;
            entry.bounty = Some(bounty_log("claim", target_id, reward));
            Ok(())
        }
        _ => Err(SimError::Sentence(format!(
            "Unknown bounty action: {action}. Use: list, accept, hunt, claim"
        ))),
    }
}

fn bounty_log(action: &str, target_id: &str, reward: i64) -> snapshot::BountyLog {
    snapshot::BountyLog {
        action: action.to_string(),
        target_id: target_id.to_string(),
        reward,
        targets: Vec::new(),
    }
}

fn active(session: &mut Option<Session>) -> Result<&mut Session, SimError> {
    session.as_mut().ok_or(SimError::NoActiveGame)
}

thread_local! {
    static SAVE_ROOT: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static SAVE_ENV_GUARD: RefCell<Option<MutexGuard<'static, ()>>> = const { RefCell::new(None) };
    static SAVE_ENV_DEPTH: Cell<u32> = const { Cell::new(0) };
}

static SAVE_SEQ: AtomicU64 = AtomicU64::new(0);
static SAVE_ENV: Mutex<()> = Mutex::new(());

/// Serializes `PORTLIGHT_SAVE_ROOT` across threads. Reentrant on one thread so
/// `save` can read the variable while a test holds the lock.
struct SaveEnvLock;

impl SaveEnvLock {
    fn acquire() -> Self {
        SAVE_ENV_DEPTH.with(|depth| {
            if depth.get() == 0 {
                let guard = SAVE_ENV.lock().unwrap_or_else(|err| err.into_inner());
                SAVE_ENV_GUARD.with(|slot| *slot.borrow_mut() = Some(guard));
            }
            depth.set(depth.get() + 1);
        });
        Self
    }
}

impl Drop for SaveEnvLock {
    fn drop(&mut self) {
        SAVE_ENV_DEPTH.with(|depth| {
            let next = depth.get().saturating_sub(1);
            depth.set(next);
            if next == 0 {
                SAVE_ENV_GUARD.with(|slot| {
                    slot.borrow_mut().take();
                });
            }
        });
    }
}

/// Removes the temp directory created for a script's `save` verb.
struct SaveScope {
    owned: Option<PathBuf>,
}

impl SaveScope {
    fn begin() -> Self {
        let _lock = SaveEnvLock::acquire();
        if let Ok(dir) = std::env::var("PORTLIGHT_SAVE_ROOT") {
            if !dir.is_empty() {
                return Self { owned: None };
            }
        }
        let dir = std::env::temp_dir().join(format!(
            "portlight-script-{}-{}",
            std::process::id(),
            SAVE_SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        SAVE_ROOT.with(|slot| *slot.borrow_mut() = Some(dir.clone()));
        Self { owned: Some(dir) }
    }
}

impl Drop for SaveScope {
    fn drop(&mut self) {
        if let Some(dir) = self.owned.take() {
            let _ = std::fs::remove_dir_all(&dir);
            SAVE_ROOT.with(|slot| {
                if slot.borrow().as_ref() == Some(&dir) {
                    *slot.borrow_mut() = None;
                }
            });
        }
    }
}

/// `PORTLIGHT_SAVE_ROOT`, or the temp directory owned by the current [`SaveScope`].
fn save_root() -> PathBuf {
    let _lock = SaveEnvLock::acquire();
    if let Ok(dir) = std::env::var("PORTLIGHT_SAVE_ROOT") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    SAVE_ROOT.with(|slot| {
        slot.borrow()
            .clone()
            .expect("save verb ran outside a script")
    })
}

fn parse_encounter_target(token: Option<&str>) -> Result<(Option<String>, Option<i64>), SimError> {
    let Some(token) = token else {
        return Ok((None, None));
    };
    if let Some(rest) = token.strip_prefix("strength:") {
        let strength = rest
            .parse()
            .map_err(|_| SimError::InvalidNumber(rest.to_string()))?;
        return Ok((None, Some(strength)));
    }
    Ok((Some(token.to_string()), None))
}

fn parse_qty(token: &str) -> Result<i64, SimError> {
    token
        .parse()
        .map_err(|_| SimError::InvalidNumber(token.to_string()))
}

fn split_stances(tokens: &[String]) -> Vec<String> {
    let mut stances = Vec::new();
    for token in tokens {
        for part in token.split(',') {
            let part = part.trim();
            if !part.is_empty() {
                stances.push(part.to_string());
            }
        }
    }
    stances
}

fn tokenize(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for ch in line.chars() {
        if ch == '"' {
            quoted = !quoted;
            continue;
        }
        if ch.is_whitespace() && !quoted {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(ch);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_command_stops_the_script() {
        let snap = run_script("new merchant Ada 1\nfly\nbuy grain 1\n");
        assert_eq!(snap.log.len(), 2);
        assert_eq!(snap.log[1].error.as_deref(), Some("Unknown command: fly"));
        assert_eq!(snap.captain.silver, 550);
    }

    #[test]
    fn save_verb_removes_its_temp_dir() {
        let _lock = SaveEnvLock::acquire();
        let previous = std::env::var("PORTLIGHT_SAVE_ROOT").ok();
        struct Restore(Option<String>);
        impl Drop for Restore {
            fn drop(&mut self) {
                match self.0.take() {
                    Some(dir) => std::env::set_var("PORTLIGHT_SAVE_ROOT", dir),
                    None => std::env::remove_var("PORTLIGHT_SAVE_ROOT"),
                }
            }
        }
        let _restore = Restore(previous);
        std::env::remove_var("PORTLIGHT_SAVE_ROOT");
        let slot = format!("parity-save-{}", std::process::id());
        let snap = run_script(&format!("new merchant Ada 1\nsave {slot}\nload {slot}\n"));
        let leaked = std::fs::read_dir(std::env::temp_dir())
            .unwrap()
            .flatten()
            .any(|entry| {
                entry
                    .path()
                    .join("saves")
                    .join(format!("{slot}.json"))
                    .is_file()
            });
        assert!(
            snap.log.iter().all(|entry| entry.error.is_none()),
            "{:?}",
            snap.log
                .iter()
                .filter_map(|entry| entry.error.as_ref())
                .collect::<Vec<_>>()
        );
        assert!(!leaked, "save verb left {slot} in the temp directory");
    }

    #[test]
    fn seed_above_i64_is_accepted() {
        let snap = run_script("new merchant Ada 9223372036854775808\n");
        assert!(snap.log[0].error.is_none(), "{:?}", snap.log[0].error);
        assert_eq!(snap.seed, 1i128 << 63);
    }
}
