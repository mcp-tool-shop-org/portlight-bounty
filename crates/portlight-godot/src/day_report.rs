//! Compact Day's report card after a successful Next day.
//!
//! Presentation only. Reads Session facts; never calls mutating Session verbs.
//! Section order: Deadlines → Health → Prices → Bounties. §13 overrides §§1–12.
//! Arrival day (sailed→InPort): Arrival → Deadlines → Health → Bounties
//! (no Prices; visit movers live under Arrival). Title `Arrived - {Port}`.
//! Movers yield first: they drop (lowest-ranked first) until the arrival row
//! budget fits, so Claim ready stays above the fold.
//!
//! Contract fail: an expiry on this advance is a `Contract expired: ...` line
//! pinned at the top of Deadlines in DUE ([`failure_lines`]). Pinned lines
//! never trim (cap, arrival yield, footer yield) and always open the card.
//!
//! Captain's week: a ride-along footer (Week deltas + one Next hint) below the
//! scroll. It never opens the card, is not a [`DayReportLine`], and does not
//! count toward [`LINE_CAP`]. The footer yields when the body would scroll at
//! the shrunk height (all-or-nothing; Next never appears without Week).

use std::collections::{HashMap, HashSet, VecDeque};

use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::classes::text_server::{AutowrapMode, OverrunBehavior};
use godot::classes::{Button, Label, PanelContainer, ScrollContainer, StyleBoxFlat, VBoxContainer};
use godot::prelude::*;
use portlight_sim::content;
use portlight_sim::model::{
    ActiveContract, ContractOutcome, InfrastructureRecord, Injury, VoyageStatus,
};
use portlight_sim::session::Session;

use crate::contracts_screen::{
    aboard_clause, ascii_sentence, failure_terms, good_display_name, meta_color, port_display_name,
};
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
/// Gap between the scroll and the foot. The column's 8 px gap left the
/// scroll's bottom edge on a body baseline; 4 px gives those rows to the
/// scroll, so the clip falls in the gap under a line. Card size is unchanged.
const SCROLL_FOOT_SEP: i32 = 4;
/// Captain's week: the scroll may give up at most this much so Close stays inside.
pub(crate) const FOOTER_SCROLL_SHRINK: f32 = 48.0;
/// Footer rows: at most Week + Next.
pub(crate) const FOOTER_MAX_LINES: usize = 2;
const FOOTER_FONT: i32 = 14;
const FOOTER_SEP: i32 = 4;
/// Deterministic body budget for footer yield (headless-reliable). Heads +
/// body lines; a line over [`FOOTER_WRAP_CHARS`] counts as 2.
pub(crate) const FOOTER_BODY_BUDGET: usize = 6;
pub(crate) const FOOTER_WRAP_CHARS: usize = 60;
/// Arrival mover yield: a body line over this many chars wraps to two rows at
/// card width (measured: the 69-char Deadlines line fits, the 78-char contract
/// line wraps). Separate from [`FOOTER_WRAP_CHARS`], which is deliberately
/// conservative for the footer.
pub(crate) const ARRIVAL_WRAP_CHARS: usize = 70;
/// Arrival row budget when the card draws at most 3 section heads.
pub(crate) const ARRIVAL_ROW_BUDGET: usize = 8;
/// Arrival row budget when the card draws 4 section heads.
pub(crate) const ARRIVAL_ROW_BUDGET_FOUR_HEADS: usize = 7;

/// Contract fail: the one prefix for an expiry line (card and Log). Mirrors
/// the T-N `Contract paid:` notice.
pub(crate) const EXPIRED_PREFIX: &str = "Contract expired: ";

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
    /// Per-port sell prices written at undock (before `depart`). Never saved.
    /// Keyed `port_id` → (`good_id` → `sell_price`). First visit / missing → no movers.
    pub visit_price_memory: HashMap<String, HashMap<String, i64>>,
    /// Active bounty ids from the last successful Next day.
    pub active_bounties: HashSet<String>,
    /// Claimable bounty ids already used as an auto-show trigger (not arrival).
    pub claimable_seen: HashSet<String>,
    /// Captain's week rolling snapshots (never saved).
    pub week: WeekWindow,
}

impl DayReportMemory {
    /// New game / load: every Godot-only snapshot goes, including the
    /// arrival `visit_price_memory` and Captain's week.
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
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
    let mut title = text_label("", 24, GOLD, false);
    title.set_name("DayReportTitle");
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
    let mut stack = VBoxContainer::new_alloc();
    stack.set_name("DayReportStack");
    stack.set_h_size_flags(SizeFlags::EXPAND_FILL);
    stack.set_v_size_flags(SizeFlags::EXPAND_FILL);
    stack.add_theme_constant_override("separation", SCROLL_FOOT_SEP);
    stack.add_child(&scroll);
    column.add_child(&stack);

    // Captain's week footer: below scroll, above Close. Not a capped section.
    // Footer and Close share one column slot (sep 4), so a shown footer costs
    // its rows + 4 px and never more than FOOTER_SCROLL_SHRINK of scroll.
    let mut foot = VBoxContainer::new_alloc();
    foot.set_name("DayReportFoot");
    foot.set_h_size_flags(SizeFlags::EXPAND_FILL);
    foot.add_theme_constant_override("separation", FOOTER_SEP);
    stack.add_child(&foot);

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
            // Contract fail: an expiry line takes the existing DUE tone.
            let color = if is_failure_line(&line.text) {
                meta_color(true)
            } else {
                CREAM
            };
            box_node.add_child(&body_line_label(&line.text, color));
        }
        nodes.body.add_child(&box_node);
    }
    apply_footer(nodes, &doc.footer, &doc.sections);
}

/// Body cost for footer yield: one per rendered section head, plus one per line
/// (two when the line is longer than [`FOOTER_WRAP_CHARS`]). Empty sections
/// contribute nothing (they are not drawn).
pub(crate) fn footer_body_cost(sections: &[DayReportSection]) -> usize {
    body_row_cost(sections, FOOTER_WRAP_CHARS)
}

/// Arrival row cost: one per drawn section head, plus one per line (two when
/// the line is longer than [`ARRIVAL_WRAP_CHARS`]).
pub(crate) fn arrival_row_cost(sections: &[DayReportSection]) -> usize {
    body_row_cost(sections, ARRIVAL_WRAP_CHARS)
}

/// Arrival row budget: 8 with at most 3 drawn section heads, 7 with 4.
pub(crate) fn arrival_row_budget(sections: &[DayReportSection]) -> usize {
    let heads = sections
        .iter()
        .filter(|section| !section.lines.is_empty())
        .count();
    if heads >= 4 {
        ARRIVAL_ROW_BUDGET_FOUR_HEADS
    } else {
        ARRIVAL_ROW_BUDGET
    }
}

/// Arrival days: movers yield before anything else (C1). `build` lays out the
/// card with the kept movers (rank order). While the arrival row cost exceeds
/// the arrival row budget, the lowest-ranked mover drops and the card is laid
/// out again. Movers may drop to zero. Dropped movers get no `+N more`, and no
/// other line trims for this.
pub(crate) fn yield_arrival_movers(
    movers: Vec<DayReportLine>,
    build: impl Fn(&[DayReportLine]) -> Vec<DayReportSection>,
) -> Vec<DayReportSection> {
    yield_arrival_rows(movers, Vec::new(), |kept, _| build(kept))
}

/// Arrival days with Health (GD ruling on the AD edge case): movers yield
/// first, then Health lines from last to first while the arrival row cost
/// is still over budget. `build` lays out the card with the kept movers and
/// kept Health lines. Still over budget once both are gone is accepted.
/// Arrival contracts, Deadlines and Bounties never trim.
pub(crate) fn yield_arrival_rows(
    mut movers: Vec<DayReportLine>,
    mut health: Vec<DayReportLine>,
    build: impl Fn(&[DayReportLine], &[DayReportLine]) -> Vec<DayReportSection>,
) -> Vec<DayReportSection> {
    loop {
        let sections = build(&movers, &health);
        if arrival_row_cost(&sections) <= arrival_row_budget(&sections) {
            return sections;
        }
        if movers.pop().is_none() && health.pop().is_none() {
            return sections;
        }
    }
}

/// Heads + lines; a line longer than `wrap_chars` counts as 2. Empty sections
/// contribute nothing (they are not drawn).
fn body_row_cost(sections: &[DayReportSection], wrap_chars: usize) -> usize {
    sections
        .iter()
        .filter(|section| !section.lines.is_empty())
        .map(|section| {
            1 + section
                .lines
                .iter()
                .map(|line| {
                    if line.text.chars().count() > wrap_chars {
                        2
                    } else {
                        1
                    }
                })
                .sum::<usize>()
        })
        .sum()
}

/// Show the footer only when Week is present and the body fits under the shrunk
/// scroll. Yield is presentation-only and all-or-nothing: Next never appears
/// without Week, and a yield hides both rows while keeping scroll at
/// [`SCROLL_MIN_H`].
pub(crate) fn footer_should_show(footer: &DayReportFooter, sections: &[DayReportSection]) -> bool {
    if footer.week.is_none() || footer.lines().is_empty() {
        return false;
    }
    footer_body_cost(sections) <= FOOTER_BODY_BUDGET
}

fn apply_footer(
    nodes: &mut DayReportNodes,
    footer: &DayReportFooter,
    sections: &[DayReportSection],
) {
    clear_children(&mut nodes.footer);
    let shown = footer_should_show(footer, sections);
    if shown {
        for text in footer.lines().iter().take(FOOTER_MAX_LINES) {
            nodes.footer.add_child(&footer_label(text));
        }
    }
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
/// for claimables that fire as notable this day. `failures` are this advance's
/// [`failure_lines`]: pinned at the top of Deadlines and never trimmed.
pub(crate) fn build_document(
    session: &Session,
    failures: &[DayReportLine],
    injuries_before: &[Injury],
    prices_before: &HashMap<String, i64>,
    memory: &mut DayReportMemory,
    arrival_day: bool,
    bounty_known: &[portlight_sim::bounty::BountyTarget],
) -> DayReportDocument {
    let world = session.world();
    let day = world.day;
    let docked_port = (world.voyage.status == VoyageStatus::InPort)
        .then_some(world.voyage.destination_id.as_str());

    let (arrival_contracts, movers) = match (arrival_day, docked_port) {
        (true, Some(port_id)) => (
            arrival_contract_lines(session.board().active.as_slice(), day, port_id),
            visit_mover_lines(session, &memory.visit_price_memory),
        ),
        _ => (Vec::new(), Vec::new()),
    };

    let exclude_port = if arrival_day { docked_port } else { None };
    let deadlines = deadline_lines(session.board().active.as_slice(), day, world, exclude_port);
    let health = health_lines(injuries_before, session.injuries());
    // Arrival day: visit movers replace day-over-day Prices (no separate section).
    let prices = if arrival_day {
        Vec::new()
    } else {
        price_lines(session, prices_before)
    };
    let bounties = bounty_lines(session, memory, arrival_day, bounty_known);

    let mut rest: Vec<(&'static str, &'static str, Vec<DayReportLine>)> = Vec::new();
    rest.push(("deadlines", "Deadlines", deadlines));
    rest.push(("health", "Health", health));
    if !arrival_day {
        rest.push(("prices", "Prices", prices));
    }
    rest.push(("bounties", "Bounties", bounties));

    let sections = if arrival_day {
        // Movers yield first, then Health: re-cap with one fewer line until
        // the rows fit.
        let health = rest
            .iter()
            .find(|(id, _, _)| *id == "health")
            .map(|(_, _, lines)| lines.clone())
            .unwrap_or_default();
        yield_arrival_rows(movers, health, |kept, kept_health| {
            let mut arrival = arrival_contracts.clone();
            arrival.extend_from_slice(kept);
            let mut raw = Vec::with_capacity(rest.len() + 1);
            raw.push(("arrival", "Arrival", arrival));
            for (id, title, lines) in &rest {
                let lines = if *id == "health" {
                    kept_health.to_vec()
                } else {
                    lines.clone()
                };
                raw.push((id, title, lines));
            }
            cap_sections_pinned(raw, failures.to_vec())
        })
    } else {
        cap_sections_pinned(rest, failures.to_vec())
    };
    let title = if arrival_day {
        if let Some(port_id) = docked_port {
            format!("Arrived - {}", port_label(world, port_id))
        } else {
            format!("Day {day}")
        }
    } else {
        format!("Day {day}")
    };
    DayReportDocument {
        day,
        title,
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
    /// Destination of the first overdue contract (deadline, title, id order),
    /// catalog name. Names the sale in `Next: Contract overdue - sell at ...`.
    pub overdue_port: Option<String>,
    pub claimable: bool,
    /// Any captain injury still asking the player to act: healing and untreated.
    pub captain_wounded: bool,
    pub provisions: i64,
}

/// Advice rung predicate (§12.7): healing wound that is not yet treated.
fn untreated_healing(injury: &Injury) -> bool {
    injury.heal_remaining.is_some() && !injury.treated
}

pub(crate) fn next_facts(session: &Session) -> NextFacts {
    let world = session.world();
    let day = world.day;
    let mut overdue: Vec<&ActiveContract> = session
        .board()
        .active
        .iter()
        .filter(|contract| contract.deadline_day < day)
        .collect();
    overdue.sort_by(|a, b| {
        a.deadline_day
            .cmp(&b.deadline_day)
            .then_with(|| a.title.cmp(&b.title))
            .then_with(|| a.offer_id.cmp(&b.offer_id))
    });
    NextFacts {
        contract_days_left: session
            .board()
            .active
            .iter()
            .map(|contract| contract.deadline_day - day)
            .collect(),
        overdue_port: overdue
            .first()
            .map(|contract| failure_name(&port_display_name(&contract.destination_port_id))),
        claimable: !claimable_ids(session).is_empty(),
        captain_wounded: session.injuries().iter().any(untreated_healing),
        provisions: world.captain.provisions,
    }
}

/// First match: overdue, due today, approaching, Claim ready, wounded, Stores low.
/// Overdue names the sale (GD: the desk cannot fix it; selling is the action),
/// without promising that the sale still pays.
pub(crate) fn next_line(facts: &NextFacts) -> Option<String> {
    let lefts = &facts.contract_days_left;
    if lefts.iter().any(|&left| left < 0) {
        let port = facts.overdue_port.as_deref().unwrap_or("the destination");
        return Some(format!("Next: Contract overdue - sell at {port}."));
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

/// Contracts deliverable at `docked_port`. Cap [`DEADLINE_N`] + `+N more`.
/// Copy: `{Title} - {d}/{r} - {timing} - sell {need} more {Good} here` (omit hint if need<=0).
pub(crate) fn arrival_contract_lines(
    active: &[ActiveContract],
    day: i64,
    docked_port: &str,
) -> Vec<DayReportLine> {
    let mut matching: Vec<&ActiveContract> = active
        .iter()
        .filter(|contract| contract.destination_port_id == docked_port)
        .collect();
    matching.sort_by(|a, b| {
        a.deadline_day
            .cmp(&b.deadline_day)
            .then_with(|| a.title.cmp(&b.title))
            .then_with(|| a.offer_id.cmp(&b.offer_id))
    });
    let total = matching.len();
    let take = DEADLINE_N as usize;
    let mut lines = Vec::new();
    for contract in matching.into_iter().take(take) {
        let title = ascii_sentence(&contract.title);
        let timing = deadline_timing(contract.deadline_day - day);
        let progress = format!(
            "{}/{}",
            contract.delivered_quantity, contract.required_quantity
        );
        let need = contract.required_quantity - contract.delivered_quantity;
        let text = if need > 0 {
            let good = good_name(&contract.good_id);
            format!("{title} - {progress} - {timing} - sell {need} more {good} here")
        } else {
            format!("{title} - {progress} - {timing}")
        };
        lines.push(DayReportLine {
            text,
            notable: true,
        });
    }
    if total > take {
        lines.push(DayReportLine {
            text: format!("+{} more", total - take),
            notable: false,
        });
    }
    lines
}

/// Visit movers at the docked pier since last undock snapshot. Empty when the
/// port has no visit memory (first visit) or none pass the 10% bar.
pub(crate) fn visit_mover_lines(
    session: &Session,
    visit_memory: &HashMap<String, HashMap<String, i64>>,
) -> Vec<DayReportLine> {
    let world = session.world();
    if world.voyage.status != VoyageStatus::InPort {
        return Vec::new();
    }
    let port_id = world.voyage.destination_id.as_str();
    let Some(before) = visit_memory.get(port_id) else {
        return Vec::new();
    };
    let Some(port) = world.port(port_id) else {
        return Vec::new();
    };
    let after: HashMap<String, i64> = port
        .market
        .iter()
        .map(|slot| (slot.good_id.clone(), slot.sell_price))
        .collect();
    rank_visit_movers(before, &after)
}

/// Pure mover ranking: abs percent >= 10%, top 3, copy `Good old to new (+N%)`.
pub(crate) fn rank_visit_movers(
    before: &HashMap<String, i64>,
    after: &HashMap<String, i64>,
) -> Vec<DayReportLine> {
    let mut movers: Vec<(i64, String, i64, i64)> = Vec::new();
    for (good_id, &now) in after {
        let Some(&prev) = before.get(good_id) else {
            continue;
        };
        let delta = now - prev;
        if delta == 0 {
            continue;
        }
        let pct = ((delta as f64) / (prev.max(1) as f64) * 100.0).round() as i64;
        if pct.abs() < 10 {
            continue;
        }
        movers.push((pct.abs(), good_id.clone(), prev, now));
    }
    movers.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    movers
        .into_iter()
        .take(3)
        .map(|(_, good_id, old, new)| {
            let good = good_name(&good_id);
            let pct = ((new - old) as f64 / (old.max(1) as f64) * 100.0).round() as i64;
            let sign = if pct > 0 { "+" } else { "" };
            DayReportLine {
                text: format!("{good} {old} to {new} ({sign}{pct}%)"),
                notable: true,
            }
        })
        .collect()
}

/// Active contracts within [`DEADLINE_N`] days. Expiries are not here: they
/// come only from [`failure_lines`] (one source), pinned above these.
fn deadline_lines(
    active: &[ActiveContract],
    day: i64,
    world: &portlight_sim::model::World,
    exclude_port: Option<&str>,
) -> Vec<DayReportLine> {
    let mut lines = Vec::new();
    for contract in active {
        if exclude_port.is_some_and(|port| contract.destination_port_id == port) {
            continue;
        }
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
        // The sale pays a fulfilled contract (R10), so `need <= 0` drops the
        // hint clause. No Complete or Deliver wording.
        let text = if need > 0 {
            let good = good_name(&contract.good_id);
            let port = port_label(world, &contract.destination_port_id);
            format!("{title} - {timing} - {progress} - sell {need} more {good} at {port}")
        } else {
            format!("{title} - {timing} - {progress}")
        };
        lines.push(DayReportLine {
            text,
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
            let old = now - delta;
            DayReportLine {
                text: price_line_text(&good, &port_name, old, now),
                notable: price_move_is_notable(old, delta),
            }
        })
        .collect()
}

/// F8: `{Good} at {Port} {old} to {new} ({+pct}%)`, the Arrival mover rounding.
/// R4: a move that rounds to 0% drops the parenthetical.
pub(crate) fn price_line_text(good: &str, port: &str, old: i64, now: i64) -> String {
    let pct = price_move_pct(old, now);
    if pct == 0 {
        format!("{good} at {port} {old} to {now}")
    } else {
        format!("{good} at {port} {old} to {now} ({pct:+}%)")
    }
}

fn price_move_pct(old: i64, now: i64) -> i64 {
    (((now - old) as f64) / (old.max(1) as f64) * 100.0).round() as i64
}

/// Q1: a price line opens the report only on a move of at least 10% and at
/// least 5 silver. Smaller picked moves still print when something else opens it.
pub(crate) fn price_move_is_notable(old: i64, delta: i64) -> bool {
    let rel = (delta.abs() as f64) / (old.max(1) as f64);
    rel >= 0.10 && delta.abs() >= 5
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

    // C1: Claim ready first (only actionable), then claimed.
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
    // F5: no `Bounty accepted` line. The player accepted it at the Hunt desk,
    // which has its own notice. The memory still tracks actives for the
    // claimed diff.
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
    cap_within(raw, LINE_CAP)
}

/// Contract fail: `pinned` failure lines bypass the cap. The rest is capped
/// at `LINE_CAP - pinned` (same C1 reserve), then the pinned lines go on top
/// of Deadlines, which is created at its canonical place when the rest left
/// it empty. Auto-show counts pinned lines too. With no pinned lines this is
/// exactly [`cap_sections`].
pub(crate) fn cap_sections_pinned(
    raw: Vec<(&'static str, &'static str, Vec<DayReportLine>)>,
    pinned: Vec<DayReportLine>,
) -> Vec<DayReportSection> {
    if pinned.is_empty() {
        return cap_sections(raw);
    }
    let order: Vec<&'static str> = raw.iter().map(|(id, _, _)| *id).collect();
    let mut sections = cap_within(raw, LINE_CAP.saturating_sub(pinned.len()));
    if let Some(section) = sections
        .iter_mut()
        .find(|section| section.id == "deadlines")
    {
        let mut lines = pinned;
        lines.append(&mut section.lines);
        section.lines = lines;
    } else {
        let before: &[&str] = match order.iter().position(|id| *id == "deadlines") {
            Some(at) => &order[..at],
            None => &[],
        };
        let index = sections
            .iter()
            .filter(|section| before.contains(&section.id))
            .count();
        sections.insert(
            index,
            DayReportSection {
                id: "deadlines",
                title: "Deadlines",
                lines: pinned,
            },
        );
    }
    sections
}

/// Fill sections in order within `cap` lines, C1 reserve for later
/// non-empty sections, trailing `+N more` where a section overflows.
fn cap_within(
    raw: Vec<(&'static str, &'static str, Vec<DayReportLine>)>,
    cap: usize,
) -> Vec<DayReportSection> {
    // C1: reserve only for later sections that actually have lines.
    let non_empty: Vec<bool> = raw.iter().map(|(_, _, lines)| !lines.is_empty()).collect();

    let mut remaining = cap;
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

/// Pre-advance snapshot `next_day` takes for the failure line: active titles
/// (the outcome has no title) and the claim count (to find this advance's
/// contract-guarantee payout). Transient; never saved.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FailureFacts {
    /// `offer_id` -> title for every active contract before the advance.
    pub titles_before: HashMap<String, String>,
    /// `infrastructure().claims.len()` before the advance.
    pub claims_before: usize,
}

impl FailureFacts {
    pub(crate) fn snapshot(session: &Session) -> Self {
        Self {
            titles_before: session
                .board()
                .active
                .iter()
                .map(|contract| (contract.offer_id.clone(), contract.title.clone()))
                .collect(),
            claims_before: session.infrastructure().claims.len(),
        }
    }
}

/// Contract-guarantee silver paid for `contract_id` on this advance: claims
/// after `claims_before`, `contract_failure`, not denied, payout > 0, whose
/// policy targets this contract (both guarantee specs are `named_contract`).
pub(crate) fn guarantee_paid(session: &Session, claims_before: usize, contract_id: &str) -> i64 {
    guarantee_in(session.infrastructure(), claims_before, contract_id)
}

/// Contract-guarantee silver ever paid for `contract_id` (the whole claims
/// list, not one advance). Recent rows are history and survive a load, so
/// they read this, never the unsaved `claims_before` snapshot.
pub(crate) fn guarantee_total(infra: &InfrastructureRecord, contract_id: &str) -> i64 {
    guarantee_in(infra, 0, contract_id)
}

fn guarantee_in(infra: &InfrastructureRecord, claims_before: usize, contract_id: &str) -> i64 {
    infra
        .claims
        .iter()
        .skip(claims_before)
        .filter(|claim| {
            claim.incident_type == "contract_failure" && !claim.denied && claim.payout > 0
        })
        .filter(|claim| {
            infra
                .policies
                .iter()
                .any(|policy| policy.id == claim.policy_id && policy.target_id == contract_id)
        })
        .map(|claim| claim.payout)
        .sum()
}

/// True only for a [`failure_lines`] line (DUE tone on the card).
pub(crate) fn is_failure_line(text: &str) -> bool {
    text.starts_with(EXPIRED_PREFIX)
}

/// The one expiry formatter for the card and the Log:
/// `Contract expired: {Title} - {d}/{r} - Silver +N - Guarantee +N - {n} {Good} still aboard`.
/// Silver-only terms (GD M2), zero terms omitted, catalog names first then
/// `humanize_id`, never the sim summary. Title falls back to `{Good} to {Port}`.
pub(crate) fn expired_line(
    outcome: &ContractOutcome,
    title: Option<&str>,
    held: i64,
    guarantee: i64,
) -> String {
    let mut text = format!(
        "{EXPIRED_PREFIX}{} - {}/{}",
        failure_title(outcome, title),
        outcome.delivered_quantity,
        outcome.required_quantity
    );
    for term in failure_terms(outcome, guarantee) {
        text.push_str(" - ");
        text.push_str(&term);
    }
    text.push_str(&aboard_clause(&outcome.good_id, held));
    text
}

/// Snapshot title (ASCII), else `{Good} to {Port}` from catalog names.
fn failure_title(outcome: &ContractOutcome, title: Option<&str>) -> String {
    title
        .map(ascii_sentence)
        .map(|title| title.trim().to_string())
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| {
            failure_name(&format!(
                "{} to {}",
                good_display_name(&outcome.good_id),
                port_display_name(&outcome.destination_port_id)
            ))
        })
}

/// ASCII guard for a catalog or humanised name.
fn failure_name(name: &str) -> String {
    ascii_sentence(name)
}

/// One `notable` line per expired outcome on this advance, sorted by
/// deadline, then title, then contract id. `held` is the flagship hold after
/// the advance; the guarantee comes from this advance's claims only.
pub(crate) fn failure_lines(
    session: &Session,
    turn_contracts: &[ContractOutcome],
    facts: &FailureFacts,
) -> Vec<DayReportLine> {
    let cargo = &session.world().captain.cargo;
    let mut rows: Vec<(i64, String, String, String)> = turn_contracts
        .iter()
        .filter(|outcome| outcome.outcome_type == "expired")
        .map(|outcome| {
            let title = facts
                .titles_before
                .get(&outcome.contract_id)
                .map(String::as_str);
            let held = cargo
                .iter()
                .filter(|item| item.good_id == outcome.good_id)
                .map(|item| item.quantity)
                .sum();
            let guarantee = guarantee_paid(session, facts.claims_before, &outcome.contract_id);
            (
                outcome.deadline_day,
                failure_title(outcome, title),
                outcome.contract_id.clone(),
                expired_line(outcome, title, held, guarantee),
            )
        })
        .collect();
    rows.sort();
    rows.into_iter()
        .map(|(_, _, _, text)| DayReportLine {
            text,
            notable: true,
        })
        .collect()
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

/// Label meta holding a body line's copy when the drawn text carries a
/// separator break (`\n`). Readers use [`label_copy`].
pub(crate) const COPY_META: &str = "portlight_copy";

/// The copy a label shows: the [`COPY_META`] text when set, else its text.
pub(crate) fn label_copy(label: &Gd<Label>) -> String {
    if label.has_meta(COPY_META) {
        label.get_meta(COPY_META).to_string()
    } else {
        label.get_text().to_string()
    }
}

/// A wrapped 15 px body line. When the card's own word wrap would end a row
/// on a ` - ` separator (`... - 5/23 -`), the row breaks at that separator
/// instead and the dash is dropped (AD #62 soft 2). Re-run on every resize
/// from the original copy, which stays in [`COPY_META`].
fn body_line_label(text: &str, color: Color) -> Gd<Label> {
    let mut label = text_label(text, 15, color, true);
    label.set_meta(COPY_META, &text.to_variant());
    let copy = text.to_string();
    let mut target = label.clone();
    label.signals().resized().connect(move || {
        let width = target.get_size().x;
        if width < 50.0 {
            return;
        }
        let Some(font) = target.get_theme_font("font") else {
            return;
        };
        let size = target.get_theme_font_size("font_size");
        let drawn = separator_wrap(&copy, width, |row| {
            font.get_string_size_ex(row).font_size(size).done().x
        });
        if target.get_text().to_string() != drawn {
            target.set_text(&drawn);
        }
    });
    label
}

/// Greedy word wrap of `text` at `width` (as the Label does). Where a row
/// would end with the separator dash of ` - `, the row ends before the
/// separator and the next row starts after it (the dash is dropped). Text
/// with no such row comes back unchanged, without `\n`.
pub(crate) fn separator_wrap(text: &str, width: f32, measure: impl Fn(&str) -> f32) -> String {
    let words: Vec<&str> = text.split(' ').collect();
    let mut rows: Vec<String> = Vec::new();
    let mut moved = false;
    let mut row: Vec<&str> = Vec::new();
    let mut index = 0;
    while index < words.len() {
        let word = words[index];
        let mut candidate = row.clone();
        candidate.push(word);
        if row.is_empty() || measure(&candidate.join(" ")) <= width {
            row = candidate;
            index += 1;
            continue;
        }
        // `word` starts the next row. A row ending on the separator dash
        // gives the dash back and breaks before it.
        if row.len() > 1 && row.last() == Some(&"-") {
            row.pop();
            moved = true;
        }
        rows.push(row.join(" "));
        row = Vec::new();
    }
    if !row.is_empty() {
        rows.push(row.join(" "));
    }
    if moved {
        rows.join("\n")
    } else {
        text.to_string()
    }
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

/// Font colour of the first Label under `node` whose text is `text` (smoke
/// tone checks). `None` when no such label exists.
pub(crate) fn label_color(node: &Gd<godot::classes::Node>, text: &str) -> Option<Color> {
    if let Ok(label) = node.clone().try_cast::<Label>() {
        if label_copy(&label) == text {
            return Some(label.get_theme_color("font_color"));
        }
    }
    node.get_children()
        .iter_shared()
        .find_map(|child| label_color(&child, text))
}

fn collect_text(node: &Gd<godot::classes::Node>, parts: &mut Vec<String>) {
    if let Ok(label) = node.clone().try_cast::<Label>() {
        let text = label_copy(&label);
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
                    text: "Grain at Al-Manar 72 to 84 (+17%)".into(),
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
                overdue_port: None,
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
                text: "Spice charter - 1 day left - 10/10".into(),
                notable: true,
            }],
        }],
        // Warm-up (one sample): Week omitted, Next only.
        footer: DayReportFooter {
            week: week_line(&smoke_week_window(day, &[(540, Some(3), 0, 0, 0)])),
            next: next_line(&NextFacts {
                contract_days_left: vec![1],
                overdue_port: None,
                claimable: false,
                captain_wounded: false,
                provisions: 20,
            }),
        },
    }
}

/// Forced arrival variant for CI frame `day-report-arrival.png`. The staged
/// body goes through the same mover yield as a live arrival.
pub(crate) fn smoke_arrival_document(day: i64) -> DayReportDocument {
    DayReportDocument {
        day,
        title: "Arrived - Al-Manar".into(),
        sections: yield_arrival_movers(smoke_arrival_movers(), smoke_arrival_sections),
        // Forced footer; the body is over the footer budget, so it yields.
        footer: DayReportFooter {
            week: week_line(&smoke_week_window(
                day,
                &[(400, Some(3), 0, 0, 0), (520, Some(3), 0, 1, 0)],
            )),
            next: next_line(&NextFacts {
                contract_days_left: vec![20],
                overdue_port: None,
                claimable: true,
                captain_wounded: false,
                provisions: 20,
            }),
        },
    }
}

/// Staged visit movers in rank order (all three, before the yield).
fn smoke_arrival_movers() -> Vec<DayReportLine> {
    [
        "Grain 12 to 16 (+33%)",
        "Spice 20 to 14 (-30%)",
        "Timber 18 to 20 (+11%)",
    ]
    .into_iter()
    .map(|text| DayReportLine {
        text: text.into(),
        notable: true,
    })
    .collect()
}

/// Staged arrival body with the given movers under the contract line. The
/// contract line matches the smoke session's live Al-Manar contract.
fn smoke_arrival_sections(movers: &[DayReportLine]) -> Vec<DayReportSection> {
    let mut arrival = vec![DayReportLine {
        text: "Porcelain for Al-Manar estate - 0/5 - 20 days left - sell 5 more Porcelain here"
            .into(),
        notable: true,
    }];
    arrival.extend_from_slice(movers);
    vec![
        DayReportSection {
            id: "arrival",
            title: "Arrival",
            lines: arrival,
        },
        // The smoke session holds only the Al-Manar contract, which Arrival
        // already shows, so the staged card carries no Deadlines row that the
        // live strip would contradict. Health keeps the same row cost.
        DayReportSection {
            id: "health",
            title: "Health",
            lines: vec![DayReportLine {
                text: "Healed: Cut hand.".into(),
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
    ]
}

/// Seed-1 probe numbers for the expired smoke and tests: Famine relief, grain
/// x23 to Corsair's Rest, deadline 19, reward 552; part sold 5/23 pays 60.
pub(crate) const SMOKE_EXPIRED_TITLE: &str = "Famine relief: grain to Corsair's Rest";

/// Live day of the expired frame: the seed-1 Spice restock (deadline 26) is
/// 3 days out, so it is on the card's Deadlines and drives Next.
pub(crate) const SMOKE_EXPIRED_FRAME_DAY: i64 = 23;

#[cfg(test)]
pub(crate) fn smoke_expired_outcome(day: i64, delivered: i64, silver: i64) -> ContractOutcome {
    ContractOutcome {
        contract_id: "71773aae754b".into(),
        outcome_type: "expired".into(),
        silver_delta: silver,
        trust_delta: if delivered > 0 { -2 } else { -3 },
        standing_delta: if delivered > 0 { -1 } else { -2 },
        heat_delta: if delivered > 0 { 1 } else { 2 },
        completion_day: day - 1,
        summary: "Contract defaulted: failed to deliver grain to corsairs_rest".into(),
        family: "shortage".into(),
        good_id: "grain".into(),
        required_quantity: 23,
        delivered_quantity: delivered,
        destination_port_id: "corsairs_rest".into(),
        deadline_day: 19,
        reward_silver: 552,
    }
}

/// Forced document for CI frame `day-report-expired.png`: `failure` (the
/// real expiry line the smoke read off the Day 21 card; DUE, pinned) above
/// the live board's deadline lines (CREAM) for `session`'s day. The Next line
/// is live (`next_facts`); Week is the staged window. The smoke accepts the
/// real seed-1 Spice contract, so the strip shows it beside the card.
pub(crate) fn smoke_expired_document(session: &Session, failure: &str) -> DayReportDocument {
    let world = session.world();
    let day = world.day;
    let failure = DayReportLine {
        text: failure.to_string(),
        notable: true,
    };
    let deadlines = deadline_lines(&session.board().active, day, world, None);
    DayReportDocument {
        day,
        title: format!("Day {day}"),
        sections: cap_sections_pinned(vec![("deadlines", "Deadlines", deadlines)], vec![failure]),
        footer: DayReportFooter {
            week: week_line(&smoke_week_window(
                day,
                &[(400, Some(3), 0, 0, 0), (457, Some(3), 0, 0, 0)],
            )),
            next: next_line(&next_facts(session)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_contract(delivered: i64, required: i64) -> ActiveContract {
        ActiveContract {
            offer_id: "offer-spice".into(),
            template_id: "test".into(),
            family: "test".into(),
            title: "Spice charter".into(),
            accepted_day: 1,
            deadline_day: 2,
            destination_port_id: "al_manar".into(),
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

    /// R10: the sale pays a fulfilled contract, so `need <= 0` keeps
    /// `{Title} - {timing} - {d}/{r}` and drops the hint clause.
    #[test]
    fn deadline_line_drops_hint_when_nothing_is_needed() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        let world = session.world();
        let active = vec![fixture_contract(10, 10), fixture_contract(12, 10)];
        let lines = deadline_lines(&active, 1, world, None);
        let texts: Vec<&str> = lines.iter().map(|line| line.text.as_str()).collect();
        assert_eq!(
            texts,
            vec![
                "Spice charter - 1 day left - 10/10",
                "Spice charter - 1 day left - 12/10"
            ]
        );
        for text in texts {
            assert!(!text.contains("Complete"), "{text}");
            assert!(!text.contains("Deliver"), "{text}");
        }
        let open = deadline_lines(&[fixture_contract(3, 10)], 1, world, None);
        assert_eq!(
            open[0].text,
            "Spice charter - 1 day left - 3/10 - sell 7 more Grain at Al-Manar"
        );
    }

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
                moves.push(price_line_text(&good_name(id), "Al-Manar", prev, now));
            }
        }
        assert!(moves.contains(&"Grain at Al-Manar 72 to 84 (+17%)".to_string()));
        assert!(moves.contains(&"Timber at Al-Manar 10 to 7 (-30%)".to_string()));
        assert!(!moves.iter().any(|line| line.contains("Spice")));
    }

    /// F8: old to new with the Arrival rounding; a 0% move drops the parenthetical.
    #[test]
    fn price_line_states_old_new_and_percent() {
        assert_eq!(
            price_line_text("Silk", "Corsair's Rest", 227, 170),
            "Silk at Corsair's Rest 227 to 170 (-25%)"
        );
        assert_eq!(
            price_line_text("Silk", "Corsair's Rest", 500, 502),
            "Silk at Corsair's Rest 500 to 502"
        );
        assert_eq!(
            price_line_text("Pelts", "Porto Novo", 6, 7),
            "Pelts at Porto Novo 6 to 7 (+17%)"
        );
        for text in [
            price_line_text("Silk", "Corsair's Rest", 227, 170),
            price_line_text("Grain", "Al-Manar", 0, 3),
        ] {
            assert!(text.is_ascii() && !text.contains("(now") && !text.contains("+0"));
        }
    }

    /// AD #62 soft 2: a row never ends on the ` - ` separator dash.
    #[test]
    fn separator_wrap_breaks_before_a_trailing_dash() {
        // One char = one unit of width.
        let measure = |row: &str| row.len() as f32;
        let line = "Contract expired: Famine relief - 0/23 - Guarantee +105";
        // Width 40: the plain wrap would end row 1 on `0/23 -`.
        assert_eq!(
            separator_wrap(line, 40.0, measure),
            "Contract expired: Famine relief - 0/23\nGuarantee +105"
        );
        // Fits on one row, or the wrap falls inside a term: unchanged.
        assert_eq!(separator_wrap(line, 80.0, measure), line);
        let week = "Spice restock - 3 days left - 0/8 - sell 8 more Spice at Al-Manar";
        assert_eq!(separator_wrap(week, 50.0, measure), week);
        // Every row is checked, and no row ends or starts with the dash.
        let long = "aaaa - bbbb - cccc - dddd";
        let wrapped = separator_wrap(long, 6.0, measure);
        for row in wrapped.split('\n') {
            assert!(
                !row.ends_with(" -") && !row.starts_with("- "),
                "{wrapped:?}"
            );
        }
        assert!(separator_wrap("Al-Manar", 3.0, measure) == "Al-Manar");
    }

    /// Q1: notable only at >= 10% and >= 5 silver.
    #[test]
    fn price_notable_needs_ten_percent_and_five_silver() {
        // Pelts 6 to 7: 17% but 1 silver - prints, never opens the report.
        assert!(!price_move_is_notable(6, 1));
        // Silk 227 to 170: notable.
        assert!(price_move_is_notable(227, -57));
        // 5 silver on 500 is 1%: not notable.
        assert!(!price_move_is_notable(500, 5));
        // Exactly 10% and 5 silver: notable.
        assert!(price_move_is_notable(50, 5));
        assert!(!price_move_is_notable(50, 4));
        // A report whose only lines are non-notable price lines does not open.
        let sections = cap_sections(vec![(
            "prices",
            "Prices",
            vec![
                line(
                    "Pelts at Porto Novo 6 to 7 (+17%)",
                    price_move_is_notable(6, 1),
                ),
                line(
                    "Silk at Porto Novo 500 to 502",
                    price_move_is_notable(500, 2),
                ),
            ],
        )]);
        assert!(sections.is_empty(), "{sections:?}");
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
            text: "Grain at Al-Manar 72 to 84 (+17%)".into(),
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

    /// C1: Claim ready then claimed inside Bounties. F5: no accepted line.
    #[test]
    fn bounty_lines_order_claim_ready_then_claimed_no_accepted() {
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
                ["Claim ready:", "Bounty claimed:"]
                    .into_iter()
                    .find(|prefix| l.text.starts_with(prefix))
                    .unwrap_or(l.text.as_str())
            })
            .collect();
        assert_eq!(
            kinds,
            vec!["Claim ready:", "Bounty claimed:"],
            "bounty_lines order: {live:?}"
        );
        assert!(memory.active_bounties.contains("the_butcher"));

        // F5: a fresh accept prints nothing (the Hunt desk already said it),
        // but memory still tracks it for the claimed diff.
        let mut fresh = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        fresh.accept_bounty("the_butcher").unwrap();
        let mut fresh_memory = DayReportMemory::default();
        let accepted_only = bounty_lines(&fresh, &mut fresh_memory, false, &known);
        assert!(accepted_only.is_empty(), "{accepted_only:?}");
        assert!(fresh_memory.active_bounties.contains("the_butcher"));

        // Cap guarantee with mixed ordered bounty lines (Claim ready first).
        let ordered = vec![
            line("Claim ready: Raj the Quiet (120 silver) - open Hunt.", true),
            line("Bounty claimed: Old Salt.", true),
            line("Bounty claimed: Red Sails.", true),
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
                    line("Grain at Al-Manar 72 to 84 (+17%)", true),
                    line("Spice at Al-Manar 40 to 37 (-8%)", true),
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
            .map(|i| {
                line(
                    &format!("Good{i} at Port {i} to {} (+{}%)", i + 1, 100 / (i + 1)),
                    true,
                )
            })
            .collect();
        let bounties = vec![
            line("Claim ready: Raj the Quiet (120 silver) - open Hunt.", true),
            line("Bounty claimed: Red Sails.", true),
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
            line("Bounty claimed: C.", true),
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
            overdue_port: None,
            claimable: false,
            captain_wounded: false,
            provisions: 20,
        };
        assert_eq!(next_line(&base), None, "empty ladder omits Next");
        let all = NextFacts {
            contract_days_left: vec![9, 2, 0, -1],
            overdue_port: Some("Corsair's Rest".into()),
            claimable: true,
            captain_wounded: true,
            provisions: 1,
        };
        // GD (contract fail, OQ2): overdue names the sale, not the desk, and
        // promises nothing about payment.
        assert_eq!(
            next_line(&all).as_deref(),
            Some("Next: Contract overdue - sell at Corsair's Rest.")
        );
        assert_eq!(
            next_line(&NextFacts {
                overdue_port: None,
                ..all.clone()
            })
            .as_deref(),
            Some("Next: Contract overdue - sell at the destination.")
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

    /// R1: full 4-section body yields the footer; a small Week body keeps it.
    /// Next without Week never shows (deadline warm-up is Next-only data).
    #[test]
    fn footer_yields_when_body_overflows_budget() {
        let full = smoke_full_document(3);
        assert!(
            footer_body_cost(&full.sections) > FOOTER_BODY_BUDGET,
            "full smoke body must exceed budget (got {})",
            footer_body_cost(&full.sections)
        );
        assert!(
            !footer_should_show(&full.footer, &full.sections),
            "full 4-section document must yield the footer"
        );

        let deadline = smoke_deadline_document(4);
        assert!(
            deadline.footer.week.is_none() && deadline.footer.next.is_some(),
            "deadline smoke still carries Next-only data"
        );
        assert!(
            !footer_should_show(&deadline.footer, &deadline.sections),
            "Next without Week never shows"
        );

        // Footer-shown case: two heads + three short rows = 5 <= 6.
        let week_body = vec![
            DayReportSection {
                id: "deadlines",
                title: "Deadlines",
                lines: vec![line("Spice charter - 3 days left.", true)],
            },
            DayReportSection {
                id: "bounties",
                title: "Bounties",
                lines: vec![
                    line("Bounty claimed: Scarlet Ana.", true),
                    line("Bounty claimed: Raj the Quiet.", true),
                ],
            },
        ];
        assert_eq!(footer_body_cost(&week_body), 5);
        let week_footer = DayReportFooter {
            week: Some("Week: -19 silver - crew 3 to 4 - bounty +1".into()),
            next: Some("Next: Contract 3 days left - open Contracts.".into()),
        };
        assert!(
            footer_should_show(&week_footer, &week_body),
            "compact Week body must keep the footer"
        );
        // Long body line counts as 2.
        let long = DayReportSection {
            id: "deadlines",
            title: "Deadlines",
            lines: vec![line(
                "Grain run - 1 day left - 3/10 - sell 7 more Grain at Corsair's Rest",
                true,
            )],
        };
        assert_eq!(footer_body_cost(&[long]), 3);
    }

    /// R3: treated healing wound skips Captain wounded and falls through.
    #[test]
    fn treated_wound_skips_captain_wounded_rung() {
        let untreated = Injury {
            injury_id: "cut_hand".into(),
            acquired_day: 1,
            heal_remaining: Some(8),
            treated: false,
        };
        let treated = Injury {
            injury_id: "cut_hand".into(),
            acquired_day: 1,
            heal_remaining: Some(4),
            treated: true,
        };
        let permanent = Injury {
            injury_id: "blinded_eye".into(),
            acquired_day: 1,
            heal_remaining: None,
            treated: false,
        };
        assert!(untreated_healing(&untreated));
        assert!(!untreated_healing(&treated));
        assert!(!untreated_healing(&permanent));

        let session = session_with_wound(true, STORES_LOW);
        let facts = next_facts(&session);
        assert!(
            !facts.captain_wounded,
            "treated healing wound must not set captain_wounded"
        );
        assert_eq!(facts.provisions, STORES_LOW);
        assert_eq!(
            next_line(&facts).as_deref(),
            Some("Next: Stores low - restock in port."),
            "treated wound falls through to Stores low"
        );

        let stocked = session_with_wound(true, STORES_LOW + 1);
        assert_eq!(
            next_line(&next_facts(&stocked)),
            None,
            "treated wound with stocked provisions omits Next"
        );

        let open = session_with_wound(false, STORES_LOW);
        let open_facts = next_facts(&open);
        assert!(open_facts.captain_wounded);
        assert_eq!(
            next_line(&open_facts).as_deref(),
            Some("Next: Captain wounded - heal in port."),
            "untreated healing still takes the wound rung over Stores"
        );

        // Week open_wounds stays heal_remaining.is_some() (treated or not).
        assert_eq!(week_sample(&session).open_wounds, 1);
        assert_eq!(week_sample(&open).open_wounds, 1);
    }

    /// Inject one cut_hand wound via save/load (Godot crate has no world_mut).
    fn session_with_wound(treated: bool, provisions: i64) -> Session {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let dir = std::env::temp_dir().join(format!(
            "portlight-day-report-wound-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        session.save(&dir, "wound").unwrap();
        let path = dir.join("saves").join("wound.json");
        let mut raw = std::fs::read_to_string(&path).unwrap();
        let treated_json = if treated { "true" } else { "false" };
        let injury = format!(
            r#"{{"injury_id":"cut_hand","acquired_day":1,"heal_remaining":8,"treated":{treated_json}}}"#
        );
        let from = "\"injuries\": []";
        let to = format!("\"injuries\": [{injury}]");
        assert!(
            raw.contains(from),
            "fresh save should have empty injuries array"
        );
        raw = raw.replacen(from, &to, 1);
        // Force provisions for Stores-low fallthrough.
        let prov_from = format!("\"provisions\": {}", session.world().captain.provisions);
        let prov_to = format!("\"provisions\": {provisions}");
        assert!(
            raw.contains(&prov_from),
            "save missing provisions field {prov_from}"
        );
        raw = raw.replacen(&prov_from, &prov_to, 1);
        std::fs::write(&path, &raw).unwrap();
        let loaded = Session::load(&dir, "wound")
            .unwrap()
            .expect("reloaded wound session");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(loaded.injuries().len(), 1);
        assert_eq!(loaded.injuries()[0].treated, treated);
        assert_eq!(loaded.world().captain.provisions, provisions);
        loaded
    }

    fn test_active(
        title: &str,
        dest: &str,
        good: &str,
        delivered: i64,
        required: i64,
        deadline: i64,
        offer_id: &str,
    ) -> ActiveContract {
        ActiveContract {
            offer_id: offer_id.into(),
            template_id: "t".into(),
            family: "procurement".into(),
            title: title.into(),
            accepted_day: 1,
            deadline_day: deadline,
            destination_port_id: dest.into(),
            good_id: good.into(),
            required_quantity: required,
            delivered_quantity: delivered,
            reward_silver: 10,
            bonus_reward: 0,
            source_region: None,
            source_port: None,
            inspection_modifier: 1.0,
            status: "active".into(),
        }
    }

    #[test]
    fn visit_movers_threshold_sort_top3_and_omit_zero() {
        let before: HashMap<String, i64> = [
            ("grain", 12),
            ("spice", 20),
            ("timber", 10),
            ("wine", 50),
            ("salt", 8),
            ("tea", 30),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v))
        .collect();
        let after: HashMap<String, i64> = [
            ("grain", 16),  // +33%
            ("spice", 14),  // -30%
            ("timber", 11), // +10%
            ("wine", 54),   // +8% — below bar
            ("salt", 8),    // zero — omit
            ("tea", 33),    // +10%
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v))
        .collect();
        let lines = rank_visit_movers(&before, &after);
        assert_eq!(lines.len(), 3, "{lines:?}");
        assert_eq!(lines[0].text, "Grain 12 to 16 (+33%)");
        assert_eq!(lines[1].text, "Spice 20 to 14 (-30%)");
        // timber and tea both 10%; alphabetical good_id: tea before timber
        assert_eq!(lines[2].text, "Tea 30 to 33 (+10%)");
        assert!(lines.iter().all(|l| l.notable));
        assert!(!lines
            .iter()
            .any(|l| l.text.contains("Wine") || l.text.contains("Salt")));
    }

    #[test]
    fn visit_movers_empty_when_no_snapshot() {
        let after: HashMap<String, i64> = [("grain".into(), 16)].into_iter().collect();
        assert!(rank_visit_movers(&HashMap::new(), &after).is_empty());
        let session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        assert!(
            visit_mover_lines(&session, &HashMap::new()).is_empty(),
            "first visit / missing snapshot must omit movers"
        );
    }

    #[test]
    fn arrival_contracts_cap_and_omit_hint_when_fulfilled() {
        let day = 10;
        let active = vec![
            test_active("A run", "al_manar", "grain", 0, 5, 15, "a"),
            test_active("B run", "al_manar", "spice", 10, 10, 12, "b"),
            test_active("C run", "al_manar", "timber", 1, 4, 14, "c"),
            test_active("D run", "al_manar", "wine", 0, 2, 20, "d"),
            test_active("Elsewhere", "porto_novo", "grain", 0, 5, 11, "e"),
        ];
        let lines = arrival_contract_lines(&active, day, "al_manar");
        // Sorted by deadline: B(12), C(14), A(15), then +1 more for D; Elsewhere excluded.
        assert_eq!(lines.len(), 4, "{lines:?}");
        assert!(lines[0].text.starts_with("B run - 10/10 - 2 days left"));
        assert!(
            !lines[0].text.contains("sell"),
            "need<=0 omits hint: {}",
            lines[0].text
        );
        assert!(lines[1].text.contains("sell 3 more Timber here"));
        assert!(lines[2].text.contains("sell 5 more Grain here"));
        assert_eq!(lines[3].text, "+1 more");
        assert!(!lines[3].notable);
        assert!(lines.iter().all(|l| !l.text.contains("due soon")));
        assert!(lines
            .iter()
            .all(|l| !l.text.contains("Complete") && !l.text.contains("Deliver")));
    }

    #[test]
    fn arrival_dedupes_this_port_out_of_deadlines() {
        let mut session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let offers = session.available_contracts();
        let al = offers
            .iter()
            .find(|o| o.destination_port_id == "al_manar")
            .expect("al_manar offer")
            .id
            .clone();
        let other = offers
            .iter()
            .find(|o| o.destination_port_id != "al_manar")
            .map(|o| o.id.clone());
        session.accept_contract(&al).unwrap();
        if let Some(id) = other {
            let _ = session.accept_contract(&id);
        }
        // Sail and arrive so build_document sees InPort at al_manar.
        session.depart("al_manar").unwrap();
        loop {
            let turn = session.advance().unwrap();
            let _ = turn;
            if session.world().voyage.status == VoyageStatus::InPort {
                break;
            }
            assert!(session.world().day < 80, "never arrived at al_manar");
        }
        assert_eq!(session.world().voyage.destination_id, "al_manar");
        let mut memory = DayReportMemory::default();
        let known: Vec<portlight_sim::bounty::BountyTarget> = Vec::new();
        let doc = build_document(
            &session,
            &[],
            &[],
            &HashMap::new(),
            &mut memory,
            true,
            &known,
        );
        assert!(doc.title.starts_with("Arrived - "), "{}", doc.title);
        assert!(doc.title.contains("Al-Manar"), "{}", doc.title);
        let arrival = doc
            .sections
            .iter()
            .find(|s| s.id == "arrival")
            .expect("arrival section");
        assert!(
            arrival
                .lines
                .iter()
                .any(|l| l.text.contains("here") || l.text.contains(" to ")),
            "{arrival:?}"
        );
        assert!(doc.has_notable(), "arrival content must auto-show");
        if let Some(deadlines) = doc.sections.iter().find(|s| s.id == "deadlines") {
            for line in &deadlines.lines {
                assert!(
                    !line.text.contains("Al-Manar"),
                    "this-port contract leaked into Deadlines: {}",
                    line.text
                );
            }
        }
        assert!(
            !doc.sections.iter().any(|s| s.id == "prices"),
            "arrival day must not show Prices"
        );
        assert!(rendered_line_count(&doc.sections) <= LINE_CAP);
    }

    #[test]
    fn arrival_notability_requires_content_else_existing_rules() {
        let session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let mut memory = DayReportMemory::default();
        let known: Vec<portlight_sim::bounty::BountyTarget> = Vec::new();
        // Fake arrival_day while docked with no contracts and no visit movers:
        // existing quiet rules → no notable sections.
        let quiet = build_document(
            &session,
            &[],
            &[],
            &HashMap::new(),
            &mut memory,
            true,
            &known,
        );
        // Prices are suppressed on arrival and there is no arrival content;
        // claimable/bounty quiet, so the card stays hidden.
        assert!(
            !quiet.has_notable(),
            "quiet arrival without content must stay hidden: {quiet:?}"
        );
        assert!(quiet.sections.is_empty(), "{quiet:?}");
    }

    /// Dedupe: a this-port contract inside the deadline window stays out of
    /// Deadlines on arrival; other ports keep their Deadlines line.
    #[test]
    fn arrival_excludes_docked_port_from_deadlines() {
        let session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let day = session.world().day;
        let active = vec![
            test_active("Porcelain run", "al_manar", "porcelain", 0, 5, day + 2, "a"),
            test_active("Grain run", "porto_novo", "grain", 0, 5, day + 2, "b"),
        ];
        let all = deadline_lines(&active, day, session.world(), None);
        assert_eq!(all.len(), 2, "{all:?}");
        assert!(all.iter().any(|l| l.text.contains("Al-Manar")), "{all:?}");
        let arrival = deadline_lines(&active, day, session.world(), Some("al_manar"));
        assert_eq!(arrival.len(), 1, "{arrival:?}");
        assert!(arrival[0].text.starts_with("Grain run"), "{arrival:?}");
        assert!(!arrival[0].text.contains("Al-Manar"), "{arrival:?}");
    }

    #[test]
    fn non_arrival_keeps_day_title_and_prices() {
        let session = Session::new("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let mut memory = DayReportMemory::default();
        let known: Vec<portlight_sim::bounty::BountyTarget> = Vec::new();
        let prices_before = snapshot_docked_prices(&session);
        let doc = build_document(
            &session,
            &[],
            &[],
            &prices_before,
            &mut memory,
            false,
            &known,
        );
        assert_eq!(doc.title, format!("Day {}", session.world().day));
        assert!(!doc.sections.iter().any(|s| s.id == "arrival"));
    }

    #[test]
    fn smoke_arrival_doc_is_notable_ascii() {
        let doc = smoke_arrival_document(12);
        assert_eq!(doc.title, "Arrived - Al-Manar");
        assert!(doc.has_notable());
        let blob = doc
            .sections
            .iter()
            .flat_map(|s| s.lines.iter())
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(blob.contains("Arrival") || doc.sections[0].id == "arrival");
        assert!(blob.contains("Grain 12 to 16 (+33%)"));
        assert!(!blob.contains('\u{2014}') && blob.contains(" - "));
        assert!(!blob.contains("due soon"));
        assert!(rendered_line_count(&doc.sections) <= LINE_CAP);
    }

    #[test]
    fn visit_price_memory_resets_with_day_report_memory() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        let mut memory = DayReportMemory::default();
        memory.visit_price_memory.insert(
            "porto_novo".into(),
            [("grain".into(), 12)].into_iter().collect(),
        );
        memory.price_memory.insert("grain".into(), 12);
        memory.active_bounties.insert("gnaw".into());
        memory.claimable_seen.insert("gnaw".into());
        memory.week.push(week_sample(&session));
        assert!(!memory.visit_price_memory.is_empty());
        memory.reset();
        assert!(memory.visit_price_memory.is_empty());
        assert!(memory.price_memory.is_empty());
        assert!(memory.active_bounties.is_empty());
        assert!(memory.claimable_seen.is_empty());
        assert_eq!(memory.week.len(), 0);
    }

    fn kept_movers<'a>(sections: &'a [DayReportSection], movers: &[DayReportLine]) -> Vec<&'a str> {
        sections
            .iter()
            .filter(|s| s.id == "arrival")
            .flat_map(|s| s.lines.iter())
            .filter(|l| movers.iter().any(|m| m.text == l.text))
            .map(|l| l.text.as_str())
            .collect()
    }

    fn has_more_row(section: &DayReportSection) -> bool {
        section
            .lines
            .iter()
            .any(|l| l.text.starts_with('+') && l.text.ends_with(" more"))
    }

    /// GD C1 (#57): staged arrival = 3 heads + wrapped contract 2 + 3 movers +
    /// deadline 1 + claim 1 = 10. Movers yield to cost 8; exactly 1 mover kept.
    #[test]
    fn arrival_movers_yield_staged_doc_to_budget() {
        let movers = smoke_arrival_movers();
        assert_eq!(movers.len(), 3);
        let staged = smoke_arrival_sections(&movers);
        assert_eq!(arrival_row_cost(&staged), 10);
        assert_eq!(arrival_row_budget(&staged), ARRIVAL_ROW_BUDGET);

        let doc = smoke_arrival_document(12);
        assert_eq!(arrival_row_cost(&doc.sections), 8);
        assert!(arrival_row_cost(&doc.sections) <= arrival_row_budget(&doc.sections));
        assert_eq!(
            kept_movers(&doc.sections, &movers),
            vec!["Grain 12 to 16 (+33%)"]
        );
        // Only movers yield: contract, Deadlines and Bounties are untouched,
        // and there is no `+N more` for dropped movers.
        assert_eq!(doc.sections.len(), staged.len());
        assert_eq!(doc.sections[0].lines[0], staged[0].lines[0]);
        assert_eq!(doc.sections[0].lines.len(), 2);
        assert!(!has_more_row(&doc.sections[0]));
        assert_eq!(doc.sections[1..], staged[1..]);
        // Claim ready is the last row and sits inside the budget.
        let last = doc.sections.last().expect("bounties");
        assert_eq!(last.id, "bounties");
        assert!(last.lines[0].text.starts_with("Claim ready"));
    }

    #[test]
    fn lone_arrival_section_keeps_all_movers() {
        let movers = smoke_arrival_movers();
        let contract = smoke_arrival_sections(&[])[0].lines[0].clone();
        let sections = yield_arrival_movers(movers.clone(), |kept| {
            let mut lines = vec![contract.clone()];
            lines.extend_from_slice(kept);
            cap_sections(vec![("arrival", "Arrival", lines)])
        });
        assert_eq!(sections.len(), 1);
        assert_eq!(kept_movers(&sections, &movers).len(), 3);
        assert_eq!(arrival_row_cost(&sections), 6);
    }

    /// 4 heads: budget 7. Movers may drop to zero; nothing else trims, and
    /// re-capping leaves no `+N more` for dropped movers.
    #[test]
    fn arrival_movers_yield_to_zero_with_four_heads() {
        let movers = smoke_arrival_movers();
        let contract = line(
            "Porcelain for Al-Manar estate - 0/5 - 20 days left - sell 5 more Porcelain here",
            true,
        );
        let deadline = line(
            "Grain run - 2 days left - 3/10 - sell 7 more Grain at Corsair's Rest",
            true,
        );
        let claim = line("Claim ready: Raj the Quiet (120 silver) - open Hunt.", true);
        let health = line("Healed: Cut hand.", true);
        // Movers-only yield (no Health stage): over budget once movers are gone.
        let sections = yield_arrival_movers(movers.clone(), |kept| {
            let mut arrival = vec![contract.clone()];
            arrival.extend_from_slice(kept);
            cap_sections(vec![
                ("arrival", "Arrival", arrival),
                ("deadlines", "Deadlines", vec![deadline.clone()]),
                ("health", "Health", vec![health.clone()]),
                ("bounties", "Bounties", vec![claim.clone()]),
            ])
        });
        assert_eq!(arrival_row_budget(&sections), ARRIVAL_ROW_BUDGET_FOUR_HEADS);
        assert!(kept_movers(&sections, &movers).is_empty());
        assert!(!has_more_row(&sections[0]));
        assert_eq!(sections[0].lines, vec![contract.clone()]);
        assert_eq!(sections[1].lines, vec![deadline.clone()]);
        assert_eq!(sections[2].lines, vec![health.clone()]);
        assert_eq!(sections[3].lines, vec![claim.clone()]);
        assert_eq!(arrival_row_cost(&sections), 9);

        // GD ruling: Health yields next, so Claim ready stays inside the budget.
        let build = |kept: &[DayReportLine], kept_health: &[DayReportLine]| {
            let mut arrival = vec![contract.clone()];
            arrival.extend_from_slice(kept);
            cap_sections(vec![
                ("arrival", "Arrival", arrival),
                ("deadlines", "Deadlines", vec![deadline.clone()]),
                ("health", "Health", kept_health.to_vec()),
                ("bounties", "Bounties", vec![claim.clone()]),
            ])
        };
        let sections = yield_arrival_rows(movers.clone(), vec![health.clone()], build);
        assert!(kept_movers(&sections, &movers).is_empty());
        assert!(sections
            .iter()
            .all(|section| section.id != "health" || section.lines.is_empty()));
        let drawn: Vec<&DayReportSection> =
            sections.iter().filter(|s| !s.lines.is_empty()).collect();
        assert_eq!(drawn.len(), 3);
        assert_eq!(arrival_row_budget(&sections), ARRIVAL_ROW_BUDGET);
        assert!(arrival_row_cost(&sections) <= arrival_row_budget(&sections));
        assert_eq!(drawn[2].lines, vec![claim.clone()]);
        assert_eq!(drawn[0].lines, vec![contract.clone()]);
        assert_eq!(drawn[1].lines, vec![deadline.clone()]);
    }

    #[test]
    fn arrival_health_yields_last_line_first_and_keeps_what_fits() {
        let contract = line("Porcelain for Al-Manar estate - 0/5 - 20 days left", true);
        let claim = line("Claim ready: Raj the Quiet (120 silver) - open Hunt.", true);
        let health = vec![
            line("Healed: Cut hand.", true),
            line("Healing: Bruised ribs.", false),
            line("Healing: Sprained wrist.", false),
        ];
        let build = |kept: &[DayReportLine], kept_health: &[DayReportLine]| {
            let mut arrival = vec![contract.clone()];
            arrival.extend_from_slice(kept);
            cap_sections(vec![
                ("arrival", "Arrival", arrival),
                ("health", "Health", kept_health.to_vec()),
                ("bounties", "Bounties", vec![claim.clone()]),
            ])
        };
        // Movers go first; with none left the card costs 3 heads + 5 rows = 8,
        // which fits, so every Health line stays.
        let movers = smoke_arrival_movers();
        let sections = yield_arrival_rows(movers.clone(), health.clone(), build);
        assert!(kept_movers(&sections, &movers).is_empty());
        let kept_health = &sections.iter().find(|s| s.id == "health").unwrap().lines;
        assert_eq!(kept_health, &health);
        assert_eq!(arrival_row_cost(&sections), 8);

        // With one more Arrival contract, the last Health line drops first.
        let second = line("Spice for Al-Manar - 0/4 - 9 days left", true);
        let build_two = |kept: &[DayReportLine], kept_health: &[DayReportLine]| {
            let mut arrival = vec![contract.clone(), second.clone()];
            arrival.extend_from_slice(kept);
            cap_sections(vec![
                ("arrival", "Arrival", arrival),
                ("health", "Health", kept_health.to_vec()),
                ("bounties", "Bounties", vec![claim.clone()]),
            ])
        };
        let sections = yield_arrival_rows(Vec::new(), health.clone(), build_two);
        let kept_health = &sections.iter().find(|s| s.id == "health").unwrap().lines;
        assert_eq!(kept_health, &health[..2].to_vec());
        assert!(arrival_row_cost(&sections) <= arrival_row_budget(&sections));
        let bounties = sections.iter().find(|s| s.id == "bounties").unwrap();
        assert_eq!(bounties.lines, vec![claim.clone()]);
    }

    #[test]
    fn arrival_still_over_budget_after_health_is_accepted() {
        let wrapped = |title: &str| {
            line(
                &format!(
                    "{title} for Al-Manar estate - 0/5 - 20 days left - sell 5 more {title} here"
                ),
                true,
            )
        };
        let contracts = vec![wrapped("Porcelain"), wrapped("Spice"), wrapped("Timber")];
        let deadline = line("Grain run - 2 days left - 3/10", true);
        let claim = line("Claim ready: Raj the Quiet (120 silver) - open Hunt.", true);
        let build = |_: &[DayReportLine], kept_health: &[DayReportLine]| {
            cap_sections(vec![
                ("arrival", "Arrival", contracts.clone()),
                ("deadlines", "Deadlines", vec![deadline.clone()]),
                ("health", "Health", kept_health.to_vec()),
                ("bounties", "Bounties", vec![claim.clone()]),
            ])
        };
        let sections = yield_arrival_rows(Vec::new(), vec![line("Healed: Cut hand.", true)], build);
        assert!(sections
            .iter()
            .all(|s| s.id != "health" || s.lines.is_empty()));
        assert_eq!(
            sections.iter().find(|s| s.id == "arrival").unwrap().lines,
            contracts
        );
        assert_eq!(
            sections.iter().find(|s| s.id == "deadlines").unwrap().lines,
            vec![deadline.clone()]
        );
        assert_eq!(
            sections.iter().find(|s| s.id == "bounties").unwrap().lines,
            vec![claim.clone()]
        );
        assert!(arrival_row_cost(&sections) > arrival_row_budget(&sections));
    }

    #[test]
    fn arrival_wrap_is_separate_from_footer_wrap() {
        assert_eq!(ARRIVAL_WRAP_CHARS, 70);
        assert_eq!(FOOTER_WRAP_CHARS, 60);
        let fits = DayReportSection {
            id: "deadlines",
            title: "Deadlines",
            lines: vec![line(&"x".repeat(70), true)],
        };
        let wraps = DayReportSection {
            id: "deadlines",
            title: "Deadlines",
            lines: vec![line(&"x".repeat(71), true)],
        };
        assert_eq!(arrival_row_cost(std::slice::from_ref(&fits)), 2);
        assert_eq!(arrival_row_cost(&[wraps]), 3);
        assert_eq!(footer_body_cost(&[fits]), 3);
    }

    // ---- Contract fail (design-brief-contract-fail.md section 12.1) ----

    use portlight_sim::model::{ActivePolicy, InsuranceClaim};

    fn probe_f1() -> ContractOutcome {
        smoke_expired_outcome(21, 0, 0)
    }

    fn probe_f2() -> ContractOutcome {
        smoke_expired_outcome(21, 5, 60)
    }

    /// Player-surface hygiene for every failure string.
    fn assert_failure_clean(text: &str) {
        assert!(text.is_ascii(), "{text}");
        for bad in [
            "\u{2014}",
            "\u{2013}",
            "->",
            "+0",
            "  ",
            "Deliver",
            "deliver",
            "Complete",
            "due soon",
            "Trust",
            "Standing",
            "Heat",
            "corsairs_rest",
            "_",
        ] {
            assert!(!text.contains(bad), "{bad:?} in {text}");
        }
    }

    /// 12.1 #1: F1 (nothing sold). GD M2: Silver-only terms, so a plain
    /// expiry is just `{Title} - {d}/{r}`.
    #[test]
    fn expired_line_f1_is_title_and_progress() {
        assert_eq!(
            expired_line(&probe_f1(), Some(SMOKE_EXPIRED_TITLE), 0, 0),
            "Contract expired: Famine relief: grain to Corsair's Rest - 0/23"
        );
    }

    /// 12.1 #2: F2 partial pay, aboard clause, guarantee order, all-zero.
    #[test]
    fn expired_line_f2_terms_guarantee_and_aboard() {
        let title = Some(SMOKE_EXPIRED_TITLE);
        assert_eq!(
            expired_line(&probe_f2(), title, 22, 0),
            "Contract expired: Famine relief: grain to Corsair's Rest - 5/23 - Silver +60 - 22 Grain still aboard"
        );
        assert_eq!(
            expired_line(&probe_f2(), title, 0, 0),
            "Contract expired: Famine relief: grain to Corsair's Rest - 5/23 - Silver +60"
        );
        assert_eq!(
            expired_line(&probe_f2(), title, 22, 65),
            "Contract expired: Famine relief: grain to Corsair's Rest - 5/23 - Silver +60 - Guarantee +65 - 22 Grain still aboard"
        );
        // Silver 0: Guarantee is the first term.
        assert_eq!(
            expired_line(&probe_f1(), title, 0, 105),
            "Contract expired: Famine relief: grain to Corsair's Rest - 0/23 - Guarantee +105"
        );
        assert_eq!(
            expired_line(&probe_f1(), title, 23, 105),
            "Contract expired: Famine relief: grain to Corsair's Rest - 0/23 - Guarantee +105 - 23 Grain still aboard"
        );
    }

    /// 12.1 #3: hygiene, title fallback, humanised unknown ids.
    #[test]
    fn expired_line_hygiene_and_fallbacks() {
        assert_eq!(
            expired_line(&probe_f1(), None, 0, 0),
            "Contract expired: Grain to Corsair's Rest - 0/23"
        );
        assert_eq!(
            expired_line(&probe_f1(), Some("  "), 0, 0),
            "Contract expired: Grain to Corsair's Rest - 0/23"
        );
        let mut unknown = probe_f2();
        unknown.good_id = "whale_oil".into();
        unknown.destination_port_id = "drowned_quay".into();
        assert!(content::content().good("whale_oil").is_none());
        assert_eq!(
            expired_line(&unknown, None, 4, 0),
            "Contract expired: Whale Oil to Drowned Quay - 5/23 - Silver +60 - 4 Whale Oil still aboard"
        );
        // Curly quotes and dashes in a title come out ASCII.
        assert_eq!(
            expired_line(
                &probe_f1(),
                Some("Famine relief \u{2014} Corsair\u{2019}s Rest"),
                0,
                0
            ),
            "Contract expired: Famine relief - Corsair's Rest - 0/23"
        );
        for text in [
            expired_line(&probe_f1(), Some(SMOKE_EXPIRED_TITLE), 0, 0),
            expired_line(&probe_f2(), Some(SMOKE_EXPIRED_TITLE), 22, 105),
            expired_line(&probe_f1(), None, 0, 0),
            expired_line(&unknown, None, 4, 0),
            crate::contracts_screen::abandon_notice(&probe_f1(), 23),
        ] {
            assert_failure_clean(&text);
        }
    }

    fn failure(text: &str) -> DayReportLine {
        DayReportLine {
            text: format!("{EXPIRED_PREFIX}{text}"),
            notable: true,
        }
    }

    fn section<'a>(sections: &'a [DayReportSection], id: &str) -> Option<&'a DayReportSection> {
        sections.iter().find(|section| section.id == id)
    }

    /// 12.1 #4 (a): 3 failures + Health 2 + Prices 3 + Bounties (Claim ready).
    #[test]
    fn pinned_cap_keeps_three_failures_and_claim_ready() {
        let pinned: Vec<DayReportLine> = (0..3).map(|i| failure(&format!("F{i} - 0/5"))).collect();
        let sections = cap_sections_pinned(
            vec![
                ("deadlines", "Deadlines", vec![]),
                (
                    "health",
                    "Health",
                    vec![
                        line("Healed: Cut hand.", true),
                        line("Healed: Bruise.", true),
                    ],
                ),
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
            ],
            pinned.clone(),
        );
        assert_eq!(sections[0].id, "deadlines");
        assert_eq!(sections[0].lines, pinned);
        let bounties = section(&sections, "bounties").expect("Bounties kept");
        assert!(bounties.lines[0].text.starts_with("Claim ready"));
        assert!(rendered_line_count(&sections) <= LINE_CAP);
        let ids: Vec<&str> = sections.iter().map(|s| s.id).collect();
        assert_eq!(ids, ["deadlines", "health", "prices", "bounties"]);
    }

    /// 12.1 #4 (b): 2 failures + 1 active deadline + Health + Prices + Bounties.
    #[test]
    fn pinned_cap_two_failures_with_an_active_deadline() {
        let pinned: Vec<DayReportLine> = (0..2).map(|i| failure(&format!("F{i} - 0/5"))).collect();
        let sections = cap_sections_pinned(
            vec![
                (
                    "deadlines",
                    "Deadlines",
                    vec![line("Spice charter - 1 day left - 0/8", true)],
                ),
                ("health", "Health", vec![line("Healed: Cut hand.", true)]),
                (
                    "prices",
                    "Prices",
                    (0..4).map(|i| line(&format!("p{i}"), true)).collect(),
                ),
                (
                    "bounties",
                    "Bounties",
                    vec![
                        line("Claim ready: A (1 silver) - open Hunt.", true),
                        line("Bounty claimed: B.", true),
                    ],
                ),
            ],
            pinned.clone(),
        );
        let deadlines = section(&sections, "deadlines").unwrap();
        assert_eq!(&deadlines.lines[..2], pinned.as_slice());
        assert_eq!(deadlines.lines[2].text, "Spice charter - 1 day left - 0/8");
        let bounties = section(&sections, "bounties").expect("Bounties kept");
        assert!(bounties.lines[0].text.starts_with("Claim ready"));
        assert!(section(&sections, "health").is_some());
        assert!(section(&sections, "prices").is_some());
        assert!(rendered_line_count(&sections) <= LINE_CAP);
    }

    /// 12.1 #4 (c): arrival with 1 failure, 2 arrival contracts, 3 movers,
    /// Health, Claim ready: movers then Health yield; failure and Claim stay.
    #[test]
    fn pinned_failure_survives_arrival_yield() {
        let pinned = vec![failure(
            "Famine relief: grain to Corsair's Rest - 5/23 - Silver +60 - 22 Grain still aboard",
        )];
        let movers = smoke_arrival_movers();
        let health = vec![line("Healed: Cut hand.", true)];
        let arrival =
            vec![
            line(
                "Porcelain for Al-Manar estate - 0/5 - 20 days left - sell 5 more Porcelain here",
                true,
            ),
            line("Spice run - 2/8 - 9 days left - sell 6 more Spice here", true),
        ];
        let sections = yield_arrival_rows(movers.clone(), health, |kept, kept_health| {
            let mut lines = arrival.clone();
            lines.extend_from_slice(kept);
            cap_sections_pinned(
                vec![
                    ("arrival", "Arrival", lines),
                    ("deadlines", "Deadlines", vec![]),
                    ("health", "Health", kept_health.to_vec()),
                    (
                        "bounties",
                        "Bounties",
                        vec![line(
                            "Claim ready: Raj the Quiet (120 silver) - open Hunt.",
                            true,
                        )],
                    ),
                ],
                pinned.clone(),
            )
        });
        let ids: Vec<&str> = sections.iter().map(|s| s.id).collect();
        assert_eq!(
            ids[..2],
            ["arrival", "deadlines"],
            "failure heads Deadlines after Arrival"
        );
        assert_eq!(section(&sections, "deadlines").unwrap().lines, pinned);
        assert!(
            kept_movers(&sections, &movers).is_empty(),
            "movers yield first"
        );
        assert!(section(&sections, "health").is_none(), "Health yields next");
        let bounties = section(&sections, "bounties").expect("Claim ready kept");
        assert!(bounties.lines[0].text.starts_with("Claim ready"));
        assert_eq!(section(&sections, "arrival").unwrap().lines, arrival);
    }

    /// 12.1 #4 (d) + section 6.6: a lone failure opens the card on a quiet day.
    #[test]
    fn lone_failure_is_notable() {
        let sections = cap_sections_pinned(
            vec![
                ("deadlines", "Deadlines", vec![]),
                (
                    "health",
                    "Health",
                    vec![line("Healing: Bruised ribs (3 days left).", false)],
                ),
            ],
            vec![failure("Grain to Corsair's Rest - 0/23")],
        );
        let doc = DayReportDocument {
            day: 21,
            title: "Day 21".into(),
            sections,
            footer: DayReportFooter::default(),
        };
        assert!(doc.has_notable());
        assert_eq!(doc.sections[0].id, "deadlines");
        // The quiet non-notable rest still renders under the failure.
        assert_eq!(doc.sections.len(), 2);
        // No pinned lines: exactly cap_sections (a quiet day stays closed).
        assert!(cap_sections_pinned(
            vec![(
                "health",
                "Health",
                vec![line("Healing: Bruised ribs (3 days left).", false)],
            )],
            Vec::new(),
        )
        .is_empty());
    }

    /// 12.1 #5: failures sit above active lines; the arrival exclude never
    /// drops a failure; deadline_lines no longer emits the old `Expired:`.
    #[test]
    fn failures_lead_deadlines_and_the_old_expired_line_is_gone() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        let day = session.world().day;
        let active = test_active("Grain run", "al_manar", "grain", 0, 10, day + 2, "a1");
        let deadlines = deadline_lines(std::slice::from_ref(&active), day, session.world(), None);
        assert!(deadlines.iter().all(|l| !l.text.starts_with("Expired")));
        assert!(deadlines.iter().all(|l| !is_failure_line(&l.text)));
        let pinned = vec![failure("Grain to Corsair's Rest - 0/23")];
        let sections = cap_sections_pinned(
            vec![("deadlines", "Deadlines", deadlines.clone())],
            pinned.clone(),
        );
        assert_eq!(sections[0].lines[0], pinned[0]);
        assert_eq!(sections[0].lines[1], deadlines[0]);
        // Arrival exclude removes the docked port's active line, never a failure.
        let excluded = deadline_lines(&[active], day, session.world(), Some("al_manar"));
        assert!(excluded.is_empty());
        let sections = cap_sections_pinned(
            vec![
                (
                    "arrival",
                    "Arrival",
                    vec![line("Grain run - 0/10 - 2 days left", true)],
                ),
                ("deadlines", "Deadlines", excluded),
            ],
            pinned.clone(),
        );
        assert_eq!(section(&sections, "deadlines").unwrap().lines, pinned);
    }

    /// 12.1 #6: footer cost counts a long failure line as 2 rows and the
    /// footer yields on the probe F2 card plus 3 other lines.
    #[test]
    fn failure_line_counts_in_footer_cost() {
        let f2 = DayReportLine {
            text: expired_line(&probe_f2(), Some(SMOKE_EXPIRED_TITLE), 22, 0),
            notable: true,
        };
        assert!(f2.text.chars().count() > FOOTER_WRAP_CHARS);
        let lone = cap_sections_pinned(vec![], vec![f2.clone()]);
        assert_eq!(footer_body_cost(&lone), 3);
        let busy = cap_sections_pinned(
            vec![
                (
                    "deadlines",
                    "Deadlines",
                    vec![line("Spice charter - 2 days left - 0/8", true)],
                ),
                ("health", "Health", vec![line("Healed: Cut hand.", true)]),
                (
                    "prices",
                    "Prices",
                    vec![line("Grain at Porto Novo 18 to 21 (+17%)", true)],
                ),
            ],
            vec![f2],
        );
        let footer = DayReportFooter {
            week: Some("Week: +57 silver".into()),
            next: None,
        };
        assert!(footer_body_cost(&busy) > FOOTER_BODY_BUDGET);
        assert!(!footer_should_show(&footer, &busy));
    }

    fn policy(id: &str, target: &str) -> ActivePolicy {
        ActivePolicy {
            id: id.into(),
            spec_id: "contract_basic".into(),
            family: "contract_guarantee".into(),
            scope: "named_contract".into(),
            purchased_day: 1,
            coverage_pct: 0.5,
            coverage_cap: 400,
            premium_paid: 20,
            target_id: target.into(),
            claims_made: 0,
            total_paid_out: 0,
            active: true,
            voyage_origin: String::new(),
            voyage_destination: String::new(),
        }
    }

    fn claim(policy_id: &str, incident: &str, payout: i64, denied: bool) -> InsuranceClaim {
        InsuranceClaim {
            policy_id: policy_id.into(),
            day: 20,
            incident_type: incident.into(),
            loss_value: 210,
            payout,
            denied,
            denial_reason: String::new(),
        }
    }

    /// 12.1 #7: only this advance's paid, undenied contract_failure claims on
    /// a policy that targets this contract.
    #[test]
    fn guarantee_matches_only_this_contract_and_this_advance() {
        let infra = InfrastructureRecord {
            policies: vec![policy("p1", "c1"), policy("p2", "c2")],
            claims: vec![
                claim("p1", "contract_failure", 50, false), // before the advance
                claim("p1", "contract_failure", 105, false),
                claim("p2", "contract_failure", 65, false), // another contract
                claim("p1", "storm", 30, false),
                claim("p1", "contract_failure", 40, true),
                claim("p1", "contract_failure", 0, false),
                claim("ghost", "contract_failure", 99, false),
            ],
            ..InfrastructureRecord::default()
        };
        assert_eq!(guarantee_in(&infra, 1, "c1"), 105);
        assert_eq!(guarantee_in(&infra, 0, "c1"), 155);
        assert_eq!(guarantee_in(&infra, 1, "c2"), 65);
        assert_eq!(guarantee_in(&infra, 1, "c3"), 0);
        assert_eq!(guarantee_in(&infra, 7, "c1"), 0);
        // Recent rows: the whole list, same filters.
        assert_eq!(guarantee_total(&infra, "c1"), 155);
        assert_eq!(guarantee_total(&infra, "c2"), 65);
        assert_eq!(guarantee_total(&infra, "c3"), 0);
    }

    /// 12.1 #8: the tone predicate matches only the expiry prefix, and no
    /// existing smoke document line matches it.
    #[test]
    fn failure_tone_predicate_is_prefix_only() {
        assert!(is_failure_line(
            "Contract expired: Grain to Corsair's Rest - 0/23"
        ));
        for text in [
            "Expired: Contract defaulted: failed to deliver grain",
            "Contract paid: Silver +612 - 23 Grain to Corsair's Rest",
            "Contract abandoned: Grain for Corsair's Rest",
            "Spice charter - 1 day left - 10/10",
            "",
        ] {
            assert!(!is_failure_line(text), "{text}");
        }
        for doc in [
            smoke_full_document(3),
            smoke_deadline_document(4),
            smoke_arrival_document(5),
        ] {
            assert!(doc
                .sections
                .iter()
                .flat_map(|s| s.lines.iter())
                .all(|l| !is_failure_line(&l.text)));
        }
    }

    /// Real Session: seed-1 Famine relief lapses on the advance that starts on
    /// day 20 (grace day). One line, from the snapshot title, no Trust copy.
    /// Then the same with `contract_basic`: the Guarantee term (+105).
    #[test]
    fn failure_lines_from_a_real_seed_one_expiry() {
        for insured in [false, true] {
            let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
            let offer = session
                .available_contracts()
                .into_iter()
                .find(|offer| {
                    offer.destination_port_id == "corsairs_rest" && offer.good_id == "grain"
                })
                .expect("seed-1 Famine relief");
            assert_eq!(offer.deadline_day, 19);
            session.accept_contract(&offer.id).unwrap();
            if insured {
                session
                    .buy_insurance("contract_basic", &offer.id, "", "")
                    .unwrap();
            }
            let mut lines = Vec::new();
            while session.world().day <= 20 {
                let facts = FailureFacts::snapshot(&session);
                let day = session.world().day;
                let turn = session.advance().unwrap();
                lines = failure_lines(&session, &turn.contracts, &facts);
                if day < 20 {
                    assert!(lines.is_empty(), "day {day}: {lines:?}");
                    assert!(session
                        .board()
                        .active
                        .iter()
                        .any(|c| c.offer_id == offer.id));
                }
            }
            assert_eq!(session.world().day, 21);
            let want = if insured {
                "Contract expired: Famine relief: grain to Corsair's Rest - 0/23 - Guarantee +105"
            } else {
                "Contract expired: Famine relief: grain to Corsair's Rest - 0/23"
            };
            assert_eq!(lines.len(), 1);
            assert_eq!(lines[0].text, want);
            assert!(lines[0].notable);
            assert_failure_clean(&lines[0].text);
            // The persisted outcome still carries the record-only numbers.
            let last = session.board().completed.last().unwrap();
            assert_eq!(last.outcome_type, "expired");
            assert_eq!(last.trust_delta, -3);
        }
    }

    /// Sort: deadline, then title, then contract id. Abandon outcomes in the
    /// slice are never failure lines (only `expired`).
    #[test]
    fn failure_lines_sort_and_ignore_other_kinds() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        let mut late = probe_f1();
        late.contract_id = "z".into();
        late.deadline_day = 20;
        let mut b = probe_f1();
        b.contract_id = "b".into();
        let mut a = probe_f1();
        a.contract_id = "a".into();
        let mut abandoned = probe_f1();
        abandoned.outcome_type = "abandoned".into();
        let facts = FailureFacts {
            titles_before: [
                ("z".to_string(), "Alpha run".to_string()),
                ("b".to_string(), "Beta run".to_string()),
                ("a".to_string(), "Beta run".to_string()),
            ]
            .into_iter()
            .collect(),
            claims_before: 0,
        };
        let lines = failure_lines(&session, &[late, abandoned, b, a], &facts);
        let texts: Vec<&str> = lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(
            texts,
            [
                "Contract expired: Beta run - 0/23",
                "Contract expired: Beta run - 0/23",
                "Contract expired: Alpha run - 0/23",
            ]
        );
    }

    /// Frame doc: F2 line pinned (DUE) above the live Spice deadline line
    /// (seed 1: Famine relief + Spice restock accepted, idle to day 23),
    /// Week + live Next fit under the existing footer rule.
    #[test]
    fn smoke_expired_doc_follows_formatters() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        let picks: Vec<String> = session
            .available_contracts()
            .into_iter()
            .filter(|offer| {
                (offer.destination_port_id == "corsairs_rest" && offer.good_id == "grain")
                    || (offer.destination_port_id == "al_manar" && offer.good_id == "spice")
            })
            .map(|offer| offer.id)
            .collect();
        assert_eq!(picks.len(), 2);
        for id in &picks {
            session.accept_contract(id).unwrap();
        }
        while session.world().day < SMOKE_EXPIRED_FRAME_DAY {
            session.advance().unwrap();
        }
        let insured = format!("{EXPIRED_PREFIX}{SMOKE_EXPIRED_TITLE} - 0/23 - Guarantee +105");
        let doc = smoke_expired_document(&session, &insured);
        assert_eq!(doc.title, "Day 23");
        assert!(doc.has_notable());
        let texts: Vec<&str> = doc.sections[0]
            .lines
            .iter()
            .map(|l| l.text.as_str())
            .collect();
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(
            texts,
            [
                "Contract expired: Famine relief: grain to Corsair's Rest - 0/23 - Guarantee +105",
                "Spice restock run to Al-Manar - 3 days left - 0/8 - sell 8 more Spice at Al-Manar",
            ]
        );
        assert!(is_failure_line(texts[0]) && !is_failure_line(texts[1]));
        assert_eq!(doc.footer.week.as_deref(), Some("Week: +57 silver"));
        assert_eq!(
            doc.footer.next.as_deref(),
            Some("Next: Contract 3 days left - open Contracts.")
        );
        assert!(footer_should_show(&doc.footer, &doc.sections));
        for text in texts {
            assert_failure_clean(text);
        }
    }

    /// GD OQ2: the overdue rung names the docked-sale port from the session.
    #[test]
    fn next_facts_names_the_overdue_port() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        let offer = session
            .available_contracts()
            .into_iter()
            .find(|offer| offer.destination_port_id == "corsairs_rest" && offer.good_id == "grain")
            .unwrap();
        session.accept_contract(&offer.id).unwrap();
        while session.world().day < 20 {
            session.advance().unwrap();
        }
        let facts = next_facts(&session);
        assert_eq!(facts.overdue_port.as_deref(), Some("Corsair's Rest"));
        assert_eq!(
            next_line(&facts).as_deref(),
            Some("Next: Contract overdue - sell at Corsair's Rest.")
        );
    }
}
