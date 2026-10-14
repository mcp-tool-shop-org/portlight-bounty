//! Pure recap model: the rows a combat recap block and a contract recap card
//! draw, built from plain values the Session already returned. No node and no
//! Godot type, so every row is tested without a Session. Nothing here is
//! saved. The screens wire it in later slices.
#![allow(dead_code)]

use portlight_sim::model::ContractOutcome;

use crate::contracts_screen::{good_display_name, port_display_name, progress_text, trim_tail};
use crate::logic::{ascii_punctuation, faction_name};

/// Longest row in chars before its cause clause is dropped, then the row is
/// cut at a word.
pub(crate) const ROW_CHARS: usize = 96;
/// Body rows drawn at most. The headline, section headings and the rule note
/// do not count.
pub(crate) const BODY_ROW_CAP: usize = 9;
/// Loot goods and notes passed through at most.
pub(crate) const PASSTHROUGH_CAP: usize = 2;
/// Injury rows at most.
pub(crate) const INJURY_CAP: usize = 2;

/// The cause on the silver and goods a sinking takes.
const WRECK_CAUSE: &str = "lost with the wreck";

pub(crate) const RULE_NOTE: &str =
    "Either ship closing counts toward boarding, unless the other evades.";
pub(crate) const BOUNTY_NOTE: &str = "The bounty stays open";
/// A win against an accepted bounty target that can now be claimed.
pub(crate) const BOUNTY_READY: &str = "Bounty ready to claim - Hunt desk";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecapKind {
    Combat,
    Contract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecapTone {
    Gold,
    Cream,
    Due,
    Muted,
}

/// Where a row sits. `bridge_name` is the `state.recap` section name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecapSection {
    /// The choice row of a win.
    Choice,
    /// `Earned` or `Cost`.
    Heading,
    Earned,
    Note,
    Cost,
    /// The boarding rule; not counted in the row cap.
    RuleNote,
    Bounty,
    Net,
    Contract,
}

impl RecapSection {
    pub(crate) fn bridge_name(self) -> &'static str {
        match self {
            Self::Choice | Self::Heading => "how",
            Self::Earned | Self::Net => "earned",
            Self::Cost => "cost",
            Self::Note | Self::RuleNote | Self::Bounty => "note",
            Self::Contract => "contract",
        }
    }

    fn counts_toward_cap(self) -> bool {
        !matches!(self, Self::Heading | Self::RuleNote)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecapRow {
    pub section: RecapSection,
    pub text: String,
    pub tone: RecapTone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecapResult {
    WonSpared,
    WonTookAll,
    WonPrize,
    WonSunk,
    Lost,
    Drew,
    BrokeAway,
    Paid,
    Abandoned,
}

impl RecapResult {
    /// The `state.recap.result` id.
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::WonSpared => "won_spared",
            Self::WonTookAll => "won_took_all",
            Self::WonPrize => "won_prize",
            Self::WonSunk => "won_sunk",
            Self::Lost => "lost",
            Self::Drew => "drew",
            Self::BrokeAway => "broke_away",
            Self::Paid => "paid",
            Self::Abandoned => "abandoned",
        }
    }

    pub(crate) fn is_win(self) -> bool {
        matches!(
            self,
            Self::WonSpared | Self::WonTookAll | Self::WonPrize | Self::WonSunk
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Recap {
    pub kind: RecapKind,
    pub result: RecapResult,
    pub headline: String,
    pub headline_tone: RecapTone,
    pub rows: Vec<RecapRow>,
}

/// The one move that decided the fight, read from step data by the screen.
/// It is named in the headline and nowhere else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DecisiveMove {
    /// The last naval step that triggered boarding: progress, threshold.
    Boarded(i64, i64),
    /// Final naval step sank the enemy with this player action.
    Sank(String),
    /// Final personal-fight step's player action.
    FinalMove(String),
    /// Final naval step sank the player with this enemy action.
    SunkBy(String),
    /// The crew reached 0 without the ship sinking.
    CrewLost,
    /// The personal duel resolved as a loss.
    BestedInDuel,
    /// Fled at the approach.
    FledApproach,
    /// Fled at sea in this naval round.
    FledAtSea(i64),
}

/// Hull loss summed per cause over the fight's steps:
/// `[their broadside, their grapeshot, parting shot]`.
pub(crate) type HullByCause = [i64; 3];
/// Crew loss summed per cause: `[their grapeshot, boarding melee]`.
pub(crate) type CrewByCause = [i64; 2];

/// What a finished fight gave the screen. Amounts are positive losses in the
/// cause sums; `deltas` is the baseline diff `[silver, crew, hull, cargo]`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CombatFacts {
    pub result: Option<RecapResult>,
    /// Naval (true) or personal fight (false).
    pub sea: bool,
    /// Display name (`captain_display_name`), never the raw id.
    pub name: String,
    pub decisive: Option<DecisiveMove>,
    /// Prize ship name on a capture.
    pub prize_ship: String,
    /// Sim reason when a sinking offered no prize.
    pub no_prize_reason: String,
    /// `VictoryReceipt.silver_delta`: the purse.
    pub purse: i64,
    /// Receipt loot messages in receipt order.
    pub loot: Vec<String>,
    /// Faction id and standing change from the receipt.
    pub standing: Option<(String, i64)>,
    /// Companion reactions and departures, passthrough.
    pub notes: Vec<String>,
    pub deltas: [i64; 4],
    pub hull_by_cause: HullByCause,
    pub crew_by_cause: CrewByCause,
    pub plundered: bool,
    /// Catalog injury names.
    pub injuries: Vec<String>,
    pub reached_boarding: bool,
    pub bounty_open: bool,
    /// Won against an accepted target whose bounty is now claimable.
    pub bounty_ready: bool,
    /// The player's hull max, for the `Your hull {h}/{max}` row after a sinking.
    pub hull_max: Option<i64>,
    /// Sinking: tow port id, silver and goods lost with the wreck, and the
    /// hull after the tow (from the sim step).
    pub towed_to: String,
    pub wreck_silver: i64,
    pub wreck_cargo: Vec<(String, i64)>,
    pub hull_after: i64,
    /// Sinking: the voyage's destination before the tow (empty when unknown).
    pub planned_destination: String,
    /// Sinking: silver cargo insurance paid for the wreck.
    pub claims_paid: i64,
    /// Sinking: `fight`, `parting_shot`, `storm`, `pirates` or `event`.
    pub sink_cause: String,
    /// Sinking: full repair at the tow port, in silver (0: none needed).
    pub repair_price: i64,
}

/// Extras a contract card needs beyond the outcome: units of the good still
/// held (abandon), per outcome in the same order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ContractExtras {
    pub held: Vec<i64>,
}

fn row(section: RecapSection, text: impl Into<String>, tone: RecapTone) -> RecapRow {
    RecapRow {
        section,
        text: ascii_punctuation(&text.into()),
        tone,
    }
}

/// A row with an optional ` - cause`. Over [`ROW_CHARS`] the cause goes
/// first; a row still too long is cut at a word.
fn caused(section: RecapSection, head: String, cause: Option<&str>, tone: RecapTone) -> RecapRow {
    let full = match cause {
        Some(cause) => format!("{head} - {cause}"),
        None => head.clone(),
    };
    let text = if full.len() <= ROW_CHARS {
        full
    } else if head.len() <= ROW_CHARS {
        head
    } else {
        trim_tail(&head, ROW_CHARS)
    };
    row(section, text, tone)
}

fn passthrough(section: RecapSection, text: &str, tone: RecapTone) -> RecapRow {
    row(section, passthrough_text(text), tone)
}

/// A sim note or loot message as a row: one trailing full stop dropped (an
/// ellipsis is kept whole), and a row over [`ROW_CHARS`] cut at a word with
/// no `...`, so a trimmed row never ends in a stop.
pub(crate) fn passthrough_text(text: &str) -> String {
    let text = ascii_punctuation(text);
    let text = text.trim_end();
    let text = if text.ends_with("...") {
        text
    } else {
        text.strip_suffix('.').unwrap_or(text)
    };
    if text.len() <= ROW_CHARS {
        return text.to_string();
    }
    let mut cut = &text[..ROW_CHARS];
    if let Some(space) = cut.rfind(' ').filter(|space| *space > 0) {
        cut = &cut[..space];
    }
    cut.trim_end_matches([' ', '-', ',', ';', ':', '.'])
        .to_string()
}

fn verdict(result: RecapResult, name: &str) -> String {
    let (head, joiner) = match result {
        r if r.is_win() => ("Won", "against"),
        RecapResult::Lost => ("Lost", "to"),
        RecapResult::Drew => ("Drew", "with"),
        _ => ("Broke away", "from"),
    };
    if name.is_empty() {
        head.to_string()
    } else {
        format!("{head} {joiner} {name}")
    }
}

/// The headline clause, only for the result and move pairs the copy table
/// allows; anything else gives no clause (never an invented cause).
pub(crate) fn decisive_clause(
    result: RecapResult,
    sea: bool,
    decisive: Option<&DecisiveMove>,
) -> Option<String> {
    let decisive = decisive?;
    let win = result.is_win();
    match decisive {
        DecisiveMove::Boarded(p, t) if win && sea => Some(format!("boarded at {p}/{t}")),
        DecisiveMove::Sank(action) if win && sea => match action.as_str() {
            "broadside" => Some("sank their ship with a broadside".into()),
            "rake" => Some("sank their ship with grapeshot".into()),
            _ => None,
        },
        DecisiveMove::FinalMove(action) if win && !sea && !action.is_empty() => {
            Some(format!("final move {}", capitalize(action)))
        }
        DecisiveMove::SunkBy(action) if result == RecapResult::Lost && sea => {
            match action.as_str() {
                "broadside" => Some("sunk by their broadside".into()),
                "rake" => Some("sunk by their grapeshot".into()),
                "parting shot" => Some("sunk by a parting shot".into()),
                _ => None,
            }
        }
        DecisiveMove::CrewLost if result == RecapResult::Lost && sea => Some("crew lost".into()),
        DecisiveMove::BestedInDuel if result == RecapResult::Lost => {
            Some("bested in the duel".into())
        }
        DecisiveMove::FledApproach if result == RecapResult::BrokeAway => {
            Some("fled before the guns".into())
        }
        DecisiveMove::FledAtSea(round) if result == RecapResult::BrokeAway => {
            Some(format!("fled in round {round}"))
        }
        _ => None,
    }
}

fn capitalize(action: &str) -> String {
    let mut chars = action.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

const INJURY_PREFIX: &str = "Injury - ";

/// The folded injury row: `Injuries - a, b`, trimmed to the row width with
/// no full stop.
fn injuries_row(names: &[&str]) -> String {
    let text = format!("Injuries - {}", names.join(", "));
    trim_tail(&text, ROW_CHARS)
        .trim_end_matches('.')
        .to_string()
}

/// The folded loot row: `Loot - {n} items - Silver +{s}`; `1 item` at one,
/// the silver part dropped at zero, `Loot - Silver +{s}` with no goods.
fn loot_summary(items: usize, silver: i64) -> String {
    let noun = if items == 1 { "item" } else { "items" };
    match (items, silver > 0) {
        (0, _) => format!("Loot - Silver +{silver}"),
        (_, true) => format!("Loot - {items} {noun} - Silver +{silver}"),
        (_, false) => format!("Loot - {items} {noun}"),
    }
}

/// `+12 silver` style loot message: the silver amount.
pub(crate) fn loot_silver(message: &str) -> Option<i64> {
    let rest = message.trim().strip_prefix('+')?;
    let (number, unit) = rest.split_once(' ')?;
    (unit.trim_end_matches('.').eq_ignore_ascii_case("silver"))
        .then(|| number.parse().ok())
        .flatten()
}

/// The single cause whose sum covers the whole loss, else none (mixed or
/// disagreeing sources render plain).
fn sole_cause<const N: usize>(
    loss: i64,
    sums: [i64; N],
    labels: [&'static str; N],
) -> Option<&'static str> {
    let covering: Vec<usize> = (0..N).filter(|i| sums[*i] != 0).collect();
    match covering.as_slice() {
        [only] if sums[*only] == loss => Some(labels[*only]),
        _ => None,
    }
}

const HULL_CAUSES: [&str; 3] = ["their broadside", "their grapeshot", "parting shot"];
const CREW_CAUSES: [&str; 2] = ["their grapeshot", "boarding melee"];

/// The per-cause sums with a sinking's overkill taken off its own cause:
/// the hull stops at 0, so the last hit can claim more than the hull lost.
/// Only the sinking cause is reduced, and only when that alone makes the
/// sums meet the real loss.
fn clamp_hull_sums(
    loss: i64,
    mut sums: HullByCause,
    decisive: Option<&DecisiveMove>,
) -> HullByCause {
    let over = sums.iter().sum::<i64>() - loss;
    if over > 0 {
        if let Some(DecisiveMove::SunkBy(action)) = decisive {
            if let Some(cause) = hull_cause(action) {
                if sums[cause] > over {
                    sums[cause] -= over;
                }
            }
        }
    }
    sums
}

/// Loss rows for one term: one row per cause when the named sums add up to
/// the whole loss (`{term} -N - {cause}` each, in cause order), else `None`
/// so the caller draws the single row.
fn split_rows<const N: usize>(
    term: &str,
    loss: i64,
    sums: [i64; N],
    labels: [&'static str; N],
) -> Option<Vec<RecapRow>> {
    let named: Vec<usize> = (0..N).filter(|i| sums[*i] > 0).collect();
    if named.len() < 2 || named.iter().map(|i| sums[*i]).sum::<i64>() != loss {
        return None;
    }
    Some(
        named
            .into_iter()
            .map(|i| {
                caused(
                    RecapSection::Cost,
                    format!("{term} -{}", sums[i]),
                    Some(labels[i]),
                    RecapTone::Due,
                )
            })
            .collect(),
    )
}

/// The combat recap, or `None` when the fight has no result yet (a missing
/// result is never shown as a draw).
pub(crate) fn combat_recap(facts: &CombatFacts) -> Option<Recap> {
    let result = facts
        .result
        .filter(|result| !matches!(result, RecapResult::Paid | RecapResult::Abandoned))?;
    let win = result.is_win();
    let mut headline = verdict(result, &facts.name);
    if let Some(clause) = decisive_clause(result, facts.sea, facts.decisive.as_ref()) {
        headline = format!("{headline} - {clause}");
    }
    // A sea-event sinking has no captain to name.
    match facts.sink_cause.as_str() {
        "storm" => headline = "Sunk by the storm".to_string(),
        "pirates" => headline = "Sunk by pirates".to_string(),
        "event" => headline = "Sunk at sea".to_string(),
        _ => {}
    }
    let headline = ascii_punctuation(&headline);
    let headline_tone = match result {
        r if r.is_win() => RecapTone::Gold,
        RecapResult::Lost => RecapTone::Due,
        _ => RecapTone::Cream,
    };

    let choice = match result {
        RecapResult::WonSpared => Some("Spared".to_string()),
        RecapResult::WonTookAll => Some("Took all - hold and purse".to_string()),
        RecapResult::WonPrize => Some(format!(
            "Prize taken - {} added to your fleet",
            facts.prize_ship
        )),
        RecapResult::WonSunk => {
            let reason = facts
                .no_prize_reason
                .trim()
                .trim_end_matches('.')
                .trim_end();
            Some(if reason.is_empty() {
                "Sunk - no prize".to_string()
            } else {
                format!("Sunk - no prize - {reason}")
            })
        }
        _ if !facts.towed_to.is_empty() => {
            Some(format!("Towed to {}", port_display_name(&facts.towed_to)))
        }
        _ => None,
    };
    let wreck =
        !facts.towed_to.is_empty() || facts.wreck_silver > 0 || !facts.wreck_cargo.is_empty();
    // `{Destination} not reached` only when the tow went somewhere else.
    let mut not_reached = (wreck
        && !facts.towed_to.is_empty()
        && !facts.planned_destination.is_empty()
        && facts.planned_destination != facts.towed_to)
        .then(|| {
            row(
                RecapSection::Choice,
                format!(
                    "{} not reached",
                    port_display_name(&facts.planned_destination)
                ),
                RecapTone::Cream,
            )
        });

    // Earned (wins only): purse, loot silver once, loot goods, standing.
    let mut earned = Vec::new();
    let mut goods = Vec::new();
    let mut loot_items = 0usize;
    let mut loot_silver_row = None;
    let mut standing = None;
    let loot_silver_total: i64 = facts.loot.iter().filter_map(|m| loot_silver(m)).sum();
    if win {
        if facts.purse > 0 {
            earned.push(caused(
                RecapSection::Earned,
                format!("Silver +{}", facts.purse),
                Some("purse"),
                RecapTone::Cream,
            ));
        }
        if loot_silver_total > 0 {
            loot_silver_row = Some(earned.len());
            earned.push(caused(
                RecapSection::Earned,
                format!("Silver +{loot_silver_total}"),
                Some("loot"),
                RecapTone::Cream,
            ));
        }
        let others: Vec<&String> = facts
            .loot
            .iter()
            .filter(|m| loot_silver(m).is_none() && !m.trim().is_empty())
            .collect();
        loot_items = others.len();
        for message in others.iter().take(PASSTHROUGH_CAP) {
            goods.push(passthrough(RecapSection::Earned, message, RecapTone::Cream));
        }
        if others.len() > PASSTHROUGH_CAP {
            goods.push(row(
                RecapSection::Earned,
                format!("+{} more", others.len() - PASSTHROUGH_CAP),
                RecapTone::Cream,
            ));
        }
        if let Some((faction, delta)) = &facts.standing {
            if *delta != 0 {
                let cause = match result {
                    RecapResult::WonSpared => Some("spared"),
                    RecapResult::WonTookAll => Some("took all"),
                    _ => None,
                };
                standing = Some(caused(
                    RecapSection::Earned,
                    format!(
                        "Underworld standing {} {}",
                        faction_name(faction),
                        crate::logic::signed_delta(*delta)
                    ),
                    cause,
                    RecapTone::Cream,
                ));
            }
        }
    }
    // A non-win draws no note: the step text stays in the Log.
    let source: Vec<&String> = if win {
        facts.notes.iter().collect()
    } else {
        Vec::new()
    };
    let notes: Vec<RecapRow> = source
        .into_iter()
        .filter(|n| !n.trim().is_empty())
        .take(PASSTHROUGH_CAP)
        .map(|n| passthrough(RecapSection::Note, n, RecapTone::Muted))
        .collect();

    // Cost: Hull, Crew, Silver, Cargo, Injury. On a win, Silver and Cargo
    // show only when the fight itself took some, with no cause: the win's
    // silver is the delta less the purse and loot silver (shown under Earned).
    let [silver, crew, mut hull, cargo] = facts.deltas;
    // The tow patches the hull before the recap reads it, so a towed ship's
    // hull loss is what the fight did to it: the causes' sum, at most the
    // hull it had before the fight (after the patch less the delta).
    if !facts.towed_to.is_empty() {
        let dealt = facts.hull_by_cause.iter().sum::<i64>();
        let before = facts.hull_after - hull;
        hull = -(if before > 0 { dealt.min(before) } else { dealt });
    }
    let silver = if win {
        silver - facts.purse - loot_silver_total
    } else {
        silver
    };
    let hull_sums = clamp_hull_sums(-hull, facts.hull_by_cause, facts.decisive.as_ref());
    let bare_hull = (hull < 0).then(|| {
        caused(
            RecapSection::Cost,
            format!("Hull {hull}"),
            sole_cause(-hull, hull_sums, HULL_CAUSES),
            RecapTone::Due,
        )
    });
    let bare_crew = (crew < 0).then(|| {
        caused(
            RecapSection::Cost,
            format!("Crew {crew}"),
            sole_cause(-crew, facts.crew_by_cause, CREW_CAUSES),
            RecapTone::Due,
        )
    });
    let split_hull = (hull < 0)
        .then(|| split_rows("Hull", -hull, hull_sums, HULL_CAUSES))
        .flatten();
    let split_crew = (crew < 0)
        .then(|| split_rows("Crew", -crew, facts.crew_by_cause, CREW_CAUSES))
        .flatten();
    let extra_split = split_hull.as_ref().map_or(0, |rows| rows.len() - 1)
        + split_crew.as_ref().map_or(0, |rows| rows.len() - 1);
    let mut cost = Vec::new();
    let plunder = (facts.plundered && result == RecapResult::Lost).then_some("plundered");
    let forfeit = (result == RecapResult::Lost
        && matches!(facts.decisive, Some(DecisiveMove::BestedInDuel)))
    .then_some("duel forfeit");
    if wreck {
        // Silver: the wreck's share from the step, then any other fall as
        // its own plain row. Goods: catalog names, two rows then a count.
        if facts.wreck_silver > 0 {
            cost.push(caused(
                RecapSection::Cost,
                format!("Silver -{}", facts.wreck_silver),
                Some(WRECK_CAUSE),
                RecapTone::Due,
            ));
        }
        // Only the wreck's own silver rows: no unexplained `Silver -N` beside
        // them (fees paid at the tow port go to the Log).
        let _ = silver;
        let goods: Vec<&(String, i64)> = facts
            .wreck_cargo
            .iter()
            .filter(|(_, qty)| *qty > 0)
            .collect();
        for (id, qty) in goods.iter().take(PASSTHROUGH_CAP) {
            cost.push(caused(
                RecapSection::Cost,
                format!("{} -{qty}", good_display_name(id)),
                Some(WRECK_CAUSE),
                RecapTone::Due,
            ));
        }
        if goods.len() > PASSTHROUGH_CAP {
            cost.push(row(
                RecapSection::Cost,
                format!(
                    "+{} more goods {WRECK_CAUSE}",
                    goods.len() - PASSTHROUGH_CAP
                ),
                RecapTone::Due,
            ));
        }
        if facts.claims_paid > 0 {
            cost.push(caused(
                RecapSection::Cost,
                format!("Silver +{}", facts.claims_paid),
                Some("insurance claim"),
                RecapTone::Cream,
            ));
        }
    } else if silver < 0 {
        cost.push(caused(
            RecapSection::Cost,
            format!("Silver {silver}"),
            if win { None } else { plunder.or(forfeit) },
            RecapTone::Due,
        ));
    }
    if cargo < 0 && !wreck {
        cost.push(caused(
            RecapSection::Cost,
            format!("Cargo {cargo}"),
            if win { None } else { plunder },
            RecapTone::Cream,
        ));
    }
    for injury in facts.injuries.iter().take(INJURY_CAP) {
        cost.push(row(
            RecapSection::Cost,
            format!("{INJURY_PREFIX}{}", injury.trim()),
            RecapTone::Cream,
        ));
    }
    if wreck {
        if let Some(max) = facts.hull_max.filter(|max| *max > 0) {
            cost.push(row(
                RecapSection::Cost,
                format!("Your hull {}/{max} - patched for the tow", facts.hull_after),
                RecapTone::Muted,
            ));
        }
    }
    let mut repair = (wreck
        && facts.repair_price > 0
        && facts.hull_max.is_some_and(|max| facts.hull_after < max))
    .then(|| {
        row(
            RecapSection::Cost,
            format!("Repair here - {} silver", facts.repair_price),
            RecapTone::Muted,
        )
    });

    // One bounty row at most: ready on a win, still open otherwise.
    let bounty = if win {
        facts
            .bounty_ready
            .then(|| row(RecapSection::Bounty, BOUNTY_READY, RecapTone::Muted))
    } else {
        facts
            .bounty_open
            .then(|| row(RecapSection::Bounty, BOUNTY_NOTE, RecapTone::Muted))
    };

    // Silver net: only when a drawn silver row rose and another fell.
    let mut drawn = earned.clone();
    drawn.extend(cost.iter().cloned());
    let net = silver_net(&drawn);

    // Cap: drop notes beyond 1, notes, loot goods beyond 1, loot goods, the
    // standing row, until the body fits. The bounty row never drops.
    let mut notes = notes;
    // The repair row drops first, then `{Destination} not reached`.
    let optional = |repair: &Option<RecapRow>, not_reached: &Option<RecapRow>| {
        usize::from(repair.is_some()) + usize::from(not_reached.is_some())
    };
    let mut fixed = usize::from(choice.is_some())
        + earned.len()
        + cost.len()
        + usize::from(net.is_some())
        + usize::from(bare_hull.is_some())
        + usize::from(bare_crew.is_some());
    while fixed + optional(&repair, &not_reached) > BODY_ROW_CAP {
        if repair.is_some() {
            repair = None;
        } else if not_reached.is_some() {
            not_reached = None;
        } else {
            break;
        }
    }
    fixed = fixed - usize::from(bare_hull.is_some()) - usize::from(bare_crew.is_some())
        + optional(&repair, &not_reached);
    // The split rows only when the whole card, notes and all, still fits;
    // otherwise each term keeps its single row.
    let all = fixed
        + notes.len()
        + goods.len()
        + usize::from(standing.is_some())
        + usize::from(bounty.is_some())
        + usize::from(bare_hull.is_some())
        + usize::from(bare_crew.is_some());
    let split = extra_split > 0 && all + extra_split <= BODY_ROW_CAP;
    let mut losses: Vec<RecapRow> = Vec::new();
    if split {
        losses.extend(
            split_hull
                .or_else(|| bare_hull.clone().map(|r| vec![r]))
                .unwrap_or_default(),
        );
        losses.extend(
            split_crew
                .or_else(|| bare_crew.clone().map(|r| vec![r]))
                .unwrap_or_default(),
        );
    } else {
        losses.extend(bare_hull);
        losses.extend(bare_crew);
    }
    let mut fixed = fixed + losses.len();
    losses.extend(cost);
    let cost = losses;
    let body = |notes: &[RecapRow],
                goods: &[RecapRow],
                standing: &Option<RecapRow>,
                bounty: &Option<RecapRow>| {
        fixed
            + notes.len()
            + goods.len()
            + usize::from(standing.is_some())
            + usize::from(bounty.is_some())
    };
    // A won card over the cap folds its loot list (silver and goods) into
    // one summary row first; Cost and the net row stay as they are.
    let loot_rows = goods.len() + usize::from(loot_silver_row.is_some());
    let folded = win && loot_rows > 1 && body(&notes, &goods, &standing, &bounty) > BODY_ROW_CAP;
    if folded {
        if let Some(index) = loot_silver_row {
            earned.remove(index);
            fixed -= 1;
        }
        goods = vec![row(
            RecapSection::Earned,
            loot_summary(loot_items, loot_silver_total),
            RecapTone::Cream,
        )];
    }
    let body = |notes: &[RecapRow],
                goods: &[RecapRow],
                standing: &Option<RecapRow>,
                bounty: &Option<RecapRow>| {
        fixed
            + notes.len()
            + goods.len()
            + usize::from(standing.is_some())
            + usize::from(bounty.is_some())
    };
    // Then the boarding rule note: on a won card it drops when the body
    // with it would pass the cap.
    let rule_note = facts.reached_boarding
        && !(win && body(&notes, &goods, &standing, &bounty) + 1 > BODY_ROW_CAP);
    // Then the injuries: a won card still over the cap shows them on one
    // `Injuries - a, b` row where the first stood.
    let mut cost = cost;
    let injured: Vec<usize> = cost
        .iter()
        .enumerate()
        .filter(|(_, r)| r.text.starts_with(INJURY_PREFIX))
        .map(|(i, _)| i)
        .collect();
    if win && injured.len() > 1 && body(&notes, &goods, &standing, &bounty) > BODY_ROW_CAP {
        let names: Vec<&str> = facts
            .injuries
            .iter()
            .take(INJURY_CAP)
            .map(|i| i.trim())
            .collect();
        let folded_row = row(RecapSection::Cost, injuries_row(&names), RecapTone::Cream);
        let first = injured[0];
        for index in injured.iter().rev() {
            cost.remove(*index);
        }
        cost.insert(first, folded_row);
        fixed -= injured.len() - 1;
    }
    let body = |notes: &[RecapRow],
                goods: &[RecapRow],
                standing: &Option<RecapRow>,
                bounty: &Option<RecapRow>| {
        fixed
            + notes.len()
            + goods.len()
            + usize::from(standing.is_some())
            + usize::from(bounty.is_some())
    };
    while body(&notes, &goods, &standing, &bounty) > BODY_ROW_CAP {
        if notes.len() > 1 {
            notes.truncate(1);
        } else if !notes.is_empty() {
            notes.clear();
        } else if !folded && goods.len() > 1 {
            goods.truncate(1);
        } else if !folded && !goods.is_empty() {
            goods.clear();
        } else if standing.is_some() {
            standing = None;
        } else {
            break;
        }
    }

    let mut rows = Vec::new();
    if let Some(choice) = choice {
        rows.push(row(
            RecapSection::Choice,
            trim_tail(&ascii_punctuation(&choice), ROW_CHARS),
            RecapTone::Cream,
        ));
    }
    rows.extend(not_reached);
    let mut earned_rows = earned;
    earned_rows.extend(goods);
    earned_rows.extend(standing);
    if !earned_rows.is_empty() {
        rows.push(row(RecapSection::Heading, "Earned", RecapTone::Muted));
        rows.extend(earned_rows);
    }
    rows.extend(notes);
    if !cost.is_empty() {
        rows.push(row(RecapSection::Heading, "Cost", RecapTone::Muted));
        rows.extend(cost);
    }
    rows.extend(repair);
    if rule_note {
        rows.push(row(RecapSection::RuleNote, RULE_NOTE, RecapTone::Muted));
    }
    rows.extend(bounty);
    rows.extend(net);
    Some(Recap {
        kind: RecapKind::Combat,
        result,
        headline,
        headline_tone,
        rows,
    })
}

/// `Silver net +N` / `-N` over the drawn `Silver` rows, only when one rose
/// and another fell.
pub(crate) fn silver_net(rows: &[RecapRow]) -> Option<RecapRow> {
    let values: Vec<i64> = rows
        .iter()
        .filter_map(|r| r.text.strip_prefix("Silver "))
        .filter_map(|rest| rest.split(' ').next()?.parse::<i64>().ok())
        .collect();
    (values.iter().any(|v| *v > 0) && values.iter().any(|v| *v < 0)).then(|| {
        row(
            RecapSection::Net,
            format!(
                "Silver net {}",
                crate::logic::signed_delta(values.iter().sum())
            ),
            RecapTone::Cream,
        )
    })
}

fn is_paid(outcome: &ContractOutcome) -> bool {
    matches!(
        outcome.outcome_type.as_str(),
        "completed" | "completed_bonus"
    )
}

/// One card for a settling sale (paid, up to 3 contracts) or a confirmed
/// abandon. The payout row is the net: there is no separate net row.
/// `None` for an empty list. The card's kind comes from the first outcome.
pub(crate) fn contract_recap(
    outcomes: &[ContractOutcome],
    extras: &ContractExtras,
) -> Option<Recap> {
    let paid = is_paid(outcomes.first()?);
    let mut rows = Vec::new();
    for (i, outcome) in outcomes.iter().enumerate() {
        let good = good_display_name(&outcome.good_id);
        let port = port_display_name(&outcome.destination_port_id);
        if paid {
            rows.push(row(
                RecapSection::Contract,
                trim_tail(
                    &format!("{} {good} to {port}", outcome.delivered_quantity),
                    ROW_CHARS,
                ),
                RecapTone::Cream,
            ));
            if outcome.silver_delta != 0 {
                rows.push(row(
                    RecapSection::Contract,
                    format!(
                        "Silver {}",
                        crate::logic::signed_delta(outcome.silver_delta)
                    ),
                    RecapTone::Cream,
                ));
            }
            let bonus = outcome.silver_delta - outcome.reward_silver;
            if outcome.outcome_type == "completed_bonus" && bonus > 0 {
                rows.push(row(
                    RecapSection::Note,
                    format!("Includes early bonus +{bonus}"),
                    RecapTone::Muted,
                ));
            }
        } else {
            rows.push(row(
                RecapSection::Contract,
                trim_tail(&format!("{good} for {port}"), ROW_CHARS),
                RecapTone::Cream,
            ));
            if outcome.delivered_quantity > 0 {
                rows.push(row(
                    RecapSection::Contract,
                    format!(
                        "Progress lost - {}",
                        progress_text(outcome.delivered_quantity, outcome.required_quantity)
                    ),
                    RecapTone::Due,
                ));
            }
            let held = extras.held.get(i).copied().unwrap_or(0);
            if held > 0 {
                rows.push(row(
                    RecapSection::Contract,
                    format!("{held} {good} still aboard"),
                    RecapTone::Cream,
                ));
            }
        }
    }
    let (result, headline, tone) = if paid {
        let headline = if outcomes.len() > 1 {
            "Contracts paid - settled by one sale"
        } else {
            "Contract paid - settled by your sale"
        };
        (RecapResult::Paid, headline, RecapTone::Gold)
    } else {
        (RecapResult::Abandoned, "Contract abandoned", RecapTone::Due)
    };
    Some(Recap {
        kind: RecapKind::Contract,
        result,
        headline: headline.to_string(),
        headline_tone: tone,
        rows,
    })
}

/// What the encounter screen carries across a fight's steps so the recap can
/// name its causes. Godot memory only: never saved, and a new encounter
/// starts a new one. Carried in `adopt_step` the way the baseline is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RecapSeed {
    /// A naval or boarding step was seen.
    pub sea: bool,
    /// Boarding meter of the last naval step that had one.
    pub boarded: Option<(i64, i64)>,
    pub decisive: Option<DecisiveMove>,
    pub hull_by_cause: HullByCause,
    pub crew_by_cause: CrewByCause,
    pub plundered: bool,
    pub injuries: Vec<String>,
    /// Set once, when the fight resolves without a win.
    pub result: Option<RecapResult>,
    /// `[silver, crew, hull, cargo]` from a copy of the baseline, read when
    /// the fight resolved.
    pub deltas: Option<[i64; 4]>,
    pub bounty_open: bool,
    /// Set at a win, read after it resolved: the target is claimable.
    pub bounty_ready: bool,
    /// A step triggered boarding or a boarding step ran.
    pub reached_boarding: bool,
    /// Win receipt: purse, loot messages, standing, reactions and departures.
    pub purse: i64,
    pub loot: Vec<String>,
    pub standing: Option<(String, i64)>,
    pub notes: Vec<String>,
    pub prize_ship: String,
    pub no_prize_reason: String,
    /// Sinking: what the sim's tow step reported.
    pub towed_to: String,
    pub wreck_silver: i64,
    pub wreck_cargo: Vec<(String, i64)>,
    pub hull_after: i64,
    /// Sinking: the voyage's destination before the tow (empty when unknown).
    pub planned_destination: String,
    /// Sinking: silver cargo insurance paid for the wreck.
    pub claims_paid: i64,
    /// Sinking: `fight`, `parting_shot`, `storm`, `pirates` or `event`.
    pub sink_cause: String,
    /// Sinking: full repair at the tow port, in silver (0: none needed).
    pub repair_price: i64,
}

/// The seed a new step starts from: the previous step's, unless that
/// encounter already ended (a fresh encounter never inherits a seed).
pub(crate) fn seed_base(previous: Option<&crate::logic::EncounterFacts>) -> RecapSeed {
    match previous {
        Some(facts) if !(facts.phase == "resolved" && !facts.pending_victory) => {
            facts.recap.clone()
        }
        _ => RecapSeed::default(),
    }
}

/// Store a Spare or Take all receipt on the seed: result, purse, loot,
/// standing and the reaction and departure notes.
pub(crate) fn seed_victory(
    seed: &mut RecapSeed,
    receipt: &portlight_sim::session::VictoryReceipt,
    deltas: Option<[i64; 4]>,
) {
    seed.result = Some(if receipt.spared {
        RecapResult::WonSpared
    } else {
        RecapResult::WonTookAll
    });
    seed.deltas = deltas;
    seed.purse = receipt.silver_delta;
    seed.loot = fold_loot(&receipt.loot, &receipt.loot_messages);
    seed.standing =
        (receipt.standing_delta != 0).then(|| (receipt.faction_id.clone(), receipt.standing_delta));
    seed.notes = receipt
        .reactions
        .iter()
        .map(|(_, _, flavor)| flavor.clone())
        .chain(
            receipt
                .departures
                .iter()
                .map(|d| format!("{} leaves. {}", d.companion_name, d.departure_line)),
        )
        .collect();
}

/// Loot rows from the receipt's drops: silver drops summed into one
/// `+N silver`, cargo drops of the same good folded into one `{n} {Good}`
/// (first-seen order), ammo as `+{gained} {Ammo}`, then the sim's other
/// loot messages (`Found: ...`).
pub(crate) fn fold_loot(
    drops: &[portlight_sim::loot::LootDrop],
    messages: &[String],
) -> Vec<String> {
    let silver: i64 = drops
        .iter()
        .filter(|drop| drop.item_type == "silver")
        .map(|drop| drop.quantity)
        .sum();
    let mut goods: Vec<(String, i64)> = Vec::new();
    for drop in drops.iter().filter(|drop| drop.item_type == "cargo") {
        match goods.iter_mut().find(|(id, _)| id == &drop.item_id) {
            Some((_, quantity)) => *quantity += drop.quantity,
            None => goods.push((drop.item_id.clone(), drop.quantity)),
        }
    }
    let mut rows = Vec::new();
    if silver > 0 {
        rows.push(format!("+{silver} silver"));
    }
    rows.extend(
        goods
            .into_iter()
            .filter(|(_, quantity)| *quantity > 0)
            .map(|(id, quantity)| format!("{quantity} {}", good_display_name(&id))),
    );
    // Ammo: the sim's gained figure (catalog bundle size times the drop),
    // folded per kind like cargo.
    let mut ammo: Vec<(String, i64)> = Vec::new();
    for drop in drops.iter().filter(|drop| drop.item_type == "ammo") {
        let Some(def) = portlight_sim::content::content().ammo(&drop.item_id) else {
            continue;
        };
        let gained = def.quantity * drop.quantity;
        match ammo.iter_mut().find(|(name, _)| name == &def.name) {
            Some((_, total)) => *total += gained,
            None => ammo.push((def.name.clone(), gained)),
        }
    }
    rows.extend(
        ammo.into_iter()
            .filter(|(_, gained)| *gained > 0)
            .map(|(name, gained)| format!("+{gained} {name}")),
    );
    rows.extend(
        messages
            .iter()
            .filter(|message| message.starts_with("Found: "))
            .cloned(),
    );
    rows
}

/// A won fight that ended at the sinking: a prize taken, the prize let go
/// under, or an enemy sunk with no prize offered.
pub(crate) fn seed_sinking_win(
    seed: &mut RecapSeed,
    step: &portlight_sim::session::EncounterStep,
    deltas: Option<[i64; 4]>,
) {
    if step.kind == "capture" && step.prize_ok {
        seed.result = Some(RecapResult::WonPrize);
        seed.prize_ship = step.prize_reason.clone();
    } else {
        seed.result = Some(RecapResult::WonSunk);
        seed.no_prize_reason = if step.prize_ok {
            String::new()
        } else {
            step.prize_reason.clone()
        };
    }
    seed.deltas = deltas;
}

/// The non-win recap result for an encounter end.
pub(crate) fn non_win_result(end: crate::logic::EncounterEnd) -> RecapResult {
    use crate::logic::EncounterEnd;
    match end {
        EncounterEnd::LostFight | EncounterEnd::LostAtSea => RecapResult::Lost,
        EncounterEnd::DrewFight => RecapResult::Drew,
        EncounterEnd::BrokeAway => RecapResult::BrokeAway,
    }
}

fn hull_cause(enemy_action: &str) -> Option<usize> {
    match enemy_action {
        "broadside" => Some(0),
        "rake" => Some(1),
        _ => None,
    }
}

/// The seed after one more step. Only step data is read; a loss with no
/// named source is not summed, so its row renders plain.
pub(crate) fn carry_seed(
    previous: &RecapSeed,
    step: &portlight_sim::session::EncounterStep,
) -> RecapSeed {
    let mut seed = previous.clone();
    let flee = step.choice == "flee";
    match step.kind.as_str() {
        "naval" if flee => {
            seed.sea = true;
            if step.hull_damage > 0 {
                // A break-away's damage is the parting shot; a failed flee
                // is their broadside (the sim's own wording for each).
                let cause = if step.escaped { 2 } else { 0 };
                seed.hull_by_cause[cause] += step.hull_damage;
            }
        }
        "naval" => {
            seed.sea = true;
            if step.player_hull_delta < 0 {
                if let Some(cause) = hull_cause(&step.enemy_action) {
                    seed.hull_by_cause[cause] += -step.player_hull_delta;
                }
            }
            if step.player_crew_delta < 0 && step.enemy_action == "rake" {
                seed.crew_by_cause[0] += -step.player_crew_delta;
            }
            if step.boarding_threshold > 0 {
                seed.boarded = Some((step.boarding_progress, step.boarding_threshold));
            }
        }
        "board" => {
            seed.sea = true;
            seed.crew_by_cause[1] += step.player_crew_lost;
        }
        "choice" if flee && step.hull_damage > 0 => {
            // Escaped: the parting shot. Caught at the approach: the sim
            // runs a round of their broadside.
            let cause = if step.escaped { 2 } else { 0 };
            seed.hull_by_cause[cause] += step.hull_damage;
        }
        _ => {}
    }
    if !step.injury.is_empty() {
        let name = portlight_sim::content::content()
            .injury(&step.injury)
            .map(|def| def.name.clone())
            .unwrap_or_else(|| crate::logic::humanize_id(&step.injury));
        if !seed.injuries.contains(&name) {
            seed.injuries.push(name);
        }
    }
    let decisive = match step.kind.as_str() {
        // Sunk by a flee's shot: a failed flee is their broadside; a parting
        // shot names no clause.
        _ if flee && step.player_sunk => Some(DecisiveMove::SunkBy(
            if step.escaped {
                "parting shot"
            } else {
                "broadside"
            }
            .to_string(),
        )),
        "choice" if flee && step.escaped => Some(DecisiveMove::FledApproach),
        "naval" if flee && step.escaped => Some(DecisiveMove::FledAtSea(step.turn)),
        "naval" if step.player_sunk => Some(DecisiveMove::SunkBy(step.enemy_action.clone())),
        "naval" if step.enemy_sunk => Some(DecisiveMove::Sank(step.player_action.clone())),
        "naval" if step.phase == "resolved" && step.player_crew <= 0 => {
            Some(DecisiveMove::CrewLost)
        }
        "naval" if step.boarding_triggered => Some(DecisiveMove::Boarded(
            step.boarding_progress,
            step.boarding_threshold,
        )),
        "fight" if step.phase == "resolved" && !step.player_won && !step.draw => {
            Some(DecisiveMove::BestedInDuel)
        }
        "fight" if !step.player_action.is_empty() => {
            Some(DecisiveMove::FinalMove(step.player_action.clone()))
        }
        _ => None,
    };
    let keeps_boarded = matches!(seed.decisive, Some(DecisiveMove::Boarded(..)))
        && matches!(decisive, Some(DecisiveMove::FinalMove(_)));
    if decisive.is_some() && !keeps_boarded {
        seed.decisive = decisive;
    }
    if step.boarding_triggered || step.kind == "board" {
        seed.reached_boarding = true;
    }
    if step.player_sunk && step.phase == "resolved" {
        seed.towed_to = step.towed_to.clone();
        seed.wreck_silver = step.sink_silver_lost;
        seed.wreck_cargo = step.sink_cargo_lost.clone();
        seed.hull_after = step.hull_after;
        seed.claims_paid = step.sink_claims_paid;
        seed.sink_cause = step.sink_cause.clone();
        seed.planned_destination = step.sink_planned_destination.clone();
    }
    if step.phase == "resolved" {
        seed.plundered = step.player_crew <= 0 && !step.player_sunk;
    }
    seed
}

/// Combat facts for a resolved fight, or `None` when the seed has no result
/// yet (still running, or a win still pending).
pub(crate) fn seed_facts(seed: &RecapSeed, name: &str) -> Option<CombatFacts> {
    let result = seed.result?;
    Some(CombatFacts {
        result: Some(result),
        sea: seed.sea,
        name: name.to_string(),
        decisive: seed.decisive.clone(),
        prize_ship: seed.prize_ship.clone(),
        no_prize_reason: seed.no_prize_reason.clone(),
        purse: seed.purse,
        loot: seed.loot.clone(),
        standing: seed.standing.clone(),
        notes: seed.notes.clone(),
        deltas: seed.deltas?,
        hull_by_cause: seed.hull_by_cause,
        crew_by_cause: seed.crew_by_cause,
        plundered: seed.plundered,
        injuries: seed.injuries.clone(),
        reached_boarding: seed.reached_boarding,
        bounty_open: seed.bounty_open,
        bounty_ready: seed.bounty_ready,
        hull_max: None,
        towed_to: seed.towed_to.clone(),
        wreck_silver: seed.wreck_silver,
        wreck_cargo: seed.wreck_cargo.clone(),
        hull_after: seed.hull_after,
        planned_destination: seed.planned_destination.clone(),
        claims_paid: seed.claims_paid,
        sink_cause: seed.sink_cause.clone(),
        repair_price: seed.repair_price,
    })
}

/// The recap a resolved Outcome card draws, or `None` while the fight runs,
/// while a win is pending, and for an end with no result (a peace).
pub(crate) fn outcome_recap(facts: &crate::logic::EncounterFacts) -> Option<Recap> {
    if facts.phase != "resolved" || facts.pending_victory {
        return None;
    }
    let name = crate::logic::captain_display_name(&facts.captain_name, &facts.captain_id);
    let mut combat = seed_facts(&facts.recap, &name)?;
    combat.hull_max = facts.player_hull_max;
    combat_recap(&combat)
}

/// The drawn recap as text, one line per row, headline first.
pub(crate) fn recap_text(recap: &Recap) -> String {
    std::iter::once(recap.headline.as_str())
        .chain(recap.rows.iter().map(|row| row.text.as_str()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use portlight_sim::session::EncounterStep;

    fn combat(facts: &CombatFacts) -> Recap {
        combat_recap(facts).expect("result set")
    }

    fn contract(outcomes: &[ContractOutcome], extras: &ContractExtras) -> Recap {
        contract_recap(outcomes, extras).expect("non-empty")
    }

    fn step(kind: &str, phase: &str) -> EncounterStep {
        let mut step = EncounterStep {
            kind: kind.into(),
            phase: phase.into(),
            ..blank_step()
        };
        step.player_crew = 10;
        step
    }

    fn blank_step() -> EncounterStep {
        EncounterStep {
            kind: String::new(),
            phase: String::new(),
            message: String::new(),
            choice: String::new(),
            success: false,
            escaped: false,
            hull_damage: 0,
            enemy_captain_id: String::new(),
            enemy_captain_name: String::new(),
            enemy_strength: 0,
            turn: 0,
            player_action: String::new(),
            enemy_action: String::new(),
            player_hull_delta: 0,
            enemy_hull_delta: 0,
            player_crew_delta: 0,
            enemy_crew_delta: 0,
            boarding_progress: 0,
            boarding_threshold: 0,
            enemy_sunk: false,
            player_sunk: false,
            boarding_triggered: false,
            flavor: String::new(),
            player_hull: 0,
            enemy_hull: 0,
            player_crew: 0,
            enemy_crew: 0,
            player_crew_lost: 0,
            enemy_crew_lost: 0,
            player_advantage: false,
            damage_to_opponent: 0,
            damage_to_player: 0,
            player_hp: 0,
            opponent_hp: 0,
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
            towed_to: String::new(),
            sink_silver_lost: 0,
            sink_cargo_lost: Vec::new(),
            hull_after: 0,
            sink_claims_paid: 0,
            sink_cause: String::new(),
            sink_planned_destination: String::new(),
            sink_notes: Vec::new(),
        }
    }

    fn naval(enemy_action: &str, hull: i64, crew: i64, progress: i64) -> EncounterStep {
        let mut s = step("naval", "naval");
        s.enemy_action = enemy_action.into();
        s.player_action = "close".into();
        s.player_hull_delta = hull;
        s.player_crew_delta = crew;
        s.boarding_progress = progress;
        s.boarding_threshold = 3;
        s
    }

    #[test]
    fn recap_seed_survives_every_step() {
        let mut seed = RecapSeed::default();
        seed = carry_seed(&seed, &naval("broadside", -8, 0, 1));
        seed = carry_seed(&seed, &naval("rake", -2, -1, 2));
        let mut boarded = naval("broadside", -4, 0, 3);
        boarded.phase = "boarding".into();
        boarded.boarding_triggered = true;
        seed = carry_seed(&seed, &boarded);
        let mut board = step("board", "duel");
        board.player_crew_lost = 2;
        seed = carry_seed(&seed, &board);
        let mut hit = step("fight", "duel");
        hit.player_action = "thrust".into();
        hit.injury = "no_such_wound".into();
        seed = carry_seed(&seed, &hit);
        let mut last = step("fight", "resolved");
        last.player_action = "parry".into();
        last.player_won = true;
        seed = carry_seed(&seed, &last);
        assert!(seed.sea);
        assert_eq!(seed.boarded, Some((3, 3)));
        assert_eq!(seed.hull_by_cause, [12, 2, 0]);
        assert_eq!(seed.crew_by_cause, [1, 2]);
        assert_eq!(seed.injuries, vec!["No Such Wound".to_string()]);
        // The boarding stays the decisive move; the duel's last move does
        // not replace it.
        assert_eq!(seed.decisive, Some(DecisiveMove::Boarded(3, 3)));
        assert!(seed.reached_boarding);
        assert!(!seed.plundered);
        // An unnamed source is not summed, so the row stays plain.
        let quiet = carry_seed(&RecapSeed::default(), &naval("evade", -5, -1, 0));
        assert_eq!((quiet.hull_by_cause, quiet.crew_by_cause), ([0; 3], [0; 2]));
    }

    #[test]
    fn seed_names_the_non_win_moves() {
        let mut sunk = naval("broadside", -20, 0, 1);
        sunk.phase = "resolved".into();
        sunk.player_sunk = true;
        let seed = carry_seed(&RecapSeed::default(), &sunk);
        assert_eq!(
            seed.decisive,
            Some(DecisiveMove::SunkBy("broadside".into()))
        );
        assert!(!seed.plundered);
        let mut crew = naval("rake", 0, -3, 1);
        crew.phase = "resolved".into();
        crew.player_crew = 0;
        let seed = carry_seed(&RecapSeed::default(), &crew);
        assert_eq!(seed.decisive, Some(DecisiveMove::CrewLost));
        assert!(seed.plundered);
        let mut away = step("naval", "resolved");
        away.choice = "flee".into();
        away.escaped = true;
        away.turn = 4;
        away.hull_damage = 6;
        let seed = carry_seed(&RecapSeed::default(), &away);
        assert_eq!(seed.decisive, Some(DecisiveMove::FledAtSea(4)));
        assert_eq!(seed.hull_by_cause, [0, 0, 6]);
        let mut failed = step("naval", "naval");
        failed.choice = "flee".into();
        failed.hull_damage = 5;
        assert_eq!(
            carry_seed(&RecapSeed::default(), &failed).hull_by_cause,
            [5, 0, 0]
        );
        let mut early = step("choice", "resolved");
        early.choice = "flee".into();
        early.escaped = true;
        early.hull_damage = 3;
        let seed = carry_seed(&RecapSeed::default(), &early);
        assert_eq!(seed.decisive, Some(DecisiveMove::FledApproach));
        assert_eq!(seed.hull_by_cause, [0, 0, 3]);
    }

    #[test]
    fn non_win_seed_builds_the_recap_rows() {
        let seed = RecapSeed {
            sea: true,
            decisive: Some(DecisiveMove::FledAtSea(2)),
            hull_by_cause: [0, 0, 4],
            result: Some(RecapResult::BrokeAway),
            deltas: Some([0, 0, -4, 0]),
            ..Default::default()
        };
        let recap = combat(&seed_facts(&seed, "Captain Vane").unwrap());
        assert_eq!(
            recap.headline,
            "Broke away from Captain Vane - fled in round 2"
        );
        assert_eq!(texts(&recap), vec!["Cost", "Hull -4 - parting shot"]);
        let drew = RecapSeed {
            decisive: Some(DecisiveMove::FinalMove("parry".into())),
            result: Some(RecapResult::Drew),
            deltas: Some([0, -1, 0, 0]),
            ..Default::default()
        };
        let recap = combat(&seed_facts(&drew, "Captain Vane").unwrap());
        assert_eq!(recap.headline, "Drew with Captain Vane");
        assert_eq!(texts(&recap), vec!["Cost", "Crew -1"]);
    }

    #[test]
    fn a_missing_result_draws_nothing() {
        assert_eq!(seed_facts(&RecapSeed::default(), "Captain Vane"), None);
        let no_deltas = RecapSeed {
            result: Some(RecapResult::Lost),
            ..Default::default()
        };
        assert_eq!(seed_facts(&no_deltas, "Captain Vane"), None);
        assert_eq!(combat_recap(&CombatFacts::default()), None);
        let mut contract_kind = loss();
        contract_kind.result = Some(RecapResult::Paid);
        assert_eq!(combat_recap(&contract_kind), None);
    }

    fn receipt(spared: bool) -> portlight_sim::session::VictoryReceipt {
        portlight_sim::session::VictoryReceipt {
            spared,
            silver_delta: 120,
            loot: vec![drop("silver", "silver", 12)],
            loot_messages: vec!["+12 silver".into(), "Found: Compass.".into()],
            standing_delta: 5,
            faction_id: "underworld".into(),
            enemy_captain_id: "vane".into(),
            enemy_captain_name: "Captain Vane".into(),
            reactions: vec![("mara".into(), 1, "Mara approves.".into())],
            departures: Vec::new(),
        }
    }

    fn drop(kind: &str, id: &str, quantity: i64) -> portlight_sim::loot::LootDrop {
        portlight_sim::loot::LootDrop {
            item_type: kind.into(),
            item_id: id.into(),
            quantity,
        }
    }

    #[test]
    fn ammo_drops_get_their_gained_row() {
        let drops = vec![
            drop("ammo", "pistol_shot", 1),
            drop("ammo", "pistol_shot", 2),
            drop("ammo", "no_such_ammo", 1),
        ];
        let def = portlight_sim::content::content()
            .ammo("pistol_shot")
            .unwrap();
        assert_eq!(
            fold_loot(&drops, &[]),
            vec![format!("+{} {}", def.quantity * 3, def.name)]
        );
    }

    #[test]
    fn same_good_loot_drops_fold_into_one_row() {
        let drops = vec![
            drop("cargo", "spice", 3),
            drop("silver", "silver", 12),
            drop("cargo", "rum", 2),
            drop("cargo", "spice", 3),
            drop("silver", "silver", 8),
        ];
        let messages: Vec<String> = [
            "+3 Spice",
            "+12 silver",
            "+2 Rum",
            "+3 Spice",
            "+8 silver",
            "Found: Compass",
        ]
        .iter()
        .map(|m| m.to_string())
        .collect();
        let spice = good_display_name("spice");
        let rum = good_display_name("rum");
        assert_eq!(
            fold_loot(&drops, &messages),
            vec![
                "+20 silver".to_string(),
                format!("6 {spice}"),
                format!("2 {rum}"),
                "Found: Compass".to_string(),
            ]
        );
        let mut receipt = receipt(false);
        receipt.loot = drops;
        receipt.loot_messages = messages;
        let mut seed = RecapSeed::default();
        // The change is the purse plus the loot silver: no fight cost.
        seed_victory(&mut seed, &receipt, Some([140, 0, 0, 0]));
        let recap = combat(&seed_facts(&seed, "Captain Vane").unwrap());
        let rows = texts(&recap);
        assert!(rows.contains(&"Silver +20 - loot"), "{rows:?}");
        assert_eq!(
            rows.iter().filter(|row| row.contains(&spice)).count(),
            1,
            "{rows:?}"
        );
        assert!(
            rows.iter().any(|row| *row == format!("6 {spice}")),
            "{rows:?}"
        );
    }

    #[test]
    fn a_victory_receipt_fills_the_win_rows() {
        let mut seed = RecapSeed {
            sea: true,
            decisive: Some(DecisiveMove::Boarded(3, 3)),
            reached_boarding: true,
            hull_by_cause: [12, 0, 0],
            ..Default::default()
        };
        seed_victory(&mut seed, &receipt(true), Some([132, 0, -12, 0]));
        let recap = combat(&seed_facts(&seed, "Captain Vane").unwrap());
        assert_eq!(recap.result.id(), "won_spared");
        assert_eq!(recap.headline, "Won against Captain Vane - boarded at 3/3");
        assert_eq!(
            texts(&recap),
            vec![
                "Spared",
                "Earned",
                "Silver +120 - purse",
                "Silver +12 - loot",
                "Found: Compass",
                &format!(
                    "Underworld standing {} +5 - spared",
                    faction_name("underworld")
                ),
                "Mara approves",
                "Cost",
                "Hull -12 - their broadside",
                RULE_NOTE,
            ]
        );
        seed_victory(&mut seed, &receipt(false), Some([120, 0, -12, 0]));
        let recap = combat(&seed_facts(&seed, "Captain Vane").unwrap());
        assert_eq!(texts(&recap)[0], "Took all - hold and purse");
    }

    #[test]
    fn sinking_wins_name_the_prize_or_its_absence() {
        let mut taken = step("capture", "resolved");
        taken.prize_ok = true;
        taken.prize_reason = "Gull".into();
        let mut seed = RecapSeed::default();
        seed_sinking_win(&mut seed, &taken, Some([0, 0, 0, 0]));
        let recap = combat(&seed_facts(&seed, "Captain Vane").unwrap());
        assert_eq!(recap.result.id(), "won_prize");
        assert_eq!(
            texts(&recap),
            vec!["Prize taken - Gull added to your fleet"]
        );
        let let_go = step("capture", "resolved");
        let mut seed = RecapSeed::default();
        seed_sinking_win(&mut seed, &let_go, Some([0, 0, 0, 0]));
        assert_eq!(
            texts(&combat(&seed_facts(&seed, "X").unwrap())),
            vec!["Sunk - no prize"]
        );
        let mut sunk = naval("broadside", 0, 0, 1);
        sunk.phase = "resolved".into();
        sunk.enemy_sunk = true;
        sunk.player_action = "broadside".into();
        sunk.prize_reason = "Their hull was lost.".into();
        let mut seed = carry_seed(&RecapSeed::default(), &sunk);
        seed_sinking_win(&mut seed, &sunk, Some([0, 0, 0, 0]));
        let recap = combat(&seed_facts(&seed, "X").unwrap());
        assert_eq!(recap.result.id(), "won_sunk");
        assert_eq!(
            recap.headline,
            "Won against X - sank their ship with a broadside"
        );
        assert_eq!(texts(&recap), vec!["Sunk - no prize - Their hull was lost"]);
        // No deltas yet (capture fills them after its own reading): no recap.
        let mut seed = RecapSeed::default();
        seed_sinking_win(&mut seed, &taken, None);
        assert_eq!(seed_facts(&seed, "X"), None);
    }

    #[test]
    fn a_new_encounter_starts_a_fresh_seed() {
        let mut facts =
            crate::logic::facts_for_catalog_captain(crate::logic::SCRIPTED_CAPTAIN, None, true)
                .expect("catalog captain");
        facts.recap.hull_by_cause = [5, 0, 0];
        facts.phase = "naval".into();
        assert_eq!(seed_base(Some(&facts)).hull_by_cause, [5, 0, 0]);
        facts.pending_victory = true;
        facts.phase = "resolved".into();
        assert_eq!(seed_base(Some(&facts)).hull_by_cause, [5, 0, 0]);
        facts.pending_victory = false;
        assert_eq!(seed_base(Some(&facts)), RecapSeed::default());
        assert_eq!(seed_base(None), RecapSeed::default());
    }

    #[test]
    fn a_non_win_draws_no_note_row() {
        let mut last = step("fight", "resolved");
        last.player_action = "parry".into();
        last.message = "You raise your guard but the bullet punches through. 6 damage!".into();
        let mut seed = carry_seed(&RecapSeed::default(), &last);
        seed.result = Some(RecapResult::Lost);
        seed.deltas = Some([-30, 0, 0, 0]);
        let recap = combat(&seed_facts(&seed, "Raj the Quiet").unwrap());
        assert!(recap.rows.iter().all(|r| r.section != RecapSection::Note));
        assert_eq!(texts(&recap), vec!["Cost", "Silver -30 - duel forfeit"]);
        for result in [RecapResult::Lost, RecapResult::Drew, RecapResult::BrokeAway] {
            let mut facts = loss();
            facts.result = Some(result);
            facts.notes = vec!["A note.".into()];
            let recap = combat(&facts);
            assert!(recap.rows.iter().all(|r| r.section != RecapSection::Note));
        }
        assert_eq!(
            recap_text(&recap),
            "Lost to Raj the Quiet - bested in the duel\nCost\nSilver -30 - duel forfeit"
        );
    }

    #[test]
    fn an_empty_contract_list_draws_nothing() {
        assert_eq!(contract_recap(&[], &ContractExtras::default()), None);
    }

    #[test]
    fn passthrough_keeps_an_ellipsis_and_trims_without_dots() {
        assert_eq!(passthrough_text("Tom waits..."), "Tom waits...");
        assert_eq!(passthrough_text("Mara approves. "), "Mara approves");
        let long = format!("{}end.", "Mara says the wind is turning. ".repeat(5));
        let row = passthrough_text(&long);
        assert!(row.len() <= ROW_CHARS, "{row}");
        assert!(!row.ends_with('.'), "{row}");
        assert!(long.starts_with(&row));
    }

    fn texts(recap: &Recap) -> Vec<&str> {
        recap.rows.iter().map(|r| r.text.as_str()).collect()
    }

    fn win(result: RecapResult) -> CombatFacts {
        CombatFacts {
            result: Some(result),
            sea: true,
            name: "Captain Vane".into(),
            decisive: Some(DecisiveMove::Boarded(3, 3)),
            purse: 120,
            deltas: [132, -1, -12, 0],
            hull_by_cause: [12, 0, 0],
            crew_by_cause: [0, 1],
            reached_boarding: true,
            ..Default::default()
        }
    }

    fn loss() -> CombatFacts {
        CombatFacts {
            result: Some(RecapResult::Lost),
            sea: true,
            name: "Captain Vane".into(),
            decisive: Some(DecisiveMove::SunkBy("broadside".into())),
            deltas: [-40, -2, -30, -5],
            hull_by_cause: [30, 0, 0],
            crew_by_cause: [2, 0],
            plundered: true,
            bounty_open: true,
            ..Default::default()
        }
    }

    #[test]
    fn mixed_losses_split_into_one_row_per_cause() {
        let mut facts = loss();
        facts.decisive = Some(DecisiveMove::CrewLost);
        facts.hull_by_cause = [18, 12, 0];
        facts.crew_by_cause = [1, 1];
        facts.bounty_open = false;
        let recap = combat(&facts);
        let rows = texts(&recap);
        assert_eq!(
            rows,
            vec![
                "Cost",
                "Hull -18 - their broadside",
                "Hull -12 - their grapeshot",
                "Crew -1 - their grapeshot",
                "Crew -1 - boarding melee",
                "Silver -40 - plundered",
                "Cargo -5 - plundered",
            ]
        );
        // Sums that miss the loss keep the bare row.
        facts.hull_by_cause = [18, 5, 0];
        let recap = combat(&facts);
        let rows = texts(&recap);
        assert!(rows.contains(&"Hull -30"), "{rows:?}");
        // A split that would push the body past the cap keeps the bare rows.
        facts.hull_by_cause = [18, 12, 0];
        facts.injuries = vec!["A".into(), "B".into()];
        facts.notes = vec![];
        facts.bounty_open = true;
        facts.decisive = Some(DecisiveMove::SunkBy("broadside".into()));
        facts.hull_max = Some(60);
        facts.towed_to = "porto_novo".into();
        facts.wreck_silver = 40;
        facts.wreck_cargo = vec![("grain".into(), 5)];
        facts.hull_after = 6;
        let recap = combat(&facts);
        let rows = texts(&recap);
        let body = recap
            .rows
            .iter()
            .filter(|r| r.section.counts_toward_cap())
            .count();
        assert!(body <= BODY_ROW_CAP, "{rows:?}");
        assert_eq!(
            rows.iter()
                .filter(|row| row.starts_with("Hull "))
                .collect::<Vec<_>>(),
            vec![&"Hull -30"],
            "{rows:?}"
        );
    }

    #[test]
    fn a_sinking_keeps_its_cause_and_shows_the_empty_hull() {
        let mut facts = loss();
        facts.plundered = false;
        facts.deltas = [0, 0, -30, 0];
        facts.crew_by_cause = [0, 0];
        facts.hull_by_cause = [41, 0, 0];
        facts.hull_max = Some(60);
        facts.bounty_open = false;
        facts.towed_to = "porto_novo".into();
        facts.hull_after = 6;
        let port = port_display_name("porto_novo");
        let recap = combat(&facts);
        assert_eq!(
            recap.headline,
            "Lost to Captain Vane - sunk by their broadside"
        );
        assert_eq!(
            texts(&recap),
            vec![
                format!("Towed to {port}").as_str(),
                "Cost",
                // Towed: the fight's own damage, not the patched delta, at
                // most the 36 the ship had (6 after the patch, delta -30).
                "Hull -36 - their broadside",
                "Your hull 6/60 - patched for the tow"
            ]
        );
        let last = recap.rows.last().unwrap();
        assert_eq!(last.tone, RecapTone::Muted);
        // Mixed: a towed ship's loss is the sum of the causes, so each
        // cause keeps its own amount.
        facts.hull_by_cause = [30, 6, 0];
        assert_eq!(
            texts(&combat(&facts))[2..4],
            ["Hull -30 - their broadside", "Hull -6 - their grapeshot"]
        );
    }

    fn sunk() -> CombatFacts {
        CombatFacts {
            result: Some(RecapResult::Lost),
            sea: true,
            name: "Captain Vane".into(),
            decisive: Some(DecisiveMove::SunkBy("broadside".into())),
            deltas: [-150, 0, -30, -20],
            hull_by_cause: [30, 0, 0],
            hull_max: Some(60),
            towed_to: "porto_novo".into(),
            wreck_silver: 150,
            wreck_cargo: vec![
                ("spice".into(), 2),
                ("silk".into(), 15),
                ("grain".into(), 3),
            ],
            hull_after: 6,
            ..Default::default()
        }
    }

    #[test]
    fn a_towed_ship_shows_the_fight_hull_loss_not_the_patch() {
        // The tow patched the hull, so the delta reads +18; the fight did 30.
        let mut facts = sunk();
        facts.deltas[2] = 18;
        facts.hull_after = 48;
        facts.hull_by_cause = [30, 0, 0];
        let recap = combat(&facts);
        let rows = texts(&recap);
        assert!(rows.contains(&"Hull -30 - their broadside"), "{rows:?}");
    }

    #[test]
    fn every_wreck_row_is_due() {
        let mut facts = sunk();
        facts.wreck_cargo.push(("rum".into(), 1));
        let recap = combat(&facts);
        let wreck: Vec<&RecapRow> = recap
            .rows
            .iter()
            .filter(|r| r.text.contains(WRECK_CAUSE))
            .collect();
        assert!(wreck.len() >= 4, "{:?}", texts(&recap));
        assert!(wreck.iter().all(|r| r.tone == RecapTone::Due));
    }

    #[test]
    fn recap_has_no_bank_row() {
        // The recap reads the wreck's own facts; banked silver is not one of
        // them, so a sinking with silver in a counting house still shows
        // only the carried loss.
        let mut facts = sunk();
        facts.claims_paid = 40;
        facts.repair_price = 90;
        let recap = combat(&facts);
        let rows = owned(&recap);
        assert!(rows.contains(&"Silver -150 - lost with the wreck".to_string()));
        for row in &rows {
            let lower = row.to_lowercase();
            assert!(!lower.contains("bank"), "{row}");
            assert!(!lower.contains("counting house"), "{row}");
            assert!(!lower.contains("letter"), "{row}");
        }
    }

    #[test]
    fn sink_recap_full_receipt_order() {
        let mut facts = sunk();
        facts.wreck_cargo.truncate(1);
        facts.planned_destination = "silva_bay".into();
        facts.claims_paid = 210;
        facts.deltas = [60, 0, -30, -2];
        facts.repair_price = 162;
        let recap = combat(&facts);
        let rows = owned(&recap);
        let towed = format!("Towed to {}", port_display_name("porto_novo"));
        let missed = format!("{} not reached", port_display_name("silva_bay"));
        assert_eq!(rows[0], towed);
        assert_eq!(rows[1], missed);
        let at = |text: &str| rows.iter().position(|r| r.as_str() == text).expect(text);
        assert!(at("Silver -150 - lost with the wreck") < at("Silver +210 - insurance claim"));
        assert!(at("Silver +210 - insurance claim") < at("Your hull 6/60 - patched for the tow"));
        assert!(at("Your hull 6/60 - patched for the tow") < at("Repair here - 162 silver"));
        assert!(rows.contains(&"Silver net +60".to_string()), "{rows:?}");
        // No plundered or Cargo rows, no unexplained Silver row, never 0/max.
        for row in &rows {
            assert!(
                !row.contains("plundered") && !row.starts_with("Cargo "),
                "{row}"
            );
            assert_ne!(row, "Silver +60");
            assert!(!row.contains("0/60") || row.contains("6/60"), "{row}");
            assert!(!row.ends_with('.') && row.is_ascii(), "{row}");
        }
    }

    #[test]
    fn destination_not_reached_only_when_different() {
        let mut facts = sunk();
        facts.planned_destination = "porto_novo".into();
        assert!(!owned(&combat(&facts))
            .iter()
            .any(|r| r.ends_with("not reached")));
        facts.planned_destination = String::new();
        assert!(!owned(&combat(&facts))
            .iter()
            .any(|r| r.ends_with("not reached")));
    }

    #[test]
    fn insurance_claim_row_only_when_paid() {
        let mut facts = sunk();
        assert!(!owned(&combat(&facts))
            .iter()
            .any(|r| r.contains("insurance")));
        facts.claims_paid = 40;
        facts.deltas[0] += 40;
        assert!(owned(&combat(&facts)).contains(&"Silver +40 - insurance claim".to_string()));
    }

    #[test]
    fn repair_price_row_uses_this_port() {
        let mut facts = sunk();
        facts.repair_price = 0;
        assert!(!owned(&combat(&facts))
            .iter()
            .any(|r| r.starts_with("Repair here")));
        facts.repair_price = 270;
        assert_eq!(
            owned(&combat(&facts)).last().map(String::as_str),
            Some("Repair here - 270 silver")
        );
        // Whole hull: no repair row.
        facts.hull_after = 60;
        assert!(!owned(&combat(&facts))
            .iter()
            .any(|r| r.starts_with("Repair here")));
    }

    #[test]
    fn sea_sink_headlines_name_the_event() {
        for (cause, headline) in [
            ("storm", "Sunk by the storm"),
            ("pirates", "Sunk by pirates"),
            ("event", "Sunk at sea"),
        ] {
            let mut facts = sunk();
            facts.name = String::new();
            facts.decisive = None;
            facts.sink_cause = cause.into();
            facts.hull_by_cause = [0, 0, 0];
            let recap = combat(&facts);
            assert_eq!(recap.headline, headline);
            assert!(owned(&recap).contains(&"Your hull 6/60 - patched for the tow".to_string()));
        }
    }

    #[test]
    fn parting_shot_sink_is_lost_not_broke_away() {
        let mut facts = sunk();
        facts.decisive = Some(DecisiveMove::SunkBy("parting shot".into()));
        facts.sink_cause = "parting_shot".into();
        assert_eq!(
            combat(&facts).headline,
            "Lost to Captain Vane - sunk by a parting shot"
        );
        let mut step = EncounterStep {
            kind: "naval".into(),
            phase: "resolved".into(),
            choice: "flee".into(),
            escaped: true,
            player_sunk: true,
            ..Default::default()
        };
        step.success = true;
        assert_eq!(
            crate::logic::encounter_end(&step, false),
            Some(crate::logic::EncounterEnd::LostAtSea)
        );
    }

    #[test]
    fn repair_then_not_reached_drop_first_at_the_cap() {
        let mut facts = sunk();
        facts.planned_destination = "silva_bay".into();
        facts.claims_paid = 10;
        facts.deltas[0] += 10;
        facts.repair_price = 99;
        facts.injuries = vec!["Broken arm".into(), "Gash".into()];
        let rows = owned(&combat(&facts));
        assert!(rows[0].starts_with("Towed to"));
        assert!(
            !rows.iter().any(|r| r.starts_with("Repair here")),
            "{rows:?}"
        );
    }

    fn owned(recap: &Recap) -> Vec<String> {
        texts(recap).iter().map(|row| row.to_string()).collect()
    }

    #[test]
    fn sink_recap_rows() {
        let recap = combat(&sunk());
        let spice = good_display_name("spice");
        let silk = good_display_name("silk");
        let port = port_display_name("porto_novo");
        assert_eq!(
            recap.headline,
            "Lost to Captain Vane - sunk by their broadside"
        );
        assert_eq!(
            texts(&recap),
            vec![
                format!("Towed to {port}"),
                "Cost".to_string(),
                "Hull -30 - their broadside".to_string(),
                "Silver -150 - lost with the wreck".to_string(),
                format!("{spice} -2 - lost with the wreck"),
                format!("{silk} -15 - lost with the wreck"),
                "+1 more goods lost with the wreck".to_string(),
                "Your hull 6/60 - patched for the tow".to_string(),
            ]
        );
        assert_eq!(recap.rows[0].section, RecapSection::Choice);
        assert_eq!(recap.rows[0].tone, RecapTone::Cream);
        assert_eq!(recap.rows.last().unwrap().tone, RecapTone::Muted);
        // No Cargo aggregate, no Silver net, no trailing stop, ASCII.
        for row in texts(&recap) {
            assert!(
                !row.starts_with("Cargo ") && !row.starts_with("Silver net"),
                "{row}"
            );
            assert!(!row.ends_with('.') && row.is_ascii(), "{row}");
        }
    }

    #[test]
    fn sink_wreck_cause_only_from_data() {
        // A sinking's card keeps to the wreck's own rows: silver lost before
        // the sink draws no extra plain Silver row beside them.
        let mut facts = sunk();
        facts.deltas = [-180, 0, -30, -20];
        let recap = combat(&facts);
        let rows = texts(&recap);
        assert!(
            rows.contains(&"Silver -150 - lost with the wreck"),
            "{rows:?}"
        );
        assert!(!rows.contains(&"Silver -30"), "{rows:?}");
        assert_eq!(rows.iter().filter(|r| r.starts_with("Silver ")).count(), 1);
        // A loss with no wreck data never names the wreck.
        let rows = texts(&combat(&loss())).join("|");
        assert!(!rows.contains("wreck") && !rows.contains("Towed"), "{rows}");
    }

    #[test]
    fn sink_goods_cap_then_more() {
        let mut facts = sunk();
        facts.wreck_cargo.truncate(2);
        assert!(!texts(&combat(&facts))
            .iter()
            .any(|r| r.contains("more goods")));
        facts.wreck_cargo = (0..5).map(|i| (format!("good_{i}"), 1)).collect();
        let recap = combat(&facts);
        let rows = texts(&recap);
        assert!(
            rows.contains(&"+3 more goods lost with the wreck"),
            "{rows:?}"
        );
    }

    #[test]
    fn sink_seed_reads_the_tow_step() {
        let mut last = naval("broadside", -30, 0, 0);
        last.phase = "resolved".into();
        last.player_sunk = true;
        last.towed_to = "porto_novo".into();
        last.sink_silver_lost = 150;
        last.sink_cargo_lost = vec![("silk".into(), 15)];
        last.hull_after = 6;
        let seed = carry_seed(&RecapSeed::default(), &last);
        assert_eq!(seed.towed_to, "porto_novo");
        assert_eq!(seed.wreck_silver, 150);
        assert_eq!(seed.wreck_cargo, vec![("silk".to_string(), 15)]);
        assert_eq!(seed.hull_after, 6);
    }

    #[test]
    fn a_flee_sinking_is_a_loss_at_sea() {
        let mut flee = step("choice", "resolved");
        flee.choice = "flee".into();
        flee.hull_damage = 6;
        flee.player_sunk = true;
        let seed = carry_seed(&RecapSeed::default(), &flee);
        assert_eq!(
            seed.decisive,
            Some(DecisiveMove::SunkBy("broadside".into()))
        );
        assert_eq!(seed.hull_by_cause, [6, 0, 0]);
        assert_eq!(
            crate::logic::encounter_end(&flee, false),
            Some(crate::logic::EncounterEnd::LostAtSea)
        );
        flee.escaped = true;
        assert_eq!(
            crate::logic::encounter_end(&flee, false),
            Some(crate::logic::EncounterEnd::LostAtSea)
        );
    }

    #[test]
    fn towed_to_uses_port_name() {
        let recap = combat(&sunk());
        let rows = texts(&recap);
        assert!(!rows[0].contains("porto_novo"), "{rows:?}");
        assert_eq!(
            rows[0],
            format!("Towed to {}", port_display_name("porto_novo"))
        );
    }

    #[test]
    fn a_lost_duel_names_the_forfeit() {
        let mut duel = step("fight", "resolved");
        duel.player_action = "parry".into();
        let seed = carry_seed(&RecapSeed::default(), &duel);
        assert_eq!(seed.decisive, Some(DecisiveMove::BestedInDuel));
        let facts = CombatFacts {
            result: Some(RecapResult::Lost),
            name: "Bram".into(),
            decisive: seed.decisive,
            deltas: [-36, 0, 0, 0],
            ..Default::default()
        };
        let recap = combat(&facts);
        assert_eq!(recap.headline, "Lost to Bram - bested in the duel");
        assert_eq!(texts(&recap), vec!["Cost", "Silver -36 - duel forfeit"]);
        // A draw gets no clause.
        let draw = CombatFacts {
            result: Some(RecapResult::Drew),
            ..facts
        };
        assert_eq!(combat(&draw).headline, "Drew with Bram");
    }

    #[test]
    fn a_failed_approach_flee_is_their_broadside() {
        let mut flee = step("choice", "naval");
        flee.choice = "flee".into();
        flee.hull_damage = 7;
        let seed = carry_seed(&RecapSeed::default(), &flee);
        assert_eq!(seed.hull_by_cause, [7, 0, 0]);
    }

    fn outcome(kind: &str, silver: i64, reward: i64, delivered: i64) -> ContractOutcome {
        ContractOutcome {
            contract_id: "c1".into(),
            outcome_type: kind.into(),
            silver_delta: silver,
            trust_delta: 0,
            standing_delta: 0,
            heat_delta: 0,
            completion_day: 9,
            summary: String::new(),
            family: "procurement".into(),
            good_id: "grain".into(),
            required_quantity: 23,
            delivered_quantity: delivered,
            destination_port_id: "porto_novo".into(),
            deadline_day: 20,
            reward_silver: reward,
        }
    }

    #[test]
    fn loot_folds_into_one_row_only_on_overflow() {
        let mut facts = win(RecapResult::WonTookAll);
        facts.loot = vec!["+12 silver".into(), "Found: Compass".into()];
        let fits = combat(&facts);
        let rows = texts(&fits);
        assert!(rows.contains(&"Silver +12 - loot"), "{rows:?}");
        assert!(rows.contains(&"Found: Compass"), "{rows:?}");
        assert!(!rows.iter().any(|r| r.starts_with("Loot -")), "{rows:?}");

        // Fight cost, standing, a note and the bounty push it over the cap.
        facts.deltas = [facts.purse + 12 - 30, -1, -12, -4];
        facts.standing = Some(("underworld".into(), 5));
        facts.notes = vec!["Mara approves".into()];
        facts.bounty_ready = true;
        let over = combat(&facts);
        let rows = texts(&over);
        assert!(rows.contains(&"Loot - 1 item - Silver +12"), "{rows:?}");
        assert!(!rows.contains(&"Silver +12 - loot"), "{rows:?}");
        assert!(!rows.contains(&"Found: Compass"), "{rows:?}");
        for kept in ["Silver -30", "Cargo -4", "Silver net +102", BOUNTY_READY] {
            assert!(rows.contains(&kept), "{kept} {rows:?}");
        }
        let body = over
            .rows
            .iter()
            .filter(|r| r.section.counts_toward_cap())
            .count();
        assert!(body <= BODY_ROW_CAP, "{rows:?}");
        assert!(!rows.contains(&RULE_NOTE), "{rows:?}");
        // The short card keeps the rule note.
        assert!(texts(&fits).contains(&RULE_NOTE));
    }

    /// A heavy won card: choice, purse, hull from three causes, crew from
    /// two, silver and cargo lost, two injuries and net (12 rows on a 9-row
    /// cap), with the target claimable.
    fn heavy_win() -> CombatFacts {
        let mut facts = win(RecapResult::WonTookAll);
        facts.purse = 120;
        facts.deltas = [90, -2, -12, -4];
        facts.hull_by_cause = [5, 4, 3];
        facts.crew_by_cause = [1, 1];
        facts.injuries = vec!["Cut".into(), "Bruise".into()];
        facts.bounty_ready = true;
        facts
    }

    #[test]
    fn injuries_fold_into_one_row_only_on_overflow() {
        let heavy = combat(&heavy_win());
        let rows = texts(&heavy);
        assert!(rows.contains(&"Injuries - Cut, Bruise"), "{rows:?}");
        assert!(!rows.iter().any(|r| r.starts_with("Injury - ")), "{rows:?}");
        assert_eq!(
            rows.iter().position(|r| *r == "Injuries - Cut, Bruise"),
            rows.iter().position(|r| *r == "Cargo -4").map(|i| i + 1),
            "{rows:?}"
        );
        let body = heavy
            .rows
            .iter()
            .filter(|r| r.section.counts_toward_cap())
            .count();
        assert!(body <= BODY_ROW_CAP, "{rows:?}");
        // A card that fits keeps one row per injury.
        let mut light = win(RecapResult::WonSpared);
        light.injuries = vec!["Cut".into(), "Bruise".into()];
        let rows = texts(&combat(&light)).join("|");
        assert!(rows.contains("Injury - Cut|Injury - Bruise"), "{rows}");
        assert!(!rows.contains("Injuries"), "{rows}");
    }

    #[test]
    fn loot_summary_copy() {
        assert_eq!(loot_summary(3, 20), "Loot - 3 items - Silver +20");
        assert_eq!(loot_summary(1, 12), "Loot - 1 item - Silver +12");
        assert_eq!(loot_summary(2, 0), "Loot - 2 items");
        assert_eq!(loot_summary(1, 0), "Loot - 1 item");
        assert_eq!(loot_summary(0, 9), "Loot - Silver +9");
        for text in [loot_summary(3, 20), loot_summary(1, 0)] {
            assert!(!text.ends_with('.'), "{text}");
        }
    }

    #[test]
    fn win_rows_with_an_injury_follow_the_order() {
        let mut facts = win(RecapResult::WonSpared);
        facts.loot = vec!["+12 silver".into(), "3 Rum".into()];
        facts.injuries = vec!["Cracked rib".into()];
        let recap = combat(&facts);
        assert_eq!(
            texts(&recap),
            vec![
                "Spared",
                "Earned",
                "Silver +120 - purse",
                "Silver +12 - loot",
                "3 Rum",
                "Cost",
                "Hull -12 - their broadside",
                "Crew -1 - boarding melee",
                "Injury - Cracked rib",
                RULE_NOTE,
            ]
        );
    }

    #[test]
    fn the_bounty_row_survives_a_heavy_win() {
        let recap = combat(&heavy_win());
        let rows = texts(&recap);
        assert_eq!(
            rows.iter().filter(|r| **r == BOUNTY_READY).count(),
            1,
            "{rows:?}"
        );
        assert!(!rows.contains(&BOUNTY_NOTE), "{rows:?}");
        let body = recap
            .rows
            .iter()
            .filter(|r| r.section.counts_toward_cap())
            .count();
        assert!(body <= BODY_ROW_CAP + 1, "{rows:?}");
    }

    #[test]
    fn a_heavy_non_win_keeps_the_bounty_open_note() {
        for result in [RecapResult::Lost, RecapResult::Drew, RecapResult::BrokeAway] {
            let mut facts = loss();
            facts.result = Some(result);
            facts.injuries = vec!["Cut".into(), "Bruise".into()];
            facts.bounty_open = true;
            facts.bounty_ready = true;
            let rows = texts(&combat(&facts)).join("|");
            assert!(rows.contains(BOUNTY_NOTE), "{rows}");
            assert!(!rows.contains(BOUNTY_READY), "{rows}");
        }
    }

    #[test]
    fn win_cost_subtracts_loot_silver() {
        let rows = |delta: i64| {
            let mut facts = win(RecapResult::WonTookAll);
            facts.purse = 120;
            facts.loot = vec!["+12 silver".into()];
            facts.deltas = [delta, 0, 0, 0];
            texts(&combat(&facts))
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
        };
        // Purse 120, loot 12, the fight cost 30: change +102.
        let cost = rows(102);
        assert!(cost.contains(&"Silver -30".to_string()), "{cost:?}");
        assert!(cost.contains(&"Silver net +102".to_string()), "{cost:?}");
        // Change +132: purse and loot only.
        let clean = rows(132);
        assert!(
            !clean.iter().any(|r| r.starts_with("Silver -")),
            "{clean:?}"
        );
        assert!(
            !clean.iter().any(|r| r.starts_with("Silver net")),
            "{clean:?}"
        );
    }

    #[test]
    fn win_cost_shows_silver_and_cargo_only_when_they_fell() {
        // Purse +55, a duel stake -30: delta +25.
        let mut facts = win(RecapResult::WonSpared);
        facts.purse = 55;
        facts.deltas = [25, 0, 0, -4];
        let rows = texts(&combat(&facts))
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>();
        assert!(rows.contains(&"Silver +55 - purse".to_string()), "{rows:?}");
        assert!(rows.contains(&"Silver -30".to_string()), "{rows:?}");
        assert!(rows.contains(&"Cargo -4".to_string()), "{rows:?}");
        assert!(rows.contains(&"Silver net +25".to_string()), "{rows:?}");
        // Silver fell with no purse: the row, and no net (nothing rose).
        facts.purse = 0;
        facts.deltas = [-30, 0, 0, 0];
        let rows = texts(&combat(&facts))
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>();
        assert!(rows.contains(&"Silver -30".to_string()), "{rows:?}");
        assert!(
            !rows.iter().any(|r| r.starts_with("Silver net")),
            "{rows:?}"
        );
        assert!(!rows.iter().any(|r| r.starts_with("Cargo")), "{rows:?}");
        // No silver or cargo change beyond the purse: no such rows.
        let rows = texts(&combat(&win(RecapResult::WonTookAll)))
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>();
        assert!(
            !rows.iter().any(|r| r.starts_with("Silver -")
                || r.starts_with("Cargo")
                || r.starts_with("Silver net")),
            "{rows:?}"
        );
        for row in rows
            .iter()
            .filter(|r| r.starts_with("Silver") || r.starts_with("Cargo"))
        {
            assert!(!row.ends_with(" 0") && !row.ends_with('.'), "{row}");
        }
    }

    #[test]
    fn won_recap_shows_bounty_ready_once() {
        let mut facts = win(RecapResult::WonSpared);
        facts.bounty_ready = true;
        let recap = combat(&facts);
        let rows: Vec<&RecapRow> = recap
            .rows
            .iter()
            .filter(|r| r.text == BOUNTY_READY || r.text == BOUNTY_NOTE)
            .collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].text, BOUNTY_READY);
        assert_eq!(rows[0].section, RecapSection::Bounty);
        assert_eq!(rows[0].tone, RecapTone::Muted);
        // A win never shows the non-win note, even with the flag set.
        facts.bounty_open = true;
        assert!(!texts(&combat(&facts)).contains(&BOUNTY_NOTE));
    }

    #[test]
    fn won_recap_without_ready_bounty_has_no_bounty_row() {
        // Not a target, or already claimed: the seed carries no flag.
        let recap = combat(&win(RecapResult::WonTookAll));
        assert!(recap.rows.iter().all(|r| r.section != RecapSection::Bounty));
    }

    #[test]
    fn non_win_keeps_only_bounty_stays_open() {
        let mut facts = loss();
        facts.bounty_ready = true;
        let recap = combat(&facts);
        let bounty: Vec<&str> = recap
            .rows
            .iter()
            .filter(|r| r.section == RecapSection::Bounty)
            .map(|r| r.text.as_str())
            .collect();
        assert_eq!(bounty, [BOUNTY_NOTE]);
    }

    fn all_combat() -> Vec<Recap> {
        let mut out = Vec::new();
        for result in [
            RecapResult::WonSpared,
            RecapResult::WonTookAll,
            RecapResult::WonPrize,
            RecapResult::WonSunk,
        ] {
            let mut facts = win(result);
            facts.prize_ship = "Gull".into();
            facts.no_prize_reason = "Their hull was lost.".into();
            facts.standing = Some(("underworld".into(), 5));
            out.push(combat(&facts));
        }
        out.push(combat(&loss()));
        out.push(combat(&CombatFacts {
            result: Some(RecapResult::Drew),
            name: "Captain Vane".into(),
            ..Default::default()
        }));
        out.push(combat(&CombatFacts {
            result: Some(RecapResult::BrokeAway),
            sea: true,
            name: "Captain Vane".into(),
            decisive: Some(DecisiveMove::FledAtSea(2)),
            deltas: [0, 0, -4, 0],
            hull_by_cause: [0, 0, 4],
            ..Default::default()
        }));
        // Notes and loot passthrough, as the sim writes them.
        let mut cards = win(RecapResult::WonTookAll);
        cards.notes = vec![
            "Mara approves.".into(),
            "Tom leaves. He wanted a bigger ship.".into(),
        ];
        cards.loot = vec!["+12 silver".into(), "Found: Compass.".into()];
        out.push(combat(&cards));
        out
    }

    fn all_contract() -> Vec<Recap> {
        vec![
            contract(
                &[outcome("completed_bonus", 672, 612, 23)],
                &ContractExtras::default(),
            ),
            contract(
                &[
                    outcome("completed", 300, 300, 10),
                    outcome("completed", 200, 200, 5),
                ],
                &ContractExtras::default(),
            ),
            contract(
                &[outcome("abandoned", 0, 612, 9)],
                &ContractExtras { held: vec![22] },
            ),
        ]
    }

    #[test]
    fn each_combat_result_has_its_headline_and_id() {
        let recaps = all_combat();
        let heads: Vec<(&str, &str)> = recaps
            .iter()
            .map(|r| (r.result.id(), r.headline.as_str()))
            .collect();
        assert_eq!(
            heads,
            vec![
                ("won_spared", "Won against Captain Vane - boarded at 3/3"),
                ("won_took_all", "Won against Captain Vane - boarded at 3/3"),
                ("won_prize", "Won against Captain Vane - boarded at 3/3"),
                ("won_sunk", "Won against Captain Vane - boarded at 3/3"),
                ("lost", "Lost to Captain Vane - sunk by their broadside"),
                ("drew", "Drew with Captain Vane"),
                (
                    "broke_away",
                    "Broke away from Captain Vane - fled in round 2"
                ),
                ("won_took_all", "Won against Captain Vane - boarded at 3/3"),
            ]
        );
        assert_eq!(texts(&recaps[0])[0], "Spared");
        assert_eq!(texts(&recaps[1])[0], "Took all - hold and purse");
        assert_eq!(
            texts(&recaps[2])[0],
            "Prize taken - Gull added to your fleet"
        );
        assert_eq!(
            texts(&recaps[3])[0],
            "Sunk - no prize - Their hull was lost"
        );
    }

    #[test]
    fn clause_table_names_only_allowed_moves() {
        use DecisiveMove::*;
        let won = RecapResult::WonSpared;
        assert_eq!(
            decisive_clause(won, true, Some(&Sank("rake".into()))).unwrap(),
            "sank their ship with grapeshot"
        );
        assert_eq!(
            decisive_clause(won, true, Some(&Sank("broadside".into()))).unwrap(),
            "sank their ship with a broadside"
        );
        assert_eq!(
            decisive_clause(won, false, Some(&FinalMove("thrust".into()))).unwrap(),
            "final move Thrust"
        );
        assert_eq!(
            decisive_clause(RecapResult::Lost, true, Some(&SunkBy("rake".into()))).unwrap(),
            "sunk by their grapeshot"
        );
        assert_eq!(
            decisive_clause(RecapResult::Lost, true, Some(&CrewLost)).unwrap(),
            "crew lost"
        );
        assert_eq!(
            decisive_clause(RecapResult::BrokeAway, true, Some(&FledApproach)).unwrap(),
            "fled before the guns"
        );
        assert_eq!(
            decisive_clause(RecapResult::Lost, false, Some(&FinalMove("thrust".into()))),
            None
        );
        assert_eq!(
            decisive_clause(RecapResult::Drew, false, Some(&FinalMove("thrust".into()))),
            None
        );
        assert_eq!(
            decisive_clause(won, true, Some(&Sank("evade".into()))),
            None
        );
        assert_eq!(
            decisive_clause(RecapResult::Lost, true, Some(&Boarded(3, 3))),
            None
        );
        assert_eq!(decisive_clause(won, true, None), None);
    }

    #[test]
    fn clause_numbers_are_verbatim() {
        let mut facts = win(RecapResult::WonSpared);
        facts.decisive = Some(DecisiveMove::Boarded(7, 12));
        assert!(combat(&facts).headline.ends_with(" - boarded at 7/12"));
        facts.result = Some(RecapResult::BrokeAway);
        facts.decisive = Some(DecisiveMove::FledAtSea(11));
        assert!(combat(&facts).headline.ends_with(" - fled in round 11"));
    }

    #[test]
    fn decisive_move_named_once() {
        for recap in all_combat() {
            for row in &recap.rows {
                for clause in [
                    "boarded at",
                    "sank their",
                    "sunk by",
                    "final move",
                    "fled ",
                    "crew lost",
                ] {
                    assert!(!row.text.contains(clause), "{} in {}", clause, row.text);
                }
            }
        }
    }

    #[test]
    fn win_rows_follow_the_order() {
        let mut facts = win(RecapResult::WonSpared);
        facts.loot = vec!["+12 silver".into(), "3 Rum".into()];
        facts.standing = Some(("underworld".into(), 5));
        facts.notes = vec!["Mara approves.".into()];
        let recap = combat(&facts);
        assert_eq!(
            texts(&recap),
            vec![
                "Spared",
                "Earned",
                "Silver +120 - purse",
                "Silver +12 - loot",
                "3 Rum",
                &format!(
                    "Underworld standing {} +5 - spared",
                    faction_name("underworld")
                ),
                "Mara approves",
                "Cost",
                "Hull -12 - their broadside",
                "Crew -1 - boarding melee",
                RULE_NOTE,
            ]
        );
    }

    #[test]
    fn took_all_standing_cause() {
        let mut facts = win(RecapResult::WonTookAll);
        facts.standing = Some(("underworld".into(), 2));
        assert!(texts(&combat(&facts))
            .iter()
            .any(|t| t.ends_with(" +2 - took all")));
    }

    #[test]
    fn loss_rows_follow_the_order() {
        let recap = combat(&loss());
        assert_eq!(recap.headline_tone, RecapTone::Due);
        assert_eq!(
            texts(&recap),
            vec![
                "Cost",
                "Hull -30 - their broadside",
                "Crew -2 - their grapeshot",
                "Silver -40 - plundered",
                "Cargo -5 - plundered",
                BOUNTY_NOTE,
            ]
        );
    }

    #[test]
    fn zero_terms_are_omitted() {
        let recap = combat(&CombatFacts {
            result: Some(RecapResult::Drew),
            name: "Captain Vane".into(),
            ..Default::default()
        });
        assert!(recap.rows.is_empty());
        let mut facts = win(RecapResult::WonSpared);
        facts.purse = 0;
        facts.deltas = [0, 0, 0, 0];
        facts.reached_boarding = false;
        assert_eq!(texts(&combat(&facts)), vec!["Spared"]);
    }

    #[test]
    fn a_win_shows_silver_and_cargo_cost_without_a_cause() {
        let mut facts = win(RecapResult::WonPrize);
        facts.purse = 0;
        facts.deltas = [-10, 0, 0, -3];
        let recap = combat(&facts);
        let rows = texts(&recap);
        assert!(rows.contains(&"Silver -10"), "{rows:?}");
        assert!(rows.contains(&"Cargo -3"), "{rows:?}");
    }

    #[test]
    fn cause_only_from_data() {
        // Sums that disagree with the diff render plain (mixed sums that
        // agree split; see mixed_losses_split_into_one_row_per_cause).
        let mut facts = loss();
        facts.hull_by_cause = [20, 0, 5];
        facts.crew_by_cause = [1, 0];
        facts.plundered = false;
        let rows = texts(&combat(&facts)).join("|");
        assert!(rows.contains("|Hull -30|"), "{rows}");
        assert!(rows.contains("|Crew -2|"), "{rows}");
        assert!(rows.contains("|Silver -40|"), "{rows}");
        let mut parting = win(RecapResult::WonSpared);
        parting.hull_by_cause = [0, 0, 12];
        assert!(texts(&combat(&parting)).contains(&"Hull -12 - parting shot"));
    }

    #[test]
    fn loot_silver_is_counted_once() {
        let mut facts = win(RecapResult::WonTookAll);
        facts.loot = vec!["+12 silver".into(), "+8 silver".into()];
        facts.deltas[0] = facts.purse + 20;
        let recap = combat(&facts);
        let rows = texts(&recap);
        assert_eq!(
            rows.iter()
                .filter(|t| t.contains("silver") || t.contains("- loot"))
                .count(),
            1
        );
        assert!(rows.contains(&"Silver +20 - loot"));
    }

    #[test]
    fn loot_goods_cap_at_two_then_more() {
        let mut facts = win(RecapResult::WonTookAll);
        facts.loot = vec![
            "3 Rum".into(),
            "2 Silk".into(),
            "1 Spice".into(),
            "4 Iron".into(),
        ];
        let recap = combat(&facts);
        let rows = texts(&recap);
        assert!(
            rows.contains(&"2 Silk") && rows.contains(&"+2 more") && !rows.contains(&"1 Spice")
        );
    }

    #[test]
    fn no_combined_net_row() {
        for recap in all_combat().iter().chain(all_contract().iter()) {
            assert!(
                !recap.rows.iter().any(|r| r.section == RecapSection::Net),
                "{recap:?}"
            );
        }
    }

    #[test]
    fn contract_payout_is_the_net() {
        let recap = &all_contract()[0];
        assert_eq!(recap.headline, "Contract paid - settled by your sale");
        assert_eq!(recap.result.id(), "paid");
        assert_eq!(
            texts(recap),
            vec![
                "23 Grain to Porto Novo",
                "Silver +672",
                "Includes early bonus +60"
            ]
        );
        assert!(!texts(recap).iter().any(|t| t.contains("net")));
    }

    /// RC4: `state.recap` mirrors the drawn card: no headings, the rule note
    /// as a `note`, folds and the bounty row as drawn; null with no card.
    #[test]
    fn state_recap_key() {
        use crate::playtest::recap_key;
        assert_eq!(recap_key(None), None);

        let mut facts = win(RecapResult::WonTookAll);
        facts.loot = vec!["+12 silver".into(), "Found: Compass".into()];
        let short = combat(&facts);
        let key = recap_key(Some(&short)).unwrap();
        assert_eq!((key.kind, key.result), ("combat", "won_took_all"));
        assert_eq!(key.headline, short.headline);
        assert!(
            key.rows.contains(&("note", RULE_NOTE.to_string())),
            "{key:?}"
        );

        facts.deltas = [facts.purse + 12 - 30, -1, -12, -4];
        facts.standing = Some(("underworld".into(), 5));
        facts.notes = vec!["Mara approves".into()];
        facts.bounty_ready = true;
        let over = combat(&facts);
        let key = recap_key(Some(&over)).unwrap();
        let drawn: Vec<(&str, String)> = over
            .rows
            .iter()
            .filter(|r| r.section != RecapSection::Heading)
            .map(|r| (r.section.bridge_name(), r.text.clone()))
            .collect();
        assert_eq!(key.rows, drawn);
        assert!(over.rows.iter().any(|r| r.section == RecapSection::Heading));
        assert!(!key.rows.iter().any(|(_, t)| t == "Cost"), "{key:?}");
        assert!(key
            .rows
            .contains(&("earned", "Loot - 1 item - Silver +12".to_string())));
        assert!(
            key.rows.contains(&("note", BOUNTY_READY.to_string())),
            "{key:?}"
        );

        let cards = all_contract();
        let paid = recap_key(Some(&cards[0])).unwrap();
        assert_eq!((paid.kind, paid.result), ("contract", "paid"));
        assert_eq!(paid.headline, "Contract paid - settled by your sale");
        assert_eq!(paid.rows.len(), cards[0].rows.len());
        let gone = recap_key(Some(&cards[2])).unwrap();
        assert_eq!((gone.kind, gone.result), ("contract", "abandoned"));
        assert_eq!(
            gone.rows
                .iter()
                .map(|(_, t)| t.as_str())
                .collect::<Vec<_>>(),
            texts(&cards[2])
        );
    }

    #[test]
    fn several_paid_contracts_share_one_card() {
        let recap = &all_contract()[1];
        assert_eq!(recap.headline, "Contracts paid - settled by one sale");
        assert_eq!(
            texts(recap),
            vec![
                "10 Grain to Porto Novo",
                "Silver +300",
                "5 Grain to Porto Novo",
                "Silver +200"
            ]
        );
    }

    #[test]
    fn abandoned_card_rows() {
        let recap = &all_contract()[2];
        assert_eq!(
            (recap.result.id(), recap.headline.as_str()),
            ("abandoned", "Contract abandoned")
        );
        assert_eq!(
            texts(recap),
            vec![
                "Grain for Porto Novo",
                "Progress lost - 9/23",
                "22 Grain still aboard"
            ]
        );
        let bare = contract(
            &[outcome("abandoned", 0, 612, 0)],
            &ContractExtras::default(),
        );
        assert_eq!(texts(&bare), vec!["Grain for Porto Novo"]);
    }

    #[test]
    fn cap_drops_in_the_ladder_order() {
        let mut facts = win(RecapResult::WonTookAll);
        facts.loot = vec![
            "+5 silver".into(),
            "3 Rum".into(),
            "2 Silk".into(),
            "1 Spice".into(),
        ];
        facts.standing = Some(("underworld".into(), 2));
        facts.notes = vec!["A.".into(), "B.".into()];
        facts.injuries = vec!["Cut".into(), "Bruise".into()];
        // body: choice, purse, loot silver, 3 goods rows, standing, 2 notes,
        // hull, crew, 2 injuries = 13 > 9. The loot folds to one row first
        // (10), the rule note drops, then the injuries fold (9), so both
        // notes stay.
        let recap = combat(&facts);
        let body = recap
            .rows
            .iter()
            .filter(|r| r.section.counts_toward_cap())
            .count();
        assert_eq!(body, BODY_ROW_CAP);
        let rows = texts(&recap);
        assert!(rows.contains(&"Loot - 3 items - Silver +5"), "{rows:?}");
        assert!(!rows.contains(&"3 Rum") && !rows.contains(&"Silver +5 - loot"));
        assert!(rows.contains(&"A") && rows.contains(&"B"), "{rows:?}");
        assert!(rows.contains(&"Injuries - Cut, Bruise"), "{rows:?}");
        assert!(rows.iter().any(|t| t.starts_with("Underworld standing")));
    }

    #[test]
    fn long_rows_drop_the_cause_then_trim() {
        let mut facts = win(RecapResult::WonPrize);
        facts.prize_ship = "X".repeat(120);
        facts.loot = vec![format!("{} cargo", "word ".repeat(30))];
        let recap = combat(&facts);
        for row in &recap.rows {
            assert!(row.text.len() <= ROW_CHARS, "{}", row.text);
        }
        let head = "Silver +1".to_string() + &"0".repeat(80);
        let dropped = caused(
            RecapSection::Earned,
            head.clone(),
            Some("purse"),
            RecapTone::Cream,
        );
        assert_eq!(dropped.text, head);
    }

    #[test]
    fn rows_are_ascii_without_raw_ids_or_banned_words() {
        let mut facts = loss();
        facts.name = "Captain \u{2014} Vane".into();
        let mut recaps = all_combat();
        recaps.push(combat(&facts));
        let mut ready = win(RecapResult::WonSpared);
        ready.bounty_ready = true;
        recaps.push(combat(&ready));
        recaps.extend(all_contract());
        for recap in &recaps {
            let all = std::iter::once(recap.headline.as_str())
                .chain(texts(recap))
                .collect::<Vec<_>>();
            for text in all {
                assert!(text.is_ascii(), "{text}");
                let lower = text.to_lowercase();
                for banned in [
                    "porto_novo",
                    "grain_",
                    "_",
                    "your closes",
                    "close actions",
                    "ledger",
                    "worth it",
                    "should",
                    "!",
                ] {
                    assert!(!lower.contains(banned), "{banned} in {text}");
                }
            }
        }
    }

    #[test]
    fn no_trailing_period_on_headlines_or_rows() {
        let mut passthrough = win(RecapResult::WonTookAll);
        passthrough.notes = vec![
            "Mara approves.".into(),
            "Tom leaves. He wanted a bigger ship.".into(),
        ];
        passthrough.loot = vec!["Found: Compass.".into()];
        let passthrough = combat(&passthrough);
        let rows = texts(&passthrough);
        for want in [
            "Mara approves",
            "Tom leaves. He wanted a bigger ship",
            "Found: Compass",
        ] {
            assert!(rows.contains(&want), "{want} in {rows:?}");
        }
        assert!(
            rows.contains(&RULE_NOTE),
            "the rule note keeps its full stop"
        );
        for recap in all_combat()
            .iter()
            .chain(all_contract().iter())
            .chain(std::iter::once(&passthrough))
        {
            assert!(!recap.headline.ends_with('.'), "{}", recap.headline);
            for row in recap
                .rows
                .iter()
                .filter(|r| r.section != RecapSection::RuleNote)
            {
                assert!(!row.text.ends_with('.'), "{}", row.text);
            }
        }
    }

    #[test]
    fn sunk_without_a_reason_reads_plain() {
        let mut facts = win(RecapResult::WonSunk);
        facts.no_prize_reason = String::new();
        assert_eq!(texts(&combat(&facts))[0], "Sunk - no prize");
        facts.no_prize_reason = " . ".into();
        assert_eq!(texts(&combat(&facts))[0], "Sunk - no prize");
    }

    #[test]
    fn personal_and_empty_name_headlines() {
        let mut facts = win(RecapResult::WonSpared);
        facts.sea = false;
        facts.decisive = Some(DecisiveMove::FinalMove("thrust".into()));
        assert_eq!(
            combat(&facts).headline,
            "Won against Captain Vane - final move Thrust"
        );
        facts.name = String::new();
        facts.decisive = None;
        assert_eq!(combat(&facts).headline, "Won");
    }

    #[test]
    fn no_ledger_wording() {
        for recap in all_combat().iter().chain(all_contract().iter()) {
            for text in std::iter::once(&recap.headline).chain(recap.rows.iter().map(|r| &r.text)) {
                let lower = text.to_lowercase();
                for word in ["ledger", "balance", "profit", "total"] {
                    assert!(!lower.contains(word), "{word} in {text}");
                }
            }
        }
    }

    #[test]
    fn silver_net_only_when_silver_rose_and_fell() {
        let up = row(RecapSection::Earned, "Silver +30 - purse", RecapTone::Cream);
        let down = row(RecapSection::Cost, "Silver -10", RecapTone::Due);
        assert_eq!(
            silver_net(&[up.clone(), down]).unwrap().text,
            "Silver net +20"
        );
        assert_eq!(silver_net(&[up]), None);
    }

    #[test]
    fn section_names_for_the_bridge() {
        assert_eq!(RecapSection::Choice.bridge_name(), "how");
        assert_eq!(RecapSection::Cost.bridge_name(), "cost");
        assert_eq!(RecapSection::RuleNote.bridge_name(), "note");
        assert_eq!(RecapKind::Contract, all_contract()[0].kind);
    }
}
