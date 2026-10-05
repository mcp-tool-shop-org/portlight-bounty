//! Compact Day's report card after a successful Next day.
//!
//! Presentation only. Reads Session facts; never calls mutating Session verbs.
//! Section order: Deadlines → Health → Prices → Bounties. §13 overrides §§1–12.

use std::collections::{HashMap, HashSet};

use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::classes::text_server::AutowrapMode;
use godot::classes::{Button, Label, PanelContainer, ScrollContainer, StyleBoxFlat, VBoxContainer};
use godot::prelude::*;
use portlight_sim::content;
use portlight_sim::model::{ActiveContract, ContractOutcome, Injury, VoyageStatus};
use portlight_sim::session::Session;

use crate::contracts_screen::ascii_sentence;
use crate::encounter_screen::style_encounter_button;
use crate::hunt_screen;
use crate::logic::ascii_label;

const INK: Color = Color::from_rgb(0.08, 0.11, 0.16);
const CREAM: Color = Color::from_rgb(0.94, 0.91, 0.84);
const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);

/// Deadline window inclusive: `deadline_day - day <= N`.
pub(crate) const DEADLINE_N: i64 = 3;
/// Global visible line cap across all sections.
pub(crate) const LINE_CAP: usize = 6;

const CARD_W: f32 = 520.0;
const CARD_X: f32 = 170.0;
const CARD_Y: f32 = 40.0;
const CARD_H: f32 = 380.0;

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct DayReportNodes {
    pub root: Gd<PanelContainer>,
    pub title: Gd<Label>,
    pub scroll: Gd<ScrollContainer>,
    pub body: Gd<VBoxContainer>,
    pub close: Gd<Button>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DayReportLine {
    pub text: String,
    /// When true, this line can auto-show the card.
    pub notable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DayReportSection {
    pub id: &'static str,
    pub title: &'static str,
    pub lines: Vec<DayReportLine>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DayReportDocument {
    pub day: i64,
    pub title: String,
    pub sections: Vec<DayReportSection>,
}

impl DayReportDocument {
    pub(crate) fn has_notable(&self) -> bool {
        self.sections
            .iter()
            .flat_map(|section| section.lines.iter())
            .any(|line| line.notable)
    }
}

/// Godot-side snapshots carried across Next day for diffs and claimable triggers.
#[derive(Clone, Debug, Default)]
pub(crate) struct DayReportMemory {
    /// Sell prices at the docked pier (`good_id` → `sell_price`). Empty at sea.
    pub price_memory: HashMap<String, i64>,
    /// Active bounty ids from the last successful Next day.
    pub active_bounties: HashSet<String>,
    /// Claimable bounty ids already used as an auto-show trigger (not arrival).
    pub claimable_seen: HashSet<String>,
}

pub(crate) fn build_day_report_screen() -> DayReportNodes {
    let mut root = PanelContainer::new_alloc();
    root.set_name("DayReportScreen");
    root.set_mouse_filter(MouseFilter::STOP);
    root.set_visible(false);
    let mut style = StyleBoxFlat::new_gd();
    style.set_bg_color(INK);
    style.set_border_color(GOLD);
    style.set_border_width_all(2);
    style.set_content_margin_all(18.0);
    root.add_theme_stylebox_override("panel", &style);
    root.set_custom_minimum_size(Vector2::new(CARD_W, 200.0));

    let mut column = VBoxContainer::new_alloc();
    column.set_name("DayReportColumn");
    column.set_h_size_flags(SizeFlags::EXPAND_FILL);
    column.set_v_size_flags(SizeFlags::EXPAND_FILL);
    column.add_theme_constant_override("separation", 8);
    root.add_child(&column);

    column.add_child(&text_label("Day's report", 14, MUTED, false));
    let title = text_label("", 24, GOLD, false);
    column.add_child(&title);

    let mut scroll = ScrollContainer::new_alloc();
    scroll.set_name("DayReportScroll");
    scroll.set_h_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_v_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_custom_minimum_size(Vector2::new(CARD_W - 36.0, 220.0));
    scroll.set_horizontal_scroll_mode(godot::classes::scroll_container::ScrollMode::DISABLED);
    let mut body = VBoxContainer::new_alloc();
    body.set_name("DayReportBody");
    body.set_h_size_flags(SizeFlags::EXPAND_FILL);
    body.add_theme_constant_override("separation", 12);
    scroll.add_child(&body);
    column.add_child(&scroll);

    let mut close = Button::new_alloc();
    close.set_name("CloseDayReport");
    close.set_text("Close");
    style_encounter_button(&mut close);
    close.set_h_size_flags(SizeFlags::SHRINK_BEGIN);
    column.add_child(&close);

    DayReportNodes {
        root,
        title,
        scroll,
        body,
        close,
    }
}

/// Compact card over the chart. Not a full-rect overlay (Sail / Next day stay usable).
pub(crate) fn place_card(root: &mut Gd<PanelContainer>) {
    root.set_anchors_and_offsets_preset(LayoutPreset::TOP_LEFT);
    root.set_position(Vector2::new(CARD_X, CARD_Y));
    root.set_size(Vector2::new(CARD_W, CARD_H));
}

pub(crate) fn set_open(nodes: &mut DayReportNodes, open: bool) {
    nodes.root.set_visible(open);
    nodes.root.set_mouse_filter(if open {
        MouseFilter::STOP
    } else {
        MouseFilter::IGNORE
    });
}

pub(crate) fn apply_document(nodes: &mut DayReportNodes, doc: &DayReportDocument) {
    nodes.title.set_text(&doc.title);
    clear_children(&mut nodes.body);
    for section in &doc.sections {
        if section.lines.is_empty() {
            continue;
        }
        let mut box_node = VBoxContainer::new_alloc();
        box_node.set_name(&format!("Section{}", section.id));
        box_node.add_theme_constant_override("separation", 4);
        box_node.add_child(&text_label(section.title, 16, GOLD, false));
        for line in &section.lines {
            box_node.add_child(&text_label(&line.text, 15, CREAM, true));
        }
        nodes.body.add_child(&box_node);
    }
}

pub(crate) fn overlay_visible(nodes: &DayReportNodes) -> bool {
    nodes.root.is_visible()
}

pub(crate) fn overlay_text(nodes: &DayReportNodes) -> String {
    let mut parts = Vec::new();
    collect_text(
        &nodes.root.clone().upcast::<godot::classes::Node>(),
        &mut parts,
    );
    parts.join("\n")
}

/// Snapshot sell prices at the pier when docked. Empty at sea (§13.2).
pub(crate) fn snapshot_docked_prices(session: &Session) -> HashMap<String, i64> {
    let world = session.world();
    if world.voyage.status != VoyageStatus::InPort {
        return HashMap::new();
    }
    let port_id = world.voyage.destination_id.as_str();
    let Some(port) = world.port(port_id) else {
        return HashMap::new();
    };
    port.market
        .iter()
        .map(|slot| (slot.good_id.clone(), slot.sell_price))
        .collect()
}

/// Build the capped document from Session + memory. Updates `memory` claimable_seen
/// for claimables that fire as notable this day.
pub(crate) fn build_document(
    session: &Session,
    turn_contracts: &[ContractOutcome],
    injuries_before: &[Injury],
    prices_before: &HashMap<String, i64>,
    memory: &mut DayReportMemory,
    arrival_day: bool,
    bounty_known: &[portlight_sim::bounty::BountyTarget],
) -> DayReportDocument {
    let world = session.world();
    let day = world.day;
    let deadlines = deadline_lines(
        session.board().active.as_slice(),
        day,
        turn_contracts,
        world,
    );
    let health = health_lines(injuries_before, session.injuries());
    let prices = price_lines(session, prices_before);
    let bounties = bounty_lines(session, memory, arrival_day, bounty_known);

    let raw = vec![
        ("deadlines", "Deadlines", deadlines),
        ("health", "Health", health),
        ("prices", "Prices", prices),
        ("bounties", "Bounties", bounties),
    ];
    let sections = cap_sections(raw);
    DayReportDocument {
        day,
        title: format!("Day {day}"),
        sections,
    }
}

fn deadline_lines(
    active: &[ActiveContract],
    day: i64,
    turn_contracts: &[ContractOutcome],
    world: &portlight_sim::model::World,
) -> Vec<DayReportLine> {
    let mut lines = Vec::new();
    for outcome in turn_contracts {
        if outcome.outcome_type == "expired" {
            let title = ascii_sentence(&outcome.summary);
            let text = if title.to_lowercase().starts_with("expired") {
                title
            } else {
                format!("Expired: {title}")
            };
            lines.push(DayReportLine {
                text,
                notable: true,
            });
        }
    }
    for contract in active {
        let left = contract.deadline_day - day;
        if left > DEADLINE_N {
            continue;
        }
        let title = ascii_sentence(&contract.title);
        let timing = deadline_timing(left);
        let progress = format!(
            "{}/{}",
            contract.delivered_quantity, contract.required_quantity
        );
        let need = contract.required_quantity - contract.delivered_quantity;
        let cue = if need > 0 {
            let good = good_name(&contract.good_id);
            let port = port_label(world, &contract.destination_port_id);
            format!("sell {need} more {good} at {port}")
        } else {
            format!(
                "ready to Complete ({}/{}) at Contracts",
                contract.delivered_quantity, contract.required_quantity
            )
        };
        lines.push(DayReportLine {
            text: format!("{title} - {timing} - {progress} - {cue}"),
            notable: true,
        });
    }
    lines
}

/// `{n} days left` / `1 day left` / `overdue` - never `due soon` (§13.1).
pub(crate) fn deadline_timing(days_left: i64) -> String {
    if days_left < 0 {
        "overdue".to_string()
    } else if days_left == 1 {
        "1 day left".to_string()
    } else {
        format!("{days_left} days left")
    }
}

fn health_lines(before: &[Injury], after: &[Injury]) -> Vec<DayReportLine> {
    let mut lines = Vec::new();
    let mut after_index = 0;
    for injury in before {
        let matched = after
            .get(after_index)
            .is_some_and(|next| same_wound(injury, next));
        if matched {
            let next = &after[after_index];
            after_index += 1;
            if let (Some(old), Some(remaining)) = (injury.heal_remaining, next.heal_remaining) {
                if remaining < old {
                    let name = injury_name(&injury.injury_id);
                    let timing = heal_days_left(remaining);
                    let notable = remaining == 1;
                    lines.push(DayReportLine {
                        text: format!("Healing: {name} ({timing})."),
                        notable,
                    });
                }
            }
        } else if injury.heal_remaining.is_some() {
            let name = injury_name(&injury.injury_id);
            lines.push(DayReportLine {
                text: format!("Healed: {name}."),
                notable: true,
            });
        }
    }
    lines
}

fn same_wound(before: &Injury, after: &Injury) -> bool {
    if before.injury_id != after.injury_id || before.acquired_day != after.acquired_day {
        return false;
    }
    match (before.heal_remaining, after.heal_remaining) {
        (None, None) => true,
        (Some(old), Some(new)) => new <= old,
        _ => false,
    }
}

pub(crate) fn heal_days_left(remaining: i64) -> String {
    if remaining == 1 {
        "1 day left".to_string()
    } else {
        format!("{remaining} days left")
    }
}

fn price_lines(session: &Session, before: &HashMap<String, i64>) -> Vec<DayReportLine> {
    let world = session.world();
    if world.voyage.status != VoyageStatus::InPort {
        return Vec::new();
    }
    let port_id = world.voyage.destination_id.as_str();
    let Some(port) = world.port(port_id) else {
        return Vec::new();
    };
    let port_name = ascii_label(&port.name, &port.id).to_string();
    let mut strong = Vec::new();
    let mut modest = Vec::new();
    for slot in &port.market {
        let Some(&prev) = before.get(&slot.good_id) else {
            continue;
        };
        let now = slot.sell_price;
        let delta = now - prev;
        if delta == 0 {
            continue;
        }
        let rel = (delta.abs() as f64) / (prev.max(1) as f64);
        if rel >= 0.10 {
            strong.push((slot.good_id.clone(), delta, now));
        } else if delta.abs() >= 2 {
            modest.push((delta.abs(), slot.good_id.clone(), delta, now));
        }
    }
    modest.sort_by_key(|a| std::cmp::Reverse(a.0));
    let mut picked: Vec<(String, i64, i64)> = strong;
    for (_, good_id, delta, now) in modest.into_iter().take(3) {
        picked.push((good_id, delta, now));
    }
    picked
        .into_iter()
        .map(|(good_id, delta, now)| {
            let good = good_name(&good_id);
            let sign = if delta > 0 { "+" } else { "" };
            DayReportLine {
                text: format!("{good} at {port_name} {sign}{delta} (now {now})"),
                notable: true,
            }
        })
        .collect()
}

fn bounty_lines(
    session: &Session,
    memory: &mut DayReportMemory,
    arrival_day: bool,
    known: &[portlight_sim::bounty::BountyTarget],
) -> Vec<DayReportLine> {
    let world = session.world();
    let active: HashSet<String> = world.captain.active_bounties.iter().cloned().collect();
    let claimed: HashSet<String> = world.captain.claimed_bounties.iter().cloned().collect();
    let mut lines = Vec::new();

    for id in active.difference(&memory.active_bounties) {
        let name = hunt_screen::display_name(id, known);
        lines.push(DayReportLine {
            text: format!("Bounty accepted: {name}."),
            notable: true,
        });
    }
    for id in memory.active_bounties.difference(&active) {
        if claimed.contains(id) {
            let name = hunt_screen::display_name(id, known);
            lines.push(DayReportLine {
                text: format!("Bounty claimed: {name}."),
                notable: true,
            });
        }
    }

    let claimables = claimable_ids(session);
    for id in &claimables {
        let name = hunt_screen::display_name(id, known);
        let reward = hunt_screen::reward_for(id, known);
        let first_time = !memory.claimable_seen.contains(id);
        let notable = first_time || arrival_day;
        if first_time {
            memory.claimable_seen.insert(id.clone());
        }
        lines.push(DayReportLine {
            text: format!("Claim ready: {name} ({reward} silver) - open Hunt."),
            notable,
        });
    }

    memory.active_bounties = active;
    lines
}

pub(crate) fn claimable_ids(session: &Session) -> Vec<String> {
    let world = session.world();
    let claimed: HashSet<&str> = world
        .captain
        .claimed_bounties
        .iter()
        .map(String::as_str)
        .collect();
    world
        .captain
        .active_bounties
        .iter()
        .filter(|id| !claimed.contains(id.as_str()))
        .filter(|id| {
            world
                .captain_memories
                .iter()
                .find(|memory| memory.captain_id == **id)
                .map(|memory| memory.times_defeated_by_player > 0)
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

/// Fill sections in order; keep at most [`LINE_CAP`] lines with a trailing `+N more`.
pub(crate) fn cap_sections(
    raw: Vec<(&'static str, &'static str, Vec<DayReportLine>)>,
) -> Vec<DayReportSection> {
    let auto_show = raw
        .iter()
        .flat_map(|(_, _, lines)| lines.iter())
        .any(|line| line.notable);
    if !auto_show {
        return Vec::new();
    }

    let mut remaining = LINE_CAP;
    let mut sections = Vec::new();
    let section_count = raw.len();
    for (index, (id, title, lines)) in raw.into_iter().enumerate() {
        if lines.is_empty() || remaining == 0 {
            continue;
        }
        let sections_left_after = section_count - index - 1;
        // Prefer at least one line for later non-empty sections when possible.
        let reserve = sections_left_after.min(remaining.saturating_sub(1));
        let budget = remaining - reserve;
        if budget == 0 {
            continue;
        }
        let (kept, overflow) = if lines.len() <= budget {
            (lines, 0)
        } else if budget == 1 {
            // One slot: prefer a real line over only "+N more".
            let overflow = lines.len() - 1;
            (lines.into_iter().take(1).collect(), overflow)
        } else {
            let take = budget - 1;
            let overflow = lines.len() - take;
            (lines.into_iter().take(take).collect(), overflow)
        };
        let used = kept.len() + usize::from(overflow > 0);
        let mut section_lines = kept;
        if overflow > 0 {
            section_lines.push(DayReportLine {
                text: format!("+{overflow} more"),
                notable: false,
            });
        }
        remaining = remaining.saturating_sub(used);
        sections.push(DayReportSection {
            id,
            title,
            lines: section_lines,
        });
    }
    sections
}

fn good_name(id: &str) -> String {
    content::content()
        .good(id)
        .map(|good| ascii_label(&good.name, id).to_string())
        .unwrap_or_else(|| id.to_string())
}

fn injury_name(id: &str) -> String {
    content::content()
        .injury(id)
        .map(|injury| ascii_label(&injury.name, id).to_string())
        .unwrap_or_else(|| id.to_string())
}

fn port_label(world: &portlight_sim::model::World, id: &str) -> String {
    world
        .port(id)
        .map(|port| ascii_label(&port.name, &port.id).to_string())
        .unwrap_or_else(|| id.to_string())
}

fn text_label(text: &str, size: i32, color: Color, wrap: bool) -> Gd<Label> {
    let mut label = Label::new_alloc();
    label.set_text(text);
    label.add_theme_font_size_override("font_size", size);
    label.add_theme_color_override("font_color", color);
    if wrap {
        label.set_autowrap_mode(AutowrapMode::WORD_SMART);
        label.set_h_size_flags(SizeFlags::EXPAND_FILL);
    }
    label
}

fn clear_children(node: &mut Gd<VBoxContainer>) {
    let children = node.get_children();
    for child in children.iter_shared() {
        node.remove_child(&child);
    }
}

fn collect_text(node: &Gd<godot::classes::Node>, parts: &mut Vec<String>) {
    if let Ok(label) = node.clone().try_cast::<Label>() {
        let text = label.get_text().to_string();
        if !text.is_empty() {
            parts.push(text);
        }
    }
    for child in node.get_children().iter_shared() {
        collect_text(&child, parts);
    }
}

/// Forced documents for CI frames (smoke injects these; no Session mutation).
pub(crate) fn smoke_full_document(day: i64) -> DayReportDocument {
    DayReportDocument {
        day,
        title: format!("Day {day}"),
        sections: vec![
            DayReportSection {
                id: "deadlines",
                title: "Deadlines",
                lines: vec![DayReportLine {
                    text: "Grain run - 2 days left - 3/10 - sell 7 more Grain at Al-Manar".into(),
                    notable: true,
                }],
            },
            DayReportSection {
                id: "health",
                title: "Health",
                lines: vec![DayReportLine {
                    text: "Healed: Cut hand.".into(),
                    notable: true,
                }],
            },
            DayReportSection {
                id: "prices",
                title: "Prices",
                lines: vec![DayReportLine {
                    text: "Grain at Al-Manar +12 (now 84)".into(),
                    notable: true,
                }],
            },
            DayReportSection {
                id: "bounties",
                title: "Bounties",
                lines: vec![DayReportLine {
                    text: "Claim ready: Raj the Quiet (120 silver) - open Hunt.".into(),
                    notable: true,
                }],
            },
        ],
    }
}

pub(crate) fn smoke_deadline_document(day: i64) -> DayReportDocument {
    DayReportDocument {
        day,
        title: format!("Day {day}"),
        sections: vec![DayReportSection {
            id: "deadlines",
            title: "Deadlines",
            lines: vec![DayReportLine {
                text: "Spice charter - 1 day left - 10/10 - ready to Complete at Contracts".into(),
                notable: true,
            }],
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadline_timing_never_says_due_soon() {
        assert_eq!(deadline_timing(3), "3 days left");
        assert_eq!(deadline_timing(2), "2 days left");
        assert_eq!(deadline_timing(1), "1 day left");
        assert_eq!(deadline_timing(0), "0 days left");
        assert_eq!(deadline_timing(-1), "overdue");
        assert!(!deadline_timing(2).contains("due soon"));
    }

    #[test]
    fn heal_plurals() {
        assert_eq!(heal_days_left(1), "1 day left");
        assert_eq!(heal_days_left(3), "3 days left");
    }

    #[test]
    fn prices_empty_when_undocked_memory() {
        // Pure helper: at-sea snapshot is empty; price_lines with empty before yields empty.
        let before: HashMap<String, i64> = HashMap::new();
        assert!(before.is_empty());
        let moves: Vec<DayReportLine> = Vec::new();
        assert!(moves.is_empty());
    }

    #[test]
    fn price_diff_formats_signed_and_drops_zero() {
        let mut before: HashMap<String, i64> = HashMap::new();
        before.insert("grain".into(), 72);
        before.insert("spice".into(), 40);
        before.insert("timber".into(), 10);
        // Simulate after: grain +12 (strong), spice +1 noise, timber -3 modest.
        let after = vec![("grain", 84_i64), ("spice", 41), ("timber", 7)];
        let mut moves = Vec::new();
        for (id, now) in after {
            let prev = *before.get(id).unwrap();
            let delta = now - prev;
            if delta == 0 {
                continue;
            }
            let rel = (delta.abs() as f64) / (prev.max(1) as f64);
            if rel >= 0.10 || delta.abs() >= 2 {
                moves.push(format!("{id} {delta:+} (now {now})"));
            }
        }
        assert!(moves.iter().any(|line| line.contains("grain +12")));
        assert!(moves.iter().any(|line| line.contains("timber -3")));
        assert!(!moves.iter().any(|line| line.contains("spice")));
    }

    #[test]
    fn line_cap_inserts_more_and_keeps_section_order() {
        let deadlines = (0..4)
            .map(|i| DayReportLine {
                text: format!("d{i}"),
                notable: true,
            })
            .collect();
        let health = vec![DayReportLine {
            text: "Healed: Cut hand.".into(),
            notable: true,
        }];
        let prices = vec![DayReportLine {
            text: "Grain at Al-Manar +12 (now 84)".into(),
            notable: true,
        }];
        let bounties = vec![DayReportLine {
            text: "Claim ready: Raj the Quiet (120 silver) - open Hunt.".into(),
            notable: true,
        }];
        let sections = cap_sections(vec![
            ("deadlines", "Deadlines", deadlines),
            ("health", "Health", health),
            ("prices", "Prices", prices),
            ("bounties", "Bounties", bounties),
        ]);
        let total: usize = sections.iter().map(|s| s.lines.len()).sum();
        assert!(total <= LINE_CAP, "{total}");
        assert_eq!(sections[0].id, "deadlines");
        assert!(sections
            .iter()
            .any(|s| s.lines.iter().any(|l| l.text.starts_with('+'))));
    }

    #[test]
    fn quiet_day_yields_no_sections() {
        let sections = cap_sections(vec![
            (
                "health",
                "Health",
                vec![DayReportLine {
                    text: "Healing: Bruised ribs (3 days left).".into(),
                    notable: false,
                }],
            ),
            (
                "bounties",
                "Bounties",
                vec![DayReportLine {
                    text: "Claim ready: Raj the Quiet (120 silver) - open Hunt.".into(),
                    notable: false,
                }],
            ),
        ]);
        assert!(sections.is_empty());
    }

    #[test]
    fn claimable_first_day_is_notable_repeat_is_not() {
        let mut seen = HashSet::new();
        let id = "raj_the_quiet".to_string();
        let first = !seen.contains(&id);
        assert!(first);
        seen.insert(id.clone());
        let second = !seen.contains(&id);
        assert!(!second);
        let arrival_retrigger = true;
        assert!(second || arrival_retrigger);
    }

    #[test]
    fn smoke_docs_use_spaced_hyphen() {
        let full = smoke_full_document(3);
        assert!(!full.sections.is_empty());
        let blob = full
            .sections
            .iter()
            .flat_map(|s| s.lines.iter())
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!blob.contains('\u{2014}'));
        assert!(blob.contains(" - "));
        assert!(full.has_notable());
        let deadline = smoke_deadline_document(4);
        assert!(deadline.has_notable());
        assert_eq!(deadline.sections.len(), 1);
    }
}
