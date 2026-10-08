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

/// `due soon` through two days left. Later deadlines stay `N days left`.
pub(crate) fn days_left_text(day: i64, deadline: i64) -> (String, bool) {
    let left = deadline - day;
    if left <= 2 {
        ("due soon".to_string(), true)
    } else {
        (format!("{left} days left"), false)
    }
}

pub(crate) fn requirement_text(tier: &str, standing: i64) -> String {
    let tier = ascii_sentence(tier);
    if standing > 0 {
        format!("trust {tier}  standing {standing}")
    } else {
        format!("trust {tier}")
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

/// Contracts desk notice after Complete or Abandon, in the Market paid-notice
/// form: `Contract abandoned: Grain for Corsair's Rest`.
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
    let tail = trim_tail(&outcome_tail(outcome), room);
    if tail.is_empty() {
        head
    } else {
        format!("{head}{sep}{tail}")
    }
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

/// The engine summary with its raw ids named: the destination port and the
/// good by their display names, any other `snake_id` title-cased.
pub(crate) fn outcome_summary(outcome: &ContractOutcome) -> String {
    let words: Vec<String> = outcome
        .summary
        .split(' ')
        .map(|word| {
            let core = word.trim_end_matches(['.', ',', ')', ':', ';']);
            let tail = &word[core.len()..];
            let name = if core.is_empty() {
                return word.to_string();
            } else if core == outcome.destination_port_id {
                port_display_name(core)
            } else if core == outcome.good_id {
                good_display_name(core)
            } else if is_snake_id(core) {
                humanize_id(core)
            } else {
                return word.to_string();
            };
            format!("{name}{tail}")
        })
        .collect();
    ascii_sentence(&words.join(" "))
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

fn port_display_name(id: &str) -> String {
    portlight_sim::content::content()
        .port(id)
        .map(|port| port.name.clone())
        .unwrap_or_else(|| humanize_id(id))
}

fn good_display_name(id: &str) -> String {
    portlight_sim::content::content()
        .good(id)
        .map(|good| good.name.clone())
        .unwrap_or_else(|| humanize_id(id))
}

fn is_snake_id(word: &str) -> bool {
    word.contains('_')
        && word
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

/// Desk Recent row, same builder as the notices: `Paid: Silver +612 - 23
/// Grain to Corsair's Rest - early bonus +60`, `Abandoned: Grain for
/// Corsair's Rest`. No raw outcome type, no sim summary.
pub(crate) fn recent_line(outcome: &ContractOutcome) -> String {
    outcome_line(recent_label(outcome), outcome, 0, DESK_CHARS)
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
    use portlight_sim::model::ContractOutcome;
    use portlight_sim::Session;

    use super::{
        abandon_prompt, ascii_sentence, availability_tag, can_complete, days_left_text,
        outcome_line, outcome_notice, outcome_summary, outcome_terms, progress_text, recent_line,
        requirement_text, reward_text, BOARD_CARD, CAP_FULL, DESK_CHARS, EMPTY_OFFERS, MAX_ACTIVE,
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
        assert_eq!(days_left_text(1, 3), ("due soon".to_string(), true));
        assert_eq!(days_left_text(1, 2), ("due soon".to_string(), true));
        assert_eq!(days_left_text(1, 12), ("11 days left".to_string(), false));
        assert_eq!(reward_text(480, 0), "480 silver");
        assert_eq!(reward_text(480, 60), "480 silver + 60 bonus");
        assert_eq!(requirement_text("unproven", 0), "trust unproven");
        assert_eq!(
            requirement_text("credible", 10),
            "trust credible  standing 10"
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

    #[test]
    fn outcome_summary_names_the_port_and_good() {
        let outcome = delivered("Delivered 23 grain to corsairs_rest (early bonus: +63 silver)");
        assert_eq!(
            outcome_summary(&outcome),
            "Delivered 23 Grain to Corsair's Rest (early bonus: +63 silver)"
        );
        let defaulted = delivered("Contract defaulted: failed to deliver grain to corsairs_rest");
        assert_eq!(
            outcome_summary(&defaulted),
            "Contract defaulted: failed to deliver Grain to Corsair's Rest"
        );
        // An id content does not know is title-cased; plain words stay.
        // `saltmarsh_guild` is in no catalog, so this is the humanize path.
        let odd = delivered("Settled with saltmarsh_guild today.");
        assert_eq!(outcome_summary(&odd), "Settled with Saltmarsh Guild today.");
        // A destination port and a good content does not list fall back too.
        let mut unknown = delivered("Delivered 4 whale_oil to drowned_quay.");
        unknown.good_id = "whale_oil".into();
        unknown.destination_port_id = "drowned_quay".into();
        assert!(portlight_sim::content::content()
            .port("drowned_quay")
            .is_none());
        assert!(portlight_sim::content::content()
            .good("whale_oil")
            .is_none());
        assert_eq!(
            outcome_summary(&unknown),
            "Delivered 4 Whale Oil to Drowned Quay."
        );
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

    /// GD M2 ruling: the one shared helper prints Silver only. Trust, standing
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

    /// GD final wording: silver first, `(+N more)`, then the tail from fields.
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

    /// GD R5: Recent rows use the same builder, no raw type or sim summary.
    #[test]
    fn recent_line_is_built_from_fields() {
        assert_eq!(
            recent_line(&bonus_paid()),
            "Paid: Silver +612 - 23 Grain to Corsair's Rest - early bonus +60"
        );
        assert_eq!(
            recent_line(&delivered("Delivered 23 grain to corsairs_rest")),
            "Paid: Silver +615 - 23 Grain to Corsair's Rest"
        );
        assert_eq!(
            recent_line(&failed("abandoned", 0)),
            "Abandoned: Grain for Corsair's Rest"
        );
        assert_eq!(
            recent_line(&failed("abandoned", -40)),
            "Abandoned: Silver -40 - Grain for Corsair's Rest"
        );
        assert_eq!(
            recent_line(&failed("expired", 0)),
            "Expired: Grain for Corsair's Rest"
        );
        assert_eq!(
            recent_line(&failed("expired", 60)),
            "Expired: Silver +60 - Grain for Corsair's Rest"
        );
        for kind in ["completed", "completed_bonus", "abandoned", "expired"] {
            let line = recent_line(&failed(kind, 0));
            assert_clean(&line);
            assert!(!line.contains("Famine relief"), "{line}");
            assert!(line.len() <= DESK_CHARS, "{line}");
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
}
