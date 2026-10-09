//! Read-only journal. Chronicle, victory paths, milestones, and captain memories.
//!
//! The overlay reads [`Session`] and does not call `evaluate_consequences`,
//! `remember_captain`, or any ledger merge. Beat titles are a Godot copy of
//! `narrative.rs` `BEATS` (see [`BEATS`]). A public `narrative::beat(id)`
//! lookup would replace that table.

use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::classes::text_server::AutowrapMode;
use godot::classes::{
    Button, HBoxContainer, Label, Node, PanelContainer, ScrollContainer, StyleBoxFlat,
    VBoxContainer,
};
use godot::prelude::*;
use portlight_sim::model::{CaptainMemory, JournalEntry, VoyageStatus};
use portlight_sim::session::Session;
use portlight_sim::{campaign::HouseBooks, content};

use crate::encounter_screen::{self, style_encounter_button};
use crate::logic::{display_or_humanized, encounter_end_name};

const INK: Color = Color::from_rgb(0.08, 0.11, 0.16);
const CREAM: Color = Color::from_rgb(0.94, 0.91, 0.84);
const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);

#[derive(Clone)]
pub(crate) struct JournalNodes {
    pub root: Gd<PanelContainer>,
    pub title: Gd<Label>,
    pub notice: Gd<Label>,
    pub scroll: Gd<ScrollContainer>,
    pub body: Gd<VBoxContainer>,
    pub plate: Gd<godot::classes::TextureRect>,
    pub plate_panel: Gd<PanelContainer>,
    pub plate_caption: Gd<Label>,
    pub placeholder: Gd<PanelContainer>,
    pub close: Gd<Button>,
    pub chronicle: Gd<VBoxContainer>,
    pub victory: Gd<VBoxContainer>,
    pub memories: Gd<VBoxContainer>,
}

pub(crate) fn build_journal_screen() -> JournalNodes {
    let mut root = PanelContainer::new_alloc();
    root.set_name("JournalScreen");
    root.set_mouse_filter(MouseFilter::STOP);
    root.set_visible(false);
    let mut style = StyleBoxFlat::new_gd();
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

    column.add_child(&text_label("Journal", 14, MUTED, false));
    let mut title = text_label("", 28, GOLD, false);
    // The playtest Journal lens reads these by name.
    title.set_name("JournalTitle");
    column.add_child(&title);
    let mut notice = text_label("", 16, CREAM, true);
    notice.set_name("JournalNotice");
    column.add_child(&notice);

    let mut scroll = ScrollContainer::new_alloc();
    scroll.set_name("JournalScroll");
    scroll.set_h_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_v_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_horizontal_scroll_mode(godot::classes::scroll_container::ScrollMode::DISABLED);
    let mut body = VBoxContainer::new_alloc();
    body.set_name("JournalBody");
    body.set_h_size_flags(SizeFlags::EXPAND_FILL);
    body.add_theme_constant_override("separation", 16);
    let chronicle = section_box("SectionChronicle");
    let victory = section_box("SectionVictory");
    let milestones = section_box("SectionMilestones");
    let memories = section_box("SectionMemories");
    let festivals = section_box("SectionFestivals");
    let day_log = section_box("SectionDayLog");
    body.add_child(&chronicle);
    body.add_child(&victory);
    body.add_child(&milestones);
    body.add_child(&memories);
    body.add_child(&festivals);
    body.add_child(&day_log);
    scroll.add_child(&body);
    column.add_child(&scroll);

    let mut close = Button::new_alloc();
    close.set_name("CloseJournal");
    close.set_text("Close");
    style_encounter_button(&mut close);
    column.add_child(&close);

    JournalNodes {
        root,
        title,
        notice,
        scroll,
        body,
        plate,
        plate_panel,
        plate_caption,
        placeholder,
        close,
        chronicle,
        victory,
        memories,
    }
}

/// Full-rect overlay. Call after the node has a parent.
pub(crate) fn fill_parent(root: &mut Gd<PanelContainer>) {
    root.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
}

pub(crate) fn set_open(nodes: &mut JournalNodes, open: bool) {
    nodes.root.set_visible(open);
    nodes.root.set_mouse_filter(if open {
        MouseFilter::STOP
    } else {
        MouseFilter::IGNORE
    });
}

pub(crate) fn apply_document(
    nodes: &mut JournalNodes,
    doc: &JournalDocument,
    mut beat_button: impl FnMut(&str, &str) -> Gd<Button>,
) {
    nodes.title.set_text(&doc.title);
    nodes.notice.set_text(&doc.notice);
    let body = nodes.body.clone();
    for section_box in body.get_children().iter_shared() {
        let Ok(mut section_box) = section_box.try_cast::<VBoxContainer>() else {
            continue;
        };
        let id = section_box.get_name().to_string();
        let section_id = match id.as_str() {
            "SectionChronicle" => "chronicle",
            "SectionVictory" => "victory",
            "SectionMilestones" => "milestones",
            "SectionMemories" => "memories",
            "SectionFestivals" => "festivals",
            "SectionDayLog" => "day_log",
            _ => continue,
        };
        fill_section(
            &mut section_box,
            doc.sections.iter().find(|section| section.id == section_id),
            &mut beat_button,
        );
    }
}

pub(crate) fn scroll_to(scroll: &mut Gd<ScrollContainer>, anchor: &Gd<VBoxContainer>) {
    let y = anchor.get_position().y.round() as i32;
    scroll.set_v_scroll(y.max(0));
}

pub(crate) fn section_visible(scroll: &Gd<ScrollContainer>, anchor: &Gd<VBoxContainer>) -> bool {
    band_visible(scroll.get_global_rect(), anchor.get_global_rect())
}

pub(crate) fn line_visible(
    scroll: &Gd<ScrollContainer>,
    anchor: &Gd<VBoxContainer>,
    needle: &str,
) -> bool {
    let view = scroll.get_global_rect();
    label_visible(&anchor.clone().upcast::<Node>(), view, needle)
}

pub(crate) fn overlay_text(root: &Gd<PanelContainer>) -> String {
    let mut parts = Vec::new();
    collect_text(&root.clone().upcast::<Node>(), &mut parts);
    parts.join("\n")
}

fn fill_section(
    section_box: &mut Gd<VBoxContainer>,
    section: Option<&JournalSection>,
    beat_button: &mut impl FnMut(&str, &str) -> Gd<Button>,
) {
    clear_children(section_box);
    let Some(section) = section else {
        section_box.set_visible(false);
        return;
    };
    section_box.set_visible(true);
    section_box.add_child(&text_label(section.heading, 16, GOLD, false));
    for row in &section.rows {
        match row {
            JournalRow::Line { text, muted } => {
                let (size, color) = if *muted { (14, MUTED) } else { (16, CREAM) };
                section_box.add_child(&text_label(text, size, color, true));
            }
            JournalRow::Beat { id, label } => {
                section_box.add_child(&beat_button(id, label));
            }
        }
    }
}

fn section_box(name: &str) -> Gd<VBoxContainer> {
    let mut section = VBoxContainer::new_alloc();
    section.set_name(name);
    section.set_h_size_flags(SizeFlags::EXPAND_FILL);
    section.add_theme_constant_override("separation", 4);
    section
}

fn text_label(text: &str, size: i32, color: Color, wrap: bool) -> Gd<Label> {
    let mut label = Label::new_alloc();
    label.set_text(text);
    label.add_theme_font_size_override("font_size", size);
    label.add_theme_color_override("font_color", color);
    label.set_h_size_flags(SizeFlags::EXPAND_FILL);
    if wrap {
        // Autowrap without clip-text. Clip plus autowrap collapses the label
        // to 1×1 in Godot 4.7 (`label.cpp` autowrap clip branch).
        label.set_autowrap_mode(AutowrapMode::WORD_SMART);
        label.set_clip_text(false);
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

fn collect_text(node: &Gd<Node>, parts: &mut Vec<String>) {
    if let Ok(label) = node.clone().try_cast::<Label>() {
        if label.is_visible_in_tree() {
            let text = label.get_text().to_string();
            if !text.is_empty() {
                parts.push(text);
            }
        }
    }
    if let Ok(button) = node.clone().try_cast::<Button>() {
        if button.is_visible_in_tree() {
            let text = button.get_text().to_string();
            if !text.is_empty() {
                parts.push(text);
            }
        }
    }
    for child in node.get_children().iter_shared() {
        collect_text(&child, parts);
    }
}

fn label_visible(node: &Gd<Node>, view: Rect2, needle: &str) -> bool {
    if let Ok(label) = node.clone().try_cast::<Label>() {
        if label.get_text().to_string().contains(needle)
            && band_visible(view, label.get_global_rect())
        {
            return true;
        }
    }
    if let Ok(button) = node.clone().try_cast::<Button>() {
        if button.get_text().to_string().contains(needle)
            && band_visible(view, button.get_global_rect())
        {
            return true;
        }
    }
    for child in node.get_children().iter_shared() {
        if label_visible(&child, view, needle) {
            return true;
        }
    }
    false
}

fn band_visible(view: Rect2, rect: Rect2) -> bool {
    if view.size.y < 8.0 || rect.size.y < 8.0 {
        return false;
    }
    let top = rect.position.y;
    let bottom = top + rect.size.y.min(48.0);
    let view_top = view.position.y;
    let view_bottom = view_top + view.size.y;
    bottom > view_top + 4.0 && top < view_bottom - 4.0
}

pub(crate) struct JournalDocument {
    pub title: String,
    pub notice: String,
    pub sections: Vec<JournalSection>,
}

pub(crate) struct JournalSection {
    pub id: &'static str,
    pub heading: &'static str,
    pub rows: Vec<JournalRow>,
}

pub(crate) enum JournalRow {
    Line { text: String, muted: bool },
    Beat { id: String, label: String },
}

pub(crate) fn document_lines(doc: &JournalDocument) -> Vec<String> {
    let mut lines = vec![doc.title.clone(), doc.notice.clone()];
    for section in &doc.sections {
        lines.push(section.heading.to_string());
        for row in &section.rows {
            lines.push(match row {
                JournalRow::Line { text, .. } => text.clone(),
                JournalRow::Beat { label, .. } => label.clone(),
            });
        }
    }
    lines
}

/// Reads narrative, victory, books, memories, and culture. `log_lines` is the
/// chart log already kept by the view. `expanded` lists beat ids whose text
/// is open. Nothing here writes the session.
pub(crate) fn journal_document(
    session: &Session,
    log_lines: &[String],
    expanded: &[String],
) -> JournalDocument {
    let mut sections = vec![
        JournalSection {
            id: "chronicle",
            heading: "Chronicle",
            rows: chronicle_rows(session, expanded),
        },
        JournalSection {
            id: "victory",
            heading: "Victory",
            rows: victory_rows(session),
        },
        JournalSection {
            id: "milestones",
            heading: "Milestones",
            rows: milestone_rows(session.books()),
        },
        JournalSection {
            id: "memories",
            heading: "Memories",
            rows: memories_rows(&session.world().captain_memories),
        },
    ];
    let festivals = festival_rows(session);
    if !festivals.is_empty() {
        sections.push(JournalSection {
            id: "festivals",
            heading: "Festivals",
            rows: festivals,
        });
    }
    let day_log = day_log_rows(log_lines);
    if !day_log.is_empty() {
        sections.push(JournalSection {
            id: "day_log",
            heading: "Day log",
            rows: day_log,
        });
    }
    JournalDocument {
        title: journal_title(session),
        notice: journal_notice(session),
        sections,
    }
}

fn chronicle_rows(session: &Session, expanded: &[String]) -> Vec<JournalRow> {
    let narrative = session.narrative();
    let mut rows = Vec::new();
    if narrative.journal.is_empty() {
        rows.push(line("No chronicle entries yet.", false));
    } else {
        for entry in narrative.journal.iter().rev() {
            let place = entry_place(session, entry);
            let open = expanded.iter().any(|id| id == &entry.beat_id);
            rows.extend(chronicle_entry_rows(entry, place.as_deref(), open));
        }
    }
    rows.push(line(
        format!("Fired beats: {}", narrative.fired.len()),
        true,
    ));
    rows
}

fn chronicle_entry_rows(
    entry: &JournalEntry,
    place: Option<&str>,
    expanded: bool,
) -> Vec<JournalRow> {
    let mut rows = vec![JournalRow::Beat {
        id: entry.beat_id.clone(),
        label: format!("Day {} - {}", entry.day, beat_title(&entry.beat_id)),
    }];
    if let Some(place) = place {
        if !place.is_empty() {
            rows.push(line(place, true));
        }
    }
    if expanded {
        if let Some(text) = beat_text(&entry.beat_id) {
            rows.push(line(text, false));
        }
    }
    rows
}

fn entry_place(session: &Session, entry: &JournalEntry) -> Option<String> {
    let port = if entry.port_id.is_empty() {
        String::new()
    } else {
        port_label(session.world(), &entry.port_id)
    };
    let region = fold_ascii(&entry.region);
    match (port.is_empty(), region.is_empty()) {
        (true, true) => None,
        (false, true) => Some(port),
        (true, false) => Some(region),
        (false, false) => Some(format!("{port}, {region}")),
    }
}

fn victory_rows(session: &Session) -> Vec<JournalRow> {
    let paths = session.victory();
    let mut rows = Vec::new();
    for path in &paths {
        rows.push(line(fold_ascii(&path.name), false));
        rows.push(line(
            format!("Strength {:.1}", path.candidate_strength),
            true,
        ));
        if path.completion_day > 0 {
            let summary = fold_ascii(&path.completion_summary);
            if summary.is_empty() {
                rows.push(line(
                    format!("Completed day {}.", path.completion_day),
                    false,
                ));
            } else {
                rows.push(line(
                    format!("Completed day {}. {summary}", path.completion_day),
                    false,
                ));
            }
        }
        for req in &path.requirements {
            rows.push(line(
                format!(
                    "[{}] {}",
                    fold_ascii(&req.status),
                    fold_ascii(&req.description)
                ),
                false,
            ));
            let detail = fold_ascii(&req.detail);
            let action = fold_ascii(&req.action);
            let extra = match (detail.is_empty(), action.is_empty()) {
                (true, true) => None,
                (false, true) => Some(detail),
                (true, false) => Some(action),
                (false, false) => Some(format!("{detail}. {action}")),
            };
            if let Some(extra) = extra {
                rows.push(line(extra, true));
            }
        }
    }
    let books = session.books();
    if !books.completed_paths.is_empty() {
        rows.push(line("Completed", false));
        for record in &books.completed_paths {
            let name = paths
                .iter()
                .find(|path| path.path_id == record.path_id)
                .map(|path| fold_ascii(&path.name))
                .unwrap_or_else(|| path_name(&record.path_id));
            rows.push(line(
                format!("{name}  day {}", record.completion_day),
                false,
            ));
            let summary = fold_ascii(&record.summary);
            if !summary.is_empty() {
                rows.push(line(summary, true));
            }
            if record.is_first {
                rows.push(line("First path.", true));
            }
        }
    }
    rows
}

fn milestone_rows(books: &HouseBooks) -> Vec<JournalRow> {
    if books.completed_milestones.is_empty() {
        return vec![line("No milestones recorded.", false)];
    }
    let mut rows = Vec::new();
    for milestone in &books.completed_milestones {
        rows.push(line(
            format!(
                "{}  day {}",
                fold_ascii(&milestone.milestone_id),
                milestone.completed_day
            ),
            false,
        ));
        let evidence = fold_ascii(&milestone.evidence);
        if !evidence.is_empty() {
            rows.push(line(evidence, true));
        }
    }
    rows
}

fn memories_rows(memories: &[CaptainMemory]) -> Vec<JournalRow> {
    if memories.is_empty() {
        return vec![line("No captains remembered yet.", false)];
    }
    let mut rows = Vec::new();
    for memory in memories {
        rows.extend(memory_rows(memory));
    }
    rows
}

fn memory_rows(memory: &CaptainMemory) -> Vec<JournalRow> {
    let mut rows = vec![line(pirate_name(&memory.captain_id), false)];
    let region = fold_ascii(&memory.last_seen_region);
    if region.is_empty() {
        rows.push(line(
            format!("Last seen day {}.", memory.last_seen_day),
            true,
        ));
    } else {
        rows.push(line(
            format!("Last seen {region}, day {}.", memory.last_seen_day),
            true,
        ));
    }
    rows.push(line(
        format!(
            "Spared {}. Defeated them {}. They defeated you {}.",
            memory.times_spared, memory.times_defeated_by_player, memory.times_defeated_player
        ),
        true,
    ));
    let rel = &memory.relationship;
    rows.push(line(
        format!(
            "Respect {}. Fear {}. Grudge {}. Familiarity {}.",
            rel.respect, rel.fear, rel.grudge, rel.familiarity
        ),
        true,
    ));
    rows
}

fn festival_rows(session: &Session) -> Vec<JournalRow> {
    session
        .world()
        .culture
        .active_festivals
        .iter()
        .map(|festival| line(festival_name(&festival.festival_id), false))
        .collect()
}

fn day_log_rows(log_lines: &[String]) -> Vec<JournalRow> {
    log_lines
        .iter()
        .map(|line| fold_ascii(line))
        .filter(|line| !line.is_empty())
        .map(|line| JournalRow::Line {
            text: line,
            muted: false,
        })
        .collect()
}

fn journal_title(session: &Session) -> String {
    let name = session.world().captain.name.trim();
    if !name.is_empty() && name.is_ascii() {
        name.to_string()
    } else {
        format!("Day {}", session.world().day)
    }
}

fn journal_notice(session: &Session) -> String {
    let world = session.world();
    let day = world.day;
    match world.voyage.status {
        VoyageStatus::AtSea => format!("Day {day}. At sea."),
        VoyageStatus::Arrived => format!(
            "Day {day}. Arrived at {}.",
            port_label(world, &world.voyage.destination_id)
        ),
        VoyageStatus::InPort => format!(
            "Day {day}. Docked at {}.",
            port_label(world, &world.voyage.destination_id)
        ),
    }
}

fn port_label(world: &portlight_sim::model::World, id: &str) -> String {
    fold_ascii(&display_or_humanized(
        world.port(id).map(|port| port.name.as_str()),
        id,
    ))
}

fn pirate_name(id: &str) -> String {
    encounter_end_name(id)
}

fn festival_name(id: &str) -> String {
    for region in &content::content().culture.regions {
        if let Some(festival) = region.festivals.iter().find(|festival| festival.id == id) {
            if !festival.name.is_empty() && festival.name.is_ascii() {
                return festival.name.clone();
            }
            return id.to_string();
        }
    }
    id.to_string()
}

fn path_name(id: &str) -> String {
    match id {
        "lawful_house" => "Lawful Trade House".to_string(),
        "shadow_network" => "Shadow Network".to_string(),
        "oceanic_reach" => "Oceanic Reach".to_string(),
        "commercial_empire" => "Commercial Empire".to_string(),
        other => fold_ascii(&other.replace('_', " ")),
    }
}

fn line(text: impl Into<String>, muted: bool) -> JournalRow {
    JournalRow::Line {
        text: text.into(),
        muted,
    }
}

pub(crate) fn beat_title(id: &str) -> String {
    BEATS
        .iter()
        .find(|beat| beat.id == id)
        .map(|beat| beat.title.to_string())
        .unwrap_or_else(|| id.replace('_', " "))
}

fn beat_text(id: &str) -> Option<String> {
    BEATS
        .iter()
        .find(|beat| beat.id == id)
        .map(|beat| beat.text.to_string())
}

/// Session strings are shown as stored, with non-ASCII punctuation folded so
/// the overlay stays ASCII. An em dash becomes a hyphen. Words stay.
fn fold_ascii(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        match ch {
            '\u{2014}' | '\u{2013}' => out.push('-'),
            '\u{2018}' | '\u{2019}' => out.push('\''),
            '\u{201C}' | '\u{201D}' => out.push('"'),
            c if c.is_ascii() => out.push(c),
            _ => {}
        }
    }
    out
}

struct BeatCopy {
    id: &'static str,
    title: &'static str,
    text: &'static str,
}

/// Titles and texts copied from `crates/portlight-sim/src/narrative.rs` `BEATS`
/// at `2d3d8b32b7806b547ea101fb28e0ddb0388dcd26`. Em dashes in that catalog
/// are ASCII hyphens here. Follow-on: public `narrative::beat(id)` should
/// replace this table so Godot does not keep a second copy.
const BEATS: &[BeatCopy] = &[
    BeatCopy {
        id: "first_trade",
        title: "The First Deal",
        text: "You count the silver from your first sale. It's not much, but it's yours. Every fortune in history started with a single trade.",
    },
    BeatCopy {
        id: "first_voyage",
        title: "Into Open Water",
        text: "The harbor shrinks behind you. The wind fills your sails and the crew settles into their watches. Whatever happens next, you've left the dock.",
    },
    BeatCopy {
        id: "first_profit",
        title: "Profit and Promise",
        text: "Your ledger shows a profit for the first time. The crew notices - a captain who can turn silver gets loyalty that gold can't buy.",
    },
    BeatCopy {
        id: "new_region",
        title: "Strange Waters",
        text: "The flags are unfamiliar. The language changes. The goods on the docks are things you've only heard about in tavern stories. You've crossed into a new world.",
    },
    BeatCopy {
        id: "first_contract",
        title: "A Binding Word",
        text: "You sign your name on a contract for the first time. The obligation weighs heavier than any cargo. Deliver on time, and doors open. Fail, and they close.",
    },
    BeatCopy {
        id: "ship_upgrade",
        title: "A Bigger Ship",
        text: "The new ship sits heavy in the water, her hold cavernous compared to your old sloop. Routes that were suicide runs become trade routes. The game just changed.",
    },
    BeatCopy {
        id: "survived_storm",
        title: "Through the Tempest",
        text: "The storm broke three spars and swept a man overboard. But you held the wheel, and the ship held together. The crew will never forget this night.",
    },
    BeatCopy {
        id: "survived_pirates",
        title: "Blood in the Water",
        text: "Pirates spotted your cargo and gave chase. Whether by speed, guile, or luck, you kept your goods and your life. Not everyone on these waters can say the same.",
    },
    BeatCopy {
        id: "first_inspection",
        title: "The Customs Man",
        text: "An inspector boards your vessel, ledger in hand. His eyes miss nothing. The nature of your cargo and the cleanness of your record decide what happens next.",
    },
    BeatCopy {
        id: "rival_encounter",
        title: "A Familiar Sail",
        text: "You spot a ship you recognize - another captain working the same routes, chasing the same margins. The sea is big, but the profitable corners of it aren't.",
    },
    BeatCopy {
        id: "mentor_wisdom",
        title: "Words from an Old Hand",
        text: "An old captain shares a drink with you in a portside tavern. \\\"The sea doesn't care about your plans,\\\" he says. \\\"She only respects the captains who listen.\\\"",
    },
    BeatCopy {
        id: "cargo_seized",
        title: "Seized",
        text: "They took your cargo. Every crate, inspected and confiscated. Your crew watches in silence as months of work vanish into a customs warehouse. The question isn't whether you'll recover. It's whether you'll try.",
    },
    BeatCopy {
        id: "near_bankruptcy",
        title: "The Abyss",
        text: "Silver: almost nothing. Provisions: running low. The crew looks at you with eyes that ask whether this is the end. Every great merchant hit bottom once. The difference is what they did next.",
    },
    BeatCopy {
        id: "contract_failed",
        title: "Broken Promise",
        text: "The deadline passed. The goods never arrived. Your name is mud at the exchange, and the trust you built evaporates like morning fog. Reputation is the hardest thing to rebuild.",
    },
    BeatCopy {
        id: "first_big_contract",
        title: "The Big Score",
        text: "A contract worth more than everything you've earned so far. The kind of deal that turns a trader into a merchant house. All those small runs were preparation for this moment.",
    },
    BeatCopy {
        id: "east_indies_arrival",
        title: "The Spice Quarter",
        text: "The East Indies. Every merchant's dream, every navigator's test. The air smells of spice and possibility. Silk and porcelain fill warehouses that stretch to the horizon. You've arrived.",
    },
    BeatCopy {
        id: "south_seas_discovery",
        title: "Beyond the Charts",
        text: "The South Seas. Your charts have blank spaces here. Pearls gleam in the shallows, volcanic islands smoke on the horizon, and kings you've never heard of trade in goods the Old World craves. This is the frontier.",
    },
    BeatCopy {
        id: "wealth_milestone",
        title: "A Captain of Substance",
        text: "Your silver reserves have crossed a line that separates traders from merchants. Ships, warehouses, contracts - you're no longer surviving. You're building.",
    },
    BeatCopy {
        id: "trade_house",
        title: "The House You Built",
        text: "Brokers in three regions know your name. Warehouses hold your goods in ports you haven't visited in weeks. Contracts arrive without you asking. You didn't just trade - you built something that will outlast you.",
    },
    BeatCopy {
        id: "galleon_master",
        title: "Master of the Long Haul",
        text: "Your galleon cuts through waters that would sink lesser ships. Routes that terrified you as a sloop captain are now your daily bread. The sea hasn't changed. You have.",
    },
    BeatCopy {
        id: "five_regions",
        title: "The Known World",
        text: "You've traded in every region the maps can show. From the Mediterranean to the South Seas, from the North Atlantic to the East Indies. Few captains can say they've seen it all. You can.",
    },
    BeatCopy {
        id: "cultural_awakening",
        title: "More Than Ledgers",
        text: "The world is bigger than your ledger. Every port has a story older than your ship. Every good you carry means something to someone beyond its price.",
    },
    BeatCopy {
        id: "festival_trader",
        title: "Festival Fortune",
        text: "The market swells with festival crowds. Prices soar, competition is fierce, and the locals remember who traded fairly during the celebration. Commerce and culture are the same thing here.",
    },
    BeatCopy {
        id: "sacred_cargo",
        title: "What They Revere",
        text: "You carry what they revere. Handle it with care - this cargo is worth more than silver to the people who receive it. Your standing grows not because you traded well, but because you traded right.",
    },
    BeatCopy {
        id: "forbidden_trade",
        title: "The Weight of Taboo",
        text: "They didn't say anything when you sold. But the silence was heavy. You've broken a cultural rule, and customs heat rises. Some profits cost more than silver.",
    },
    BeatCopy {
        id: "cultural_bridge",
        title: "Bridge Between Worlds",
        text: "You belong everywhere and nowhere. The merchant who speaks every tongue and respects every custom is trusted by all. Three regions greet you as one of their own.",
    },
    BeatCopy {
        id: "festival_patron",
        title: "Friend of the Festivals",
        text: "Word spreads along the trade routes: you are a friend of the festivals. Not just a buyer who arrives when prices rise, but a captain who respects the celebration. The ports remember.",
    },
    BeatCopy {
        id: "the_known_world_culture",
        title: "A Citizen of the Sea",
        text: "From the columned exchanges of the Mediterranean to the coral thrones of the South Seas, you've seen how every people makes meaning from trade. Commerce isn't just numbers. It's the story of how strangers become neighbors.",
    },
    BeatCopy {
        id: "proverb_collector",
        title: "Wisdom of the Ports",
        text: "Every port taught you something. You carry their wisdom like ballast - invisible, but it keeps you steady. The proverbs of twenty harbors live in your captain's log.",
    },
    BeatCopy {
        id: "first_winter",
        title: "The Cold Season",
        text: "Winter closes in. The northern ports grow quiet and the sea turns grey. Experienced captains planned for this - they stocked medicines and tea when prices were low. The unprepared pay winter rates.",
    },
    BeatCopy {
        id: "monsoon_survivor",
        title: "Through the Monsoon",
        text: "The monsoon season tried to swallow your ship whole. Rain so heavy it felt solid, waves that blocked out the sky. But you kept the crew alive and the cargo dry. The East Indies respect a captain who dares the monsoon.",
    },
    BeatCopy {
        id: "harvest_trader",
        title: "Riding the Harvest",
        text: "You timed it perfectly. When the harvest flooded the market with cheap grain and cotton, you were there to buy. When winter drove demand through the roof, you were there to sell. The calendar is a captain's secret weapon.",
    },
    BeatCopy {
        id: "four_seasons_captain",
        title: "Captain for All Seasons",
        text: "You've sailed through spring calms and winter gales, monsoon fury and autumn harvests. The sea has shown you every face it has. You trade with the rhythm of the world, not against it.",
    },
    BeatCopy {
        id: "first_contraband",
        title: "Crossing the Line",
        text: "You bought something the law says you shouldn't have. It sits in your hold like a secret - valuable, dangerous, and impossible to un-know. The legitimate world just got a little smaller.",
    },
    BeatCopy {
        id: "underworld_contact",
        title: "A Name in the Dark",
        text: "Word travels in the underworld. A pirate faction knows your name - not as prey, but as someone worth talking to. The line between trader and smuggler just blurred.",
    },
    BeatCopy {
        id: "pirate_deal",
        title: "Trading with Wolves",
        text: "You traded with a pirate captain on the open sea. No port, no witnesses, no manifest. Just two captains, a price, and a handshake. The underworld does business differently.",
    },
    BeatCopy {
        id: "first_duel_win",
        title: "Blood and Steel",
        text: "Your blade found its mark. The pirate captain yielded, and for a heartbeat the world narrowed to two people and a single truth: you earned what you carry. The crew looks at you differently now.",
    },
    BeatCopy {
        id: "nemesis_born",
        title: "A Grudge on the Water",
        text: "A pirate captain remembers you. Not as a trade partner or a neutral ship - as an enemy. They'll be watching for your sails, and next time, the conversation starts with steel.",
    },
    BeatCopy {
        id: "faction_trusted",
        title: "The Shadow's Trust",
        text: "A pirate faction trusts you completely. Their captains greet you as an ally, their ports treat you as family. You've earned what money alone can't buy: a place in the underworld's inner circle.",
    },
    BeatCopy {
        id: "duel_master",
        title: "Blade of the Sea",
        text: "Five captains have felt your steel. Your name is spoken with respect in every pirate port and with fear on every patrol ship. The blade is part of who you are now.",
    },
    BeatCopy {
        id: "shadow_master",
        title: "Lord of the Grey",
        text: "Three factions count you as a friend. The underworld's politics flow through your hold as surely as the legitimate trade. You've built something no customs inspector can confiscate: a network that spans every shadow port in the Known World.",
    },
    BeatCopy {
        id: "faction_spillover",
        title: "The Price of Friends",
        text: "You helped one faction, and another noticed. In the underworld, every friendship casts a shadow. The enemies of your friends are now watching you with different eyes.",
    },
    BeatCopy {
        id: "vendetta_declared",
        title: "Blood in the Ledger",
        text: "A faction has declared vendetta. Your alliance with their enemy has made you a target - not just a stranger, but a marked captain. Their ships will hunt you in their waters. Choose your routes carefully.",
    },
    BeatCopy {
        id: "political_survivor",
        title: "Walking the Wire",
        text: "You've navigated the underworld's politics without being destroyed by them. Trade partners on one side, enemies on the other, and you in the middle - still sailing, still trading, still alive. That's an achievement few can claim.",
    },
    BeatCopy {
        id: "faction_diplomat",
        title: "The Pirate's Diplomat",
        text: "Factions that hate each other both trust you. You've done what no navy, no governor, no merchant guild has managed: earned standing on both sides of a pirate war. The sea's politics flow through you.",
    },
];

#[cfg(test)]
mod tests {
    use super::{
        beat_text, beat_title, chronicle_entry_rows, document_lines, journal_document, memory_rows,
        BEATS,
    };
    use portlight_sim::model::{CaptainMemory, JournalEntry};
    use portlight_sim::Session;

    fn row_text(rows: &[super::JournalRow]) -> String {
        rows.iter()
            .map(|row| match row {
                super::JournalRow::Line { text, .. } => text.clone(),
                super::JournalRow::Beat { label, .. } => label.clone(),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn beat_table_matches_the_narrative_catalog() {
        assert_eq!(BEATS.len(), 45);
        let mut ids = BEATS.iter().map(|beat| beat.id).collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 45);
        assert_eq!(beat_title("first_trade"), "The First Deal");
        assert_eq!(beat_title("faction_diplomat"), "The Pirate's Diplomat");
        assert_eq!(beat_title("not_a_real_beat"), "not a real beat");
        let text = beat_text("first_trade").unwrap();
        assert!(text.contains("single trade"));
        assert!(text.is_ascii());
        for beat in BEATS {
            assert!(beat.id.is_ascii() && beat.title.is_ascii() && beat.text.is_ascii());
            assert!(!beat.title.is_empty() && !beat.text.is_empty());
        }
    }

    #[test]
    fn a_new_voyage_lists_four_paths_and_empty_records() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        let doc = journal_document(
            &session,
            &["New game. Docked at Porto Novo.".to_string()],
            &[],
        );
        assert_eq!(doc.title, "Ada");
        assert!(doc
            .notice
            .starts_with(&format!("Day {}.", session.world().day)));
        assert!(doc.notice.contains("Docked at"));
        let text = document_lines(&doc).join("\n");
        assert!(text.is_ascii(), "{text}");
        assert!(text.contains("No chronicle entries yet."));
        assert!(text.contains("Fired beats: 0"));
        for name in [
            "Lawful Trade House",
            "Shadow Network",
            "Oceanic Reach",
            "Commercial Empire",
        ] {
            assert!(text.contains(name), "{name}");
        }
        assert_eq!(text.matches("Strength ").count(), 4);
        assert!(text.contains("No milestones recorded."));
        assert!(text.contains("No captains remembered yet."));
        assert!(text.contains("New game. Docked at Porto Novo."));
        assert!(!text.contains("Ledger:"));
        assert!(doc.sections.iter().all(|section| section.id != "festivals"));
        let victory = doc
            .sections
            .iter()
            .find(|section| section.id == "victory")
            .unwrap();
        let paths = session.victory();
        let names: Vec<&str> = paths.iter().map(|path| path.name.as_str()).collect();
        let shown = row_text(&victory.rows);
        let mut cursor = 0;
        for name in names {
            let at = shown[cursor..].find(name).expect(name);
            cursor += at + name.len();
        }
        for row in &victory.rows {
            if let super::JournalRow::Line { text, .. } = row {
                if let Some(rest) = text.strip_prefix('[') {
                    let status = rest.split(']').next().unwrap();
                    assert!(matches!(status, "met" | "missing" | "blocked"), "{text}");
                }
            }
        }
        assert!(text.contains("network collapses above 40"));
    }

    #[test]
    fn chronicle_entries_expand_mapped_text_only() {
        let entry = JournalEntry {
            beat_id: "first_trade".to_string(),
            day: 4,
            port_id: "porto_novo".to_string(),
            region: "Mediterranean".to_string(),
        };
        let closed = row_text(&chronicle_entry_rows(
            &entry,
            Some("Porto Novo, Mediterranean"),
            false,
        ));
        assert_eq!(closed, "Day 4 - The First Deal\nPorto Novo, Mediterranean");
        let open = row_text(&chronicle_entry_rows(
            &entry,
            Some("Porto Novo, Mediterranean"),
            true,
        ));
        assert!(open.contains("single trade"));
        let unknown = JournalEntry {
            beat_id: "made_up_beat".to_string(),
            day: 2,
            port_id: String::new(),
            region: String::new(),
        };
        let rows = row_text(&chronicle_entry_rows(&unknown, None, true));
        assert_eq!(rows, "Day 2 - made up beat");
    }

    #[test]
    fn a_memory_row_uses_the_pirate_catalog_name() {
        let mut memory = CaptainMemory::new("raj_the_quiet");
        memory.last_seen_day = 12;
        memory.last_seen_region = "Mediterranean".to_string();
        memory.times_spared = 1;
        memory.times_defeated_by_player = 2;
        memory.times_defeated_player = 0;
        memory.relationship.respect = 4;
        memory.relationship.grudge = 3;
        let text = row_text(&memory_rows(&memory));
        assert!(text.contains("Raj the Quiet"));
        assert!(text.contains("Last seen Mediterranean, day 12."));
        assert!(text.contains("Spared 1. Defeated them 2. They defeated you 0."));
        assert!(text.contains("Respect 4. Fear 0. Grudge 3. Familiarity 0."));
        assert!(text.is_ascii());
    }

    /// A catalog miss reads as humanized copy, never the raw id.
    #[test]
    fn catalog_misses_never_print_raw_ids() {
        let session = Session::new(
            crate::logic::SCRIPTED_NAME,
            crate::logic::SCRIPTED_CAPTAIN_TYPE,
            crate::logic::SCRIPTED_SEED,
            None,
        )
        .expect("scripted session");
        let world = session.world();
        assert_eq!(super::port_label(world, "salt_spit_cove"), "Salt Spit Cove");
        let known = world.port("porto_novo").expect("porto_novo");
        assert_eq!(super::port_label(world, "porto_novo"), known.name);
        assert_eq!(super::pirate_name("salt_spit_cove"), "Salt Spit Cove");
        assert_eq!(
            super::pirate_name(crate::logic::SCRIPTED_CAPTAIN),
            crate::logic::encounter_end_name(crate::logic::SCRIPTED_CAPTAIN)
        );
    }
}
