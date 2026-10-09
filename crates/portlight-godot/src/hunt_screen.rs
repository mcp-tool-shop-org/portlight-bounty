//! Hunt overlay. Forage is one day of [`portlight_sim::Session::hunt`].
//! The bounty board is a separate desk: accept, hunt target, claim.
//!
//! `hunt_bounty` hands the returned encounter to the existing encounter
//! screen. This module does not draw a second fight.
//!
//! Follow-on, not this screen: a GameSession runner `bounty` verb. The
//! `bounty_*` parity scripts stay whole-script skips until that lands.

use godot::classes::control::{MouseFilter, SizeFlags};
use godot::classes::text_server::AutowrapMode;
use godot::classes::{Button, HBoxContainer, Label, PanelContainer, StyleBoxFlat, VBoxContainer};
use godot::prelude::*;
use portlight_sim::bounty::BountyTarget;
use portlight_sim::hunting::HuntResult;
use portlight_sim::model::VoyageStatus;
use portlight_sim::Session;

use crate::encounter_screen;
use crate::logic::{ascii_label, captain_display_name, faction_name, humanize_id};

const INK: Color = Color::from_rgb(0.08, 0.11, 0.16);
const CREAM: Color = Color::from_rgb(0.94, 0.91, 0.84);
const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);

/// Same cap `accept_bounty` enforces.
pub(crate) const ACTIVE_CAP: usize = 3;

pub(crate) const FORAGE_BUTTON: &str = "Forage";
/// Muted helper under the Forage status line. Shown always, at sea and in
/// port, whatever the bounty state. Forage costs a day; it never opens a fight.
pub(crate) const FORAGE_HELPER: &str = "Spends a day gathering stores. Does not start a fight.";
/// Forage result line when every term is zero.
pub(crate) const FORAGE_NO_CHANGE: &str = "No gain, no loss.";
pub(crate) const HUNT_TARGET_BUTTON: &str = "Hunt target";
pub(crate) const CLAIM_BUTTON: &str = "Claim";
pub(crate) const EMPTY_BOARD: &str = "No bounties on the board.";
/// Eyebrow above the gold title. The title is location only, like the
/// sibling desks; docked status sits here.
pub(crate) const EYEBROW_DOCKED: &str = "Hunt - Docked";
pub(crate) const EYEBROW_AT_SEA: &str = "Hunt";

/// Display row. Mirrors `PIRATE_BOUNTIES` in `portlight-sim` so an active id
/// still has a name after the ephemeral board is replaced. Not a sim API.
struct CatalogRow {
    id: &'static str,
    name: &'static str,
    faction_id: &'static str,
    region: &'static str,
    reward: i64,
    difficulty: &'static str,
    description: &'static str,
}

const CATALOG: &[CatalogRow] = &[
    CatalogRow {
        id: "scarlet_ana",
        name: "Scarlet Ana",
        faction_id: "crimson_tide",
        region: "North Atlantic",
        reward: 150,
        difficulty: "moderate",
        description: "Captain of the Crimson Tide's diplomatic fleet. Deals first, fights well.",
    },
    CatalogRow {
        id: "the_butcher",
        name: "The Butcher",
        faction_id: "crimson_tide",
        region: "Mediterranean",
        reward: 220,
        difficulty: "hard",
        description: "Crimson Tide enforcer. No diplomacy. Takes what he wants.",
    },
    CatalogRow {
        id: "raj_the_quiet",
        name: "Raj the Quiet",
        faction_id: "monsoon_syndicate",
        region: "East Indies",
        reward: 120,
        difficulty: "easy",
        description: "Syndicate spymaster. Knows your hold before you open it.",
    },
    CatalogRow {
        id: "typhoon_mei",
        name: "Typhoon Mei",
        faction_id: "monsoon_syndicate",
        region: "East Indies",
        reward: 200,
        difficulty: "hard",
        description: "The monsoon in human form. Controls the eastern sea lanes with chaos.",
    },
    CatalogRow {
        id: "old_coral",
        name: "Old Coral",
        faction_id: "deep_reef",
        region: "South Seas",
        reward: 160,
        difficulty: "moderate",
        description: "Brotherhood elder. Fifty years on the reef. Respects courage.",
    },
    CatalogRow {
        id: "the_diver",
        name: "The Diver",
        faction_id: "deep_reef",
        region: "West Africa",
        reward: 180,
        difficulty: "hard",
        description: "Boards from the waterline and is gone before steel is drawn.",
    },
    CatalogRow {
        id: "sergeant_kruze",
        name: "Sergeant Kruze",
        faction_id: "iron_wolves",
        region: "North Atlantic",
        reward: 200,
        difficulty: "hard",
        description: "Former garrison sergeant. Runs piracy like a military operation.",
    },
    CatalogRow {
        id: "gnaw",
        name: "Gnaw",
        faction_id: "iron_wolves",
        region: "North Atlantic",
        reward: 200,
        difficulty: "hard",
        description: "Most feared pirate in the North Atlantic. Destroys what he cannot take.",
    },
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BountyCard {
    pub captain_id: String,
    pub captain_name: String,
    pub faction_id: String,
    pub region: String,
    pub reward: i64,
    pub difficulty: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub(crate) enum HuntConfirm {
    HuntTarget(String),
    Claim(String),
}

#[derive(Clone, Debug)]
pub(crate) enum HuntAction {
    Forage,
    RefreshBoard,
    Accept(String),
    AskHunt(String),
    AskClaim(String),
    Confirm,
    Cancel,
    Close,
}

/// Last board sample and cards cached when they were listed. Refresh replaces
/// `board` and must not drop `known`.
#[derive(Clone, Debug, Default)]
pub(crate) struct HuntDesk {
    pub notice: String,
    pub board: Vec<BountyTarget>,
    pub posted: bool,
    pub known: Vec<BountyTarget>,
    pub confirm: Option<HuntConfirm>,
}

#[derive(Clone)]
pub(crate) struct HuntNodes {
    pub root: Gd<PanelContainer>,
    pub eyebrow: Gd<Label>,
    pub title: Gd<Label>,
    pub notice: Gd<Label>,
    pub scroll: Gd<godot::classes::ScrollContainer>,
    pub body: Gd<VBoxContainer>,
    pub confirm_row: Gd<HBoxContainer>,
    pub footer: Gd<VBoxContainer>,
    pub plate: Gd<godot::classes::TextureRect>,
    pub plate_panel: Gd<PanelContainer>,
    pub plate_caption: Gd<Label>,
    pub placeholder: Gd<PanelContainer>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OfferRow {
    pub id: String,
    pub text: String,
    pub accept_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActiveRow {
    pub id: String,
    pub text: String,
    pub claim_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HuntModel {
    pub eyebrow: String,
    pub title: String,
    pub notice: String,
    pub forage_status: String,
    pub forage_enabled: bool,
    pub posted: bool,
    pub offers: Vec<OfferRow>,
    pub active_heading: String,
    pub active: Vec<ActiveRow>,
    pub claimed: Vec<String>,
    pub wanted: String,
    pub pact_note: Option<String>,
    pub known: Vec<String>,
}

pub(crate) fn build_hunt_screen() -> HuntNodes {
    let mut root = PanelContainer::new_alloc();
    root.set_name("HuntScreen");
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

    let eyebrow = text_label(EYEBROW_AT_SEA, 14, MUTED);
    column.add_child(&eyebrow);
    let title = text_label("At sea", 28, GOLD);
    column.add_child(&title);
    let mut notice = text_label("", 16, CREAM);
    notice.set_autowrap_mode(AutowrapMode::WORD_SMART);
    notice.set_h_size_flags(SizeFlags::EXPAND_FILL);
    column.add_child(&notice);

    let mut confirm_row = HBoxContainer::new_alloc();
    confirm_row.set_name("HuntConfirm");
    confirm_row.add_theme_constant_override("separation", 8);
    confirm_row.set_visible(false);
    column.add_child(&confirm_row);

    let mut scroll = godot::classes::ScrollContainer::new_alloc();
    scroll.set_name("HuntScroll");
    scroll.set_h_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_v_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_horizontal_scroll_mode(godot::classes::scroll_container::ScrollMode::DISABLED);
    let mut body = VBoxContainer::new_alloc();
    body.set_name("HuntBody");
    body.set_h_size_flags(SizeFlags::EXPAND_FILL);
    body.add_theme_constant_override("separation", 14);
    scroll.add_child(&body);
    column.add_child(&scroll);

    let mut footer = VBoxContainer::new_alloc();
    footer.set_name("HuntFooter");
    footer.set_h_size_flags(SizeFlags::EXPAND_FILL);
    column.add_child(&footer);

    HuntNodes {
        root,
        eyebrow,
        title,
        notice,
        scroll,
        body,
        confirm_row,
        footer,
        plate,
        plate_panel,
        plate_caption,
        placeholder,
    }
}

pub(crate) fn fill_parent(root: &mut Gd<PanelContainer>) {
    root.set_anchors_and_offsets_preset(godot::classes::control::LayoutPreset::FULL_RECT);
}

pub(crate) fn show_hunt(nodes: &mut HuntNodes, open: bool) {
    nodes.root.set_visible(open);
    nodes.root.set_mouse_filter(if open {
        MouseFilter::STOP
    } else {
        MouseFilter::IGNORE
    });
}

pub(crate) fn catalog_card(id: &str) -> Option<BountyCard> {
    CATALOG
        .iter()
        .find(|row| row.id == id)
        .map(card_from_catalog)
}

pub(crate) fn card_from_target(target: &BountyTarget) -> BountyCard {
    BountyCard {
        captain_id: target.captain_id.clone(),
        // Live name, else catalog name, else humanised id; never the raw id.
        captain_name: captain_display_name(&target.captain_name, &target.captain_id),
        // An id for `faction_name` (catalog, else humanised). An empty or
        // non-ASCII faction falls back to the catalog captain's faction, never
        // to the captain id.
        faction_id: target_faction_id(target),
        region: ascii_label(&target.region, "").to_string(),
        reward: target.reward,
        difficulty: ascii_label(&target.difficulty, "").to_string(),
        description: ascii_text(&target.description),
    }
}

fn target_faction_id(target: &BountyTarget) -> String {
    if !target.faction_id.is_empty() && target.faction_id.is_ascii() {
        return target.faction_id.clone();
    }
    portlight_sim::content::content()
        .pirate(&target.captain_id)
        .map(|pirate| pirate.faction_id.clone())
        .unwrap_or_default()
}

pub(crate) fn remember(known: &mut Vec<BountyTarget>, target: BountyTarget) {
    if let Some(slot) = known
        .iter_mut()
        .find(|row| row.captain_id == target.captain_id)
    {
        *slot = target;
    } else {
        known.push(target);
    }
}

pub(crate) fn forage_notice(result: &HuntResult) -> String {
    let mut lines = Vec::new();
    let flavor = ascii_text(&result.flavor);
    if !flavor.is_empty() {
        lines.push(flavor);
    }
    let danger = ascii_text(&result.danger_text);
    if !danger.is_empty() {
        lines.push(danger);
    }
    lines.push(forage_deltas(result));
    lines.join("\n")
}

/// One clause per non-zero term: gains are `+N`, losses `-N`. Morale, crew,
/// and hull are costs, so they read as negatives. All zero is one line.
pub(crate) fn forage_deltas(result: &HuntResult) -> String {
    let terms = [
        ("Provisions", result.provisions_gained),
        ("Pelts", result.pelts_gained),
        ("Silver", result.silver_gained),
        ("Morale", -result.morale_cost),
        ("Crew", -result.crew_lost),
        ("Hull", -result.hull_damage),
    ];
    let clauses: Vec<String> = terms
        .iter()
        .filter(|(_, value)| *value != 0)
        .map(|(name, value)| format!("{name} {value:+}."))
        .collect();
    if clauses.is_empty() {
        FORAGE_NO_CHANGE.to_string()
    } else {
        clauses.join(" ")
    }
}

pub(crate) fn claim_notice(silver: i64) -> String {
    format!("Claimed {silver} silver.")
}

/// Hunt confirm. `crew` is `(crew, template_crew_min, cap)` where `cap` comes
/// from [`portlight_sim::naval::hunt_crew_loss_cap`]. No odds.
pub(crate) fn hunt_confirm_text(name: &str, crew: Option<(i64, i64, i64)>) -> String {
    match crew {
        Some((crew, min, cap)) if cap > 0 => format!(
            "Hunt {name}? This opens a fight. Boarding can cost up to {cap} crew - you have {crew}, need {min} to sail."
        ),
        Some((_, min, _)) => format!(
            "Hunt {name}? This opens a fight. Boarding won't cost crew below the {min} you need to sail."
        ),
        None => format!("Hunt {name}? This opens a fight."),
    }
}

pub(crate) fn claim_confirm_text(name: &str) -> String {
    format!("Claim the bounty on {name}?")
}

/// Forage status stays a location line. It does not name bounty targets.
pub(crate) fn forage_status(at_sea: bool, port: Option<&str>, morale: Option<i64>) -> String {
    if at_sea {
        let morale = morale.unwrap_or(0);
        if morale < 20 {
            format!("At sea. Morale {morale}. Crew morale too low for hunting at sea (need 20+).")
        } else {
            format!("At sea. Morale {morale}. Need 20 or more.")
        }
    } else {
        let port = port.unwrap_or("port");
        format!("In port at {port}.")
    }
}

pub(crate) fn forage_enabled(at_sea: bool, morale: Option<i64>) -> bool {
    !(at_sea && morale.unwrap_or(0) < 20)
}

pub(crate) fn wanted_copy(level: i64) -> (String, Option<String>) {
    let line = format!("Wanted level {level}");
    let note = if level >= 3 {
        Some("Pact hunters are a separate sea event.".to_string())
    } else {
        None
    };
    (line, note)
}

pub(crate) fn accept_enabled(id: &str, active: &[String], claimed: &[String]) -> bool {
    !active.iter().any(|row| row == id)
        && !claimed.iter().any(|row| row == id)
        && active.len() < ACTIVE_CAP
}

pub(crate) fn claim_enabled(defeats: i64) -> bool {
    defeats > 0
}

pub(crate) fn hunt_model(session: &Session, desk: &HuntDesk) -> HuntModel {
    let world = session.world();
    let at_sea = world.voyage.status == VoyageStatus::AtSea;
    let port_name = if at_sea {
        None
    } else {
        let id = world.voyage.destination_id.as_str();
        Some(
            world
                .port(id)
                .map(|port| port.name.clone())
                .unwrap_or_else(|| humanize_id(id)),
        )
    };
    let morale = world.captain.ship.as_ref().map(|ship| ship.morale);
    // Location only, matching the sibling desks. Docked status is the eyebrow.
    let (eyebrow, title) = if at_sea {
        (EYEBROW_AT_SEA, "At sea".to_string())
    } else {
        (
            EYEBROW_DOCKED,
            port_name.clone().unwrap_or_else(|| "port".to_string()),
        )
    };
    let active_ids = world.captain.active_bounties.clone();
    let claimed_ids = world.captain.claimed_bounties.clone();
    let offers = desk
        .board
        .iter()
        .map(|target| {
            let card = card_from_target(target);
            let enabled = accept_enabled(&card.captain_id, &active_ids, &claimed_ids);
            OfferRow {
                id: card.captain_id.clone(),
                text: offer_text(&card),
                accept_enabled: enabled,
            }
        })
        .collect();
    let active = active_ids
        .iter()
        .map(|id| {
            let card = resolve_card(id, &desk.known);
            let defeats = times_defeated(session, id);
            ActiveRow {
                id: id.clone(),
                text: active_text(&card, defeats),
                claim_enabled: claim_enabled(defeats),
            }
        })
        .collect();
    let claimed = claimed_ids
        .iter()
        .map(|id| resolve_card(id, &desk.known).captain_name)
        .collect();
    let (wanted, pact_note) = wanted_copy(world.captain.wanted_level);
    let known = world
        .captain_memories
        .iter()
        .map(|memory| {
            let name = resolve_card(&memory.captain_id, &desk.known).captain_name;
            format!("{name}  defeated {}", memory.times_defeated_by_player)
        })
        .collect();
    HuntModel {
        eyebrow: eyebrow.to_string(),
        title,
        notice: desk.notice.clone(),
        forage_status: forage_status(at_sea, port_name.as_deref(), morale),
        forage_enabled: forage_enabled(at_sea, morale),
        posted: desk.posted,
        offers,
        active_heading: format!("Active ({}/{})", active_ids.len(), ACTIVE_CAP),
        active,
        claimed,
        wanted,
        pact_note,
        known,
    }
}

pub(crate) fn display_name(id: &str, known: &[BountyTarget]) -> String {
    resolve_card(id, known).captain_name
}

pub(crate) fn reward_for(id: &str, known: &[BountyTarget]) -> i64 {
    resolve_card(id, known).reward
}

pub(crate) fn rebuild_body(
    body: &mut Gd<VBoxContainer>,
    model: &HuntModel,
    make: &impl Fn(&str, HuntAction) -> Gd<Button>,
) {
    clear_box(body);
    let mut forage = section("Forage");
    forage.add_child(&wrapped(&model.forage_status, 15, CREAM));
    forage.add_child(&wrapped(FORAGE_HELPER, 14, MUTED));
    let mut forage_button = make(FORAGE_BUTTON, HuntAction::Forage);
    forage_button.set_disabled(!model.forage_enabled);
    forage.add_child(&forage_button);
    body.add_child(&forage);

    let mut board = section("Bounty board");
    board.add_child(&make("Refresh board", HuntAction::RefreshBoard));
    if model.posted && model.offers.is_empty() {
        board.add_child(&wrapped(EMPTY_BOARD, 15, CREAM));
    }
    for offer in &model.offers {
        board.add_child(&wrapped(&offer.text, 15, CREAM));
        let mut accept = make("Accept", HuntAction::Accept(offer.id.clone()));
        accept.set_disabled(!offer.accept_enabled);
        board.add_child(&accept);
    }
    body.add_child(&board);

    let mut active = section(&model.active_heading);
    for row in &model.active {
        active.add_child(&wrapped(&row.text, 15, CREAM));
        let mut buttons = HBoxContainer::new_alloc();
        buttons.add_theme_constant_override("separation", 8);
        buttons.add_child(&make(
            HUNT_TARGET_BUTTON,
            HuntAction::AskHunt(row.id.clone()),
        ));
        let mut claim = make(CLAIM_BUTTON, HuntAction::AskClaim(row.id.clone()));
        claim.set_disabled(!row.claim_enabled);
        buttons.add_child(&claim);
        active.add_child(&buttons);
    }
    body.add_child(&active);

    let mut claimed = section("Claimed");
    if model.claimed.is_empty() {
        claimed.add_child(&wrapped("None.", 15, MUTED));
    } else {
        claimed.add_child(&wrapped(&model.claimed.join("\n"), 15, CREAM));
    }
    body.add_child(&claimed);

    let mut wanted = section("Wanted");
    wanted.add_child(&wrapped(&model.wanted, 15, CREAM));
    if let Some(note) = &model.pact_note {
        wanted.add_child(&wrapped(note, 14, MUTED));
    }
    body.add_child(&wanted);

    if !model.known.is_empty() {
        let mut known = section("Known captains");
        known.add_child(&wrapped(&model.known.join("\n"), 14, MUTED));
        body.add_child(&known);
    }
}

pub(crate) fn clear_box(node: &mut Gd<VBoxContainer>) {
    let children = node.get_children();
    for mut child in children.iter_shared() {
        node.remove_child(&child);
        child.queue_free();
    }
}

/// Bring a named section to the top of the scroll. Frame captures use this
/// so the board and the active list are on screen, not under the fold.
pub(crate) fn scroll_to(
    scroll: &mut godot::classes::ScrollContainer,
    body: &Gd<VBoxContainer>,
    name: &str,
) {
    for child in body.get_children().iter_shared() {
        if child.get_name() == name {
            if let Ok(control) = child.try_cast::<godot::classes::Control>() {
                scroll.ensure_control_visible(&control);
            }
        }
    }
}

/// True when the Forage section in `body` shows [`FORAGE_HELPER`]. Smoke check.
pub(crate) fn forage_helper_shown(body: &Gd<VBoxContainer>) -> bool {
    body.get_children()
        .iter_shared()
        .filter(|section| section.get_name() == "HuntForage")
        .flat_map(|section| section.get_children().iter_shared().collect::<Vec<_>>())
        .filter_map(|child| child.try_cast::<Label>().ok())
        .any(|label| label.get_text() == FORAGE_HELPER)
}

pub(crate) fn clear_row(node: &mut Gd<HBoxContainer>) {
    let children = node.get_children();
    for mut child in children.iter_shared() {
        node.remove_child(&child);
        child.queue_free();
    }
}

fn resolve_card(id: &str, known: &[BountyTarget]) -> BountyCard {
    if let Some(target) = known.iter().find(|row| row.captain_id == id) {
        return card_from_target(target);
    }
    catalog_card(id).unwrap_or_else(|| BountyCard {
        captain_id: id.to_string(),
        captain_name: captain_display_name("", id),
        faction_id: String::new(),
        region: String::new(),
        reward: 0,
        difficulty: String::new(),
        description: String::new(),
    })
}

fn card_from_catalog(row: &CatalogRow) -> BountyCard {
    BountyCard {
        captain_id: row.id.to_string(),
        captain_name: row.name.to_string(),
        faction_id: row.faction_id.to_string(),
        region: row.region.to_string(),
        reward: row.reward,
        difficulty: row.difficulty.to_string(),
        description: row.description.to_string(),
    }
}

fn offer_text(card: &BountyCard) -> String {
    format!(
        "{}\n{} | {} | {} | {} silver\n{}",
        card.captain_name,
        card.region,
        faction_name(&card.faction_id),
        card.difficulty,
        card.reward,
        card.description
    )
}

fn active_text(card: &BountyCard, defeats: i64) -> String {
    if card.reward == 0 && card.difficulty.is_empty() {
        return format!("{}\nDefeated {defeats}.", card.captain_name);
    }
    format!(
        "{}\n{} silver | {} | defeated {defeats}",
        card.captain_name, card.reward, card.difficulty
    )
}

fn times_defeated(session: &Session, id: &str) -> i64 {
    session
        .world()
        .captain_memories
        .iter()
        .find(|memory| memory.captain_id == id)
        .map(|memory| memory.times_defeated_by_player)
        .unwrap_or(0)
}

fn section(title: &str) -> Gd<VBoxContainer> {
    let mut column = VBoxContainer::new_alloc();
    column.set_name(section_name(title));
    column.set_h_size_flags(SizeFlags::EXPAND_FILL);
    column.add_theme_constant_override("separation", 6);
    // Gold subhead, same as Contracts, Shipyard, Journal, and Crew.
    column.add_child(&text_label(title, 16, GOLD));
    column
}

fn section_name(title: &str) -> &'static str {
    if title == "Forage" {
        "HuntForage"
    } else if title == "Bounty board" {
        "HuntBoard"
    } else if title.starts_with("Active") {
        "HuntActive"
    } else if title == "Claimed" {
        "HuntClaimed"
    } else if title == "Wanted" {
        "HuntWanted"
    } else {
        "HuntKnown"
    }
}

fn wrapped(text: &str, size: i32, color: Color) -> Gd<Label> {
    let mut label = text_label(text, size, color);
    label.set_autowrap_mode(AutowrapMode::WORD_SMART);
    label.set_h_size_flags(SizeFlags::EXPAND_FILL);
    label
}

fn text_label(text: &str, size: i32, color: Color) -> Gd<Label> {
    let mut label = Label::new_alloc();
    label.set_text(text);
    label.add_theme_font_size_override("font_size", size);
    label.add_theme_color_override("font_color", color);
    label
}

/// Maps em/en dashes to ASCII '-', drops other non-ASCII for the default font,
/// then collapses leftover double spaces. Display only; the sim string is unchanged.
fn ascii_text(text: &str) -> String {
    if text.is_ascii() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        let mapped = match ch {
            '\u{2014}' | '\u{2013}' => '-',
            c if c.is_ascii() => c,
            _ => continue,
        };
        if mapped == ' ' && out.ends_with(' ') {
            continue;
        }
        out.push(mapped);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() {
                if !name.starts_with('.') && name != "assets" {
                    source_files(&path, out);
                }
            } else if matches!(path.extension().and_then(|e| e.to_str()), Some("rs" | "gd")) {
                out.push(path);
            }
        }
    }

    /// Hunt crew-loss acceptance test 12: the cap comes from
    /// `naval::hunt_crew_loss_cap`; Godot code never re-derives it.
    #[test]
    fn hunt_crew_formula_lives_only_in_the_sim() {
        let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = Vec::new();
        source_files(&crate_dir.join("src"), &mut files);
        source_files(&crate_dir.join("../../godot"), &mut files);
        assert!(files.len() > 10, "source scan found {} files", files.len());
        // Split so this test does not match itself.
        let banned = [["0.0", "3"].concat(), ["ce", "il"].concat()];
        for path in &files {
            let text = std::fs::read_to_string(path).unwrap_or_default();
            for word in &banned {
                assert!(
                    !text.contains(word.as_str()),
                    "{word:?} in {}: the hunt cap belongs to portlight_sim::naval",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn hunt_confirm_states_crew_against_the_sail_minimum() {
        assert_eq!(
            hunt_confirm_text("Raj the Quiet", Some((4, 3, 1))),
            "Hunt Raj the Quiet? This opens a fight. Boarding can cost up to 1 crew - you have 4, need 3 to sail."
        );
        assert_eq!(
            hunt_confirm_text("Raj the Quiet", Some((5, 5, 0))),
            "Hunt Raj the Quiet? This opens a fight. Boarding won't cost crew below the 5 you need to sail."
        );
        assert_eq!(
            hunt_confirm_text("Raj the Quiet", None),
            "Hunt Raj the Quiet? This opens a fight."
        );
        for crew in [Some((4, 3, 1)), Some((5, 5, 0)), None] {
            let text = hunt_confirm_text("Raj the Quiet", crew);
            assert!(text.is_ascii(), "{text}");
            assert!(!text.contains('%'), "{text}");
            assert!(!text.to_ascii_lowercase().contains("chance"), "{text}");
            assert!(
                !text.contains('\u{2014}') && !text.contains('\u{2013}'),
                "{text}"
            );
            assert!(!text.contains(" will "), "{text}");
        }
    }

    #[test]
    fn catalog_matches_every_live_board_row() {
        let mut seen = std::collections::BTreeSet::new();
        for seed in 0..48 {
            let mut session = Session::new("Ada", "merchant", seed, None).unwrap();
            for target in session.bounty_board() {
                let card = catalog_card(&target.captain_id).unwrap_or_else(|| {
                    panic!("board id {} is not in the Godot catalog", target.captain_id)
                });
                assert_eq!(card.captain_name, target.captain_name);
                assert_eq!(card.faction_id, target.faction_id);
                assert_eq!(card.region, target.region);
                assert_eq!(card.reward, target.reward);
                assert_eq!(card.difficulty, target.difficulty);
                assert_eq!(card.description, target.description);
                seen.insert(target.captain_id);
            }
        }
        let mut expected: Vec<&str> = CATALOG.iter().map(|row| row.id).collect();
        expected.sort_unstable();
        let got: Vec<&str> = seen.iter().map(String::as_str).collect();
        assert_eq!(got, expected);
    }

    #[test]
    fn offer_text_names_every_faction_by_its_catalog_name() {
        let content = portlight_sim::content::content();
        for row in CATALOG {
            let text = offer_text(&card_from_catalog(row));
            let name = &content.faction(row.faction_id).unwrap().name;
            let meta = text.lines().nth(1).unwrap();
            assert!(meta.contains(&format!(" | {name} | ")), "{meta}");
        }
        let deep_reef = CATALOG
            .iter()
            .find(|row| row.faction_id == "deep_reef")
            .unwrap();
        assert!(
            offer_text(&card_from_catalog(deep_reef)).contains(" | The Deep Reef Brotherhood | ")
        );
    }

    #[test]
    fn forage_copy_lists_deltas_and_is_not_a_bounty_hunt() {
        let sample = HuntResult {
            success: false,
            provisions_gained: 0,
            pelts_gained: 0,
            silver_gained: 0,
            morale_cost: 3,
            crew_lost: 0,
            hull_damage: 0,
            flavor: "The sea gives nothing today. Your crew stares at empty nets.".into(),
            danger_text: String::new(),
        };
        let notice = forage_notice(&sample);
        assert!(notice.contains("empty nets"));
        assert!(!notice.contains("Provisions"));
        assert!(notice.ends_with("\nMorale -3."));
        assert!(!notice.to_lowercase().contains("hunt bounty"));
        assert_eq!(FORAGE_BUTTON, "Forage");
        assert!(!FORAGE_BUTTON.to_lowercase().contains("bounty"));
    }

    fn forage_sample(terms: [i64; 6]) -> HuntResult {
        HuntResult {
            success: true,
            provisions_gained: terms[0],
            pelts_gained: terms[1],
            silver_gained: terms[2],
            morale_cost: terms[3],
            crew_lost: terms[4],
            hull_damage: terms[5],
            flavor: String::new(),
            danger_text: String::new(),
        }
    }

    #[test]
    fn forage_deltas_all_zero_is_one_line() {
        let sample = forage_sample([0; 6]);
        assert_eq!(forage_deltas(&sample), "No gain, no loss.");
        assert_eq!(forage_notice(&sample), FORAGE_NO_CHANGE);
        assert!(!forage_notice(&sample).contains("0"));
    }

    #[test]
    fn forage_deltas_mixed_drops_zero_terms() {
        let sample = forage_sample([4, 1, 0, 2, 0, 0]);
        assert_eq!(
            forage_deltas(&sample),
            "Provisions +4. Pelts +1. Morale -2."
        );
        let silver = forage_sample([0, 0, 7, 0, 0, 0]);
        assert_eq!(forage_deltas(&silver), "Silver +7.");
        assert!(!forage_deltas(&sample).contains("-0"));
        assert!(!forage_deltas(&sample).contains("+0"));
    }

    #[test]
    fn forage_deltas_all_loss_reads_negative() {
        let sample = forage_sample([-2, 0, -5, 3, 1, 4]);
        assert_eq!(
            forage_deltas(&sample),
            "Provisions -2. Silver -5. Morale -3. Crew -1. Hull -4."
        );
        assert!(!forage_deltas(&sample).contains('+'));
    }

    #[test]
    fn forage_helper_copy_is_pinned_and_ascii() {
        assert_eq!(
            FORAGE_HELPER,
            "Spends a day gathering stores. Does not start a fight."
        );
        assert!(FORAGE_HELPER.is_ascii());
        assert!(!FORAGE_HELPER.contains("  "));
    }

    #[test]
    fn ascii_filter_maps_dashes_and_collapses_leftover_spaces() {
        assert_eq!(
            ascii_text("Shore birds and shellfish \u{2014} not glamorous."),
            "Shore birds and shellfish - not glamorous."
        );
        assert_eq!(
            ascii_text("Shore birds and shellfish \u{2013} not glamorous."),
            "Shore birds and shellfish - not glamorous."
        );
        assert_eq!(ascii_text("Plain  ascii stays."), "Plain  ascii stays.");
        let sample = HuntResult {
            success: true,
            provisions_gained: 4,
            pelts_gained: 1,
            silver_gained: 0,
            morale_cost: 0,
            crew_lost: 0,
            hull_damage: 0,
            flavor: "Shore birds and shellfish \u{2014} not glamorous.".into(),
            danger_text: String::new(),
        };
        assert!(forage_notice(&sample).contains(" - "));
        assert!(!forage_notice(&sample).contains("  "));
    }

    #[test]
    fn sea_forage_status_names_the_morale_gate() {
        let open = forage_status(true, None, Some(50));
        assert!(open.contains("At sea"));
        assert!(open.contains("Morale 50"));
        assert!(open.contains("20"));
        assert!(forage_enabled(true, Some(50)));
        let shut = forage_status(true, None, Some(19));
        assert!(shut.contains("need 20+"));
        assert!(!forage_enabled(true, Some(19)));
        assert_eq!(
            forage_status(false, Some("Porto Novo"), Some(50)),
            "In port at Porto Novo."
        );
    }

    #[test]
    fn accept_stops_at_three_and_claim_needs_a_defeat() {
        let active = vec!["a".into(), "b".into(), "c".into()];
        assert!(!accept_enabled("d", &active, &[]));
        assert!(accept_enabled("d", &["a".into()], &[]));
        assert!(!accept_enabled("a", &["a".into()], &[]));
        assert!(!accept_enabled("z", &[], &["z".into()]));
        assert!(!claim_enabled(0));
        assert!(claim_enabled(1));
    }

    #[test]
    fn wanted_footnote_is_only_the_pact_sea_event() {
        assert_eq!(wanted_copy(0).1, None);
        assert_eq!(wanted_copy(2).1, None);
        assert_eq!(
            wanted_copy(3).1.as_deref(),
            Some("Pact hunters are a separate sea event.")
        );
        assert!(!wanted_copy(4).0.contains("board"));
    }

    #[test]
    fn porto_novo_forage_does_not_open_a_duel_and_claim_waits() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        assert_eq!(session.world().voyage.destination_id, "porto_novo");
        let day = session.world().day;
        let result = session.hunt().unwrap();
        assert_eq!(session.world().day, day + 1);
        assert_eq!(session.world().captain.day, day + 1);
        assert!(session.world().pending_duel.is_none());
        let notice = forage_notice(&result);
        assert!(notice.contains("Provisions"));
        let mut desk = HuntDesk {
            board: session.bounty_board(),
            posted: true,
            ..HuntDesk::default()
        };
        assert!(!desk.board.is_empty() && desk.board.len() <= 3);
        for target in desk.board.clone() {
            remember(&mut desk.known, target);
        }
        let id = desk.board[0].captain_id.clone();
        session.accept_bounty(&id).unwrap();
        let again = session.accept_bounty(&id).unwrap_err().to_string();
        assert_eq!(again, "Already hunting this target");
        let early = session.claim_bounty(&id).unwrap_err().to_string();
        assert_eq!(
            early,
            "Target not yet defeated. Find and defeat them at sea."
        );
        let model = hunt_model(&session, &desk);
        assert_eq!(model.title, "Porto Novo");
        assert_eq!(model.eyebrow, EYEBROW_DOCKED);
        assert!(!model.title.contains("Docked"));
        assert_eq!(model.active.len(), 1);
        assert!(!model.active[0].claim_enabled);
        assert!(model
            .offers
            .iter()
            .any(|row| row.id == id && !row.accept_enabled));
        assert!(model.forage_status.starts_with("In port"));
        assert!(!model.forage_status.contains(&model.active[0].id));
        let kept = desk.known.len();
        desk.board.clear();
        assert_eq!(desk.known.len(), kept);
        let still = hunt_model(&session, &desk);
        assert!(still.active[0].text.contains("silver"));
    }

    #[test]
    fn refresh_keeps_an_active_card_that_left_the_board() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        session.accept_bounty("gnaw").unwrap();
        let mut desk = HuntDesk {
            posted: true,
            board: session.bounty_board(),
            ..HuntDesk::default()
        };
        desk.board.retain(|row| row.captain_id != "gnaw");
        let model = hunt_model(&session, &desk);
        let row = model.active.iter().find(|row| row.id == "gnaw").unwrap();
        assert!(row.text.contains("Gnaw"));
        assert!(row.text.contains("200 silver"));
        assert!(!row.claim_enabled);
    }
}
