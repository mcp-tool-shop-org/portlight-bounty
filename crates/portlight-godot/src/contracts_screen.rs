//! Docked contract board. Same ink overlay and side column as the encounter
//! and new-game screens. The simulation is only [`portlight_sim::Session`]:
//! open with `available_contracts`, then accept, complete, and abandon.
//!
//! Character art stays the held placeholder. `man_of_war` stays the galleon
//! plate through [`crate::encounter_screen::set_ship_plate`].

use godot::classes::control::{MouseFilter, SizeFlags};
use godot::classes::text_server::AutowrapMode;
use godot::classes::{HBoxContainer, Label, PanelContainer, ScrollContainer, VBoxContainer};
use godot::prelude::*;
use portlight_sim::model::ContractOutcome;

use crate::encounter_screen;
use crate::logic::humanize_id;

const INK: Color = Color::from_rgb(0.08, 0.11, 0.16);
const CREAM: Color = Color::from_rgb(0.94, 0.91, 0.84);
const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);
/// Due-soon copy. Same warm tone the chart uses for a pending duel.
const DUE: Color = Color::from_rgb(0.93, 0.55, 0.42);

pub(crate) const EMPTY_OFFERS: &str = "No offers at this port. Try another day or another port.";
pub(crate) const CAP_FULL: &str = "Active contracts full (3/3).";
pub(crate) const BOARD_CARD: &str =
    "Sell the goods at the destination - the contract pays out on the sale.";
pub(crate) const MAX_ACTIVE: usize = 3;
/// Contracts list headings. The playtest lens splits rows on these texts.
pub(crate) const SECTION_BOARD: &str = "Board";
pub(crate) const SECTION_ACTIVE: &str = "Active";
pub(crate) const SECTION_RECENT: &str = "Recent";

#[derive(Clone)]
pub(crate) struct ContractsNodes {
    pub root: Gd<PanelContainer>,
    pub title: Gd<Label>,
    pub card: Gd<Label>,
    pub notice: Gd<Label>,
    pub cap: Gd<Label>,
    pub confirm_row: Gd<HBoxContainer>,
    pub scroll: Gd<ScrollContainer>,
    pub list: Gd<VBoxContainer>,
    pub footer: Gd<HBoxContainer>,
    pub plate: Gd<godot::classes::TextureRect>,
    pub plate_panel: Gd<PanelContainer>,
    pub plate_caption: Gd<Label>,
    pub placeholder: Gd<PanelContainer>,
}

pub(crate) fn build_contracts_screen() -> ContractsNodes {
    let mut root = PanelContainer::new_alloc();
    root.set_name("ContractsScreen");
    root.set_mouse_filter(MouseFilter::STOP);
    root.set_visible(false);
    let mut style = godot::classes::StyleBoxFlat::new_gd();
    style.set_bg_color(INK);
    style.set_content_margin_all(28.0);
    root.add_theme_stylebox_override("panel", &style);

    let mut row = HBoxContainer::new_alloc();
    row.set_h_size_flags(SizeFlags::EXPAND_FILL);
    row.set_v_size_flags(SizeFlags::EXPAND_FILL);
    row.add_theme_constant_override("separation", 20);
    root.add_child(&row);

    let side = encounter_screen::side_column();
    let plate = side.plate.clone();
    let plate_panel = side.panel.clone();
    let plate_caption = side.caption.clone();
    let placeholder = side.placeholder.clone();
    row.add_child(&side.column);

    let mut column = VBoxContainer::new_alloc();
    column.set_h_size_flags(SizeFlags::EXPAND_FILL);
    column.set_v_size_flags(SizeFlags::EXPAND_FILL);
    column.add_theme_constant_override("separation", 10);
    row.add_child(&column);

    column.add_child(&text_label("Contracts", 14, MUTED));
    let title = text_label("Contract board", 28, GOLD);
    column.add_child(&title);
    let mut card = text_label(BOARD_CARD, 18, CREAM);
    card.set_autowrap_mode(AutowrapMode::WORD_SMART);
    column.add_child(&card);

    let mut notice = text_label("", 16, CREAM);
    notice.set_name("ContractsNotice");
    notice.set_autowrap_mode(AutowrapMode::WORD_SMART);
    column.add_child(&notice);

    let mut cap = text_label("", 15, DUE);
    cap.set_name("ContractsCap");
    cap.set_visible(false);
    column.add_child(&cap);

    let mut confirm_row = HBoxContainer::new_alloc();
    confirm_row.set_name("AbandonConfirm");
    confirm_row.add_theme_constant_override("separation", 8);
    confirm_row.set_visible(false);
    column.add_child(&confirm_row);

    let mut scroll = ScrollContainer::new_alloc();
    scroll.set_name("ContractsScroll");
    scroll.set_h_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_v_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_horizontal_scroll_mode(godot::classes::scroll_container::ScrollMode::DISABLED);
    let mut list = VBoxContainer::new_alloc();
    list.set_name("ContractList");
    list.set_h_size_flags(SizeFlags::EXPAND_FILL);
    list.add_theme_constant_override("separation", 4);
    scroll.add_child(&list);
    column.add_child(&scroll);

    let mut footer = HBoxContainer::new_alloc();
    footer.set_name("ContractsFooter");
    footer.add_theme_constant_override("separation", 8);
    column.add_child(&footer);

    ContractsNodes {
        root,
        title,
        card,
        notice,
        cap,
        confirm_row,
        scroll,
        list,
        footer,
        plate,
        plate_panel,
        plate_caption,
        placeholder,
    }
}

pub(crate) fn section_label(text: &str) -> Gd<Label> {
    text_label(text, 16, GOLD)
}

pub(crate) fn body_line(text: &str, size: i32, color: Color) -> Gd<Label> {
    let mut label = text_label(text, size, color);
    label.set_autowrap_mode(AutowrapMode::WORD_SMART);
    label.set_h_size_flags(SizeFlags::EXPAND_FILL);
    label
}

pub(crate) fn meta_color(due_soon: bool) -> Color {
    if due_soon {
        DUE
    } else {
        MUTED
    }
}

/// Engine copy for the screen. An em dash in a shortage reason becomes a hyphen.
pub(crate) fn ascii_sentence(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\u{2014}' | '\u{2013}' | '\u{2212}' => out.push('-'),
            '\u{2018}' | '\u{2019}' => out.push('\''),
            '\u{201c}' | '\u{201d}' => out.push('"'),
            c if c.is_ascii() => out.push(c),
            _ => {}
        }
    }
    out
}

pub(crate) fn reward_text(silver: i64, bonus: i64) -> String {
    if bonus > 0 {
        format!("{silver} silver + {bonus} bonus")
    } else {
        format!("{silver} silver")
    }
}

/// Desk timing words come from [`day_report::deadline_timing`], the same as
/// the strip, the Day's report and the Departure check (Q2: `due soon` is
/// retired). Two days left or fewer still tints DUE.
pub(crate) fn days_left_text(day: i64, deadline: i64) -> (String, bool) {
    let left = deadline - day;
    (crate::day_report::deadline_timing(left), left <= 2)
}

/// Board meta requirement, e.g. `Trust Credible - Standing 10`. Same casing as
/// the Harbour offer line (`Trust Credible. Standing 3.`): the tier is a
/// title-cased name, never the raw id.
pub(crate) fn requirement_text(tier: &str, standing: i64) -> String {
    let tier = humanize_id(&ascii_sentence(tier));
    if standing > 0 {
        format!("Trust {tier} - Standing {standing}")
    } else {
        format!("Trust {tier}")
    }
}

/// Complete is only for an accepted obligation that has already been delivered.
pub(crate) fn can_complete(status: &str, delivered: i64, required: i64) -> bool {
    status == "accepted" && delivered >= required
}

pub(crate) fn progress_text(delivered: i64, required: i64) -> String {
    format!("{delivered}/{required}")
}

pub(crate) fn abandon_prompt(title: &str) -> String {
    format!("Abandon {}? Confirm to drop it.", ascii_sentence(title))
}

/// Char budget for one Contracts desk line (notice 16 px, Recent 14 px). The
/// desk is about 1000 px wide and its labels wrap, so this only caps a
/// runaway tail; no catalog outcome reaches it.
pub(crate) const DESK_CHARS: usize = 96;

/// Contracts desk notice after an abandon (or a desk settle), in the Market
/// paid-notice form: `Contract abandoned: Grain for Corsair's Rest`.
pub(crate) fn outcome_notice(outcome: &ContractOutcome) -> String {
    outcome_line(notice_label(outcome), outcome, 0, DESK_CHARS)
}

/// `Silver +615`, or empty when no silver moved. The one Silver term for every
/// outcome surface. Silver is the only outcome term the sim applies: trust,
/// standing and heat stay on the record (they size a contract-guarantee
/// claim) and never reach the captain, as in Python, so no line prints them.
pub(crate) fn outcome_terms(outcome: &ContractOutcome) -> String {
    if outcome.silver_delta == 0 {
        String::new()
    } else {
        format!("Silver {:+}", outcome.silver_delta)
    }
}

/// Failure terms: the Silver term from [`outcome_terms`] (partial pay on an
/// expiry), then `Guarantee +{p}` when a contract guarantee paid out. Zero
/// terms are omitted (R4). Same Silver-only rule as every outcome surface.
pub(crate) fn failure_terms(outcome: &ContractOutcome, guarantee: i64) -> Vec<String> {
    let mut terms = Vec::new();
    let silver = outcome_terms(outcome);
    if !silver.is_empty() {
        terms.push(silver);
    }
    if guarantee > 0 {
        terms.push(format!("Guarantee +{guarantee}"));
    }
    terms
}

/// ` - 22 Grain still aboard` (catalog good name), or empty when none of the
/// contract's good is held. The one next-step fact on a failure line.
pub(crate) fn aboard_clause(good_id: &str, held: i64) -> String {
    if held > 0 {
        ascii_sentence(&format!(
            " - {held} {} still aboard",
            good_display_name(good_id)
        ))
    } else {
        String::new()
    }
}

/// Desk notice after Abandon: the shared [`outcome_notice`] form plus the
/// aboard clause. The head is never trimmed; the field tail gives way first
/// so the whole line stays within [`DESK_CHARS`].
pub(crate) fn abandon_notice(outcome: &ContractOutcome, held: i64) -> String {
    let aboard = aboard_clause(&outcome.good_id, held);
    let line = outcome_line(
        notice_label(outcome),
        outcome,
        0,
        DESK_CHARS.saturating_sub(aboard.len()),
    );
    format!("{line}{aboard}")
}

/// A desk notice for a failure outcome (abandon, or an expiry if one is ever
/// shown here). These render in DUE; every other notice stays CREAM.
pub(crate) fn is_failure_notice(text: &str) -> bool {
    text.starts_with("Contract abandoned: ") || text.starts_with("Contract expired: ")
}

/// Desk notice tone: DUE for a failure outcome, CREAM otherwise.
pub(crate) fn notice_color(text: &str) -> Color {
    if is_failure_notice(text) {
        DUE
    } else {
        CREAM
    }
}

/// Notice label by outcome kind: `Contract paid` / `Contract abandoned` /
/// `Contract expired`.
pub(crate) fn notice_label(outcome: &ContractOutcome) -> &'static str {
    match outcome.outcome_type.as_str() {
        "completed" | "completed_bonus" => "Contract paid",
        "abandoned" => "Contract abandoned",
        "expired" => "Contract expired",
        _ => "Contract settled",
    }
}

/// Recent row label by outcome kind: `Paid` / `Abandoned` / `Expired`.
fn recent_label(outcome: &ContractOutcome) -> &'static str {
    match outcome.outcome_type.as_str() {
        "completed" | "completed_bonus" => "Paid",
        "abandoned" => "Abandoned",
        "expired" => "Expired",
        _ => "Settled",
    }
}

/// The tail, built from contract fields (never the sim summary) with catalog
/// names: `23 Grain to Corsair's Rest - early bonus +60` for a payment,
/// `Grain for Corsair's Rest` otherwise.
pub(crate) fn outcome_tail(outcome: &ContractOutcome) -> String {
    let good = good_display_name(&outcome.good_id);
    let port = port_display_name(&outcome.destination_port_id);
    let tail = match outcome.outcome_type.as_str() {
        "completed" | "completed_bonus" => {
            let mut tail = format!("{} {good} to {port}", outcome.delivered_quantity);
            let bonus = outcome.silver_delta - outcome.reward_silver;
            if outcome.outcome_type == "completed_bonus" && bonus > 0 {
                tail.push_str(&format!(" - early bonus +{bonus}"));
            }
            tail
        }
        _ => format!("{good} for {port}"),
    };
    ascii_sentence(&tail)
}

/// `{label}: Silver +612 (+1 more) - {tail}`. Zero silver and `+0 more` are
/// omitted. The head (label, silver, more) is never trimmed; a tail that
/// would push the line past `budget` chars is cut at a word and ends `...`.
pub(crate) fn outcome_line(
    label: &str,
    outcome: &ContractOutcome,
    more: usize,
    budget: usize,
) -> String {
    outcome_line_with_tail(label, outcome, more, budget, &outcome_tail(outcome))
}

/// [`outcome_line`] with the tail given by the caller (the Market notice uses
/// the bonus alone, so the early bonus is never trimmed away).
pub(crate) fn outcome_line_with_tail(
    label: &str,
    outcome: &ContractOutcome,
    more: usize,
    budget: usize,
    full_tail: &str,
) -> String {
    let mut head = format!("{label}:");
    let silver = outcome_terms(outcome);
    if !silver.is_empty() {
        head.push(' ');
        head.push_str(&silver);
    }
    if more > 0 {
        head.push_str(&format!(" (+{more} more)"));
    }
    let sep = if head.ends_with(':') { " " } else { " - " };
    let room = budget.saturating_sub(head.len() + sep.len());
    let tail = trim_tail(full_tail, room);
    if tail.is_empty() {
        head
    } else {
        format!("{head}{sep}{tail}")
    }
}

/// `early bonus +60` when the outcome paid an early bonus, else `None`.
pub(crate) fn early_bonus_tail(outcome: &ContractOutcome) -> Option<String> {
    let bonus = outcome.silver_delta - outcome.reward_silver;
    (outcome.outcome_type == "completed_bonus" && bonus > 0)
        .then(|| format!("early bonus +{bonus}"))
}

/// `Contract progress - Grain for Corsair's Rest - 10/23`. The tail trims
/// first; the head and the `{d}/{r}` token never do. No Silver, Trust,
/// Standing or Heat: nothing was paid.
pub(crate) fn progress_line(
    good_id: &str,
    port_id: &str,
    delivered: i64,
    required: i64,
    budget: usize,
) -> String {
    let head = "Contract progress";
    let token = format!(" - {}", progress_text(delivered, required));
    let tail = ascii_sentence(&format!(
        "{} for {}",
        good_display_name(good_id),
        port_display_name(port_id)
    ));
    let room = budget.saturating_sub(head.len() + 3 + token.len());
    let tail = trim_tail(&tail, room);
    if tail.is_empty() {
        format!("{head}{token}")
    } else {
        format!("{head} - {tail}{token}")
    }
}

/// One progress line per contract still active whose delivered count rose
/// between `before` and `after` (matched by offer id). Empty when nothing was
/// credited, so a sale that moves no contract prints nothing extra.
pub(crate) fn progress_lines(
    before: &[portlight_sim::model::ActiveContract],
    after: &[portlight_sim::model::ActiveContract],
    budget: usize,
) -> Vec<String> {
    after
        .iter()
        .filter(|now| {
            before
                .iter()
                .find(|old| old.offer_id == now.offer_id)
                .is_some_and(|old| now.delivered_quantity > old.delivered_quantity)
        })
        .map(|now| {
            progress_line(
                &now.good_id,
                &now.destination_port_id,
                now.delivered_quantity,
                now.required_quantity,
                budget,
            )
        })
        .collect()
}

/// ASCII `tail` cut to `room` chars: whole words, then `...`. Empty when
/// not even one char and the dots fit.
fn trim_tail(tail: &str, room: usize) -> String {
    if tail.len() <= room {
        return tail.to_string();
    }
    if room < 4 {
        return String::new();
    }
    let mut cut = &tail[..room - 3];
    if let Some(space) = cut.rfind(' ').filter(|space| *space > 0) {
        cut = &cut[..space];
    }
    format!("{}...", cut.trim_end_matches([' ', '-']))
}

/// T-S. One muted fact about the docked port's market for a board card.
/// `None` when not docked. No remote prices, no advice.
pub(crate) fn availability_tag(
    market: Option<&[portlight_sim::model::MarketSlot]>,
    good_id: &str,
) -> Option<String> {
    let slot = market?.iter().find(|slot| slot.good_id == good_id);
    Some(match slot {
        Some(slot) if slot.stock_current > 0 => format!("Sold here - buy {}", slot.buy_price),
        Some(_) => "Out of stock here".to_string(),
        None => "Not sold here".to_string(),
    })
}

pub(crate) fn port_display_name(id: &str) -> String {
    portlight_sim::content::content()
        .port(id)
        .map(|port| port.name.clone())
        .unwrap_or_else(|| humanize_id(id))
}

pub(crate) fn good_display_name(id: &str) -> String {
    portlight_sim::content::content()
        .good(id)
        .map(|good| good.name.clone())
        .unwrap_or_else(|| humanize_id(id))
}

/// Desk Recent row, same builder as the notices: `Paid: Silver +612 - 23
/// Grain to Corsair's Rest - early bonus +60`, `Abandoned: Grain for
/// Corsair's Rest`. No raw outcome type, no sim summary. An expiry adds its
/// failure terms and progress:
/// `Expired: Silver +60 - Guarantee +105 - Grain for Corsair's Rest - 5/23`.
/// `guarantee` is the contract-guarantee silver paid for this contract; only
/// expired rows read it.
pub(crate) fn recent_line(outcome: &ContractOutcome, guarantee: i64) -> String {
    if outcome.outcome_type == "expired" {
        expired_recent_line(outcome, guarantee)
    } else {
        outcome_line(recent_label(outcome), outcome, 0, DESK_CHARS)
    }
}

/// `Expired: Silver +N - Guarantee +N - {Good} for {Port} - {d}/{r}`, zero
/// terms omitted (silver-first, like every trimmed one-liner). The head and
/// the `{d}/{r}` token are never trimmed; only the field tail gives way.
fn expired_recent_line(outcome: &ContractOutcome, guarantee: i64) -> String {
    let mut head = format!("{}:", recent_label(outcome));
    let terms = failure_terms(outcome, guarantee);
    if !terms.is_empty() {
        head.push(' ');
        head.push_str(&terms.join(" - "));
    }
    let progress = format!(
        " - {}",
        progress_text(outcome.delivered_quantity, outcome.required_quantity)
    );
    let sep = if head.ends_with(':') { " " } else { " - " };
    let room = DESK_CHARS.saturating_sub(head.len() + sep.len() + progress.len());
    let tail = trim_tail(&outcome_tail(outcome), room);
    if tail.is_empty() {
        format!("{head}{progress}")
    } else {
        format!("{head}{sep}{tail}{progress}")
    }
}

fn text_label(text: &str, size: i32, color: Color) -> Gd<Label> {
    let mut label = Label::new_alloc();
    label.set_text(text);
    label.add_theme_font_size_override("font_size", size);
    label.add_theme_color_override("font_color", color);
    label
}

#[cfg(test)]
mod tests {
    use portlight_sim::model::{ActiveContract, ContractOutcome};
    use portlight_sim::Session;

    use super::{
        abandon_notice, abandon_prompt, aboard_clause, ascii_sentence, availability_tag,
        can_complete, days_left_text, failure_terms, is_failure_notice, notice_color, outcome_line,
        outcome_notice, outcome_terms, progress_line, progress_lines, progress_text, recent_line,
        requirement_text, reward_text, BOARD_CARD, CAP_FULL, CREAM, DESK_CHARS, DUE, EMPTY_OFFERS,
        MAX_ACTIVE,
    };

    #[test]
    fn copy_is_ascii_and_names_the_empty_board_and_the_cap() {
        for line in [EMPTY_OFFERS, CAP_FULL, BOARD_CARD] {
            assert!(line.is_ascii(), "{line}");
        }
        assert_eq!(MAX_ACTIVE, 3);
        // R10: the sale pays a fulfilled contract. No Complete step in the copy.
        assert_eq!(
            BOARD_CARD,
            "Sell the goods at the destination - the contract pays out on the sale."
        );
        assert!(!BOARD_CARD.contains("Complete"));
        assert_eq!(
            ascii_sentence("Shortage at Corsair's Rest \u{2014} urgent demand for grain"),
            "Shortage at Corsair's Rest - urgent demand for grain"
        );
    }

    #[test]
    fn days_reward_and_complete_follow_the_obligation() {
        assert_eq!(days_left_text(1, 3), ("2 days left".to_string(), true));
        assert_eq!(days_left_text(1, 2), ("1 day left".to_string(), true));
        assert_eq!(days_left_text(1, 1), ("due today".to_string(), true));
        assert_eq!(days_left_text(2, 1), ("overdue".to_string(), true));
        assert_eq!(days_left_text(1, 4), ("3 days left".to_string(), false));
        assert_eq!(days_left_text(1, 12), ("11 days left".to_string(), false));
        for (day, deadline) in [(1, 3), (1, 2), (1, 1), (2, 1), (1, 12)] {
            assert!(!days_left_text(day, deadline).0.contains("due soon"));
        }
        assert_eq!(reward_text(480, 0), "480 silver");
        assert_eq!(reward_text(480, 60), "480 silver + 60 bonus");
        assert_eq!(requirement_text("unproven", 0), "Trust Unproven");
        assert_eq!(
            requirement_text("credible", 10),
            "Trust Credible - Standing 10"
        );
        assert_eq!(progress_text(0, 23), "0/23");
        assert!(!can_complete("accepted", 0, 23));
        assert!(!can_complete("accepted", 22, 23));
        assert!(can_complete("accepted", 23, 23));
        assert!(!can_complete("completed", 23, 23));
    }

    #[test]
    fn abandon_prompt_and_outcome_print_only_applied_terms() {
        assert_eq!(
            abandon_prompt("Grain for Corsair's Rest"),
            "Abandon Grain for Corsair's Rest? Confirm to drop it."
        );
        let outcome = ContractOutcome {
            contract_id: "abc".into(),
            outcome_type: "abandoned".into(),
            silver_delta: 0,
            trust_delta: -2,
            standing_delta: -1,
            heat_delta: 1,
            completion_day: 1,
            summary: "Abandoned contract: Grain for Corsair's Rest".into(),
            family: "shortage".into(),
            good_id: "grain".into(),
            required_quantity: 23,
            delivered_quantity: 0,
            destination_port_id: "corsairs_rest".into(),
            deadline_day: 12,
            reward_silver: 480,
        };
        let notice = outcome_notice(&outcome);
        assert!(notice.is_ascii(), "{notice}");
        // Abandon applies nothing: no Silver term, the tail from fields.
        assert_eq!(notice, "Contract abandoned: Grain for Corsair's Rest");
        for term in ["Silver", "Trust", "Standing", "Heat", "+0", "  "] {
            assert!(!notice.contains(term), "{notice}");
        }
    }

    /// Accept, fail an early complete, abandon with no captain, then clear the
    /// rest of the board without asking for a new day's offers.
    #[test]
    fn first_playable_board_clears_without_a_later_refresh() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        let day = session.world().day;
        let silver = session.world().captain.silver;
        let trust = session.world().captain.standing.commercial_trust;
        let wanted = session.world().captain.wanted_level;
        let heat = session.world().captain.standing.heat;
        let regional = session.world().captain.standing.regional;
        let offers = session.available_contracts();
        assert_eq!(offers.len(), 5);
        let first = offers[0].id.clone();
        let active = session.accept_contract(&first).unwrap();
        assert!(active.delivered_quantity < active.required_quantity);
        let err = session.complete_contract(&first).unwrap_err();
        assert_eq!(err.to_string(), "Contract is not yet fulfilled");
        assert_eq!(session.world().captain.silver, silver);

        let mut saw_cap = false;
        loop {
            let offer_ids: Vec<String> = session
                .board()
                .offers
                .iter()
                .map(|offer| offer.id.clone())
                .collect();
            if offer_ids.is_empty() {
                break;
            }
            if session.board().active.len() >= MAX_ACTIVE {
                let err = session.accept_contract(&offer_ids[0]).unwrap_err();
                assert_eq!(err.to_string(), "Too many active contracts (max 3)");
                saw_cap = true;
                let active_ids: Vec<String> = session
                    .board()
                    .active
                    .iter()
                    .map(|contract| contract.offer_id.clone())
                    .collect();
                for id in active_ids {
                    session.abandon_contract(&id).unwrap();
                }
            }
            session.accept_contract(&offer_ids[0]).unwrap();
        }
        assert!(
            saw_cap,
            "seed 1 should fill the 3-cap before the board is empty"
        );
        let leftover: Vec<String> = session
            .board()
            .active
            .iter()
            .map(|contract| contract.offer_id.clone())
            .collect();
        for id in leftover {
            let outcome = session.abandon_contract(&id).unwrap();
            assert_eq!(outcome.outcome_type, "abandoned");
            assert_eq!(outcome.trust_delta, -2);
        }
        assert!(session.board().offers.is_empty());
        assert!(session.board().active.is_empty());
        assert!(session.board().breaches.is_empty());
        assert!(!session.board().completed.is_empty());
        assert_eq!(session.world().day, day);
        assert_eq!(session.world().captain.silver, silver);
        assert_eq!(session.world().captain.wanted_level, wanted);
        assert_eq!(session.world().captain.standing.commercial_trust, trust);
        assert_eq!(session.world().captain.standing.heat, heat);
        assert_eq!(session.world().captain.standing.regional, regional);
        // Same day: a second read must not invent offers.
        assert!(session.available_contracts().is_empty());
    }

    fn delivered(summary: &str) -> ContractOutcome {
        ContractOutcome {
            contract_id: "abc".into(),
            outcome_type: "completed".into(),
            silver_delta: 615,
            trust_delta: 1,
            standing_delta: 2,
            heat_delta: -1,
            completion_day: 9,
            summary: summary.into(),
            family: "shortage".into(),
            good_id: "grain".into(),
            required_quantity: 23,
            delivered_quantity: 23,
            destination_port_id: "corsairs_rest".into(),
            deadline_day: 19,
            reward_silver: 552,
        }
    }

    fn bonus_paid() -> ContractOutcome {
        let mut paid = delivered("Delivered 23 grain to corsairs_rest (early bonus: +60 silver)");
        paid.outcome_type = "completed_bonus".into();
        paid.silver_delta = 612;
        paid
    }

    fn failed(kind: &str, silver: i64) -> ContractOutcome {
        let mut outcome = delivered("Abandoned contract: Famine relief: grain to Corsair's Rest");
        outcome.outcome_type = kind.into();
        outcome.silver_delta = silver;
        outcome.trust_delta = -2;
        outcome.standing_delta = -1;
        outcome.heat_delta = 1;
        outcome.delivered_quantity = 0;
        outcome
    }

    /// Player-surface hygiene for every outcome line.
    fn assert_clean(line: &str) {
        assert!(line.is_ascii(), "{line}");
        for bad in [
            "Trust",
            "Standing",
            "Heat",
            "+0",
            "  ",
            "Delivered",
            "deliver",
            "completed",
            "corsairs_rest",
            "_",
        ] {
            assert!(!line.contains(bad), "{bad:?} in {line}");
        }
    }

    /// The one shared helper prints Silver only. Trust, standing
    /// and heat are never applied by the sim (nor by Python), so no outcome
    /// surface shows them. Zero silver is omitted (R4).
    #[test]
    fn outcome_terms_is_silver_only_and_drops_zero() {
        let mut outcome = delivered("Delivered 23 grain to corsairs_rest");
        assert_eq!(outcome_terms(&outcome), "Silver +615");
        outcome.silver_delta = 60;
        outcome.trust_delta = -2;
        outcome.standing_delta = -1;
        outcome.heat_delta = 1;
        assert_eq!(outcome_terms(&outcome), "Silver +60");
        outcome.silver_delta = -40;
        assert_eq!(outcome_terms(&outcome), "Silver -40");
        outcome.silver_delta = 0;
        assert_eq!(outcome_terms(&outcome), "");
    }

    /// Silver first, `(+N more)`, then the tail from fields.
    #[test]
    fn outcome_line_leads_with_silver_and_more_then_the_field_tail() {
        let paid = bonus_paid();
        assert_eq!(
            outcome_line("Contract paid", &paid, 1, DESK_CHARS),
            "Contract paid: Silver +612 (+1 more) - 23 Grain to Corsair's Rest - early bonus +60"
        );
        // N = 0 drops the count; no bonus drops the bonus term.
        assert_eq!(
            outcome_notice(&paid),
            "Contract paid: Silver +612 - 23 Grain to Corsair's Rest - early bonus +60"
        );
        assert_eq!(
            outcome_notice(&delivered("Delivered 23 grain to corsairs_rest")),
            "Contract paid: Silver +615 - 23 Grain to Corsair's Rest"
        );
        let abandoned = failed("abandoned", -40);
        assert_eq!(
            outcome_line("Contract abandoned", &abandoned, 1, DESK_CHARS),
            "Contract abandoned: Silver -40 (+1 more) - Grain for Corsair's Rest"
        );
        assert_eq!(
            outcome_notice(&abandoned),
            "Contract abandoned: Silver -40 - Grain for Corsair's Rest"
        );
        // Silver 0: the term is gone, the count still leads the tail.
        let quiet = failed("abandoned", 0);
        assert_eq!(
            outcome_notice(&quiet),
            "Contract abandoned: Grain for Corsair's Rest"
        );
        assert_eq!(
            outcome_line("Contract abandoned", &quiet, 1, DESK_CHARS),
            "Contract abandoned: (+1 more) - Grain for Corsair's Rest"
        );
        assert_eq!(
            outcome_notice(&failed("expired", 0)),
            "Contract expired: Grain for Corsair's Rest"
        );
        // Ids no catalog lists go through humanize_id.
        let mut unknown = delivered("Delivered 4 whale_oil to drowned_quay.");
        unknown.good_id = "whale_oil".into();
        unknown.destination_port_id = "drowned_quay".into();
        unknown.delivered_quantity = 4;
        assert_eq!(
            outcome_notice(&unknown),
            "Contract paid: Silver +615 - 4 Whale Oil to Drowned Quay"
        );
        for line in [
            outcome_notice(&paid),
            outcome_notice(&abandoned),
            outcome_notice(&quiet),
            outcome_notice(&unknown),
        ] {
            assert_clean(&line);
        }
    }

    /// Overflow trims the tail at a word with ASCII `...`; the head
    /// `Contract paid: Silver +N (+N more)` is never cut.
    #[test]
    fn outcome_line_trims_only_the_tail() {
        let paid = bonus_paid();
        assert_eq!(
            outcome_line("Contract paid", &paid, 0, 56),
            "Contract paid: Silver +612 - 23 Grain to Corsair's..."
        );
        assert_eq!(
            outcome_line("Contract paid", &paid, 1, 56),
            "Contract paid: Silver +612 (+1 more) - 23 Grain to..."
        );
        // Exactly fits: no dots.
        let plain = delivered("Delivered 23 grain to corsairs_rest");
        let full = "Contract paid: Silver +615 - 23 Grain to Corsair's Rest";
        assert_eq!(outcome_line("Contract paid", &plain, 0, full.len()), full);
        assert_eq!(
            outcome_line("Contract paid", &plain, 0, full.len() - 1),
            "Contract paid: Silver +615 - 23 Grain to Corsair's..."
        );
        // No room for the tail: the head stands alone, whole.
        assert_eq!(
            outcome_line("Contract paid", &paid, 9, 10),
            "Contract paid: Silver +612 (+9 more)"
        );
        for budget in 30..=96 {
            for more in [0, 1, 9] {
                let line = outcome_line("Contract paid", &paid, more, budget);
                let head = if more == 0 {
                    "Contract paid: Silver +612".to_string()
                } else {
                    format!("Contract paid: Silver +612 (+{more} more)")
                };
                assert!(line.starts_with(&head), "{line}");
                assert!(line.len() <= budget.max(head.len()), "{budget}: {line}");
                assert!(
                    !line.ends_with(" -...") && !line.ends_with(" ..."),
                    "{line}"
                );
                assert_clean(&line);
            }
        }
    }

    /// Recent rows use the same builder, no raw type or sim summary.
    #[test]
    fn recent_line_is_built_from_fields() {
        assert_eq!(
            recent_line(&bonus_paid(), 0),
            "Paid: Silver +612 - 23 Grain to Corsair's Rest - early bonus +60"
        );
        assert_eq!(
            recent_line(&delivered("Delivered 23 grain to corsairs_rest"), 0),
            "Paid: Silver +615 - 23 Grain to Corsair's Rest"
        );
        assert_eq!(
            recent_line(&failed("abandoned", 0), 0),
            "Abandoned: Grain for Corsair's Rest"
        );
        assert_eq!(
            recent_line(&failed("abandoned", -40), 0),
            "Abandoned: Silver -40 - Grain for Corsair's Rest"
        );
        // Abandoned rows never read the guarantee.
        assert_eq!(
            recent_line(&failed("abandoned", 0), 105),
            "Abandoned: Grain for Corsair's Rest"
        );
        for kind in ["completed", "completed_bonus", "abandoned", "expired"] {
            let line = recent_line(&failed(kind, 0), 0);
            assert_clean(&line);
            assert!(!line.contains("Famine relief"), "{line}");
            assert!(line.len() <= DESK_CHARS, "{line}");
        }
    }

    /// Expired Recent rows carry Silver, Guarantee and
    /// `{d}/{r}`, zero terms omitted, catalog names, silver-first.
    #[test]
    fn expired_recent_row_has_terms_and_progress() {
        assert_eq!(
            recent_line(&failed("expired", 0), 0),
            "Expired: Grain for Corsair's Rest - 0/23"
        );
        let mut part = failed("expired", 60);
        part.delivered_quantity = 5;
        assert_eq!(
            recent_line(&part, 0),
            "Expired: Silver +60 - Grain for Corsair's Rest - 5/23"
        );
        assert_eq!(
            recent_line(&failed("expired", 0), 105),
            "Expired: Guarantee +105 - Grain for Corsair's Rest - 0/23"
        );
        assert_eq!(
            recent_line(&part, 105),
            "Expired: Silver +60 - Guarantee +105 - Grain for Corsair's Rest - 5/23"
        );
        // Non-catalog ids humanize; never raw.
        let mut odd = failed("expired", 0);
        odd.good_id = "whale_oil".into();
        odd.destination_port_id = "drowned_quay".into();
        assert_eq!(
            recent_line(&odd, 0),
            "Expired: Whale Oil for Drowned Quay - 0/23"
        );
        // A runaway port id trims the field tail; head and progress stay.
        let mut long = part.clone();
        long.destination_port_id = "a_very_long_port_name".repeat(6);
        let line = recent_line(&long, 105);
        assert!(
            line.starts_with("Expired: Silver +60 - Guarantee +105 - "),
            "{line}"
        );
        assert!(line.ends_with("... - 5/23"), "{line}");
        assert!(line.len() <= DESK_CHARS, "{line}");
        for line in [
            recent_line(&failed("expired", 0), 0),
            recent_line(&part, 105),
            recent_line(&odd, 0),
            line,
        ] {
            assert_clean(&line);
            for bad in [
                "\u{2014}",
                "\u{2013}",
                "->",
                "sold",
                "delivered",
                "Famine relief",
            ] {
                assert!(!line.contains(bad), "{bad:?} in {line}");
            }
        }
    }

    /// Contract fail: Silver (partial pay) then Guarantee, zero terms dropped.
    #[test]
    fn failure_terms_put_guarantee_after_silver_and_drop_zero() {
        let mut outcome = failed("expired", 60);
        assert_eq!(failure_terms(&outcome, 0), vec!["Silver +60".to_string()]);
        assert_eq!(
            failure_terms(&outcome, 65),
            vec!["Silver +60".to_string(), "Guarantee +65".to_string()]
        );
        outcome.silver_delta = 0;
        assert_eq!(
            failure_terms(&outcome, 105),
            vec!["Guarantee +105".to_string()]
        );
        assert!(failure_terms(&outcome, 0).is_empty());
        // A denied or empty claim never prints a term.
        assert!(failure_terms(&outcome, -5).is_empty());
    }

    /// Contract fail: the aboard clause, and the abandon notice that carries it.
    #[test]
    fn abandon_notice_adds_the_aboard_clause_and_keeps_the_head() {
        assert_eq!(aboard_clause("grain", 22), " - 22 Grain still aboard");
        assert_eq!(aboard_clause("grain", 0), "");
        assert_eq!(aboard_clause("whale_oil", 3), " - 3 Whale Oil still aboard");
        let quiet = failed("abandoned", 0);
        assert_eq!(
            abandon_notice(&quiet, 23),
            "Contract abandoned: Grain for Corsair's Rest - 23 Grain still aboard"
        );
        // Nothing held: exactly the shared notice.
        assert_eq!(abandon_notice(&quiet, 0), outcome_notice(&quiet));
        // A runaway tail trims before the aboard clause; the head stays whole.
        let mut long = failed("abandoned", 0);
        long.destination_port_id =
            "the_very_long_and_winding_harbour_of_the_far_southern_reaches_beyond".into();
        let notice = abandon_notice(&long, 23);
        assert!(
            notice.starts_with("Contract abandoned: Grain for "),
            "{notice}"
        );
        assert!(notice.ends_with("... - 23 Grain still aboard"), "{notice}");
        assert!(notice.len() <= DESK_CHARS, "{notice}");
        for line in [abandon_notice(&quiet, 23), notice] {
            assert_clean(&line);
            assert!(
                !line.contains("Complete") && !line.contains("due soon"),
                "{line}"
            );
        }
    }

    /// Contract fail: the desk notice is DUE after an abandon, CREAM otherwise.
    #[test]
    fn failure_notice_tone_is_due_only_for_failures() {
        let abandoned = abandon_notice(&failed("abandoned", 0), 23);
        assert!(is_failure_notice(&abandoned));
        assert_eq!(notice_color(&abandoned), DUE);
        assert!(is_failure_notice(&outcome_notice(&failed("expired", 0))));
        for other in [
            outcome_notice(&bonus_paid()),
            "Accepted Famine relief: grain to Corsair's Rest.".to_string(),
            abandon_prompt("Grain for Corsair's Rest"),
            "Contract is not yet fulfilled".to_string(),
            String::new(),
        ] {
            assert!(!is_failure_notice(&other), "{other}");
            assert_eq!(notice_color(&other), CREAM, "{other}");
        }
    }

    #[test]
    fn availability_tag_reads_the_docked_market_only() {
        let session = Session::new("Ada", "merchant", 1, None).expect("seed-1 session");
        let mut market = session
            .world()
            .port("porto_novo")
            .expect("Porto Novo")
            .market
            .clone();
        let grain = market
            .iter()
            .find(|slot| slot.good_id == "grain")
            .expect("grain slot")
            .buy_price;
        assert_eq!(
            availability_tag(Some(&market), "grain").as_deref(),
            Some(format!("Sold here - buy {grain}").as_str())
        );
        assert_eq!(
            availability_tag(Some(&market), "weapons").as_deref(),
            Some("Not sold here")
        );
        for slot in market.iter_mut().filter(|slot| slot.good_id == "grain") {
            slot.stock_current = 0;
        }
        assert_eq!(
            availability_tag(Some(&market), "grain").as_deref(),
            Some("Out of stock here")
        );
        // At sea there is no docked market, so no tag.
        assert_eq!(availability_tag(None, "grain"), None);
        for tag in ["Sold here - buy 8", "Out of stock here", "Not sold here"] {
            assert!(tag.is_ascii(), "{tag}");
        }
    }

    fn active(offer: &str, good: &str, port: &str, done: i64, need: i64) -> ActiveContract {
        ActiveContract {
            offer_id: offer.into(),
            template_id: "t".into(),
            family: "shortage".into(),
            title: "Grain run".into(),
            accepted_day: 1,
            deadline_day: 20,
            destination_port_id: port.into(),
            good_id: good.into(),
            required_quantity: need,
            delivered_quantity: done,
            reward_silver: 500,
            bonus_reward: 60,
            source_region: None,
            source_port: None,
            inspection_modifier: 1.0,
            status: "active".into(),
        }
    }

    #[test]
    fn progress_line_formats_head_tail_and_token() {
        assert_eq!(
            progress_line("grain", "corsairs_rest", 10, 23, 96),
            "Contract progress - Grain for Corsair's Rest - 10/23"
        );
        let before = [active("a", "grain", "corsairs_rest", 0, 23)];
        let after = [active("a", "grain", "corsairs_rest", 10, 23)];
        assert_eq!(
            progress_lines(&before, &after, 96),
            vec!["Contract progress - Grain for Corsair's Rest - 10/23"]
        );
        // No credit, no line (R4); a contract that left the board is paid, not progress.
        assert!(progress_lines(&after, &after, 96).is_empty());
        assert!(progress_lines(&before, &[], 96).is_empty());
    }

    #[test]
    fn progress_line_trims_the_tail_not_the_token() {
        let long = progress_line("black_powder", "typhoon_anchorage", 999, 999, 40);
        assert!(long.starts_with("Contract progress - "), "{long}");
        assert!(long.ends_with(" - 999/999"), "{long}");
        assert!(long.contains("..."), "{long}");
        // No room for any tail: head and token stand whole.
        assert_eq!(
            progress_line("grain", "corsairs_rest", 10, 23, 10),
            "Contract progress - 10/23"
        );
        for budget in 20..=96 {
            let line = progress_line("black_powder", "typhoon_anchorage", 12, 345, budget);
            assert!(line.starts_with("Contract progress"), "{line}");
            assert!(line.ends_with("12/345"), "{line}");
            assert_clean(&line);
        }
    }

    #[test]
    fn progress_line_has_no_trust_standing_heat_or_complete() {
        let line = progress_line("grain", "corsairs_rest", 10, 23, 96);
        for term in [
            "Silver",
            "Trust",
            "Standing",
            "Heat",
            "Complete",
            "Deliver",
            "corsairs_rest",
        ] {
            assert!(!line.contains(term), "{line}");
        }
        assert!(line.is_ascii());
    }
}
