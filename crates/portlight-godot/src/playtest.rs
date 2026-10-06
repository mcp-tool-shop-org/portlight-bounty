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
//!
//! [`PortlightPlaytestLens`] adds the contract strip, the Contracts board
//! cards, and the Contracts active rows to an observation. It reads the
//! drawn nodes, the same way the offered choices are read, so it reports
//! what a player can see and never touches the session.

use godot::classes::{Button, Container, Control, Label};
use godot::prelude::*;

use crate::contracts_screen::{SECTION_ACTIVE, SECTION_BOARD, SECTION_RECENT};
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
        Action::CycleTradeQty => "chart.market.qty".into(),
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
        "chart.market.qty" => Action::CycleTradeQty,
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

const STRIP_ROOT: &str = "ContractStrip";
const STRIP_ROW: &str = "ContractStripRow";
const CONTRACTS_ROOT: &str = "ContractsScreen";
const CONTRACT_LIST: &str = "ContractList";
/// The strip draws a `|` label between segments. It is not a segment.
const STRIP_SEPARATOR: &str = "|";
const DAY_REPORT_ROOT: &str = "DayReportScreen";
const DAY_REPORT_COLUMN: &str = "DayReportColumn";
const DAY_REPORT_TITLE: &str = "DayReportTitle";
const DAY_REPORT_BODY: &str = "DayReportBody";
const DAY_REPORT_FOOTER: &str = "DayReportFooter";
/// `apply_document` names each drawn section box `Section{id}`.
const DAY_REPORT_SECTION_PREFIX: &str = "Section";

/// One contract block on the Contracts screen, read from its labels and
/// its offered buttons.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ContractRow {
    /// Offer id, taken from the row's accept, complete, or abandon id.
    pub id: String,
    pub title: String,
    pub detail: String,
    pub meta: String,
    /// Playtest ids of the row's visible, enabled buttons, in draw order.
    pub actions: Vec<String>,
}

/// What the lens adds to an observation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ContractsView {
    /// Strip segments joined with ` | `. Empty when the strip is hidden.
    pub strip: String,
    /// Board cards. Empty unless the Contracts desk is open.
    pub board: Vec<ContractRow>,
    /// Active rows. Empty unless the Contracts desk is open.
    pub active: Vec<ContractRow>,
}

impl ContractsView {
    /// Lines for the observation text, before `Actions:`.
    pub(crate) fn text_lines(&self, contracts_open: bool) -> Vec<String> {
        let mut lines = Vec::new();
        if !self.strip.is_empty() {
            lines.push(format!("Contract strip: {}", self.strip));
        }
        if !contracts_open {
            return lines;
        }
        lines.push("Board:".to_string());
        if self.board.is_empty() {
            lines.push("- none".to_string());
        }
        for row in &self.board {
            lines.push(row_line(row));
        }
        lines.push("Active:".to_string());
        if self.active.is_empty() {
            lines.push("- none".to_string());
        }
        for row in &self.active {
            lines.push(row_line(row));
        }
        lines
    }
}

fn row_line(row: &ContractRow) -> String {
    let mut parts = vec![row.title.as_str()];
    for part in [row.detail.as_str(), row.meta.as_str()] {
        if !part.is_empty() {
            parts.push(part);
        }
    }
    let mut line = format!("- {}", parts.join(" — "));
    if !row.actions.is_empty() {
        line.push_str(&format!(" [{}]", row.actions.join(", ")));
    }
    line
}

/// One drawn Day's report section: its id, heading, and body lines.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DayReportSectionView {
    pub id: String,
    pub title: String,
    pub lines: Vec<String>,
}

/// The Day's report card as drawn. Only read while the card is showing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DayReportView {
    pub title: String,
    pub eyebrow: String,
    pub sections: Vec<DayReportSectionView>,
    /// Captain's week rows. Empty when the footer is hidden.
    pub footer: Vec<String>,
}

impl DayReportView {
    /// Lines for the observation text, before `Actions:`.
    pub(crate) fn text_lines(&self) -> Vec<String> {
        let mut lines = vec![format!("Day's report: {}", self.title)];
        for section in &self.sections {
            lines.push(format!("{}:", section.title));
            for line in &section.lines {
                lines.push(format!("- {line}"));
            }
        }
        if !self.footer.is_empty() {
            lines.push("Footer:".to_string());
            for line in &self.footer {
                lines.push(format!("- {line}"));
            }
        }
        lines
    }

    fn to_dictionary(&self) -> VarDictionary {
        let mut sections = VarArray::new();
        for section in &self.sections {
            let mut lines = VarArray::new();
            for line in &section.lines {
                lines.push(line.as_str());
            }
            let mut dict = vdict! {
                "id" => section.id.as_str(),
                "title" => section.title.as_str(),
            };
            dict.set("lines", &lines);
            sections.push(&dict);
        }
        let mut footer = VarArray::new();
        for line in &self.footer {
            footer.push(line.as_str());
        }
        let mut dict = vdict! {
            "title" => self.title.as_str(),
            "eyebrow" => self.eyebrow.as_str(),
        };
        dict.set("sections", &sections);
        dict.set("footer", &footer);
        dict
    }
}

/// A section from its box name (`Section{id}`) and its label texts in draw
/// order: heading first, then the lines. Other nodes are not sections.
pub(crate) fn day_report_section(name: &str, labels: &[String]) -> Option<DayReportSectionView> {
    let id = name.strip_prefix(DAY_REPORT_SECTION_PREFIX)?;
    if id.is_empty() {
        return None;
    }
    let (title, lines) = labels.split_first()?;
    Some(DayReportSectionView {
        id: id.to_string(),
        title: title.clone(),
        lines: lines.to_vec(),
    })
}

/// Strip segments without the drawn separators.
pub(crate) fn strip_text(labels: &[String]) -> String {
    labels
        .iter()
        .map(|text| text.trim())
        .filter(|text| !text.is_empty() && *text != STRIP_SEPARATOR)
        .collect::<Vec<_>>()
        .join(" | ")
}

/// The offer id a row's button names, if it is a contract id.
pub(crate) fn contract_id_of(action: &str) -> Option<&str> {
    for prefix in [
        "contracts.accept.",
        "contracts.complete.",
        "contracts.abandon.",
    ] {
        if let Some(rest) = action.strip_prefix(prefix) {
            if rest != "confirm" && rest != "cancel" && !rest.is_empty() {
                return Some(rest);
            }
        }
    }
    None
}

/// A row from a block's label texts (title, detail, meta) and button ids.
pub(crate) fn contract_row(labels: &[String], actions: Vec<String>) -> ContractRow {
    let id = actions
        .iter()
        .find_map(|action| contract_id_of(action))
        .unwrap_or_default()
        .to_string();
    let field = |index: usize| labels.get(index).cloned().unwrap_or_default();
    ContractRow {
        id,
        title: field(0),
        detail: field(1),
        meta: field(2),
        actions,
    }
}

/// Which Contracts list section a row belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ListSection {
    Board,
    Active,
    Recent,
}

/// The section a heading label starts, keyed on the same texts the desk
/// draws. Any other label text is not a heading.
pub(crate) fn list_section(heading: &str) -> Option<ListSection> {
    match heading {
        SECTION_BOARD => Some(ListSection::Board),
        SECTION_ACTIVE => Some(ListSection::Active),
        SECTION_RECENT => Some(ListSection::Recent),
        _ => None,
    }
}

/// One child of the Contracts list, in draw order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ListItem {
    Label(String),
    Row(ContractRow),
}

/// Board and Active rows by the heading drawn above them. Rows before any
/// heading and Recent rows are dropped.
pub(crate) fn split_rows(items: Vec<ListItem>) -> (Vec<ContractRow>, Vec<ContractRow>) {
    let mut board = Vec::new();
    let mut active = Vec::new();
    let mut section = None;
    for item in items {
        match item {
            ListItem::Label(text) => {
                if let Some(next) = list_section(&text) {
                    section = Some(next);
                }
            }
            ListItem::Row(row) => match section {
                Some(ListSection::Board) => board.push(row),
                Some(ListSection::Active) => active.push(row),
                Some(ListSection::Recent) | None => {}
            },
        }
    }
    (board, active)
}

/// Insert lines before the `Actions:` line, or append when there is none.
pub(crate) fn splice_text(text: &str, extra: &[String]) -> String {
    if extra.is_empty() {
        return text.to_string();
    }
    let mut lines: Vec<&str> = text.split('\n').collect();
    let at = lines
        .iter()
        .position(|line| *line == "Actions:")
        .unwrap_or(lines.len());
    let extra_refs: Vec<&str> = extra.iter().map(String::as_str).collect();
    lines.splice(at..at, extra_refs);
    lines.join("\n")
}

/// Read-only lens for the playtest bridge. `playtest_bridge.gd` passes every
/// observation through [`PortlightPlaytestLens::augment`].
#[derive(GodotClass)]
#[class(init, base = RefCounted)]
pub(crate) struct PortlightPlaytestLens {}

#[godot_api]
impl PortlightPlaytestLens {
    /// Adds `state.contract_strip`, `state.contract_board`,
    /// `state.contracts_active`, and `state.day_report` (null unless the
    /// card is showing), and the same facts as text lines before `Actions:`.
    /// Returns the observation unchanged when `game` is null.
    ///
    /// `VarDictionary::clone()` shares the dictionary; it is not a copy. With
    /// a game node, the returned dictionary is the caller's `observation`,
    /// and it and its nested `state` are updated in place. The bridge builds
    /// a fresh observation for every call, so nothing else sees the change.
    #[func]
    fn augment(game: Option<Gd<Node>>, observation: VarDictionary) -> VarDictionary {
        let Some(game) = game else {
            return observation;
        };
        let contracts_open = find_named(&game, CONTRACTS_ROOT)
            .and_then(|node| node.try_cast::<Control>().ok())
            .is_some_and(|control| control.is_visible_in_tree());
        let view = read_contracts_view(&game, contracts_open);
        let day_report = read_day_report_view(&game);

        let mut result = observation.clone();
        let mut state = observation
            .get("state")
            .and_then(|value| value.try_to::<VarDictionary>().ok())
            .unwrap_or_default();
        state.set("contract_strip", view.strip.as_str());
        state.set("contract_board", &rows_array(&view.board));
        state.set("contracts_active", &rows_array(&view.active));
        match &day_report {
            Some(report) => state.set("day_report", &report.to_dictionary()),
            None => state.set("day_report", &Variant::nil()),
        }
        result.set("state", &state);

        let text = observation
            .get("text")
            .and_then(|value| value.try_to::<GString>().ok())
            .map(|text| text.to_string())
            .unwrap_or_default();
        let mut extra = view.text_lines(contracts_open);
        if let Some(report) = &day_report {
            extra.extend(report.text_lines());
        }
        let spliced = splice_text(&text, &extra);
        result.set("text", spliced.as_str());
        result
    }
}

fn rows_array(rows: &[ContractRow]) -> VarArray {
    let mut out = VarArray::new();
    for row in rows {
        let mut actions = VarArray::new();
        for action in &row.actions {
            actions.push(action.as_str());
        }
        let mut dict = vdict! {
            "id" => row.id.as_str(),
            "title" => row.title.as_str(),
            "detail" => row.detail.as_str(),
            "meta" => row.meta.as_str(),
        };
        dict.set("actions", &actions);
        out.push(&dict);
    }
    out
}

fn read_contracts_view(game: &Gd<Node>, contracts_open: bool) -> ContractsView {
    let mut view = ContractsView::default();
    if let Some(root) = find_named(game, STRIP_ROOT) {
        let shown = root
            .clone()
            .try_cast::<Control>()
            .is_ok_and(|control| control.is_visible_in_tree());
        if shown {
            let row = find_named(&root, STRIP_ROW).unwrap_or(root);
            let mut labels = Vec::new();
            collect_labels(&row, &mut labels);
            view.strip = strip_text(&labels);
        }
    }
    if !contracts_open {
        return view;
    }
    // The list keeps its last build while the desk is closed, so it is read
    // only while the desk is open.
    let Some(list) = find_named(game, CONTRACT_LIST) else {
        return view;
    };
    let mut items = Vec::new();
    for child in list.get_children().iter_shared() {
        if let Ok(label) = child.clone().try_cast::<Label>() {
            items.push(ListItem::Label(label.get_text().to_string()));
            continue;
        }
        if child.clone().try_cast::<Container>().is_err() {
            continue;
        }
        let mut labels = Vec::new();
        collect_labels(&child, &mut labels);
        let mut actions = Vec::new();
        collect_actions(&child, &mut actions);
        items.push(ListItem::Row(contract_row(&labels, actions)));
    }
    (view.board, view.active) = split_rows(items);
    view
}

fn read_day_report_view(game: &Gd<Node>) -> Option<DayReportView> {
    let root = find_named(game, DAY_REPORT_ROOT)?;
    let shown = root
        .clone()
        .try_cast::<Control>()
        .is_ok_and(|control| control.is_visible_in_tree());
    if !shown {
        return None;
    }
    let mut view = DayReportView::default();
    if let Some(title) =
        find_named(&root, DAY_REPORT_TITLE).and_then(|node| node.try_cast::<Label>().ok())
    {
        view.title = title.get_text().to_string();
    }
    // The eyebrow is the column's first label, above the title.
    if let Some(column) = find_named(&root, DAY_REPORT_COLUMN) {
        view.eyebrow = column
            .get_children()
            .iter_shared()
            .find_map(|child| child.try_cast::<Label>().ok())
            .map(|label| label.get_text().to_string())
            .unwrap_or_default();
    }
    if let Some(body) = find_named(&root, DAY_REPORT_BODY) {
        for child in body.get_children().iter_shared() {
            let mut labels = Vec::new();
            collect_labels(&child, &mut labels);
            let name = child.get_name().to_string();
            if let Some(section) = day_report_section(&name, &labels) {
                view.sections.push(section);
            }
        }
    }
    if let Some(footer) = find_named(&root, DAY_REPORT_FOOTER) {
        let footer_shown = footer
            .clone()
            .try_cast::<Control>()
            .is_ok_and(|control| control.is_visible_in_tree());
        if footer_shown {
            collect_labels(&footer, &mut view.footer);
        }
    }
    Some(view)
}

fn find_named(root: &Gd<Node>, name: &str) -> Option<Gd<Node>> {
    // Built in code, so these nodes have no owner. `owned` must be false.
    root.find_child_ex(name).recursive(true).owned(false).done()
}

fn collect_labels(node: &Gd<Node>, out: &mut Vec<String>) {
    if let Ok(label) = node.clone().try_cast::<Label>() {
        if label.is_visible_in_tree() {
            let text = label.get_text().to_string();
            if !text.is_empty() {
                out.push(text);
            }
        }
    }
    for child in node.get_children().iter_shared() {
        collect_labels(&child, out);
    }
}

fn collect_actions(node: &Gd<Node>, out: &mut Vec<String>) {
    if let Ok(button) = node.clone().try_cast::<Button>() {
        if button.is_visible_in_tree() && !button.is_disabled() && button.has_meta("playtest_id") {
            if let Ok(id) = button.get_meta("playtest_id").try_to::<GString>() {
                let id = id.to_string();
                if !id.is_empty() && !out.contains(&id) {
                    out.push(id);
                }
            }
        }
    }
    for child in node.get_children().iter_shared() {
        collect_actions(&child, out);
    }
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
        roundtrip_action(Action::CycleTradeQty);
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

    #[test]
    fn strip_drops_separators_and_hides_when_empty() {
        let labels = vec![
            "Grain run - 0/5 - 4 days left".to_string(),
            "|".to_string(),
            "+1 more".to_string(),
        ];
        assert_eq!(
            strip_text(&labels),
            "Grain run - 0/5 - 4 days left | +1 more"
        );
        assert_eq!(strip_text(&[]), "");
    }

    #[test]
    fn rows_take_the_offer_id_from_their_buttons() {
        let labels = vec![
            "Grain for Al-Manar".to_string(),
            "Grain x5 to Al-Manar   120 silver".to_string(),
            "9 days left   trust unproven   Shortage".to_string(),
        ];
        let board = contract_row(&labels, vec!["contracts.accept.abc123".into()]);
        assert_eq!(board.id, "abc123");
        assert_eq!(board.title, "Grain for Al-Manar");
        assert_eq!(board.meta, "9 days left   trust unproven   Shortage");
        let active = contract_row(
            &labels[..2],
            vec![
                "contracts.complete.abc123".into(),
                "contracts.abandon.abc123".into(),
            ],
        );
        assert_eq!(active.id, "abc123");
        assert_eq!(active.meta, "");
        assert_eq!(contract_id_of("contracts.abandon.confirm"), None);
        assert_eq!(contract_id_of("chart.contracts.open"), None);
        let blind = contract_row(&labels, Vec::new());
        assert_eq!(blind.id, "");
    }

    #[test]
    fn board_and_active_split_on_the_desk_heading_texts() {
        // The lens keys on the same constants the Contracts desk draws.
        assert_eq!(SECTION_BOARD, "Board");
        assert_eq!(SECTION_ACTIVE, "Active");
        assert_eq!(SECTION_RECENT, "Recent");
        assert_eq!(list_section(SECTION_BOARD), Some(ListSection::Board));
        assert_eq!(list_section(SECTION_ACTIVE), Some(ListSection::Active));
        assert_eq!(list_section(SECTION_RECENT), Some(ListSection::Recent));
        assert_eq!(list_section("No active contracts."), None);
        let row =
            |id: &str| contract_row(&[id.to_string()], vec![format!("contracts.accept.{id}")]);
        let items = vec![
            ListItem::Row(row("orphan")),
            ListItem::Label(SECTION_BOARD.into()),
            ListItem::Row(row("b1")),
            ListItem::Label("No offers at this port.".into()),
            ListItem::Row(row("b2")),
            ListItem::Label(SECTION_ACTIVE.into()),
            ListItem::Row(row("a1")),
            ListItem::Label(SECTION_RECENT.into()),
            ListItem::Row(row("r1")),
        ];
        let (board, active) = split_rows(items);
        let ids = |rows: &[ContractRow]| rows.iter().map(|row| row.id.clone()).collect::<Vec<_>>();
        assert_eq!(ids(&board), vec!["b1", "b2"]);
        assert_eq!(ids(&active), vec!["a1"]);
        let (none_board, none_active) = split_rows(vec![ListItem::Row(row("x"))]);
        assert!(none_board.is_empty() && none_active.is_empty());
    }

    #[test]
    fn day_report_sections_come_from_named_boxes() {
        let labels = vec![
            "Arrival".to_string(),
            "Grain run - 0/8 - 5 days left - sell 8 more Grain here".to_string(),
            "Grain 12 to 16 (+33%)".to_string(),
        ];
        let section = day_report_section("Sectionarrival", &labels).unwrap();
        assert_eq!(section.id, "arrival");
        assert_eq!(section.title, "Arrival");
        assert_eq!(section.lines, labels[1..].to_vec());
        assert_eq!(day_report_section("DayReportFooter", &labels), None);
        assert_eq!(day_report_section("Section", &labels), None);
        assert_eq!(day_report_section("Sectionhealth", &[]), None);
        let heading_only = day_report_section("Sectionhealth", &labels[..1]).unwrap();
        assert!(heading_only.lines.is_empty());
    }

    #[test]
    fn day_report_text_lines_go_before_actions() {
        let view = DayReportView {
            title: "Arrived - Al-Manar".into(),
            eyebrow: "Day's report".into(),
            sections: vec![
                DayReportSectionView {
                    id: "arrival".into(),
                    title: "Arrival".into(),
                    lines: vec!["Grain 12 to 16 (+33%)".into()],
                },
                DayReportSectionView {
                    id: "bounties".into(),
                    title: "Bounties".into(),
                    lines: vec!["Claim ready: Raj the Quiet (120 silver) - open Hunt.".into()],
                },
            ],
            footer: vec!["Week: +40 silver".into()],
        };
        let lines = view.text_lines();
        assert_eq!(
            lines,
            vec![
                "Day's report: Arrived - Al-Manar",
                "Arrival:",
                "- Grain 12 to 16 (+33%)",
                "Bounties:",
                "- Claim ready: Raj the Quiet (120 silver) - open Hunt.",
                "Footer:",
                "- Week: +40 silver",
            ]
        );
        let bare = DayReportView {
            title: "Day 3".into(),
            ..DayReportView::default()
        };
        assert_eq!(bare.text_lines(), vec!["Day's report: Day 3"]);
        let text = "Screen: day-report\nActions:\nchart.day_report.close — Close";
        assert_eq!(
            splice_text(text, &bare.text_lines()),
            "Screen: day-report\nDay's report: Day 3\nActions:\nchart.day_report.close — Close"
        );
    }

    #[test]
    fn text_lines_name_strip_board_and_active_before_actions() {
        let view = ContractsView {
            strip: "Grain run - 0/5 - 4 days left - to Al-Manar".into(),
            board: vec![contract_row(
                &[
                    "Spice run".into(),
                    "Spice x3 to Corsair's Rest   90 silver".into(),
                ],
                vec!["contracts.accept.s1".into()],
            )],
            active: Vec::new(),
        };
        let closed = view.text_lines(false);
        assert_eq!(
            closed,
            vec!["Contract strip: Grain run - 0/5 - 4 days left - to Al-Manar".to_string()]
        );
        let open = view.text_lines(true);
        assert_eq!(open[1], "Board:");
        assert_eq!(
            open[2],
            "- Spice run — Spice x3 to Corsair's Rest   90 silver [contracts.accept.s1]"
        );
        assert_eq!(open[3], "Active:");
        assert_eq!(open[4], "- none");
        assert!(ContractsView::default().text_lines(false).is_empty());

        let text = "Screen: chart\nstatus\nActions:\nchart.hire — Hire";
        let spliced = splice_text(text, &closed);
        assert_eq!(
            spliced,
            "Screen: chart\nstatus\nContract strip: Grain run - 0/5 - 4 days left - to Al-Manar\nActions:\nchart.hire — Hire"
        );
        assert_eq!(splice_text(text, &[]), text);
        assert_eq!(splice_text("x", &["y".into()]), "x\ny");
    }
}
