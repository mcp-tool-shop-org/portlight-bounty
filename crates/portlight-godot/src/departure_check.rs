//! Departure check: a small ink panel before a docked Sail, only when the
//! voyage needs a decision (brief `design-brief-departure-check.md`).
//!
//! Presentation only. Reads `world()`, `board()` and `sail_lanes()` plus
//! public sim helpers; never calls a mutating Session verb. Voyage days are
//! the lane's own `estimated_days` (the number behind the chart caption), and
//! the stores threshold is the lane's `provisions_needed`. No formula is
//! copied. Gate cases (not docked, unknown lane, Blocked lane, crew below the
//! template minimum) build no facts, so Sail departs exactly as before and the
//! sim's own refusal logs.
//!
//! Order: unsold here, late (here and elsewhere), at risk, stores, crew
//! (reserved, empty in v1), cargo. Cap [`DEPARTURE_LINE_CAP`] + `+N more`.

use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::classes::text_server::AutowrapMode;
use godot::classes::{Button, HBoxContainer, Label, PanelContainer, StyleBoxFlat, VBoxContainer};
use godot::prelude::*;
use portlight_sim::model::{ActiveContract, VoyageStatus, World};
use portlight_sim::session::Session;
use portlight_sim::{content, economy, ship, LaneSuitability, SailLane};

use crate::contracts_screen::{ascii_sentence, meta_color};
use crate::day_report::deadline_timing;
use crate::encounter_screen::style_encounter_button;
use crate::logic::humanize_id;

const INK: Color = Color::from_rgb(0.08, 0.11, 0.16);
const CREAM: Color = Color::from_rgb(0.94, 0.91, 0.84);
const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);

/// Warning lines shown before `+N more`. The context line is outside the cap.
/// Four (#63): four two-row lines plus `+N more` stay under
/// [`WORST_CASE_MAX_H`]. Play reaches four lines at most today.
pub(crate) const DEPARTURE_LINE_CAP: usize = 4;
/// `at risk` band for a contract due at the destination: `0 <= left - E <= 1`.
pub(crate) const AT_RISK_SLACK: i64 = 1;

/// Day's report slot: the two are mutually exclusive, so they never overlap.
pub(crate) const PANEL_X: f32 = 170.0;
pub(crate) const PANEL_Y: f32 = 40.0;
pub(crate) const PANEL_W: f32 = 520.0;
pub(crate) const PANEL_MIN_H: f32 = 160.0;
/// Height cap (clamp). A safety ceiling only: content is held to
/// [`WORST_CASE_MAX_H`] by the smoke, so the clamp never cuts content.
pub(crate) const PANEL_MAX_H: f32 = 420.0;
/// Most height the worst case (four two-row lines plus `+N more`, about
/// 373 px today) may need. It keeps the panel inside the Day's report card
/// area (same slot, 380 px tall); the smoke fails above it.
pub(crate) const WORST_CASE_MAX_H: f32 = 380.0;
const PANEL_MARGIN: f32 = 18.0;
/// Text column width inside the 2 px border and 18 px margin.
const COLUMN_W: f32 = PANEL_W - 2.0 * PANEL_MARGIN;

pub(crate) const STAY_TEXT: &str = "Stay in port";
pub(crate) const SAIL_TEXT: &str = "Sail anyway";

/// One active contract as the panel sees it. Names are already player copy.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ContractFacts {
    pub title: String,
    pub destination_id: String,
    pub destination_name: String,
    pub good_name: String,
    pub deadline_day: i64,
    pub need: i64,
    pub held: i64,
}

/// Plain data read at press time while docked.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DepartureFacts {
    pub day: i64,
    pub here_id: String,
    pub dest_id: String,
    pub dest_name: String,
    /// `SailLane.estimated_days` for `dest_id`.
    pub voyage_days: i64,
    /// `SailLane.provisions_needed` for `dest_id`.
    pub provisions_needed: i64,
    pub provisions: i64,
    pub hold_used: f64,
    pub hold_cap: i64,
    pub contracts: Vec<ContractFacts>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DepartureKind {
    UnsoldHere,
    Late,
    AtRisk,
    Stores,
    // Crew sits here in the order. Empty in v1 (brief D1): below the
    // minimum the panel is bypassed and the lane list carries the line.
    Cargo,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DepartureLine {
    pub text: String,
    pub kind: DepartureKind,
    /// DUE tone: the verdict is `will be late` or `may run out`.
    pub warn: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DepartureDoc {
    pub title: String,
    pub context: String,
    pub lines: Vec<DepartureLine>,
    /// Hidden warning lines past the cap (drawn as `+{n} more`).
    pub more: usize,
}

/// F9 (brief section 19): the lane-list line when the flagship is below its
/// template crew minimum. `None` at or above it (the `depart` predicate is
/// strict `<`).
pub(crate) fn crew_short_line(crew: i64, min: i64) -> Option<String> {
    (crew < min)
        .then(|| format!("Need {min} crew to sail - you have {crew}. Hire at the Crew desk."))
}

/// The gate. False means no panel: `depart` runs and its own refusal (or a
/// clean depart) happens exactly as before.
pub(crate) fn gate_allows_panel(
    docked: bool,
    lane: Option<&SailLane>,
    crew: i64,
    crew_min: i64,
) -> bool {
    docked
        && lane.is_some_and(|lane| lane.suitability != LaneSuitability::Blocked)
        && crew >= crew_min
}

/// Facts for a Sail press toward `dest`. `None` on any gate case.
pub(crate) fn departure_facts(session: &Session, dest: &str) -> Option<DepartureFacts> {
    let world = session.world();
    let docked = world.voyage.status == VoyageStatus::InPort;
    let flagship = world.captain.ship.as_ref()?;
    let lanes = session.sail_lanes();
    let lane = lanes.iter().find(|lane| lane.destination_id == dest);
    if !gate_allows_panel(
        docked,
        lane,
        flagship.crew,
        ship::template_crew_min(flagship),
    ) {
        return None;
    }
    let lane = lane?;
    let cargo = &world.captain.cargo;
    let contracts = session
        .board()
        .active
        .iter()
        .filter(|contract| contract.status == "accepted")
        .filter_map(|contract| contract_facts(world, contract, cargo))
        .collect();
    Some(DepartureFacts {
        day: world.day,
        here_id: world.voyage.destination_id.clone(),
        dest_id: lane.destination_id.clone(),
        dest_name: display_name(Some(lane.destination_name.as_str()), &lane.destination_id),
        voyage_days: lane.estimated_days,
        provisions_needed: lane.provisions_needed,
        provisions: world.captain.provisions,
        hold_used: economy::cargo_weight(cargo),
        hold_cap: ship::resolve_cargo_capacity(flagship),
        contracts,
    })
}

fn contract_facts(
    world: &World,
    contract: &ActiveContract,
    cargo: &[portlight_sim::model::CargoItem],
) -> Option<ContractFacts> {
    let need = contract.required_quantity - contract.delivered_quantity;
    if need <= 0 {
        return None;
    }
    let title = ascii_sentence(&contract.title);
    let title = if title.trim().is_empty() {
        humanize_id(&contract.template_id)
    } else {
        title
    };
    let port_name = world
        .port(&contract.destination_port_id)
        .map(|port| port.name.as_str());
    let good_name = content::content()
        .good(&contract.good_id)
        .map(|good| good.name.as_str());
    Some(ContractFacts {
        title,
        destination_id: contract.destination_port_id.clone(),
        destination_name: display_name(port_name, &contract.destination_port_id),
        good_name: display_name(good_name, &contract.good_id),
        deadline_day: contract.deadline_day,
        need,
        held: crate::game::cargo_held(cargo, &contract.good_id),
    })
}

/// Catalog name when it is ASCII; otherwise the shared humaniser on the id
/// (never a raw id).
fn display_name(name: Option<&str>, id: &str) -> String {
    match name {
        Some(name) if !name.is_empty() && name.is_ascii() => name.to_string(),
        _ => humanize_id(id),
    }
}

fn days_text(days: i64) -> String {
    if days == 1 {
        "1 day".to_string()
    } else {
        format!("{days} days")
    }
}

/// `12` when whole, else one decimal (`12.5`).
fn hold_amount(used: f64) -> String {
    if used.fract() == 0.0 {
        format!("{}", used as i64)
    } else {
        format!("{used:.1}")
    }
}

pub(crate) fn context_line(facts: &DepartureFacts) -> String {
    let voyage = format!("Voyage {}", days_text(facts.voyage_days));
    if facts.hold_used <= 0.0 {
        format!("{voyage} - Hold empty")
    } else {
        format!(
            "{voyage} - Hold {}/{}",
            hold_amount(facts.hold_used),
            facts.hold_cap
        )
    }
}

/// True when the ASCII title already names the port (case-insensitive), so a
/// `to {Port}` tail would repeat it. Shared with the contract strip.
pub(crate) fn title_names_port(title: &str, port: &str) -> bool {
    !port.is_empty()
        && title
            .to_ascii_lowercase()
            .contains(&port.to_ascii_lowercase())
}

struct Ranked {
    line: DepartureLine,
    deadline_day: i64,
    title: String,
}

/// Pure document builder. `None` means no warning line: no panel.
pub(crate) fn build_departure_check(facts: &DepartureFacts) -> Option<DepartureDoc> {
    let e = facts.voyage_days;
    let mut ranked: Vec<Ranked> = Vec::new();
    for contract in &facts.contracts {
        let left = contract.deadline_day - facts.day;
        let timing = deadline_timing(left);
        let title = contract.title.as_str();
        let line = if contract.destination_id == facts.here_id && contract.held > 0 {
            let n = contract.held.min(contract.need);
            Some(DepartureLine {
                text: format!(
                    "{title} - {n} {} aboard - sell here before you sail",
                    contract.good_name
                ),
                kind: DepartureKind::UnsoldHere,
                warn: false,
            })
        } else if contract.destination_id == facts.dest_id {
            if e > left {
                Some(DepartureLine {
                    text: format!("{title} - {timing} - will be late"),
                    kind: DepartureKind::Late,
                    warn: true,
                })
            } else if left - e <= AT_RISK_SLACK {
                Some(DepartureLine {
                    text: format!("{title} - {timing} - at risk"),
                    kind: DepartureKind::AtRisk,
                    warn: false,
                })
            } else if contract.held == 0 {
                Some(DepartureLine {
                    text: format!(
                        "{title} - no {} aboard - needs {}",
                        contract.good_name, contract.need
                    ),
                    kind: DepartureKind::Cargo,
                    warn: false,
                })
            } else {
                None
            }
        } else if e >= left {
            let text = if title_names_port(title, &contract.destination_name) {
                format!("{title} - {timing} - will be late")
            } else {
                format!(
                    "{title} - to {} - {timing} - will be late",
                    contract.destination_name
                )
            };
            Some(DepartureLine {
                text,
                kind: DepartureKind::Late,
                warn: true,
            })
        } else {
            None
        };
        if let Some(line) = line {
            ranked.push(Ranked {
                line,
                deadline_day: contract.deadline_day,
                title: contract.title.clone(),
            });
        }
    }
    if facts.provisions < facts.provisions_needed {
        let mut text = format!(
            "Stores {} - {} advised for {} at sea",
            facts.provisions,
            facts.provisions_needed,
            days_text(e)
        );
        let short = facts.provisions < e;
        if short {
            text.push_str(" - may run out");
        }
        ranked.push(Ranked {
            line: DepartureLine {
                text,
                kind: DepartureKind::Stores,
                warn: short,
            },
            deadline_day: 0,
            title: String::new(),
        });
    }
    if ranked.is_empty() {
        return None;
    }
    ranked.sort_by(|a, b| {
        a.line
            .kind
            .cmp(&b.line.kind)
            .then_with(|| a.deadline_day.cmp(&b.deadline_day))
            .then_with(|| a.title.cmp(&b.title))
    });
    let total = ranked.len();
    let lines = ranked
        .into_iter()
        .take(DEPARTURE_LINE_CAP)
        .map(|ranked| ranked.line)
        .collect::<Vec<_>>();
    Some(DepartureDoc {
        title: format!("Before you sail - {}", facts.dest_name),
        context: context_line(facts),
        more: total - lines.len(),
        lines,
    })
}

/// Facts + document in one call (the `request_sail` gate).
pub(crate) fn departure_document(session: &Session, dest: &str) -> Option<DepartureDoc> {
    departure_facts(session, dest).and_then(|facts| build_departure_check(&facts))
}

// ---- UI -------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct DepartureNodes {
    pub root: Gd<PanelContainer>,
    pub title: Gd<Label>,
    pub context: Gd<Label>,
    pub lines: Gd<VBoxContainer>,
    pub stay: Gd<Button>,
    pub sail: Gd<Button>,
}

pub(crate) fn build_departure_screen() -> DepartureNodes {
    let mut root = PanelContainer::new_alloc();
    root.set_name("DepartureCheck");
    root.set_mouse_filter(MouseFilter::IGNORE);
    root.set_visible(false);
    let mut style = StyleBoxFlat::new_gd();
    style.set_bg_color(INK);
    style.set_border_color(GOLD);
    style.set_border_width_all(2);
    style.set_content_margin_all(PANEL_MARGIN);
    root.add_theme_stylebox_override("panel", &style);
    root.set_custom_minimum_size(Vector2::new(PANEL_W, PANEL_MIN_H));

    let mut column = VBoxContainer::new_alloc();
    column.set_name("DepartureColumn");
    column.set_h_size_flags(SizeFlags::EXPAND_FILL);
    column.add_theme_constant_override("separation", 8);
    root.add_child(&column);

    let mut head = VBoxContainer::new_alloc();
    head.set_name("DepartureHead");
    head.add_theme_constant_override("separation", 2);
    let mut title = text_label("", 24, GOLD);
    title.set_name("DepartureTitle");
    head.add_child(&title);
    let mut context = text_label("", 14, MUTED);
    context.set_name("DepartureContext");
    head.add_child(&context);
    column.add_child(&head);

    let mut lines = VBoxContainer::new_alloc();
    lines.set_name("DepartureLines");
    lines.set_h_size_flags(SizeFlags::EXPAND_FILL);
    lines.add_theme_constant_override("separation", 4);
    column.add_child(&lines);

    let mut row = HBoxContainer::new_alloc();
    row.set_name("DepartureButtons");
    row.add_theme_constant_override("separation", 12);
    let mut stay = Button::new_alloc();
    stay.set_name("DepartureStay");
    stay.set_text(STAY_TEXT);
    style_encounter_button(&mut stay);
    stay.set_h_size_flags(SizeFlags::SHRINK_BEGIN);
    row.add_child(&stay);
    let mut sail = Button::new_alloc();
    sail.set_name("DepartureSail");
    sail.set_text(SAIL_TEXT);
    style_encounter_button(&mut sail);
    sail.set_h_size_flags(SizeFlags::SHRINK_BEGIN);
    row.add_child(&sail);
    column.add_child(&row);

    DepartureNodes {
        root,
        title,
        context,
        lines,
        stay,
        sail,
    }
}

/// Day's report slot, height fit to content (min 160). The cap keeps it
/// under [`PANEL_MAX_H`]; the smoke asserts that.
pub(crate) fn place_panel(root: &mut Gd<PanelContainer>) {
    root.set_anchors_and_offsets_preset(LayoutPreset::TOP_LEFT);
    root.set_position(Vector2::new(PANEL_X, PANEL_Y));
    let height = root
        .get_combined_minimum_size()
        .y
        .clamp(PANEL_MIN_H, PANEL_MAX_H);
    root.set_size(Vector2::new(PANEL_W, height));
}

pub(crate) fn set_open(nodes: &mut DepartureNodes, open: bool) {
    nodes.root.set_visible(open);
    nodes.root.set_mouse_filter(if open {
        MouseFilter::STOP
    } else {
        MouseFilter::IGNORE
    });
}

pub(crate) fn apply_document(nodes: &mut DepartureNodes, doc: &DepartureDoc) {
    nodes.title.set_text(&doc.title);
    nodes.context.set_text(&doc.context);
    let children = nodes.lines.get_children();
    for mut child in children.iter_shared() {
        nodes.lines.remove_child(&child);
        child.queue_free();
    }
    for line in &doc.lines {
        let color = if line.warn { meta_color(true) } else { CREAM };
        nodes.lines.add_child(&text_label(&line.text, 15, color));
    }
    if doc.more > 0 {
        nodes
            .lines
            .add_child(&text_label(&more_text(doc.more), 15, MUTED));
    }
}

/// The MUTED overflow row: `+{n} more`.
pub(crate) fn more_text(more: usize) -> String {
    format!("+{more} more")
}

/// Drawn button labels left to right (the smoke pins `[Stay in port, Sail anyway]`).
pub(crate) fn button_texts(nodes: &DepartureNodes) -> Vec<String> {
    nodes
        .stay
        .get_parent()
        .map(|row| {
            row.get_children()
                .iter_shared()
                .filter_map(|child| child.try_cast::<Button>().ok())
                .map(|button| button.get_text().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// The tallest document the panel can get: a long title, the cap of
/// two-row warning lines, and `+N more`. Used by the smoke's height check.
/// The hold is the largest play can reach: a Royal Man-of-War (200) with all
/// six upgrade slots on Reinforced Bulkheads (+20 each), nearly full.
pub(crate) fn worst_case_document() -> DepartureDoc {
    let long = "Premium charter: black powder to Corsair's Rest - 0/40 - 1 day left - will be late";
    DepartureDoc {
        title: "Before you sail - Corsair's Rest".to_string(),
        context: "Voyage 12 days - Hold 319.5/320".to_string(),
        lines: (0..DEPARTURE_LINE_CAP)
            .map(|_| DepartureLine {
                text: format!("{long} - sell 40 more Black Powder"),
                kind: DepartureKind::Late,
                warn: true,
            })
            .collect(),
        more: 9,
    }
}

pub(crate) fn overlay_visible(nodes: &DepartureNodes) -> bool {
    nodes.root.is_visible()
}

/// Drawn warning lines (incl. `+N more`), for smoke asserts.
pub(crate) fn line_texts(nodes: &DepartureNodes) -> Vec<String> {
    nodes
        .lines
        .get_children()
        .iter_shared()
        .filter_map(|child| child.try_cast::<Label>().ok())
        .map(|label| label.get_text().to_string())
        .collect()
}

/// Open-panel fit: `Some` names what broke.
pub(crate) fn panel_fit_error(nodes: &DepartureNodes) -> Option<String> {
    let pos = nodes.root.get_position();
    let size = nodes.root.get_size();
    let min = nodes.root.get_combined_minimum_size();
    let panel = nodes.root.get_global_rect();
    let mut problems = Vec::new();
    if (pos.x - PANEL_X).abs() > 1.0 || (pos.y - PANEL_Y).abs() > 1.0 {
        problems.push(format!(
            "at ({}, {}) want ({PANEL_X}, {PANEL_Y})",
            pos.x, pos.y
        ));
    }
    if (size.x - PANEL_W).abs() > 1.0 {
        problems.push(format!("width {} want {PANEL_W}", size.x));
    }
    if size.y > PANEL_MAX_H + 0.5 || min.y > PANEL_MAX_H + 0.5 {
        problems.push(format!(
            "height {} min {} over {PANEL_MAX_H}",
            size.y, min.y
        ));
    }
    for button in [&nodes.stay, &nodes.sail] {
        let rect = button.get_global_rect();
        let inside = rect.position.y >= panel.position.y
            && rect.position.y + rect.size.y <= panel.position.y + panel.size.y + 1.0
            && rect.position.x + rect.size.x <= panel.position.x + panel.size.x + 1.0;
        let styled = button.get_theme_stylebox("normal").is_some();
        if !button.is_visible_in_tree() || !inside || !styled || rect.size.y <= 0.0 {
            problems.push(format!(
                "button '{}' visible {} inside {inside} styled {styled} rect {}x{} at ({}, {})",
                button.get_text(),
                button.is_visible_in_tree(),
                rect.size.x,
                rect.size.y,
                rect.position.x,
                rect.position.y
            ));
        }
    }
    if problems.is_empty() {
        None
    } else {
        Some(problems.join("; "))
    }
}

fn text_label(text: &str, size: i32, color: Color) -> Gd<Label> {
    let mut label = Label::new_alloc();
    label.set_text(text);
    label.add_theme_font_size_override("font_size", size);
    label.add_theme_color_override("font_color", color);
    label.set_autowrap_mode(AutowrapMode::WORD_SMART);
    label.set_h_size_flags(SizeFlags::EXPAND_FILL);
    // A known width from the start, so the wrapped height is right before
    // the first container sort (the panel height fits its content).
    label.set_custom_minimum_size(Vector2::new(COLUMN_W, 0.0));
    label
}

#[cfg(test)]
mod tests {
    use super::*;
    use portlight_chart::project_chart;

    fn base_facts(contracts: Vec<ContractFacts>) -> DepartureFacts {
        DepartureFacts {
            day: 10,
            here_id: "porto_novo".into(),
            dest_id: "corsairs_rest".into(),
            dest_name: "Corsair's Rest".into(),
            voyage_days: 2,
            provisions_needed: 4,
            provisions: 30,
            hold_used: 0.0,
            hold_cap: 30,
            contracts,
        }
    }

    fn contract(
        title: &str,
        dest: &str,
        dest_name: &str,
        deadline: i64,
        held: i64,
    ) -> ContractFacts {
        ContractFacts {
            title: title.into(),
            destination_id: dest.into(),
            destination_name: dest_name.into(),
            good_name: "Grain".into(),
            deadline_day: deadline,
            need: 5,
            held,
        }
    }

    fn here(deadline: i64, held: i64) -> ContractFacts {
        contract(
            "Famine relief: grain to Corsair's Rest",
            "corsairs_rest",
            "Corsair's Rest",
            deadline,
            held,
        )
    }

    fn texts(doc: &DepartureDoc) -> Vec<String> {
        doc.lines.iter().map(|line| line.text.clone()).collect()
    }

    fn one_line(facts: &DepartureFacts) -> DepartureLine {
        let doc = build_departure_check(facts).expect("panel");
        assert_eq!(doc.lines.len(), 1, "{:?}", texts(&doc));
        doc.lines[0].clone()
    }

    // 1
    #[test]
    fn late_here_fires_when_voyage_outruns_the_deadline() {
        let line = one_line(&base_facts(vec![here(11, 5)]));
        assert_eq!(
            line.text,
            "Famine relief: grain to Corsair's Rest - 1 day left - will be late"
        );
        assert_eq!(line.kind, DepartureKind::Late);
        assert!(line.warn);
        let due = one_line(&base_facts(vec![here(10, 5)]));
        assert!(
            due.text.ends_with(" - due today - will be late"),
            "{}",
            due.text
        );
        let over = one_line(&base_facts(vec![here(9, 5)]));
        assert!(
            over.text.ends_with(" - overdue - will be late"),
            "{}",
            over.text
        );
        // E == left is not late (it is at risk).
        let even = one_line(&base_facts(vec![here(12, 5)]));
        assert_eq!(even.kind, DepartureKind::AtRisk);
    }

    // 2
    #[test]
    fn at_risk_band_is_zero_or_one_day_of_slack() {
        let zero = one_line(&base_facts(vec![here(12, 5)]));
        assert_eq!(
            zero.text,
            "Famine relief: grain to Corsair's Rest - 2 days left - at risk"
        );
        assert!(!zero.warn);
        let one = one_line(&base_facts(vec![here(13, 5)]));
        assert_eq!(
            one.text,
            "Famine relief: grain to Corsair's Rest - 3 days left - at risk"
        );
        assert_eq!(build_departure_check(&base_facts(vec![here(14, 5)])), None);
    }

    // 3
    #[test]
    fn late_elsewhere_names_the_port_unless_the_title_does() {
        let grain = contract("Grain run", "al_manar", "Al-Manar", 12, 5);
        let line = one_line(&base_facts(vec![grain.clone()]));
        assert_eq!(
            line.text,
            "Grain run - to Al-Manar - 2 days left - will be late"
        );
        assert!(line.warn);
        let titled = contract(
            "Spice restock run to Al-Manar",
            "al_manar",
            "Al-Manar",
            12,
            0,
        );
        assert_eq!(
            one_line(&base_facts(vec![titled])).text,
            "Spice restock run to Al-Manar - 2 days left - will be late"
        );
        let lower = contract("spice run to al-manar", "al_manar", "Al-Manar", 11, 0);
        assert_eq!(
            one_line(&base_facts(vec![lower])).text,
            "spice run to al-manar - 1 day left - will be late"
        );
        // E < left: silent. Elsewhere never says at risk.
        let calm = contract("Grain run", "al_manar", "Al-Manar", 13, 0);
        assert_eq!(build_departure_check(&base_facts(vec![calm])), None);
        let near = contract("Grain run", "al_manar", "Al-Manar", 13, 0);
        let mut facts = base_facts(vec![near]);
        facts.voyage_days = 2;
        assert!(build_departure_check(&facts).is_none());
    }

    // 4
    #[test]
    fn stores_line_uses_the_lane_advice() {
        let mut facts = base_facts(Vec::new());
        facts.voyage_days = 3;
        facts.provisions_needed = 5;
        facts.provisions = 4;
        let line = one_line(&facts);
        assert_eq!(line.text, "Stores 4 - 5 advised for 3 days at sea");
        assert_eq!(line.kind, DepartureKind::Stores);
        assert!(!line.warn);
        facts.voyage_days = 5;
        facts.provisions_needed = 7;
        facts.provisions = 2;
        let short = one_line(&facts);
        assert_eq!(
            short.text,
            "Stores 2 - 7 advised for 5 days at sea - may run out"
        );
        assert!(short.warn);
        facts.voyage_days = 1;
        facts.provisions_needed = 3;
        facts.provisions = 2;
        assert_eq!(
            one_line(&facts).text,
            "Stores 2 - 3 advised for 1 day at sea"
        );
        facts.provisions = 3;
        assert_eq!(build_departure_check(&facts), None);
    }

    // 5
    #[test]
    fn cargo_line_only_for_a_calm_destination_contract_with_none_aboard() {
        let mut porcelain = contract(
            "Porcelain for Al-Manar estate",
            "corsairs_rest",
            "Corsair's Rest",
            30,
            0,
        );
        porcelain.good_name = "Porcelain".into();
        let line = one_line(&base_facts(vec![porcelain.clone()]));
        assert_eq!(
            line.text,
            "Porcelain for Al-Manar estate - no Porcelain aboard - needs 5"
        );
        assert_eq!(line.kind, DepartureKind::Cargo);
        assert!(!line.warn);
        porcelain.held = 1;
        assert_eq!(build_departure_check(&base_facts(vec![porcelain])), None);
        // Elsewhere contract with nothing aboard and time to spare: silent.
        let far = contract("Grain run", "al_manar", "Al-Manar", 30, 0);
        assert_eq!(build_departure_check(&base_facts(vec![far])), None);
        // A deadline line suppresses the cargo line for the same contract.
        let late = one_line(&base_facts(vec![here(11, 0)]));
        assert_eq!(late.kind, DepartureKind::Late);
        let risky = one_line(&base_facts(vec![here(12, 0)]));
        assert_eq!(risky.kind, DepartureKind::AtRisk);
    }

    // 6
    #[test]
    fn nothing_to_warn_means_no_panel() {
        let mut facts = base_facts(vec![
            here(40, 5),
            contract("Grain run", "al_manar", "Al-Manar", 40, 0),
        ]);
        facts.hold_used = 30.0;
        facts.provisions = facts.provisions_needed;
        assert_eq!(build_departure_check(&facts), None);
        assert_eq!(build_departure_check(&base_facts(Vec::new())), None);
    }

    fn lane(suitability: LaneSuitability) -> SailLane {
        SailLane {
            destination_id: "corsairs_rest".into(),
            destination_name: "Corsair's Rest".into(),
            region: "mediterranean".into(),
            distance: 16,
            danger: 0.1,
            min_ship_class: "sloop".into(),
            suitability,
            suitability_note: None,
            estimated_days: 2,
            provisions_needed: 4,
        }
    }

    // 7
    #[test]
    fn gate_cases_build_no_panel() {
        let ok = lane(LaneSuitability::Ok);
        let warning = lane(LaneSuitability::Warning);
        let blocked = lane(LaneSuitability::Blocked);
        assert!(gate_allows_panel(true, Some(&ok), 3, 3));
        assert!(gate_allows_panel(true, Some(&warning), 3, 3));
        assert!(!gate_allows_panel(true, Some(&blocked), 3, 3));
        assert!(!gate_allows_panel(true, Some(&ok), 2, 3));
        assert!(!gate_allows_panel(false, Some(&ok), 3, 3));
        assert!(!gate_allows_panel(true, None, 3, 3));

        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        assert!(departure_facts(&session, "corsairs_rest").is_some());
        assert!(departure_facts(&session, "nowhere").is_none());
        assert!(departure_facts(&session, "porto_novo").is_none());
        session.fire_crew(1, "sailor").unwrap();
        assert!(departure_facts(&session, "corsairs_rest").is_none());
        session.hire_crew(1, "sailor").unwrap();
        assert!(departure_facts(&session, "corsairs_rest").is_some());
        session.depart("corsairs_rest").unwrap();
        assert!(departure_facts(&session, "silva_bay").is_none());
        assert!(departure_facts(&session, "corsairs_rest").is_none());
    }

    // 8
    #[test]
    fn order_is_unsold_late_risk_stores_cargo() {
        let mut facts = base_facts(vec![
            {
                let mut c = contract("Zeta cargo", "corsairs_rest", "Corsair's Rest", 40, 0);
                c.good_name = "Silk".into();
                c
            },
            contract("Beta risk", "corsairs_rest", "Corsair's Rest", 13, 3),
            contract("Bravo late", "al_manar", "Al-Manar", 12, 0),
            contract("Alpha late", "silva_bay", "Silva Bay", 12, 0),
            contract("Early late", "corsairs_rest", "Corsair's Rest", 11, 0),
            contract("Sell me", "porto_novo", "Porto Novo", 40, 2),
        ]);
        facts.provisions = 3;
        let doc = build_departure_check(&facts).unwrap();
        let kinds = doc.lines.iter().map(|line| line.kind).collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                DepartureKind::UnsoldHere,
                DepartureKind::Late,
                DepartureKind::Late,
                DepartureKind::Late,
            ]
        );
        assert!(doc.lines[1].text.starts_with("Early late - "));
        assert!(doc.lines[2].text.starts_with("Alpha late - "));
        assert!(doc.lines[3].text.starts_with("Bravo late - "));
        // At risk, stores and cargo sit past the cap.
        assert_eq!(doc.more, 3);
        facts
            .contracts
            .retain(|c| c.title.contains("late") || c.title == "Zeta cargo");
        let doc = build_departure_check(&facts).unwrap();
        let kinds = doc.lines.iter().map(|line| line.kind).collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                DepartureKind::Late,
                DepartureKind::Late,
                DepartureKind::Late,
                DepartureKind::Stores,
            ]
        );
        assert_eq!(doc.more, 1);
        facts.contracts.retain(|c| c.title != "Bravo late");
        let doc = build_departure_check(&facts).unwrap();
        let kinds = doc.lines.iter().map(|line| line.kind).collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                DepartureKind::Late,
                DepartureKind::Late,
                DepartureKind::Stores,
                DepartureKind::Cargo,
            ]
        );
        assert_eq!(doc.more, 0);
    }

    // 9
    #[test]
    fn cap_is_four_plus_more() {
        assert_eq!(DEPARTURE_LINE_CAP, 4);
        let four = (0..4)
            .map(|i| contract(&format!("Run {i}"), "al_manar", "Al-Manar", 11, 0))
            .collect::<Vec<_>>();
        let mut facts = base_facts(four);
        facts.provisions = 1;
        let doc = build_departure_check(&facts).unwrap();
        assert_eq!(doc.lines.len(), DEPARTURE_LINE_CAP);
        assert_eq!(doc.more, 1);
        facts.provisions = 30;
        let doc = build_departure_check(&facts).unwrap();
        assert_eq!(doc.lines.len(), 4);
        assert_eq!(doc.more, 0);
    }

    /// #63 follow-up: the drawn overflow row and the two button labels.
    #[test]
    fn more_row_and_button_labels() {
        assert_eq!(more_text(1), "+1 more");
        assert_eq!(more_text(12), "+12 more");
        assert_eq!(STAY_TEXT, "Stay in port");
        assert_eq!(SAIL_TEXT, "Sail anyway");
        let worst = worst_case_document();
        assert_eq!(worst.lines.len(), DEPARTURE_LINE_CAP);
        assert!(worst.more > 0);
        for text in worst.lines.iter().map(|line| &line.text) {
            assert!(text.is_ascii() && text.len() > 64, "{text}");
        }
    }

    // 10
    #[test]
    fn copy_is_ascii_and_uses_the_shared_timing_words() {
        let mut facts = base_facts(vec![
            here(11, 0),
            contract("Grain run", "al_manar", "Al-Manar", 9, 0),
            contract("Sell me", "porto_novo", "Porto Novo", 40, 9),
        ]);
        facts.provisions = 1;
        let doc = build_departure_check(&facts).unwrap();
        let mut all = vec![doc.title.clone(), doc.context.clone()];
        all.extend(texts(&doc));
        for text in &all {
            assert!(text.is_ascii(), "{text}");
            for banned in [
                "\u{2014}", "\u{2013}", "->", "due soon", "Complete", "Deliver", "Claim",
            ] {
                assert!(!text.contains(banned), "{text} has {banned}");
            }
        }
        assert!(texts(&doc).iter().any(|t| t.contains(&deadline_timing(1))));
        assert!(texts(&doc).iter().any(|t| t.contains(&deadline_timing(-1))));
        assert_eq!(doc.title, "Before you sail - Corsair's Rest");
        assert_eq!(doc.context, "Voyage 2 days - Hold empty");
        facts.hold_used = 12.5;
        facts.voyage_days = 1;
        assert_eq!(context_line(&facts), "Voyage 1 day - Hold 12.5/30");
        facts.hold_used = 12.0;
        assert_eq!(context_line(&facts), "Voyage 1 day - Hold 12/30");
        // Raw ids go through the shared humaniser, never print as-is.
        assert_eq!(display_name(None, "corsairs_rest"), "Corsairs Rest");
        assert_eq!(display_name(Some("Al-Man\u{e2}r"), "al_manar"), "Al Manar");
        assert_eq!(display_name(Some("Silva Bay"), "silva_bay"), "Silva Bay");
    }

    // 11
    #[test]
    fn voyage_days_match_the_chart_lane() {
        let session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let chart = project_chart(&session);
        assert!(!chart.lanes.is_empty());
        for lane in &chart.lanes {
            let facts = departure_facts(&session, &lane.destination_id).expect("facts");
            assert_eq!(
                facts.voyage_days, lane.estimated_days,
                "{}",
                lane.destination_id
            );
        }
    }

    #[test]
    fn unsold_here_fires_first_and_clamps_to_need() {
        let mut sell = contract("Grain for Porto Novo", "porto_novo", "Porto Novo", 40, 9);
        let line = one_line(&base_facts(vec![sell.clone()]));
        assert_eq!(
            line.text,
            "Grain for Porto Novo - 5 Grain aboard - sell here before you sail"
        );
        assert_eq!(line.kind, DepartureKind::UnsoldHere);
        assert!(!line.warn);
        sell.held = 2;
        assert_eq!(
            one_line(&base_facts(vec![sell.clone()])).text,
            "Grain for Porto Novo - 2 Grain aboard - sell here before you sail"
        );
        sell.held = 0;
        assert_eq!(build_departure_check(&base_facts(vec![sell.clone()])), None);
        let elsewhere = contract("Grain run", "al_manar", "Al-Manar", 40, 9);
        assert_eq!(build_departure_check(&base_facts(vec![elsewhere])), None);
        sell.held = 3;
        let doc = build_departure_check(&base_facts(vec![here(11, 0), sell])).unwrap();
        assert_eq!(doc.lines[0].kind, DepartureKind::UnsoldHere);
        assert_eq!(doc.lines[1].kind, DepartureKind::Late);
    }

    // Section 19 (F9)
    #[test]
    fn crew_short_line_only_below_the_minimum() {
        let line = crew_short_line(1, 3).unwrap();
        assert_eq!(
            line,
            "Need 3 crew to sail - you have 1. Hire at the Crew desk."
        );
        assert!(line.is_ascii());
        for banned in ["Deliver", "Complete", "\u{2014}", "\u{2013}"] {
            assert!(!line.contains(banned));
        }
        assert_eq!(crew_short_line(3, 3), None);
        assert_eq!(crew_short_line(4, 3), None);
    }

    #[test]
    fn seed_one_famine_relief_reads_late_at_day_18() {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let offer = session
            .available_contracts()
            .into_iter()
            .find(|offer| offer.good_id == "grain" && offer.destination_port_id == "corsairs_rest")
            .expect("famine relief");
        session.accept_contract(&offer.id).unwrap();
        let mut facts = departure_facts(&session, "corsairs_rest").unwrap();
        assert_eq!(facts.contracts.len(), 1);
        assert_eq!(facts.voyage_days, 2);
        facts.day = facts.contracts[0].deadline_day - 1;
        let doc = build_departure_check(&facts).unwrap();
        assert!(doc.lines[0].text.ends_with("1 day left - will be late"));
    }
}
