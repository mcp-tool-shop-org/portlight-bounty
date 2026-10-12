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

pub(crate) const RULE_NOTE: &str =
    "Either ship closing counts toward boarding, unless the other evades.";
pub(crate) const BOUNTY_NOTE: &str = "The bounty stays open";

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

    fn is_win(self) -> bool {
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
    row(
        section,
        trim_tail(
            ascii_punctuation(text).trim_end().trim_end_matches('.'),
            ROW_CHARS,
        ),
        tone,
    )
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
                _ => None,
            }
        }
        DecisiveMove::CrewLost if result == RecapResult::Lost && sea => Some("crew lost".into()),
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

/// `+12 silver` style loot message: the silver amount.
fn loot_silver(message: &str) -> Option<i64> {
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

pub(crate) fn combat_recap(facts: &CombatFacts) -> Recap {
    let result = facts.result.unwrap_or(RecapResult::Drew);
    let win = result.is_win();
    let mut headline = verdict(result, &facts.name);
    if let Some(clause) = decisive_clause(result, facts.sea, facts.decisive.as_ref()) {
        headline = format!("{headline} - {clause}");
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
        _ => None,
    };

    // Earned (wins only): purse, loot silver once, loot goods, standing.
    let mut earned = Vec::new();
    let mut goods = Vec::new();
    let mut standing = None;
    if win {
        if facts.purse > 0 {
            earned.push(caused(
                RecapSection::Earned,
                format!("Silver +{}", facts.purse),
                Some("purse"),
                RecapTone::Cream,
            ));
        }
        let loot_total: i64 = facts.loot.iter().filter_map(|m| loot_silver(m)).sum();
        if loot_total > 0 {
            earned.push(caused(
                RecapSection::Earned,
                format!("Silver +{loot_total}"),
                Some("loot"),
                RecapTone::Cream,
            ));
        }
        let others: Vec<&String> = facts
            .loot
            .iter()
            .filter(|m| loot_silver(m).is_none() && !m.trim().is_empty())
            .collect();
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
    let notes: Vec<RecapRow> = if win {
        facts
            .notes
            .iter()
            .filter(|n| !n.trim().is_empty())
            .take(PASSTHROUGH_CAP)
            .map(|n| passthrough(RecapSection::Note, n, RecapTone::Muted))
            .collect()
    } else {
        Vec::new()
    };

    // Cost: Hull, Crew, Silver, Cargo, Injury. A win shows Hull, Crew and
    // Injury only.
    let [silver, crew, hull, cargo] = facts.deltas;
    let mut cost = Vec::new();
    if hull < 0 {
        cost.push(caused(
            RecapSection::Cost,
            format!("Hull {hull}"),
            sole_cause(
                -hull,
                facts.hull_by_cause,
                ["their broadside", "their grapeshot", "parting shot"],
            ),
            RecapTone::Due,
        ));
    }
    if crew < 0 {
        cost.push(caused(
            RecapSection::Cost,
            format!("Crew {crew}"),
            sole_cause(
                -crew,
                facts.crew_by_cause,
                ["their grapeshot", "boarding melee"],
            ),
            RecapTone::Due,
        ));
    }
    let plunder = (facts.plundered && result == RecapResult::Lost).then_some("plundered");
    if !win && silver < 0 {
        cost.push(caused(
            RecapSection::Cost,
            format!("Silver {silver}"),
            plunder,
            RecapTone::Due,
        ));
    }
    if !win && cargo < 0 {
        cost.push(caused(
            RecapSection::Cost,
            format!("Cargo {cargo}"),
            plunder,
            RecapTone::Cream,
        ));
    }
    for injury in facts.injuries.iter().take(INJURY_CAP) {
        cost.push(row(
            RecapSection::Cost,
            format!("Injury - {}", injury.trim()),
            RecapTone::Cream,
        ));
    }

    let bounty = (!win && facts.bounty_open)
        .then(|| row(RecapSection::Bounty, BOUNTY_NOTE, RecapTone::Muted));

    // Silver net: only when a drawn silver row rose and another fell.
    let mut drawn = earned.clone();
    drawn.extend(cost.iter().cloned());
    let net = silver_net(&drawn);

    // Cap: drop notes beyond 1, notes, loot goods beyond 1, loot goods, the
    // standing row, the bounty note, until the body fits.
    let mut notes = notes;
    let mut bounty = bounty;
    let fixed =
        usize::from(choice.is_some()) + earned.len() + cost.len() + usize::from(net.is_some());
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
        } else if goods.len() > 1 {
            goods.truncate(1);
        } else if !goods.is_empty() {
            goods.clear();
        } else if standing.is_some() {
            standing = None;
        } else if bounty.is_some() {
            bounty = None;
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
    if facts.reached_boarding {
        rows.push(row(RecapSection::RuleNote, RULE_NOTE, RecapTone::Muted));
    }
    rows.extend(bounty);
    rows.extend(net);
    Recap {
        kind: RecapKind::Combat,
        result,
        headline,
        headline_tone,
        rows,
    }
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
pub(crate) fn contract_recap(outcomes: &[ContractOutcome], extras: &ContractExtras) -> Recap {
    let paid = outcomes.first().is_none_or(is_paid);
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
    Recap {
        kind: RecapKind::Contract,
        result,
        headline: headline.to_string(),
        headline_tone: tone,
        rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            deltas: [120, -1, -12, 0],
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
            out.push(combat_recap(&facts));
        }
        out.push(combat_recap(&loss()));
        out.push(combat_recap(&CombatFacts {
            result: Some(RecapResult::Drew),
            name: "Captain Vane".into(),
            ..Default::default()
        }));
        out.push(combat_recap(&CombatFacts {
            result: Some(RecapResult::BrokeAway),
            sea: true,
            name: "Captain Vane".into(),
            decisive: Some(DecisiveMove::FledAtSea(2)),
            deltas: [0, 0, -4, 0],
            hull_by_cause: [0, 0, 4],
            ..Default::default()
        }));
        out
    }

    fn all_contract() -> Vec<Recap> {
        vec![
            contract_recap(
                &[outcome("completed_bonus", 672, 612, 23)],
                &ContractExtras::default(),
            ),
            contract_recap(
                &[
                    outcome("completed", 300, 300, 10),
                    outcome("completed", 200, 200, 5),
                ],
                &ContractExtras::default(),
            ),
            contract_recap(
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
        assert!(combat_recap(&facts)
            .headline
            .ends_with(" - boarded at 7/12"));
        facts.result = Some(RecapResult::BrokeAway);
        facts.decisive = Some(DecisiveMove::FledAtSea(11));
        assert!(combat_recap(&facts)
            .headline
            .ends_with(" - fled in round 11"));
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
        facts.injuries = vec!["Cracked rib".into()];
        let recap = combat_recap(&facts);
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
                "Injury - Cracked rib",
                RULE_NOTE,
            ]
        );
    }

    #[test]
    fn took_all_standing_cause() {
        let mut facts = win(RecapResult::WonTookAll);
        facts.standing = Some(("underworld".into(), 2));
        assert!(texts(&combat_recap(&facts))
            .iter()
            .any(|t| t.ends_with(" +2 - took all")));
    }

    #[test]
    fn loss_rows_follow_the_order() {
        let recap = combat_recap(&loss());
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
        let recap = combat_recap(&CombatFacts {
            result: Some(RecapResult::Drew),
            name: "Captain Vane".into(),
            ..Default::default()
        });
        assert!(recap.rows.is_empty());
        let mut facts = win(RecapResult::WonSpared);
        facts.purse = 0;
        facts.deltas = [0, 0, 0, 0];
        facts.reached_boarding = false;
        assert_eq!(texts(&combat_recap(&facts)), vec!["Spared"]);
    }

    #[test]
    fn a_win_shows_no_silver_or_cargo_cost() {
        let mut facts = win(RecapResult::WonPrize);
        facts.deltas = [-10, 0, 0, -3];
        let rows = texts(&combat_recap(&facts)).join("|");
        assert!(!rows.contains("Silver -10") && !rows.contains("Cargo"));
    }

    #[test]
    fn cause_only_from_data() {
        // Mixed hull sources and a sum that disagrees with the diff both
        // render plain.
        let mut facts = loss();
        facts.hull_by_cause = [20, 0, 10];
        facts.crew_by_cause = [1, 0];
        facts.plundered = false;
        let rows = texts(&combat_recap(&facts)).join("|");
        assert!(rows.contains("|Hull -30|"), "{rows}");
        assert!(rows.contains("|Crew -2|"), "{rows}");
        assert!(rows.contains("|Silver -40|"), "{rows}");
        let mut parting = win(RecapResult::WonSpared);
        parting.hull_by_cause = [0, 0, 12];
        assert!(texts(&combat_recap(&parting)).contains(&"Hull -12 - parting shot"));
    }

    #[test]
    fn loot_silver_is_counted_once() {
        let mut facts = win(RecapResult::WonTookAll);
        facts.loot = vec!["+12 silver".into(), "+8 silver".into()];
        let recap = combat_recap(&facts);
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
        let recap = combat_recap(&facts);
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
        let bare = contract_recap(
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
        // hull, crew, 2 injuries = 13 > 9.
        let recap = combat_recap(&facts);
        let body = recap
            .rows
            .iter()
            .filter(|r| r.section.counts_toward_cap())
            .count();
        assert_eq!(body, BODY_ROW_CAP);
        let rows = texts(&recap);
        assert!(!rows.contains(&"A.") && !rows.contains(&"B."));
        assert!(rows.contains(&"3 Rum") && !rows.contains(&"2 Silk") && !rows.contains(&"+1 more"));
        assert!(rows.iter().any(|t| t.starts_with("Underworld standing")));
    }

    #[test]
    fn long_rows_drop_the_cause_then_trim() {
        let mut facts = win(RecapResult::WonPrize);
        facts.prize_ship = "X".repeat(120);
        facts.loot = vec![format!("{} cargo", "word ".repeat(30))];
        let recap = combat_recap(&facts);
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
        recaps.push(combat_recap(&facts));
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
        let passthrough = combat_recap(&passthrough);
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
        assert_eq!(texts(&combat_recap(&facts))[0], "Sunk - no prize");
        facts.no_prize_reason = " . ".into();
        assert_eq!(texts(&combat_recap(&facts))[0], "Sunk - no prize");
    }

    #[test]
    fn personal_and_empty_name_headlines() {
        let mut facts = win(RecapResult::WonSpared);
        facts.sea = false;
        facts.decisive = Some(DecisiveMove::FinalMove("thrust".into()));
        assert_eq!(
            combat_recap(&facts).headline,
            "Won against Captain Vane - final move Thrust"
        );
        facts.name = String::new();
        facts.decisive = None;
        assert_eq!(combat_recap(&facts).headline, "Won");
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
