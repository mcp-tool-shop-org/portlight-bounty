//! Compact Day's report card after a successful Next day.
//!
//! Presentation only. Reads Session facts; never calls mutating Session verbs.
//! Section order: Deadlines → Health → Prices → Bounties. §13 overrides §§1–12.
//!
//! Captain's week: a ride-along footer (Week deltas + one Next hint) below the
//! scroll. It never opens the card, is not a [`DayReportLine`], and does not
//! count toward [`LINE_CAP`].

use std::collections::{HashMap, HashSet, VecDeque};

use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::classes::text_server::{AutowrapMode, OverrunBehavior};
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
/// Scroll minimum height with no footer.
const SCROLL_MIN_H: f32 = 220.0;
/// Captain's week: the scroll may give up at most this much so Close stays inside.
pub(crate) const FOOTER_SCROLL_SHRINK: f32 = 48.0;
/// Footer rows: at most Week + Next.
pub(crate) const FOOTER_MAX_LINES: usize = 2;
const FOOTER_FONT: i32 = 14;
const FOOTER_SEP: i32 = 4;

/// Captain's week rolling window (Godot memory only, never saved).
pub(crate) const WEEK_N: usize = 5;
/// GD presentation threshold for `Next: Stores low`.
pub(crate) const STORES_LOW: i64 = 3;

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct DayReportNodes {
    pub root: Gd<PanelContainer>,
    pub title: Gd<Label>,
    pub scroll: Gd<ScrollContainer>,
    pub body: Gd<VBoxContainer>,
    pub footer: Gd<VBoxContainer>,
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
    /// Captain's week ride-along. Never read by [`Self::has_notable`].
    pub footer: DayReportFooter,
}

/// Captain's week footer rows. Plain strings, not [`DayReportLine`]s: they never
/// carry `notable`, never enter [`cap_sections`], and never open the card.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DayReportFooter {
    pub week: Option<String>,
    pub next: Option<String>,
}

impl DayReportFooter {
    pub(crate) fn lines(&self) -> Vec<&str> {
        [self.week.as_deref(), self.next.as_deref()]
            .into_iter()
            .flatten()
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.week.is_none() && self.next.is_none()
    }
}

/// One post-advance snapshot for Captain's week deltas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WeekSample {
    pub day: i64,
    pub silver: i64,
    pub crew: Option<i64>,
    pub open_wounds: usize,
    pub active_bounties: usize,
    pub claimable: usize,
}

/// Rolling window of at most [`WEEK_N`] samples. Lives in [`DayReportMemory`],
/// so it resets with it on new game / load. Never saved.
#[derive(Clone, Debug, Default)]
pub(crate) struct WeekWindow {
    samples: VecDeque<WeekSample>,
}

impl WeekWindow {
    /// Push after a successful advance. Same `day` replaces the last sample.
    pub(crate) fn push(&mut self, sample: WeekSample) {
        if self
            .samples
            .back()
            .is_some_and(|last| last.day == sample.day)
        {
            self.samples.pop_back();
        }
        self.samples.push_back(sample);
        while self.samples.len() > WEEK_N {
            self.samples.pop_front();
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.samples.len()
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    fn ends(&self) -> Option<(&WeekSample, &WeekSample)> {
        if self.samples.len() < 2 {
            return None;
        }
        Some((self.samples.front()?, self.samples.back()?))
    }
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
    /// Captain's week rolling snapshots (never saved).
    pub week: WeekWindow,
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
    scroll.set_custom_minimum_size(Vector2::new(CARD_W - 36.0, SCROLL_MIN_H));
    scroll.set_horizontal_scroll_mode(godot::classes::scroll_container::ScrollMode::DISABLED);
    let mut body = VBoxContainer::new_alloc();
    body.set_name("DayReportBody");
    body.set_h_size_flags(SizeFlags::EXPAND_FILL);
    body.add_theme_constant_override("separation", 12);
    scroll.add_child(&body);
    column.add_child(&scroll);

    // Captain's week footer: below scroll, above Close. Not a capped section.
    // Footer and Close share one column slot (sep 4), so a shown footer costs
    // its rows + 4 px and never more than FOOTER_SCROLL_SHRINK of scroll.
    let mut foot = VBoxContainer::new_alloc();
    foot.set_name("DayReportFoot");
    foot.set_h_size_flags(SizeFlags::EXPAND_FILL);
    foot.add_theme_constant_override("separation", FOOTER_SEP);
    column.add_child(&foot);

    let mut footer = VBoxContainer::new_alloc();
    footer.set_name("DayReportFooter");
    footer.set_h_size_flags(SizeFlags::EXPAND_FILL);
    footer.add_theme_constant_override("separation", FOOTER_SEP);
    footer.set_visible(false);
    foot.add_child(&footer);

    let mut close = Button::new_alloc();
    close.set_name("CloseDayReport");
    close.set_text("Close");
    style_encounter_button(&mut close);
    close.set_h_size_flags(SizeFlags::SHRINK_BEGIN);
    foot.add_child(&close);

    DayReportNodes {
        root,
        title,
        scroll,
        body,
        footer,
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
    apply_footer(nodes, &doc.footer);
}

fn apply_footer(nodes: &mut DayReportNodes, footer: &DayReportFooter) {
    clear_children(&mut nodes.footer);
    let lines = footer.lines();
    for text in lines.iter().take(FOOTER_MAX_LINES) {
        nodes.footer.add_child(&footer_label(text));
    }
    let shown = !lines.is_empty();
    nodes.footer.set_visible(shown);
    let scroll_h = if shown {
        SCROLL_MIN_H - FOOTER_SCROLL_SHRINK
    } else {
        SCROLL_MIN_H
    };
    nodes
        .scroll
        .set_custom_minimum_size(Vector2::new(CARD_W - 36.0, scroll_h));
}

/// One muted row; clipped so a long Week line never widens or heightens the card.
fn footer_label(text: &str) -> Gd<Label> {
    let mut label = text_label(text, FOOTER_FONT, MUTED, false);
    label.set_h_size_flags(SizeFlags::EXPAND_FILL);
    label.set_clip_text(true);
    label.set_text_overrun_behavior(OverrunBehavior::TRIM_ELLIPSIS);
    label
}

/// Open-card fit: size stays `CARD_W` x `CARD_H` (±1), children fit, and Close
/// sits inside the card. `Some` names what broke.
pub(crate) fn card_fit_error(nodes: &DayReportNodes) -> Option<String> {
    let size = nodes.root.get_size();
    let min = nodes.root.get_combined_minimum_size();
    let size_ok = (size.x - CARD_W).abs() <= 1.0 && (size.y - CARD_H).abs() <= 1.0;
    let min_ok = min.x <= CARD_W + 1.0 && min.y <= CARD_H + 1.0;
    let card = nodes.root.get_global_rect();
    let close = nodes.close.get_global_rect();
    let close_ok = close.size.y <= 0.0
        || (close.position.y >= card.position.y
            && close.position.y + close.size.y <= card.position.y + card.size.y + 1.0);
    let scroll_min = nodes.scroll.get_custom_minimum_size().y;
    let scroll_ok = scroll_min >= SCROLL_MIN_H - FOOTER_SCROLL_SHRINK;
    let footer_rows = nodes.footer.get_child_count() as usize;
    // Shown footer: rows + the 4 px slot gap must fit inside the allowed shrink.
    let footer_h = if nodes.footer.is_visible() {
        nodes.footer.get_combined_minimum_size().y + FOOTER_SEP as f32
    } else {
        0.0
    };
    let footer_ok = footer_rows <= FOOTER_MAX_LINES && footer_h <= FOOTER_SCROLL_SHRINK + 0.5;
    if size_ok && min_ok && close_ok && scroll_ok && footer_ok {
        return None;
    }
    Some(format!(
        "card {w}x{h} min {mw}x{mh} (want {CARD_W}x{CARD_H}); close y {cy}+{ch} in card y {ky}+{kh}; scroll min {scroll_min}; footer rows {footer_rows} cost {footer_h}",
        w = size.x,
        h = size.y,
        mw = min.x,
        mh = min.y,
        cy = close.position.y,
        ch = close.size.y,
        ky = card.position.y,
        kh = card.size.y,
    ))
}

/// Footer text only (for smoke assertions).
pub(crate) fn footer_text(nodes: &DayReportNodes) -> String {
    let mut parts = Vec::new();
    collect_text(
        &nodes.footer.clone().upcast::<godot::classes::Node>(),
        &mut parts,
    );
    parts.join("\n")
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
        footer: DayReportFooter::default(),
    }
}

/// Post-advance Captain's week snapshot. Reads only; no Session mutation.
pub(crate) fn week_sample(session: &Session) -> WeekSample {
    let world = session.world();
    WeekSample {
        day: world.day,
        silver: world.captain.silver,
        crew: world.captain.ship.as_ref().map(|ship| ship.crew),
        open_wounds: session
            .injuries()
            .iter()
            .filter(|injury| injury.heal_remaining.is_some())
            .count(),
        active_bounties: world.captain.active_bounties.len(),
        claimable: claimable_ids(session).len(),
    }
}

/// Footer for a card that is already open for a notable document.
pub(crate) fn build_footer(session: &Session, window: &WeekWindow) -> DayReportFooter {
    DayReportFooter {
        week: week_line(window),
        next: next_line(&next_facts(session)),
    }
}

/// `Week: ...` from the oldest to the newest sample. `None` while warming up
/// (< 2 samples) or when every clause is zero. Zero clauses are omitted.
pub(crate) fn week_line(window: &WeekWindow) -> Option<String> {
    let (first, last) = window.ends()?;
    let mut clauses = Vec::new();
    let silver = last.silver - first.silver;
    if silver != 0 {
        clauses.push(format!("{silver:+} silver"));
    }
    if let (Some(old), Some(new)) = (first.crew, last.crew) {
        if old != new {
            clauses.push(format!("crew {old} to {new}"));
        }
    }
    let wounds = last.open_wounds as i64 - first.open_wounds as i64;
    if wounds < 0 {
        clauses.push(format!("{} healed", -wounds));
    } else if wounds == 1 {
        clauses.push("1 wound open".to_string());
    } else if wounds > 1 {
        clauses.push(format!("{wounds} wounds open"));
    }
    let bounties = last.active_bounties as i64 - first.active_bounties as i64;
    if bounties != 0 {
        clauses.push(format!("bounty {bounties:+}"));
    }
    let claimable = last.claimable as i64 - first.claimable as i64;
    if claimable > 0 {
        clauses.push(format!("{claimable} claimable"));
    }
    if clauses.is_empty() {
        return None;
    }
    Some(format!("Week: {}", clauses.join(" - ")))
}

/// Session facts for the Next ladder (pure, so the order is unit-testable).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct NextFacts {
    /// `deadline_day - day` for each active contract.
    pub contract_days_left: Vec<i64>,
    pub claimable: bool,
    /// Any captain injury with `heal_remaining.is_some()`.
    pub captain_wounded: bool,
    pub provisions: i64,
}

pub(crate) fn next_facts(session: &Session) -> NextFacts {
    let world = session.world();
    let day = world.day;
    NextFacts {
        contract_days_left: session
            .board()
            .active
            .iter()
            .map(|contract| contract.deadline_day - day)
            .collect(),
        claimable: !claimable_ids(session).is_empty(),
        captain_wounded: session
            .injuries()
            .iter()
            .any(|injury| injury.heal_remaining.is_some()),
        provisions: world.captain.provisions,
    }
}

/// First match: overdue, due today, approaching, Claim ready, wounded, Stores low.
pub(crate) fn next_line(facts: &NextFacts) -> Option<String> {
    let lefts = &facts.contract_days_left;
    if lefts.iter().any(|&left| left < 0) {
        return Some("Next: Contract overdue - open Contracts.".to_string());
    }
    if lefts.contains(&0) {
        return Some("Next: Contract due today - open Contracts.".to_string());
    }
    if let Some(left) = lefts
        .iter()
        .copied()
        .filter(|left| (1..=DEADLINE_N).contains(left))
        .min()
    {
        let timing = deadline_timing(left);
        return Some(format!("Next: Contract {timing} - open Contracts."));
    }
    if facts.claimable {
        return Some("Next: Claim ready - open Hunt.".to_string());
    }
    if facts.captain_wounded {
        return Some("Next: Captain wounded - heal in port.".to_string());
    }
    if facts.provisions <= STORES_LOW {
        return Some("Next: Stores low - restock in port.".to_string());
    }
    None
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
            "ready to Complete at Contracts".to_string()
        };
        lines.push(DayReportLine {
            text: format!("{title} - {timing} - {progress} - {cue}"),
            notable: true,
        });
    }
    lines
}

/// `{n} days left` / `1 day left` / `due today` / `overdue` - never `due soon` (§13.1).
pub(crate) fn deadline_timing(days_left: i64) -> String {
    if days_left < 0 {
        "overdue".to_string()
    } else if days_left == 0 {
        "due today".to_string()
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

pub(crate) fn price_lines(session: &Session, before: &HashMap<String, i64>) -> Vec<DayReportLine> {
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

pub(crate) fn bounty_lines(
    session: &Session,
    memory: &mut DayReportMemory,
    arrival_day: bool,
    known: &[portlight_sim::bounty::BountyTarget],
) -> Vec<DayReportLine> {
    let world = session.world();
    let active: HashSet<String> = world.captain.active_bounties.iter().cloned().collect();
    let claimed: HashSet<String> = world.captain.claimed_bounties.iter().cloned().collect();
    let mut lines = Vec::new();

    // C1: Claim ready first (only actionable), then claimed, then accepted.
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
    for id in memory.active_bounties.difference(&active) {
        if claimed.contains(id) {
            let name = hunt_screen::display_name(id, known);
            lines.push(DayReportLine {
                text: format!("Bounty claimed: {name}."),
                notable: true,
            });
        }
    }
    for id in active.difference(&memory.active_bounties) {
        let name = hunt_screen::display_name(id, known);
        lines.push(DayReportLine {
            text: format!("Bounty accepted: {name}."),
            notable: true,
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

    // C1: reserve only for later sections that actually have lines.
    let non_empty: Vec<bool> = raw.iter().map(|(_, _, lines)| !lines.is_empty()).collect();

    let mut remaining = LINE_CAP;
    let mut sections = Vec::new();
    for (index, (id, title, lines)) in raw.into_iter().enumerate() {
        if lines.is_empty() || remaining == 0 {
            continue;
        }
        let sections_left_after = non_empty[index + 1..].iter().filter(|&&has| has).count();
        // Prefer at least one line for later non-empty sections when possible.
        let reserve = sections_left_after.min(remaining.saturating_sub(1));
        let budget = remaining - reserve;
        if budget == 0 {
            continue;
        }
        let (kept, overflow) = if lines.len() <= budget {
            (lines, 0)
        } else if budget == 1 {
            // One slot: keep a real line. Do not also emit "+N more" (would exceed budget / LINE_CAP).
            (lines.into_iter().take(1).collect(), 0)
        } else {
            let take = budget - 1;
            let overflow = lines.len() - take;
            (lines.into_iter().take(take).collect(), overflow)
        };
        let used = kept.len() + usize::from(overflow > 0);
        debug_assert!(used <= budget);
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
        // Forced footer through the real formatters: +340 silver, crew 12 to 14,
        // 1 healed; the 2-days-left deadline wins the Next ladder over Claim ready.
        footer: DayReportFooter {
            week: week_line(&smoke_week_window(
                day,
                &[(340, Some(12), 1, 1, 0), (680, Some(14), 0, 1, 0)],
            )),
            next: next_line(&NextFacts {
                contract_days_left: vec![2],
                claimable: true,
                captain_wounded: false,
                provisions: 20,
            }),
        },
    }
}

/// Window ending on `day` from `(silver, crew, open_wounds, active, claimable)`.
fn smoke_week_window(day: i64, rows: &[(i64, Option<i64>, usize, usize, usize)]) -> WeekWindow {
    let mut window = WeekWindow::default();
    let start = day - rows.len() as i64 + 1;
    for (offset, &(silver, crew, open_wounds, active_bounties, claimable)) in
        rows.iter().enumerate()
    {
        window.push(WeekSample {
            day: start + offset as i64,
            silver,
            crew,
            open_wounds,
            active_bounties,
            claimable,
        });
    }
    window
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
        // Warm-up (one sample): Week omitted, Next only.
        footer: DayReportFooter {
            week: week_line(&smoke_week_window(day, &[(540, Some(3), 0, 0, 0)])),
            next: next_line(&NextFacts {
                contract_days_left: vec![1],
                claimable: false,
                captain_wounded: false,
                provisions: 20,
            }),
        },
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
        assert_eq!(deadline_timing(0), "due today");
        assert_eq!(deadline_timing(-1), "overdue");
        assert!(!deadline_timing(0).contains("due soon"));
        assert!(!deadline_timing(2).contains("due soon"));
    }

    #[test]
    fn heal_plurals() {
        assert_eq!(heal_days_left(1), "1 day left");
        assert_eq!(heal_days_left(3), "3 days left");
    }

    #[test]
    fn prices_empty_when_undocked() {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        assert_eq!(session.world().voyage.status, VoyageStatus::InPort);
        let docked = snapshot_docked_prices(&session);
        assert!(!docked.is_empty(), "docked pier should expose sell prices");
        session.depart("al_manar").unwrap();
        assert_eq!(session.world().voyage.status, VoyageStatus::AtSea);
        assert!(
            snapshot_docked_prices(&session).is_empty(),
            "at-sea snapshot must be empty (§13.2)"
        );
        assert!(
            price_lines(&session, &docked).is_empty(),
            "price_lines at sea must be empty even with prior memory"
        );
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

    /// Build a session with an active bounty that is already defeated (claimable).
    fn session_with_claimable(id: &str) -> Session {
        session_with_defeated(&[id])
    }

    /// Accept each bounty and mark it defeated once (claimable).
    /// Uses save/load so we do not need a Session world_mut API (Godot-only crate).
    fn session_with_defeated(ids: &[&str]) -> Session {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        for id in ids {
            session.accept_bounty(id).unwrap();
        }
        let dir = std::env::temp_dir().join(format!(
            "portlight-day-report-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        session.save(&dir, "claim").unwrap();
        let path = dir.join("saves").join("claim.json");
        let mut raw = std::fs::read_to_string(&path).unwrap();
        let entries = ids
            .iter()
            .map(|id| defeated_memory_json(id))
            .collect::<Vec<_>>()
            .join(",");
        let memory = format!(
            r#"
    "captain_memories": {{{entries}
    }},"#
        );
        let marker = "\"pirate_state\": {";
        let Some(idx) = raw.find(marker) else {
            panic!("save missing pirate_state object");
        };
        let insert_at = idx + marker.len();
        raw.insert_str(insert_at, &memory);
        std::fs::write(&path, &raw).unwrap();
        let loaded = Session::load(&dir, "claim")
            .unwrap()
            .expect("reloaded claimable session");
        let _ = std::fs::remove_dir_all(&dir);
        loaded
    }

    fn defeated_memory_json(id: &str) -> String {
        format!(
            r#"
      "{id}": {{
        "captain_id": "{id}",
        "relationship": {{
          "respect": 0,
          "fear": 0,
          "grudge": 0,
          "familiarity": 0
        }},
        "encounters": [],
        "last_seen_day": 1,
        "last_seen_region": "",
        "times_spared": 0,
        "times_defeated_by_player": 1,
        "times_defeated_player": 0,
        "player_sank_their_ship": false
      }}"#
        )
    }

    #[test]
    fn claimable_first_day_is_notable_repeat_is_not() {
        let session = session_with_claimable("raj_the_quiet");
        assert!(
            claimable_ids(&session)
                .iter()
                .any(|id| id == "raj_the_quiet"),
            "production claimable_ids should see defeated active bounty"
        );
        let known: Vec<portlight_sim::bounty::BountyTarget> = Vec::new();
        let mut memory = DayReportMemory::default();

        let first = bounty_lines(&session, &mut memory, false, &known);
        let claim = first
            .iter()
            .find(|line| line.text.contains("Claim ready:"))
            .expect("first day should list claimable via bounty_lines");
        assert!(claim.notable, "first claimable day must be notable");
        assert!(memory.claimable_seen.contains("raj_the_quiet"));

        let second = bounty_lines(&session, &mut memory, false, &known);
        let claim2 = second
            .iter()
            .find(|line| line.text.contains("Claim ready:"))
            .expect("ride-along still lists claimable");
        assert!(!claim2.notable, "repeat docked day is not notable");

        let arrival = bounty_lines(&session, &mut memory, true, &known);
        let claim3 = arrival
            .iter()
            .find(|line| line.text.contains("Claim ready:"))
            .expect("arrival day still lists claimable");
        assert!(claim3.notable, "arrival re-triggers notable");

        // Also exercise build_document on the production path.
        let mut memory2 = DayReportMemory::default();
        let doc = build_document(
            &session,
            &[],
            &[],
            &HashMap::new(),
            &mut memory2,
            false,
            &known,
        );
        assert!(doc.has_notable());
        assert!(doc
            .sections
            .iter()
            .flat_map(|s| s.lines.iter())
            .any(|l| l.text.contains("Claim ready:")));
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

    fn line(text: &str, notable: bool) -> DayReportLine {
        DayReportLine {
            text: text.into(),
            notable,
        }
    }

    fn rendered_line_count(sections: &[DayReportSection]) -> usize {
        sections.iter().map(|s| s.lines.len()).sum()
    }

    fn more_row_count(sections: &[DayReportSection]) -> usize {
        sections
            .iter()
            .flat_map(|s| s.lines.iter())
            .filter(|l| l.text.starts_with('+') && l.text.ends_with(" more"))
            .count()
    }

    /// C1: Claim ready / claimed / accepted push order inside Bounties.
    #[test]
    fn bounty_lines_order_claim_ready_then_claimed_then_accepted() {
        // Live Session: raj stays claimable, scarlet_ana is claimed this day,
        // the_butcher is newly accepted. Memory holds last Next day's actives.
        let mut session = session_with_defeated(&["raj_the_quiet", "scarlet_ana"]);
        session.claim_bounty("scarlet_ana").unwrap();
        session.accept_bounty("the_butcher").unwrap();
        let known: Vec<portlight_sim::bounty::BountyTarget> = Vec::new();
        let mut memory = DayReportMemory::default();
        memory.active_bounties.insert("raj_the_quiet".into());
        memory.active_bounties.insert("scarlet_ana".into());
        let live = bounty_lines(&session, &mut memory, false, &known);
        let kinds: Vec<&str> = live
            .iter()
            .map(|l| {
                ["Claim ready:", "Bounty claimed:", "Bounty accepted:"]
                    .into_iter()
                    .find(|prefix| l.text.starts_with(prefix))
                    .unwrap_or(l.text.as_str())
            })
            .collect();
        assert_eq!(
            kinds,
            vec!["Claim ready:", "Bounty claimed:", "Bounty accepted:"],
            "bounty_lines order: {live:?}"
        );

        // A fresh accept without a defeat is accepted only, never claimable.
        let mut fresh = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        fresh.accept_bounty("the_butcher").unwrap();
        let mut fresh_memory = DayReportMemory::default();
        let accepted_only = bounty_lines(&fresh, &mut fresh_memory, false, &known);
        assert!(
            accepted_only
                .iter()
                .any(|l| l.text.starts_with("Bounty accepted:")),
            "new accept yields Bounty accepted"
        );
        assert!(
            !accepted_only
                .iter()
                .any(|l| l.text.starts_with("Claim ready:")),
            "fresh accept without defeat is not claimable"
        );

        // Cap guarantee with mixed ordered bounty lines (Claim ready first).
        let ordered = vec![
            line("Claim ready: Raj the Quiet (120 silver) - open Hunt.", true),
            line("Bounty claimed: Old Salt.", true),
            line("Bounty accepted: Red Sails.", true),
        ];
        let sections = cap_sections(vec![
            (
                "deadlines",
                "Deadlines",
                (0..3).map(|i| line(&format!("d{i}"), true)).collect(),
            ),
            ("health", "Health", vec![line("Healed: Cut hand.", true)]),
            (
                "prices",
                "Prices",
                vec![
                    line("Grain at Al-Manar +12 (now 84)", true),
                    line("Spice at Al-Manar -3 (now 37)", true),
                ],
            ),
            ("bounties", "Bounties", ordered),
        ]);
        let bounties = sections
            .iter()
            .find(|s| s.id == "bounties")
            .expect("bounties section must survive the cap");
        assert!(
            !bounties.lines.is_empty(),
            "guarantee: at least one Bounties line visible when any exist"
        );
        assert!(
            bounties.lines[0].text.starts_with("Claim ready:"),
            "first visible Bounties line must be Claim ready, got {:?}",
            bounties.lines[0].text
        );
        let total = rendered_line_count(&sections);
        assert!(total <= LINE_CAP, "hard cap including +N more: {total}");
    }

    /// C1 worst case: 3 deadlines + health + several prices + 2 bounty lines.
    #[test]
    fn notable_gate_worst_case_keeps_claim_ready_visible() {
        let deadlines = (0..3)
            .map(|i| line(&format!("deadline {i}"), true))
            .collect();
        let health = vec![line("Healed: Cut hand.", true)];
        let prices = (0..4)
            .map(|i| line(&format!("good{i} at Port +{i} (now {i})"), true))
            .collect();
        let bounties = vec![
            line("Claim ready: Raj the Quiet (120 silver) - open Hunt.", true),
            line("Bounty accepted: Red Sails.", true),
        ];
        let sections = cap_sections(vec![
            ("deadlines", "Deadlines", deadlines),
            ("health", "Health", health),
            ("prices", "Prices", prices),
            ("bounties", "Bounties", bounties),
        ]);
        let total = rendered_line_count(&sections);
        assert!(total <= LINE_CAP, "hard cap: {total} > {LINE_CAP}");
        // Every budget==1 section keeps its one real line and adds no +N more.
        assert_eq!(more_row_count(&sections), 0, "{sections:?}");
        assert!(
            sections
                .iter()
                .any(|s| s.id == "bounties" && !s.lines.is_empty()),
            "guarantee: at least one Bounties line visible"
        );
        let bounty = sections.iter().find(|s| s.id == "bounties").unwrap();
        assert!(
            bounty.lines[0].text.starts_with("Claim ready:"),
            "first visible Bounties line must be Claim ready"
        );
    }

    /// C1: empty later sections must not steal reserve slots.
    #[test]
    fn reserve_skips_empty_later_sections() {
        // Deadlines (4) + empty health + empty prices + bounties (1).
        // Old code reserved 3 slots for the three later section slots (incl. empty),
        // leaving budget=3 for deadlines → 2 lines + "+2 more", then empty, empty, 1 bounty.
        // With non-empty-only reserve, reserve=1 → deadlines get budget=5.
        let deadlines = (0..4).map(|i| line(&format!("d{i}"), true)).collect();
        let bounties = vec![line(
            "Claim ready: Raj the Quiet (120 silver) - open Hunt.",
            true,
        )];
        let sections = cap_sections(vec![
            ("deadlines", "Deadlines", deadlines),
            ("health", "Health", Vec::new()),
            ("prices", "Prices", Vec::new()),
            ("bounties", "Bounties", bounties),
        ]);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].id, "deadlines");
        assert_eq!(sections[1].id, "bounties");
        // Budget 5 with reserve 1: all 4 deadlines fit with no "+N more"
        // (the old layout kept only 2 real lines + "+2 more").
        let deadline_real = sections[0]
            .lines
            .iter()
            .filter(|l| !l.text.starts_with('+'))
            .count();
        assert_eq!(
            deadline_real, 4,
            "non-empty reserve should fit all 4 deadlines, got {deadline_real}"
        );
        assert_eq!(more_row_count(&sections), 0);
        assert!(sections[1].lines[0].text.starts_with("Claim ready:"));
        assert!(rendered_line_count(&sections) <= LINE_CAP);
    }

    /// C1: budget==1 must not emit "+N more" on top of the kept line (hard cap).
    #[test]
    fn budget_one_does_not_exceed_line_cap_with_more_row() {
        // Deadlines overflow with reserve=1 → budget=5 → 4 real + "+N more" uses 5.
        // Bounties then see remaining=1 with 3 lines → budget==1 path.
        let deadlines = (0..10).map(|i| line(&format!("d{i}"), true)).collect();
        let bounties = vec![
            line("Claim ready: A (1 silver) - open Hunt.", true),
            line("Bounty claimed: B.", true),
            line("Bounty accepted: C.", true),
        ];
        let sections = cap_sections(vec![
            ("deadlines", "Deadlines", deadlines),
            ("health", "Health", Vec::new()),
            ("prices", "Prices", Vec::new()),
            ("bounties", "Bounties", bounties),
        ]);
        let total = rendered_line_count(&sections);
        assert!(
            total <= LINE_CAP,
            "budget==1 path must not push total over LINE_CAP (got {total}); more_rows={}",
            more_row_count(&sections)
        );
        let bounty = sections
            .iter()
            .find(|s| s.id == "bounties")
            .expect("bounties");
        assert_eq!(bounty.lines.len(), 1, "budget 1 keeps one line, no +N more");
        assert!(bounty.lines[0].text.starts_with("Claim ready:"));
        assert!(!bounty.lines[0].text.starts_with('+'));
        // Deadlines must have consumed a +N more row toward the cap.
        assert!(more_row_count(&sections) >= 1);
    }

    /// C1 hard cap: every rendered row including each +N more counts toward LINE_CAP.
    #[test]
    fn hard_cap_counts_more_rows() {
        let deadlines = (0..5).map(|i| line(&format!("d{i}"), true)).collect();
        let health = (0..3).map(|i| line(&format!("h{i}"), true)).collect();
        let prices = (0..3).map(|i| line(&format!("p{i}"), true)).collect();
        let bounties = (0..3)
            .map(|i| line(&format!("Claim ready: b{i} (1 silver) - open Hunt."), true))
            .collect();
        let sections = cap_sections(vec![
            ("deadlines", "Deadlines", deadlines),
            ("health", "Health", health),
            ("prices", "Prices", prices),
            ("bounties", "Bounties", bounties),
        ]);
        let total = rendered_line_count(&sections);
        let mores = more_row_count(&sections);
        assert!(total <= LINE_CAP, "total={total} more_rows={mores}");
        assert!(
            sections.iter().any(|s| s.id == "bounties"),
            "bounties must remain when notable bounty lines exist"
        );
    }

    fn sample(
        day: i64,
        silver: i64,
        crew: Option<i64>,
        wounds: usize,
        active: usize,
    ) -> WeekSample {
        WeekSample {
            day,
            silver,
            crew,
            open_wounds: wounds,
            active_bounties: active,
            claimable: 0,
        }
    }

    fn window(samples: &[WeekSample]) -> WeekWindow {
        let mut window = WeekWindow::default();
        for s in samples {
            window.push(*s);
        }
        window
    }

    /// Captain's week: signed snapshot deltas, zero clauses omitted, ASCII ` - `.
    #[test]
    fn week_line_formats_signed_deltas_and_drops_zero_clauses() {
        let up = window(&[
            sample(1, 200, Some(12), 1, 0),
            sample(2, 540, Some(14), 0, 0),
        ]);
        assert_eq!(
            week_line(&up).as_deref(),
            Some("Week: +340 silver - crew 12 to 14 - 1 healed")
        );
        let down = window(&[
            sample(1, 500, Some(14), 0, 1),
            sample(3, 420, Some(11), 0, 1),
        ]);
        assert_eq!(
            week_line(&down).as_deref(),
            Some("Week: -80 silver - crew 14 to 11")
        );
        // Zero silver clause dropped, never +0.
        let wounds = window(&[sample(1, 300, Some(3), 0, 0), sample(2, 300, Some(3), 2, 1)]);
        assert_eq!(
            week_line(&wounds).as_deref(),
            Some("Week: 2 wounds open - bounty +1")
        );
        let one = window(&[sample(1, 300, Some(3), 0, 2), sample(2, 300, Some(3), 1, 1)]);
        assert_eq!(
            week_line(&one).as_deref(),
            Some("Week: 1 wound open - bounty -1")
        );
        let mut claim = window(&[sample(1, 300, Some(3), 0, 1)]);
        claim.push(WeekSample {
            claimable: 1,
            ..sample(2, 300, Some(3), 0, 1)
        });
        assert_eq!(week_line(&claim).as_deref(), Some("Week: 1 claimable"));
        for text in [&up, &down, &wounds, &one, &claim]
            .into_iter()
            .filter_map(week_line)
        {
            assert!(text.is_ascii(), "{text}");
            assert!(!text.contains("+0") && !text.contains("-0"), "{text}");
            assert!(!text.contains("now "), "deltas only: {text}");
        }
    }

    #[test]
    fn week_line_omitted_while_warming_up_or_flat() {
        assert_eq!(week_line(&WeekWindow::default()), None);
        let single = window(&[sample(4, 900, Some(5), 0, 0)]);
        assert_eq!(week_line(&single), None, "one sample is warm-up");
        let flat = window(&[sample(4, 900, Some(5), 1, 1), sample(5, 900, Some(5), 1, 1)]);
        assert_eq!(week_line(&flat), None, "all-zero clauses omit Week");
    }

    #[test]
    fn week_window_caps_at_week_n_and_replaces_same_day() {
        let mut w = WeekWindow::default();
        assert!(w.is_empty());
        for day in 1..=8 {
            w.push(sample(day, day * 10, Some(3), 0, 0));
        }
        assert_eq!(w.len(), WEEK_N);
        // Oldest kept is day 4 (8 - WEEK_N + 1): delta 80 - 40.
        assert_eq!(week_line(&w).as_deref(), Some("Week: +40 silver"));
        w.push(sample(8, 100, Some(3), 0, 0));
        assert_eq!(w.len(), WEEK_N, "same day replaces, does not grow");
        assert_eq!(week_line(&w).as_deref(), Some("Week: +60 silver"));
        // DayReportMemory reset clears the window (new game / load).
        let mut memory = DayReportMemory {
            week: w,
            ..DayReportMemory::default()
        };
        assert_eq!(memory.week.len(), WEEK_N);
        memory = DayReportMemory::default();
        assert!(memory.week.is_empty());
    }

    #[test]
    fn next_ladder_first_match_order() {
        let base = NextFacts {
            contract_days_left: vec![],
            claimable: false,
            captain_wounded: false,
            provisions: 20,
        };
        assert_eq!(next_line(&base), None, "empty ladder omits Next");
        let all = NextFacts {
            contract_days_left: vec![9, 2, 0, -1],
            claimable: true,
            captain_wounded: true,
            provisions: 1,
        };
        assert_eq!(
            next_line(&all).as_deref(),
            Some("Next: Contract overdue - open Contracts.")
        );
        let today = NextFacts {
            contract_days_left: vec![3, 0],
            ..all.clone()
        };
        assert_eq!(
            next_line(&today).as_deref(),
            Some("Next: Contract due today - open Contracts.")
        );
        let soon = NextFacts {
            contract_days_left: vec![5, 3, 1],
            ..all.clone()
        };
        assert_eq!(
            next_line(&soon).as_deref(),
            Some("Next: Contract 1 day left - open Contracts.")
        );
        let three = NextFacts {
            contract_days_left: vec![3],
            ..all.clone()
        };
        assert_eq!(
            next_line(&three).as_deref(),
            Some("Next: Contract 3 days left - open Contracts.")
        );
        let far = NextFacts {
            contract_days_left: vec![DEADLINE_N + 1],
            ..all.clone()
        };
        assert_eq!(
            next_line(&far).as_deref(),
            Some("Next: Claim ready - open Hunt.")
        );
        let wounded = NextFacts {
            claimable: false,
            ..far.clone()
        };
        assert_eq!(
            next_line(&wounded).as_deref(),
            Some("Next: Captain wounded - heal in port.")
        );
        let stores = NextFacts {
            captain_wounded: false,
            provisions: STORES_LOW,
            ..wounded.clone()
        };
        assert_eq!(
            next_line(&stores).as_deref(),
            Some("Next: Stores low - restock in port.")
        );
        let stocked = NextFacts {
            provisions: STORES_LOW + 1,
            ..stores.clone()
        };
        assert_eq!(next_line(&stocked), None);
        for facts in [&all, &today, &soon, &three, &far, &wounded, &stores] {
            let text = next_line(facts).unwrap();
            assert!(text.is_ascii() && !text.contains("due soon"), "{text}");
            assert!(text.contains(" - "), "{text}");
        }
    }

    /// Footer is ride-along: it never flips `has_notable` and never enters the cap.
    #[test]
    fn footer_never_flips_notable_or_counts_toward_cap() {
        let footer = DayReportFooter {
            week: Some("Week: +340 silver".into()),
            next: Some("Next: Claim ready - open Hunt.".into()),
        };
        let quiet = DayReportDocument {
            day: 5,
            title: "Day 5".into(),
            sections: cap_sections(vec![(
                "health",
                "Health",
                vec![line("Healing: Bruised ribs (3 days left).", false)],
            )]),
            footer: footer.clone(),
        };
        assert!(quiet.sections.is_empty());
        assert!(!quiet.has_notable(), "footer must not open a quiet day");

        let sections = cap_sections(vec![
            (
                "deadlines",
                "Deadlines",
                (0..5).map(|i| line(&format!("d{i}"), true)).collect(),
            ),
            ("health", "Health", vec![line("Healed: Cut hand.", true)]),
            (
                "prices",
                "Prices",
                (0..3).map(|i| line(&format!("p{i}"), true)).collect(),
            ),
            (
                "bounties",
                "Bounties",
                vec![line("Claim ready: A (1 silver) - open Hunt.", true)],
            ),
        ]);
        let doc = DayReportDocument {
            day: 6,
            title: "Day 6".into(),
            sections,
            footer,
        };
        assert!(doc.has_notable());
        assert!(rendered_line_count(&doc.sections) <= LINE_CAP);
        assert_eq!(doc.footer.lines().len(), FOOTER_MAX_LINES);
        assert!(doc
            .sections
            .iter()
            .flat_map(|s| s.lines.iter())
            .all(|l| !l.text.starts_with("Week:") && !l.text.starts_with("Next:")));
    }

    #[test]
    fn week_sample_reads_session_snapshot() {
        let session = session_with_claimable("raj_the_quiet");
        let snap = week_sample(&session);
        let world = session.world();
        assert_eq!(snap.day, world.day);
        assert_eq!(snap.silver, world.captain.silver);
        assert_eq!(snap.crew, world.captain.ship.as_ref().map(|s| s.crew));
        assert_eq!(snap.active_bounties, 1);
        assert_eq!(snap.claimable, 1);
        assert_eq!(snap.open_wounds, 0);
        let facts = next_facts(&session);
        assert!(facts.claimable);
        assert_eq!(
            next_line(&facts).as_deref(),
            Some("Next: Claim ready - open Hunt.")
        );
        let footer = build_footer(&session, &WeekWindow::default());
        assert_eq!(footer.week, None, "no window yet");
        assert!(!footer.is_empty());
    }

    #[test]
    fn smoke_footers_follow_formatters() {
        let full = smoke_full_document(3);
        assert_eq!(
            full.footer.week.as_deref(),
            Some("Week: +340 silver - crew 12 to 14 - 1 healed")
        );
        assert_eq!(
            full.footer.next.as_deref(),
            Some("Next: Contract 2 days left - open Contracts.")
        );
        let deadline = smoke_deadline_document(4);
        assert_eq!(deadline.footer.week, None, "warm-up omits Week");
        assert_eq!(
            deadline.footer.next.as_deref(),
            Some("Next: Contract 1 day left - open Contracts.")
        );
    }
}
