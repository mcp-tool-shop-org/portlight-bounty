//! Choice ids for the playtest bridge.
//!
//! A button's `playtest_id` meta and [`parse_playtest_id`] are the same
//! strings, so a `choose` applies the action the button would have called.
//!
//! Six tip rows are named by the harness and must stay spelled this way:
//! `chart.hire`, `chart.day_report.close`, `pier.hunt.open`, `pier.hunt.close`,
//! `pier.crew.open`, `pier.crew.close`. Every other id is a local scheme.
//! The choice-id contract file was not on disk or in the handoff PR, so these
//! strings are not a recovered contract. Do not treat them as locked.

use crate::game::{Action, ShipyardArm, Stance};
use crate::harbour_screen::HarbourIntent;
use crate::hunt_screen::HuntAction;
use crate::logic::PointPool;

pub(crate) enum PlaytestCommand {
    Action(Action),
    Hunt(HuntAction),
}

pub(crate) fn action_playtest_id(action: &Action) -> String {
    match action {
        Action::NewGame => "chart.new_game".into(),
        Action::SaveGame => "chart.save".into(),
        Action::OpenCaptains => "newgame.captains".into(),
        Action::OpenCustom => "newgame.custom".into(),
        Action::OpenLoad => "newgame.load".into(),
        Action::NewgameBack => "newgame.back".into(),
        Action::StartCaptain(id) => format!("newgame.start.{id}"),
        Action::BeginCustom => "newgame.begin_custom".into(),
        Action::AdjustPoints(pool, delta) => {
            format!("newgame.points.{}.{}", pool_slug(*pool), delta)
        }
        Action::CycleRegion(delta) => format!("newgame.cycle.region.{delta}"),
        Action::CyclePort(delta) => format!("newgame.cycle.port.{delta}"),
        Action::CycleBloc(delta) => format!("newgame.cycle.bloc.{delta}"),
        Action::CycleFaction(delta) => format!("newgame.cycle.faction.{delta}"),
        Action::CycleMentor(delta) => format!("newgame.cycle.mentor.{delta}"),
        Action::LoadSlot(slot) => format!("newgame.load.{slot}"),
        Action::NextDay => "chart.next_day".into(),
        Action::Work => "chart.work".into(),
        Action::OpenHunt => "pier.hunt.open".into(),
        Action::ToggleMarket => "chart.market".into(),
        Action::HireSailor => "chart.hire".into(),
        Action::Provision => "chart.provisions".into(),
        Action::OpenCrew => "pier.crew.open".into(),
        Action::CloseCrew => "pier.crew.close".into(),
        Action::CrewHire { role, count } => format!("crew.hire.{role}.{count}"),
        Action::CrewFire { role, count } => format!("crew.fire.{role}.{count}"),
        Action::CrewProvision(count) => format!("crew.provision.{count}"),
        Action::CrewTrain(role) => format!("crew.train.{role}"),
        Action::CrewSkill(id) => format!("crew.skill.{id}"),
        Action::CrewRecruit(id) => format!("crew.recruit.{id}"),
        Action::CrewConfirm => "crew.confirm".into(),
        Action::CrewCancel => "crew.cancel".into(),
        Action::Stance(stance) => format!("encounter.stance.{}", stance.as_str()),
        Action::ClearStances => "encounter.clear_stances".into(),
        Action::Duel => "encounter.duel".into(),
        Action::AutoResolve => "encounter.auto_resolve".into(),
        Action::Sail(dest) => format!("chart.sail.{dest}"),
        Action::Buy(good) => format!("chart.buy.{good}"),
        Action::Sell(good) => format!("chart.sell.{good}"),
        Action::EncounterChoice(id) => format!("encounter.choice.{id}"),
        Action::Naval(id) => format!("encounter.naval.{id}"),
        Action::Board => "encounter.board".into(),
        Action::Combat(id) => format!("encounter.combat.{id}"),
        Action::Spare => "encounter.spare".into(),
        Action::Capture => "encounter.capture".into(),
        Action::TakeAll => "encounter.take_all".into(),
        Action::CaptureCrew(count) => format!("encounter.capture_crew.{count}"),
        Action::LeaveEncounter => "encounter.leave".into(),
        Action::OpenContracts => "chart.contracts.open".into(),
        Action::CloseContracts => "chart.contracts.close".into(),
        Action::RefreshContracts => "contracts.refresh".into(),
        Action::AcceptContract(id) => format!("contracts.accept.{id}"),
        Action::CompleteContract(id) => format!("contracts.complete.{id}"),
        Action::ArmAbandon(id) => format!("contracts.abandon.{id}"),
        Action::ConfirmAbandon => "contracts.abandon.confirm".into(),
        Action::CancelAbandon => "contracts.abandon.cancel".into(),
        Action::OpenShipyard => "chart.shipyard.open".into(),
        Action::CloseShipyard => "chart.shipyard.close".into(),
        Action::ShipyardRepair => "shipyard.repair".into(),
        Action::ShipyardRename => "shipyard.rename".into(),
        Action::ShipyardArm(arm) => shipyard_arm_id(arm),
        Action::ShipyardConfirm => "shipyard.confirm".into(),
        Action::ShipyardCancel => "shipyard.cancel".into(),
        Action::OpenJournal => "chart.journal.open".into(),
        Action::CloseJournal => "chart.journal.close".into(),
        Action::CloseDayReport => "chart.day_report.close".into(),
        Action::ToggleBeat(id) => format!("journal.beat.{id}"),
        Action::OpenHarbour => "chart.harbour.open".into(),
        Action::CloseHarbour => "chart.harbour.close".into(),
        Action::HarbourPrepare(intent) => format!("harbour.prepare.{}", harbour_intent_id(intent)),
        Action::HarbourConfirm => "harbour.confirm".into(),
        Action::HarbourCancel => "harbour.cancel".into(),
        Action::HarbourDeposit(id) => format!("harbour.deposit.{id}"),
        Action::HarbourWithdraw(id) => format!("harbour.withdraw.{id}"),
        Action::HarbourRepayField => "harbour.repay.field".into(),
        Action::HarbourRepayAll => "harbour.repay.all".into(),
        Action::HarbourDraw => "harbour.draw".into(),
        Action::HarbourEmergency => "harbour.emergency".into(),
    }
}

pub(crate) fn hunt_playtest_id(action: &HuntAction) -> String {
    match action {
        HuntAction::Forage => "pier.hunt.forage".into(),
        HuntAction::RefreshBoard => "pier.hunt.refresh".into(),
        HuntAction::Accept(id) => format!("pier.hunt.accept.{id}"),
        HuntAction::AskHunt(id) => format!("pier.hunt.ask.{id}"),
        HuntAction::AskClaim(id) => format!("pier.hunt.claim.{id}"),
        HuntAction::Confirm => "pier.hunt.confirm".into(),
        HuntAction::Cancel => "pier.hunt.cancel".into(),
        HuntAction::Close => "pier.hunt.close".into(),
    }
}

pub(crate) fn parse_playtest_id(id: &str) -> Option<PlaytestCommand> {
    if let Some(action) = hunt_from_id(id) {
        return Some(PlaytestCommand::Hunt(action));
    }
    action_from_id(id).map(PlaytestCommand::Action)
}

fn hunt_from_id(id: &str) -> Option<HuntAction> {
    Some(match id {
        "pier.hunt.forage" => HuntAction::Forage,
        "pier.hunt.refresh" => HuntAction::RefreshBoard,
        "pier.hunt.confirm" => HuntAction::Confirm,
        "pier.hunt.cancel" => HuntAction::Cancel,
        "pier.hunt.close" => HuntAction::Close,
        _ => {
            if let Some(rest) = id.strip_prefix("pier.hunt.accept.") {
                return Some(HuntAction::Accept(rest.to_string()));
            }
            if let Some(rest) = id.strip_prefix("pier.hunt.ask.") {
                return Some(HuntAction::AskHunt(rest.to_string()));
            }
            if let Some(rest) = id.strip_prefix("pier.hunt.claim.") {
                return Some(HuntAction::AskClaim(rest.to_string()));
            }
            return None;
        }
    })
}

fn action_from_id(id: &str) -> Option<Action> {
    if let Some(action) = static_action(id) {
        return Some(action);
    }
    if let Some(rest) = id.strip_prefix("newgame.start.") {
        return Some(Action::StartCaptain(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("newgame.load.") {
        return Some(Action::LoadSlot(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("newgame.points.") {
        let (pool, delta) = rest.rsplit_once('.')?;
        return Some(Action::AdjustPoints(pool_from(pool)?, delta.parse().ok()?));
    }
    if let Some(rest) = id.strip_prefix("newgame.cycle.") {
        let (which, delta) = rest.rsplit_once('.')?;
        let delta = delta.parse().ok()?;
        return Some(match which {
            "region" => Action::CycleRegion(delta),
            "port" => Action::CyclePort(delta),
            "bloc" => Action::CycleBloc(delta),
            "faction" => Action::CycleFaction(delta),
            "mentor" => Action::CycleMentor(delta),
            _ => return None,
        });
    }
    if let Some(rest) = id.strip_prefix("crew.hire.") {
        let (role, count) = rest.rsplit_once('.')?;
        return Some(Action::CrewHire {
            role: role.to_string(),
            count: count.parse().ok()?,
        });
    }
    if let Some(rest) = id.strip_prefix("crew.fire.") {
        let (role, count) = rest.rsplit_once('.')?;
        return Some(Action::CrewFire {
            role: role.to_string(),
            count: count.parse().ok()?,
        });
    }
    if let Some(rest) = id.strip_prefix("crew.provision.") {
        return Some(Action::CrewProvision(rest.parse().ok()?));
    }
    if let Some(rest) = id.strip_prefix("crew.train.") {
        return Some(Action::CrewTrain(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("crew.skill.") {
        return Some(Action::CrewSkill(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("crew.recruit.") {
        return Some(Action::CrewRecruit(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("encounter.stance.") {
        return Some(Action::Stance(stance_from(rest)?));
    }
    if let Some(rest) = id.strip_prefix("chart.sail.") {
        return Some(Action::Sail(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("chart.buy.") {
        return Some(Action::Buy(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("chart.sell.") {
        return Some(Action::Sell(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("encounter.choice.") {
        return Some(Action::EncounterChoice(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("encounter.naval.") {
        return Some(Action::Naval(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("encounter.combat.") {
        return Some(Action::Combat(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("encounter.capture_crew.") {
        return Some(Action::CaptureCrew(rest.parse().ok()?));
    }
    if let Some(rest) = id.strip_prefix("contracts.accept.") {
        return Some(Action::AcceptContract(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("contracts.complete.") {
        return Some(Action::CompleteContract(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("contracts.abandon.") {
        if rest == "confirm" {
            return Some(Action::ConfirmAbandon);
        }
        if rest == "cancel" {
            return Some(Action::CancelAbandon);
        }
        return Some(Action::ArmAbandon(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("shipyard.arm.") {
        return Some(Action::ShipyardArm(shipyard_arm_from(rest)?));
    }
    if let Some(rest) = id.strip_prefix("journal.beat.") {
        return Some(Action::ToggleBeat(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("harbour.prepare.") {
        return Some(Action::HarbourPrepare(harbour_intent_from(rest)?));
    }
    if let Some(rest) = id.strip_prefix("harbour.deposit.") {
        return Some(Action::HarbourDeposit(rest.to_string()));
    }
    if let Some(rest) = id.strip_prefix("harbour.withdraw.") {
        return Some(Action::HarbourWithdraw(rest.to_string()));
    }
    None
}

fn static_action(id: &str) -> Option<Action> {
    Some(match id {
        "chart.new_game" => Action::NewGame,
        "chart.save" => Action::SaveGame,
        "newgame.captains" => Action::OpenCaptains,
        "newgame.custom" => Action::OpenCustom,
        "newgame.load" => Action::OpenLoad,
        "newgame.back" => Action::NewgameBack,
        "newgame.begin_custom" => Action::BeginCustom,
        "chart.next_day" => Action::NextDay,
        "chart.work" => Action::Work,
        "pier.hunt.open" => Action::OpenHunt,
        "chart.market" => Action::ToggleMarket,
        "chart.hire" => Action::HireSailor,
        "chart.provisions" => Action::Provision,
        "pier.crew.open" => Action::OpenCrew,
        "pier.crew.close" => Action::CloseCrew,
        "crew.confirm" => Action::CrewConfirm,
        "crew.cancel" => Action::CrewCancel,
        "encounter.clear_stances" => Action::ClearStances,
        "encounter.duel" => Action::Duel,
        "encounter.auto_resolve" => Action::AutoResolve,
        "encounter.board" => Action::Board,
        "encounter.spare" => Action::Spare,
        "encounter.capture" => Action::Capture,
        "encounter.take_all" => Action::TakeAll,
        "encounter.leave" => Action::LeaveEncounter,
        "chart.contracts.open" => Action::OpenContracts,
        "chart.contracts.close" => Action::CloseContracts,
        "contracts.refresh" => Action::RefreshContracts,
        "chart.shipyard.open" => Action::OpenShipyard,
        "chart.shipyard.close" => Action::CloseShipyard,
        "shipyard.repair" => Action::ShipyardRepair,
        "shipyard.rename" => Action::ShipyardRename,
        "shipyard.confirm" => Action::ShipyardConfirm,
        "shipyard.cancel" => Action::ShipyardCancel,
        "chart.journal.open" => Action::OpenJournal,
        "chart.journal.close" => Action::CloseJournal,
        "chart.day_report.close" => Action::CloseDayReport,
        "chart.harbour.open" => Action::OpenHarbour,
        "chart.harbour.close" => Action::CloseHarbour,
        "harbour.confirm" => Action::HarbourConfirm,
        "harbour.cancel" => Action::HarbourCancel,
        "harbour.repay.field" => Action::HarbourRepayField,
        "harbour.repay.all" => Action::HarbourRepayAll,
        "harbour.draw" => Action::HarbourDraw,
        "harbour.emergency" => Action::HarbourEmergency,
        _ => return None,
    })
}

fn pool_slug(pool: PointPool) -> &'static str {
    match pool {
        PointPool::Trade => "trade",
        PointPool::Sailing => "sailing",
        PointPool::Shadow => "shadow",
        PointPool::Reputation => "reputation",
    }
}

fn pool_from(slug: &str) -> Option<PointPool> {
    Some(match slug {
        "trade" => PointPool::Trade,
        "sailing" => PointPool::Sailing,
        "shadow" => PointPool::Shadow,
        "reputation" => PointPool::Reputation,
        _ => return None,
    })
}

fn stance_from(slug: &str) -> Option<Stance> {
    Some(match slug {
        "thrust" => Stance::Thrust,
        "slash" => Stance::Slash,
        "parry" => Stance::Parry,
        _ => return None,
    })
}

fn shipyard_arm_id(arm: &ShipyardArm) -> String {
    match arm {
        ShipyardArm::Buy(id) => format!("shipyard.arm.buy.{id}"),
        ShipyardArm::Install(id) => format!("shipyard.arm.install.{id}"),
        ShipyardArm::Sell(id) => format!("shipyard.arm.sell.{id}"),
        ShipyardArm::Dock => "shipyard.arm.dock".into(),
        ShipyardArm::Board(id) => format!("shipyard.arm.board.{id}"),
    }
}

fn shipyard_arm_from(rest: &str) -> Option<ShipyardArm> {
    if rest == "dock" {
        return Some(ShipyardArm::Dock);
    }
    if let Some(id) = rest.strip_prefix("buy.") {
        return Some(ShipyardArm::Buy(id.to_string()));
    }
    if let Some(id) = rest.strip_prefix("install.") {
        return Some(ShipyardArm::Install(id.to_string()));
    }
    if let Some(id) = rest.strip_prefix("sell.") {
        return Some(ShipyardArm::Sell(id.to_string()));
    }
    if let Some(id) = rest.strip_prefix("board.") {
        return Some(ShipyardArm::Board(id.to_string()));
    }
    None
}

fn harbour_intent_id(intent: &HarbourIntent) -> String {
    match intent {
        HarbourIntent::LeaseWarehouse(id) => format!("warehouse.{id}"),
        HarbourIntent::OpenBroker { region, tier } => format!("broker.{region}.{tier}"),
        HarbourIntent::BuyLicense(id) => format!("license.{id}"),
        HarbourIntent::OpenCredit(id) => format!("credit.{id}"),
        HarbourIntent::Draw { tier, amount } => format!("draw.{tier}.{amount}"),
        HarbourIntent::Emergency(amount) => format!("emergency.{amount}"),
        HarbourIntent::BuyInsurance {
            policy_id,
            target_id,
            origin,
            destination,
        } => format!("insurance.{policy_id}.{target_id}.{origin}.{destination}"),
    }
}

fn harbour_intent_from(rest: &str) -> Option<HarbourIntent> {
    if let Some(id) = rest.strip_prefix("warehouse.") {
        return Some(HarbourIntent::LeaseWarehouse(id.to_string()));
    }
    if let Some(id) = rest.strip_prefix("license.") {
        return Some(HarbourIntent::BuyLicense(id.to_string()));
    }
    if let Some(id) = rest.strip_prefix("credit.") {
        return Some(HarbourIntent::OpenCredit(id.to_string()));
    }
    if let Some(rest) = rest.strip_prefix("broker.") {
        let (region, tier) = rest.split_once('.')?;
        return Some(HarbourIntent::OpenBroker {
            region: region.to_string(),
            tier: tier.to_string(),
        });
    }
    if let Some(rest) = rest.strip_prefix("draw.") {
        let (tier, amount) = rest.rsplit_once('.')?;
        return Some(HarbourIntent::Draw {
            tier: tier.to_string(),
            amount: amount.parse().ok()?,
        });
    }
    if let Some(amount) = rest.strip_prefix("emergency.") {
        return Some(HarbourIntent::Emergency(amount.parse().ok()?));
    }
    if let Some(rest) = rest.strip_prefix("insurance.") {
        let mut parts = rest.splitn(4, '.');
        return Some(HarbourIntent::BuyInsurance {
            policy_id: parts.next()?.to_string(),
            target_id: parts.next()?.to_string(),
            origin: parts.next()?.to_string(),
            destination: parts.next()?.to_string(),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip_action(action: Action) {
        let id = action_playtest_id(&action);
        let Some(PlaytestCommand::Action(back)) = parse_playtest_id(&id) else {
            panic!("did not parse {id}");
        };
        assert_eq!(action_playtest_id(&back), id, "{id}");
    }

    #[test]
    fn tip_rows_and_payloads_roundtrip() {
        roundtrip_action(Action::HireSailor);
        roundtrip_action(Action::CloseDayReport);
        roundtrip_action(Action::OpenHunt);
        roundtrip_action(Action::OpenCrew);
        roundtrip_action(Action::CloseCrew);
        roundtrip_action(Action::StartCaptain("merchant".into()));
        roundtrip_action(Action::Sail("al_manar".into()));
        roundtrip_action(Action::Buy("grain".into()));
        roundtrip_action(Action::AcceptContract("grain_run".into()));
        roundtrip_action(Action::AdjustPoints(PointPool::Trade, -1));
        roundtrip_action(Action::CycleRegion(1));
        roundtrip_action(Action::CrewHire {
            role: "sailor".into(),
            count: 1,
        });
        roundtrip_action(Action::Stance(Stance::Parry));
        roundtrip_action(Action::ShipyardArm(ShipyardArm::Buy("sloop".into())));
        roundtrip_action(Action::HarbourPrepare(HarbourIntent::OpenBroker {
            region: "mediterranean".into(),
            tier: "small".into(),
        }));
        roundtrip_action(Action::HarbourPrepare(HarbourIntent::BuyInsurance {
            policy_id: "hull".into(),
            target_id: "ship".into(),
            origin: "porto_novo".into(),
            destination: "al_manar".into(),
        }));
        let hunt = hunt_playtest_id(&HuntAction::Close);
        assert_eq!(hunt, "pier.hunt.close");
        assert!(matches!(
            parse_playtest_id(&hunt),
            Some(PlaytestCommand::Hunt(HuntAction::Close))
        ));
        let ask = hunt_playtest_id(&HuntAction::AskHunt("red".into()));
        assert!(matches!(
            parse_playtest_id(&ask),
            Some(PlaytestCommand::Hunt(HuntAction::AskHunt(id))) if id == "red"
        ));
    }
}
