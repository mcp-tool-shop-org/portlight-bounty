//! Chart HUD contract deadline/progress strip.
//!
//! Presentation only. Reads Session board/world facts; never mutates Session.
//! Reuses [`crate::day_report::deadline_timing`]. Cap 2 + `+N more`. Per-segment
//! urgency colour. Click opens Contracts when docked (handler lives in game.rs).

use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::classes::{Button, HBoxContainer, Label, PanelContainer, StyleBoxFlat};
use godot::prelude::*;
use portlight_sim::model::ActiveContract;
use portlight_sim::session::Session;

use crate::contracts_screen::{ascii_sentence, meta_color, progress_text};
use crate::day_report::deadline_timing;
use crate::departure_check::title_names_port;
use crate::logic::display_or_humanized;

const INK: Color = Color::from_rgb(0.08, 0.11, 0.16);
const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);

pub(crate) const STRIP_X: f32 = 12.0;
pub(crate) const STRIP_Y: f32 = 8.0;
pub(crate) const STRIP_W: f32 = 832.0;
pub(crate) const STRIP_H: f32 = 22.0;
const STRIP_FONT: i32 = 13;
/// Soft fit budget for a single-segment destination hint (size 13 default font).
const DEST_CHAR_BUDGET: usize = 110;
const SHOW_CAP: usize = 2;

#[derive(Clone)]
pub(crate) struct ContractStripNodes {
    pub root: Gd<PanelContainer>,
    pub hit: Gd<Button>,
    pub row: Gd<HBoxContainer>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StripSegment {
    pub text: String,
    /// True when this segment uses the DUE warning tone.
    pub urgent: bool,
    /// True for the trailing `+N more` token (MUTED).
    pub more: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StripDocument {
    pub segments: Vec<StripSegment>,
}

impl StripDocument {
    #[allow(dead_code)]
    pub(crate) fn joined_text(&self) -> String {
        self.segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

pub(crate) fn build_contract_strip() -> ContractStripNodes {
    let mut root = PanelContainer::new_alloc();
    root.set_name("ContractStrip");
    root.set_mouse_filter(MouseFilter::STOP);
    root.set_visible(false);
    let mut style = StyleBoxFlat::new_gd();
    style.set_bg_color(INK);
    style.set_border_color(GOLD);
    style.set_border_width_all(1);
    style.set_content_margin_all(4.0);
    root.add_theme_stylebox_override("panel", &style);
    root.set_custom_minimum_size(Vector2::new(STRIP_W, STRIP_H));

    let mut hit = Button::new_alloc();
    hit.set_name("ContractStripHit");
    hit.set_text("");
    hit.set_flat(true);
    hit.set_focus_mode(godot::classes::control::FocusMode::NONE);
    hit.set_mouse_filter(MouseFilter::STOP);
    hit.set_h_size_flags(SizeFlags::EXPAND_FILL);
    hit.set_v_size_flags(SizeFlags::EXPAND_FILL);
    // Keep the ink panel visible; transparent button chrome.
    let mut empty = StyleBoxFlat::new_gd();
    empty.set_bg_color(Color::from_rgba(0.0, 0.0, 0.0, 0.0));
    empty.set_content_margin_all(0.0);
    for state in ["normal", "hover", "pressed", "focus", "disabled"] {
        hit.add_theme_stylebox_override(state, &empty);
    }

    let mut row = HBoxContainer::new_alloc();
    row.set_name("ContractStripRow");
    row.set_mouse_filter(MouseFilter::IGNORE);
    row.set_h_size_flags(SizeFlags::EXPAND_FILL);
    row.set_v_size_flags(SizeFlags::EXPAND_FILL);
    row.add_theme_constant_override("separation", 6);
    hit.add_child(&row);
    root.add_child(&hit);

    ContractStripNodes { root, hit, row }
}

pub(crate) fn place_strip(root: &mut Gd<PanelContainer>) {
    root.set_anchors_and_offsets_preset(LayoutPreset::TOP_LEFT);
    root.set_position(Vector2::new(STRIP_X, STRIP_Y));
    root.set_size(Vector2::new(STRIP_W, STRIP_H));
}

pub(crate) fn set_visible(nodes: &mut ContractStripNodes, visible: bool) {
    nodes.root.set_visible(visible);
    let filter = if visible {
        MouseFilter::STOP
    } else {
        MouseFilter::IGNORE
    };
    nodes.root.set_mouse_filter(filter);
    nodes.hit.set_mouse_filter(filter);
}

pub(crate) fn apply_document(nodes: &mut ContractStripNodes, doc: &StripDocument) {
    clear_children(&mut nodes.row);
    for (index, segment) in doc.segments.iter().enumerate() {
        if index > 0 {
            let mut sep = Label::new_alloc();
            sep.set_text("|");
            sep.set_mouse_filter(MouseFilter::IGNORE);
            sep.add_theme_font_size_override("font_size", STRIP_FONT);
            sep.add_theme_color_override("font_color", MUTED);
            nodes.row.add_child(&sep);
        }
        let color = if segment.more {
            MUTED
        } else if segment.urgent {
            meta_color(true)
        } else {
            GOLD
        };
        let mut label = Label::new_alloc();
        label.set_text(&segment.text);
        label.set_mouse_filter(MouseFilter::IGNORE);
        label.add_theme_font_size_override("font_size", STRIP_FONT);
        label.add_theme_color_override("font_color", color);
        nodes.row.add_child(&label);
    }
}

pub(crate) fn overlay_visible(nodes: &ContractStripNodes) -> bool {
    nodes.root.is_visible()
}

pub(crate) fn overlay_text(nodes: &ContractStripNodes) -> String {
    let mut parts = Vec::new();
    collect_text(
        &nodes.root.clone().upcast::<godot::classes::Node>(),
        &mut parts,
    );
    parts.join(" | ")
}

/// Build the strip document from the live board. `None` when empty (hide).
pub(crate) fn build_document(session: &Session) -> Option<StripDocument> {
    let day = session.world().day;
    build_document_from_active(session.board().active.as_slice(), day, session.world())
}

pub(crate) fn build_document_from_active(
    active: &[ActiveContract],
    day: i64,
    world: &portlight_sim::model::World,
) -> Option<StripDocument> {
    if active.is_empty() {
        return None;
    }
    let mut ordered: Vec<&ActiveContract> = active.iter().collect();
    ordered.sort_by(|a, b| {
        a.deadline_day
            .cmp(&b.deadline_day)
            .then_with(|| a.title.cmp(&b.title))
    });
    let total = ordered.len();
    let shown = ordered.into_iter().take(SHOW_CAP).collect::<Vec<_>>();
    let mut segments = Vec::new();
    let single = shown.len() == 1 && total == 1;
    for contract in &shown {
        segments.push(segment_for(contract, day, world, single));
    }
    if total > SHOW_CAP {
        let more = total - SHOW_CAP;
        segments.push(StripSegment {
            text: format!("+{more} more"),
            urgent: false,
            more: true,
        });
    }
    // Destination only when exactly one segment (no +N). Soft-fit budget.
    if segments.len() == 1 {
        if let Some(contract) = shown.first() {
            let with_dest = segment_with_destination(contract, day, world);
            if with_dest.text.chars().count() <= DEST_CHAR_BUDGET {
                segments[0] = with_dest;
            }
        }
    }
    Some(StripDocument { segments })
}

fn segment_for(
    contract: &ActiveContract,
    day: i64,
    _world: &portlight_sim::model::World,
    _single: bool,
) -> StripSegment {
    let title = ascii_sentence(&contract.title);
    let progress = progress_text(contract.delivered_quantity, contract.required_quantity);
    let days_left = contract.deadline_day - day;
    let urgent = days_left <= 1;
    // R10: the sale settles a filled contract, so the cue is always timing.
    let cue = deadline_timing(days_left);
    StripSegment {
        text: format!("{title} - {progress} - {cue}"),
        urgent,
        more: false,
    }
}

fn segment_with_destination(
    contract: &ActiveContract,
    day: i64,
    world: &portlight_sim::model::World,
) -> StripSegment {
    let mut segment = segment_for(contract, day, world, true);
    let port = port_label(world, &contract.destination_port_id);
    // Drop `to {Port}` when the title already names the port (the Departure
    // check's elsewhere-line rule).
    if !title_names_port(&ascii_sentence(&contract.title), &port) {
        segment.text = format!("{} - to {port}", segment.text);
    }
    segment
}

fn port_label(world: &portlight_sim::model::World, id: &str) -> String {
    let name = world.port(id).map(|port| port.name.as_str());
    display_or_humanized(name, id)
}

fn clear_children(node: &mut Gd<HBoxContainer>) {
    let children = node.get_children();
    for mut child in children.iter_shared() {
        node.remove_child(&child);
        child.queue_free();
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

/// Fixture contract for pure unit tests (no Session mutation).
#[cfg(test)]
fn test_contract(
    title: &str,
    deadline_day: i64,
    delivered: i64,
    required: i64,
    dest: &str,
) -> ActiveContract {
    ActiveContract {
        offer_id: format!("offer-{title}"),
        template_id: "test".into(),
        family: "test".into(),
        title: title.into(),
        accepted_day: 1,
        deadline_day,
        destination_port_id: dest.into(),
        good_id: "grain".into(),
        required_quantity: required,
        delivered_quantity: delivered,
        reward_silver: 100,
        bonus_reward: 0,
        source_region: None,
        source_port: None,
        inspection_modifier: 1.0,
        status: "accepted".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portlight_sim::session::Session;

    /// #63 GD note 5: the single-contract `- to {Port}` tail drops when the
    /// title already names the port (case-insensitive), else it stays.
    #[test]
    fn single_contract_drops_the_port_the_title_names() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        let world = session.world();
        let named = test_contract(
            "Famine relief: grain to Corsair's Rest",
            5,
            0,
            23,
            "corsairs_rest",
        );
        let doc = build_document_from_active(&[named], 1, world).unwrap();
        assert_eq!(doc.segments.len(), 1);
        assert!(
            !doc.segments[0].text.contains("- to "),
            "{}",
            doc.segments[0].text
        );
        assert_eq!(doc.segments[0].text.matches("Corsair's Rest").count(), 1);
        let plain = test_contract("Grain run", 5, 0, 5, "corsairs_rest");
        let doc = build_document_from_active(&[plain], 1, world).unwrap();
        assert!(
            doc.segments[0].text.ends_with(" - to Corsair's Rest"),
            "{}",
            doc.segments[0].text
        );
    }

    #[test]
    fn empty_active_hides() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        assert!(build_document_from_active(&[], 1, session.world()).is_none());
    }

    #[test]
    fn sorts_nearest_deadline_then_title() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        let world = session.world();
        let active = vec![
            test_contract("Zebra run", 10, 0, 5, "al_manar"),
            test_contract("Apple run", 10, 0, 5, "al_manar"),
            test_contract("Grain run", 5, 0, 5, "al_manar"),
        ];
        let doc = build_document_from_active(&active, 1, world).unwrap();
        assert_eq!(
            doc.segments[0].text.split(" - ").next().unwrap(),
            "Grain run"
        );
        assert_eq!(
            doc.segments[1].text.split(" - ").next().unwrap(),
            "Apple run"
        );
        assert!(doc.segments[2].text.starts_with("+1 more"));
    }

    #[test]
    fn caps_at_two_plus_more() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        let world = session.world();
        let active = vec![
            test_contract("A", 5, 1, 5, "al_manar"),
            test_contract("B", 6, 1, 5, "al_manar"),
            test_contract("C", 7, 1, 5, "al_manar"),
            test_contract("D", 8, 1, 5, "al_manar"),
        ];
        let doc = build_document_from_active(&active, 1, world).unwrap();
        assert_eq!(doc.segments.len(), 3);
        assert_eq!(doc.segments[2].text, "+2 more");
        assert!(doc.segments[2].more);
        assert!(!doc.segments[2].urgent);
    }

    #[test]
    fn copy_reuses_deadline_timing_never_due_soon() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        let world = session.world();
        let cases = [
            (5_i64, "4 days left"),
            (2, "1 day left"),
            (1, "due today"),
            (0, "overdue"),
        ];
        for (deadline, timing) in cases {
            let active = vec![test_contract("Grain run", deadline, 3, 10, "al_manar")];
            let doc = build_document_from_active(&active, 1, world).unwrap();
            let text = doc.joined_text();
            assert!(text.contains(timing), "{text} missing {timing}");
            assert!(!text.contains("due soon"));
            assert!(!text.contains('\u{2014}'));
            assert!(text.contains(" - "));
            assert!(text.contains("3/10"));
        }
    }

    #[test]
    fn deliverable_shows_timing_not_a_complete_cue() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        let world = session.world();
        let active = vec![test_contract("Spice charter", 10, 10, 10, "al_manar")];
        let doc = build_document_from_active(&active, 1, world).unwrap();
        let text = doc.joined_text();
        assert!(!text.contains("Complete"), "{text}");
        assert!(text.contains("10/10"));
        assert!(text.contains(&deadline_timing(9)), "{text}");
    }

    #[test]
    fn urgency_flags_due_today_one_day_overdue() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        let world = session.world();
        let active = vec![
            test_contract("Calm", 10, 0, 5, "al_manar"),
            test_contract("Hot", 2, 0, 5, "porto_novo"),
        ];
        let doc = build_document_from_active(&active, 1, world).unwrap();
        // Hot (deadline 2) sorts first; urgent. Calm is not.
        assert!(doc.segments[0].urgent);
        assert!(!doc.segments[1].urgent);
        assert!(doc.segments[0].text.contains("1 day left"));
    }

    #[test]
    fn destination_only_on_single_segment_when_fits() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        let world = session.world();
        let one = vec![test_contract("Grain run", 5, 3, 10, "al_manar")];
        let doc = build_document_from_active(&one, 1, world).unwrap();
        assert!(
            doc.joined_text().contains(" - to "),
            "single segment should include destination: {}",
            doc.joined_text()
        );

        let two = vec![
            test_contract("Grain run", 5, 3, 10, "al_manar"),
            test_contract("Spice charter", 8, 1, 10, "porto_novo"),
        ];
        let doc2 = build_document_from_active(&two, 1, world).unwrap();
        assert!(
            !doc2.joined_text().contains(" - to "),
            "two segments omit destination: {}",
            doc2.joined_text()
        );

        let three = vec![
            test_contract("A", 5, 0, 5, "al_manar"),
            test_contract("B", 6, 0, 5, "al_manar"),
            test_contract("C", 7, 0, 5, "al_manar"),
        ];
        let doc3 = build_document_from_active(&three, 1, world).unwrap();
        assert!(doc3.joined_text().contains("+1 more"));
        assert!(!doc3.joined_text().contains(" - to "));
    }

    #[test]
    fn session_board_path_builds() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        assert!(build_document(&session).is_none());
        let offers = session.available_contracts();
        let id = offers[0].id.clone();
        session.accept_contract(&id).unwrap();
        let doc = build_document(&session).expect("one active should show");
        assert_eq!(doc.segments.len(), 1);
        assert!(doc.joined_text().contains(" - "));
    }
}
