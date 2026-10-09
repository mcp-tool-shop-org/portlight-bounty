//! Decisions the Godot view can make without a display server.
//!
//! Projection and facing stay in `portlight-chart`. This module is the view's
//! use of that math, plus the button rules that sit beside `Session`.
//! Sampling a captured image only reads pixels; it does not draw.

use godot::classes::Image;
use godot::prelude::*;
use portlight_chart::{plates_for_class, ship_draw, Facing, ShipDraw};
use portlight_sim::encounter::EncounterState;
use portlight_sim::session::{EncounterStep, Session, VictoryReceipt};
use portlight_sim::SimError;

/// Playable window in `godot/project.godot`.
pub(crate) const WINDOW_W: f32 = 1280.0;
pub(crate) const WINDOW_H: f32 = 720.0;
/// Side panel. Wide enough for a lane label and a Sail button.
pub(crate) const PANEL_MIN_W: f32 = 420.0;
/// `HBoxContainer` separation set on the chart row.
pub(crate) const ROW_SEPARATION: i32 = 4;

/// Width left for the chart once the panel and the row gap are on screen.
pub(crate) fn chart_host_width() -> f32 {
    WINDOW_W - PANEL_MIN_W - ROW_SEPARATION as f32
}

pub(crate) fn layout_fits_window() -> bool {
    chart_host_width() > 0.0
        && chart_host_width() + PANEL_MIN_W + ROW_SEPARATION as f32 <= WINDOW_W
        && WINDOW_H == 720.0
}

/// Lines for one `Session::advance`, in the order the chart log already used,
/// with [`portlight_sim::Turn::notes`] included.
///
/// Empty strings are dropped. A missing voyage line adds nothing. Notes sit
/// with the day's events so upkeep, seizure, default, and claim text are not
/// discarded before the docked or at-sea line.
pub(crate) fn day_log_lines(
    event_messages: &[String],
    shocks: &[String],
    notes: &[String],
    voyage_line: Option<&str>,
) -> Vec<String> {
    let mut lines: Vec<String> = event_messages
        .iter()
        .chain(shocks.iter())
        .chain(notes.iter())
        .filter(|line| !line.is_empty())
        .cloned()
        .collect();
    if let Some(line) = voyage_line.filter(|line| !line.is_empty()) {
        lines.push(line.to_string());
    }
    lines
}

/// The Duel button is available whenever a duel is pending. Stance count is
/// the sim's check (`Session::duel`), not a second gate in the view.
pub(crate) fn duel_button_enabled(pending: bool) -> bool {
    pending
}

/// The lane box at sea. Sailing is refused there, so the note points at the
/// one action that moves the voyage.
pub(crate) const AT_SEA_LANE_NOTE: &str = "At sea. Use Next day to continue.";

/// A docked port that offers no lane. Not reachable with the shipped
/// content; a neutral line so the box is never left blank or wrong.
pub(crate) const NO_LANES_NOTE: &str = "No lanes from here.";

/// What the lane box says when the chart has no lane to list: the at-sea
/// note while a voyage is on, else the neutral docked line.
pub(crate) fn empty_lane_note(voyage_present: bool) -> &'static str {
    if voyage_present {
        AT_SEA_LANE_NOTE
    } else {
        NO_LANES_NOTE
    }
}

/// First line of the chart status block. The resource is called Stores, as
/// on the Stores button, with the count last.
pub(crate) fn status_header(
    day: i64,
    season: &str,
    captain: &str,
    silver: i64,
    stores: i64,
) -> String {
    format!("Day {day}   {season}   {captain}   {silver} silver   Stores {stores}")
}

/// The docked price line under the port row.
pub(crate) fn services_line(sailor_cost: i64, stores_cost: i64) -> String {
    format!("Sailor listed {sailor_cost} silver. Stores listed {stores_cost} silver a day.")
}

/// A captured window is useless when one colour covers almost every sample.
/// The broken 1280×720 chart shot was the clear colour below a 28 px strip.
pub(crate) fn frame_mostly_flat(samples: &[[u8; 3]]) -> bool {
    dominant_color_fraction(samples) >= 0.80
}

/// `PORTLIGHT_SHOT` and the harbour seam share this rejection.
/// An empty image, the wrong size, or one colour over most of the frame fails.
pub(crate) fn capture_frame_rejected(
    width: i32,
    height: i32,
    expect_w: i32,
    expect_h: i32,
    samples: &[[u8; 3]],
) -> bool {
    samples.is_empty() || width != expect_w || height != expect_h || frame_mostly_flat(samples)
}

/// A desk dimmed behind a card is mostly the ink ground (about 90%), so the
/// 80% flat rule would reject a good frame. It still fails when it is empty,
/// the wrong size, or one colour over almost every sample (a blank frame).
pub(crate) fn dimmed_frame_rejected(
    width: i32,
    height: i32,
    expect_w: i32,
    expect_h: i32,
    samples: &[[u8; 3]],
) -> bool {
    samples.is_empty()
        || width != expect_w
        || height != expect_h
        || dominant_color_fraction(samples) >= 0.97
}

/// Plate-panel fill, `Color::from_rgb(0.72, 0.58, 0.36)`.
const ENCOUNTER_PLATE_RGB: [u8; 3] = [184, 148, 92];
/// Action-button fill, `Color::from_rgb(0.55, 0.42, 0.24)`.
const ENCOUNTER_BUTTON_RGB: [u8; 3] = [140, 107, 61];

/// Encounter captures do not use [`capture_frame_rejected`].
///
/// The UI plate layout shrink-wraps the plate, so the ink ground `(20, 28, 41)`
/// covers most of a 1280×720 frame (measured at about 93%). That is the
/// signed-off layout, not a blank shot. A frame still fails when it is empty,
/// the wrong size, or missing the plate panel and a filled button.
pub(crate) fn encounter_frame_rejected(
    width: i32,
    height: i32,
    expect_w: i32,
    expect_h: i32,
    samples: &[[u8; 3]],
) -> bool {
    ink_frame_rejected(width, height, expect_w, expect_h, samples)
}

/// New-game captures are 1280×800. The same plate and button fills have to
/// be on the frame. An ink ground that covers most of the shot is the layout.
pub(crate) const NEWGAME_SHOT_W: i32 = 1280;
pub(crate) const NEWGAME_SHOT_H: i32 = 800;

pub(crate) fn newgame_frame_rejected(width: i32, height: i32, samples: &[[u8; 3]]) -> bool {
    ink_frame_rejected(width, height, NEWGAME_SHOT_W, NEWGAME_SHOT_H, samples)
}

fn ink_frame_rejected(
    width: i32,
    height: i32,
    expect_w: i32,
    expect_h: i32,
    samples: &[[u8; 3]],
) -> bool {
    if samples.is_empty() || width != expect_w || height != expect_h {
        return true;
    }
    let plate = sample_fraction(samples, ENCOUNTER_PLATE_RGB);
    let button = sample_fraction(samples, ENCOUNTER_BUTTON_RGB);
    // A 64×64 plate at 2× plus padding is ~3% of a 720-tall frame, and still
    // above 1% at 800. One stray pixel must not pass a blank ink overlay.
    plate < 0.01 || button < 0.002
}

fn sample_fraction(samples: &[[u8; 3]], color: [u8; 3]) -> f32 {
    let hits = samples.iter().filter(|sample| **sample == color).count();
    hits as f32 / samples.len() as f32
}

/// Process status for `harbour_seam.tscn`.
/// An illegal layout and a rejected frame both exit non-zero.
/// The scene must not write a PNG when `layout_ok` is false.
pub(crate) fn seam_exit_code(layout_ok: bool, frames_ok: bool) -> i32 {
    if layout_ok && frames_ok {
        0
    } else {
        1
    }
}

/// Every eighth pixel. `PORTLIGHT_SHOT` and the harbour seam both use this.
pub(crate) fn frame_samples(image: &Gd<Image>) -> Vec<[u8; 3]> {
    let width = image.get_width();
    let height = image.get_height();
    let mut samples = Vec::new();
    let step = 8;
    let mut y = 0;
    while y < height {
        let mut x = 0;
        while x < width {
            let color = image.get_pixel(x, y);
            samples.push([
                (color.r.clamp(0.0, 1.0) * 255.0).round() as u8,
                (color.g.clamp(0.0, 1.0) * 255.0).round() as u8,
                (color.b.clamp(0.0, 1.0) * 255.0).round() as u8,
            ]);
            x += step;
        }
        y += step;
    }
    samples
}

/// Character art is held. The encounter portrait slot is a placeholder.
pub(crate) const PORTRAIT_PLACEHOLDER: &str = "Placeholder";

/// `parity/scripts/boarding.txt`, sailed out of port first.
/// These are the script's commands, not a rules table the screen invents.
pub(crate) const SCRIPTED_CAPTAIN_TYPE: &str = "privateer";
pub(crate) const SCRIPTED_NAME: &str = "Ada";
pub(crate) const SCRIPTED_SEED: i128 = 4;
pub(crate) const SCRIPTED_DEPART: &str = "thornport";
pub(crate) const SCRIPTED_CAPTAIN: &str = "raj_the_quiet";
pub(crate) const SCRIPTED_NAVAL: &[&str] =
    &["broadside", "broadside", "rake", "evade", "close", "close"];
pub(crate) const SCRIPTED_FIGHT: &[&str] = &[
    "thrust", "slash", "parry", "dodge", "thrust", "thrust", "thrust", "thrust",
];

/// The stance duel on the chart stays in charge when this screen is closed.
/// An open encounter covers it. Closing the encounter hands that duel back.
pub(crate) fn stance_duel_visible(screen_open: bool, pending_duel: bool) -> bool {
    pending_duel && !screen_open
}

/// Exact integer scale for a plate in a UI panel.
pub(crate) const UI_PLATE_SCALE: i32 = 2;
/// Padding on each side of that plate. A 64×64 sloop is a 168×168 panel.
pub(crate) const UI_PLATE_PAD: i32 = 20;

pub(crate) fn ui_plate_panel(canvas_w: i32, canvas_h: i32) -> (i32, i32) {
    (
        canvas_w * UI_PLATE_SCALE + UI_PLATE_PAD * 2,
        canvas_h * UI_PLATE_SCALE + UI_PLATE_PAD * 2,
    )
}

/// Side-on plate for one ship template, via [`ship_draw`].
///
/// `royal_man_of_war` draws the galleon f3 plate and keeps the class name
/// `man_of_war`. An unknown template logs one warning on the chart's
/// warned-once set, then uses the sloop. An empty id is the plate widget
/// before a ship exists, so it does not log.
pub(crate) fn encounter_plate(template_id: &str) -> ShipDraw {
    if template_id.is_empty() {
        return plates_for_class("sloop", Facing::F3);
    }
    ship_draw(template_id, Facing::F3)
}

/// Hull and crew the screen can read off [`Session::world`].
#[derive(Debug, Clone, Copy)]
pub(crate) struct PlayerShip {
    pub hull: i64,
    pub hull_max: i64,
    pub crew: i64,
}

/// Hull and crew a newly fitted template has. `Ship::from_template` is what
/// `Session::buy_ship` stores: hull starts at `hull_max`, crew at `crew_min`.
pub(crate) fn template_player_ship(template_id: &str) -> Option<PlayerShip> {
    let template = portlight_sim::content::content().ship(template_id)?;
    let ship = portlight_sim::model::Ship::from_template(template);
    Some(PlayerShip {
        hull: ship.hull,
        hull_max: ship.hull_max,
        crew: ship.crew,
    })
}

pub(crate) fn player_ship(session: &Session) -> Option<PlayerShip> {
    session
        .world()
        .captain
        .ship
        .as_ref()
        .map(|ship| PlayerShip {
            hull: ship.hull,
            hull_max: ship.hull_max,
            crew: ship.crew,
        })
}

pub(crate) fn at_sea(session: &Session) -> bool {
    session.world().voyage.status == portlight_sim::model::VoyageStatus::AtSea
}

/// Sentence [`Session`] already produced. Empty when both fields are empty.
pub(crate) fn session_text(message: &str, flavor: &str) -> String {
    if !message.is_empty() {
        ascii_punctuation(message)
    } else {
        ascii_punctuation(flavor)
    }
}

/// Display filter for sim text: an em or en dash becomes the ASCII
/// ` - ` separator, a Unicode minus becomes `-`, curly quotes become straight
/// quotes, an ellipsis becomes `...` and a middle dot becomes ` - `. Letters
/// are left alone, so a name keeps its accents.
pub(crate) fn ascii_punctuation(text: &str) -> String {
    if text.is_ascii() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\u{2014}' | '\u{2013}' | '\u{00b7}' => {
                let trimmed = out.trim_end_matches(' ').len();
                out.truncate(trimmed);
                out.push_str(" - ");
            }
            '\u{2212}' | '\u{2010}' | '\u{2011}' => out.push('-'),
            '\u{2018}' | '\u{2019}' => out.push('\''),
            '\u{201c}' | '\u{201d}' => out.push('"'),
            '\u{2026}' => out.push_str("..."),
            '\u{00a0}' => out.push(' '),
            c => out.push(c),
        }
    }
    // ` - ` already carries its spaces; fold the doubled ones it made.
    while out.contains("-  ") {
        out = out.replace("-  ", "- ");
    }
    out.trim_start_matches(" - ").to_string()
}

/// `SimError::InvalidAction` lists the actions [`Session`] would accept.
/// A rejected probe is how the screen learns that list.
pub(crate) fn action_list_from_error(err: &SimError) -> Option<Vec<String>> {
    match err {
        SimError::InvalidAction(list) => Some(split_session_actions(list)),
        _ => None,
    }
}

pub(crate) fn split_session_actions(list: &str) -> Vec<String> {
    list.split(',')
        .map(str::trim)
        .filter(|action| !action.is_empty())
        .map(str::to_string)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScreenPhase {
    Approach,
    Naval,
    Boarding,
    /// Personal fight (`Session::fight`). Not the voyage stance duel.
    Personal,
    Outcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ScreenAction {
    Choice(&'static str),
    Naval(String),
    Board,
    Combat(String),
    Spare,
    Capture,
    TakeAll,
    Return { at_sea: bool },
}

/// What the last Session call (or the content catalog) exposed.
/// The screen does not fill these by running combat math.
#[derive(Debug, Clone)]
pub(crate) struct EncounterFacts {
    pub phase: String,
    pub kind: String,
    pub captain_id: String,
    pub captain_name: String,
    pub faction_id: String,
    pub strength: i64,
    pub player_hull: i64,
    pub player_hull_max: Option<i64>,
    pub player_crew: i64,
    pub enemy_hull: Option<i64>,
    pub enemy_hull_max: Option<i64>,
    pub enemy_crew: Option<i64>,
    pub player_hp: i64,
    pub opponent_hp: i64,
    /// Signed hull, crew, and HP changes from the last resolved step.
    /// Zero means that side did not change. HP is the negation of
    /// `damage_to_player` / `damage_to_opponent` (those amounts are not signed).
    /// Stamina stays on the step; this line does not show it.
    pub player_hull_delta: i64,
    pub enemy_hull_delta: i64,
    pub player_crew_delta: i64,
    pub enemy_crew_delta: i64,
    pub player_hp_delta: i64,
    pub opponent_hp_delta: i64,
    pub log: String,
    pub pending_victory: bool,
    pub at_sea: bool,
    /// `false` until [`Session::encounter_choice_with`] has opened it.
    pub on_session: bool,
    pub naval_actions: Vec<String>,
    pub combat_actions: Vec<String>,
    /// World reading taken when the encounter opened (section 10.3). Godot
    /// memory only, never saved. Carried across steps; taken (set to
    /// `None`) when the non-win receipt is logged, so it logs once.
    pub baseline: Option<EncounterBaseline>,
}

/// Silver, flagship crew and hull, and total cargo units when an encounter
/// opens. The step carries no silver or cargo delta, so a non-win receipt
/// diffs the world against this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EncounterBaseline {
    pub silver: i64,
    pub crew: i64,
    pub hull: i64,
    pub cargo_units: i64,
}

impl EncounterBaseline {
    pub(crate) fn read(world: &portlight_sim::model::World) -> Self {
        let ship = world.captain.ship.as_ref();
        Self {
            silver: world.captain.silver,
            crew: ship.map_or(0, |ship| ship.crew),
            hull: ship.map_or(0, |ship| ship.hull),
            cargo_units: world.captain.cargo.iter().map(|item| item.quantity).sum(),
        }
    }

    /// `[silver, crew, hull, cargo]` as `now - self`.
    pub(crate) fn deltas(&self, now: &Self) -> [i64; 4] {
        [
            now.silver - self.silver,
            now.crew - self.crew,
            now.hull - self.hull,
            now.cargo_units - self.cargo_units,
        ]
    }
}

/// How a fight ended without a win (section 10.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EncounterEnd {
    LostFight,
    DrewFight,
    LostAtSea,
    BrokeAway,
}

/// The non-win end a resolved step reports, or `None` (still running, a
/// pending win, an enemy sunk, a negotiated peace, a capture).
pub(crate) fn encounter_end(step: &EncounterStep, pending_victory: bool) -> Option<EncounterEnd> {
    if step.phase != "resolved" || pending_victory || step.player_won || step.enemy_sunk {
        return None;
    }
    if step.escaped {
        return Some(EncounterEnd::BrokeAway);
    }
    match step.kind.as_str() {
        "fight" if step.draw => Some(EncounterEnd::DrewFight),
        "fight" => Some(EncounterEnd::LostFight),
        "naval" => Some(EncounterEnd::LostAtSea),
        _ => None,
    }
}

/// A pirate captain's name for player copy: the live name when it is ASCII,
/// else the catalog name, else [`humanize_id`]. Never the raw id.
pub(crate) fn captain_display_name(name: &str, captain_id: &str) -> String {
    if !name.is_empty() && name.is_ascii() {
        name.to_string()
    } else {
        encounter_end_name(captain_id)
    }
}

/// A ship's name for player copy: its own name when it is ASCII, else the
/// catalog hull name, else [`humanize_id`] of the template id.
pub(crate) fn ship_display_name(name: &str, template_id: &str) -> String {
    if !name.is_empty() && name.is_ascii() {
        return name.to_string();
    }
    let catalog = portlight_sim::content::content()
        .ship(template_id)
        .map(|template| template.name.as_str());
    display_or_humanized(catalog, template_id)
}

/// `{Name}` in the non-win receipt: the catalog display name, or
/// [`humanize_id`] when the catalog has none or it isn't ASCII. Never the
/// raw id. Empty only when the id is empty.
pub(crate) fn encounter_end_name(captain_id: &str) -> String {
    let catalog = portlight_sim::content::content()
        .pirate(captain_id)
        .map(|pirate| pirate.name.as_str());
    display_or_humanized(catalog, captain_id)
}

/// A display name, else [`humanize_id`]. Never the raw id.
pub(crate) fn display_or_humanized(display: Option<&str>, id: &str) -> String {
    match display {
        Some(name) if !name.is_empty() && name.is_ascii() => name.to_string(),
        _ => humanize_id(id),
    }
}

/// One plain receipt line for a fight that ended without a win. Terms in
/// order Silver, Crew, Hull, Cargo; zero terms omitted (R4). No odds, no
/// advice, no Trust or Standing.
pub(crate) fn encounter_end_line(
    end: EncounterEnd,
    name: &str,
    deltas: [i64; 4],
    bounty_open: bool,
) -> String {
    let (head, joiner) = match end {
        EncounterEnd::LostFight => ("Lost the fight", "with"),
        EncounterEnd::DrewFight => ("Drew the fight", "with"),
        EncounterEnd::LostAtSea => ("Lost the sea fight", "with"),
        EncounterEnd::BrokeAway => ("Broke away", "from"),
    };
    let mut line = if name.is_empty() {
        head.to_string()
    } else {
        format!("{head} {joiner} {name}")
    };
    for (term, value) in ["Silver", "Crew", "Hull", "Cargo"].iter().zip(deltas) {
        if value != 0 {
            line.push_str(&format!(" - {term} {}", signed_delta(value)));
        }
    }
    if bounty_open {
        line.push_str(" - the bounty stays open");
    }
    line
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EncounterView {
    pub phase: ScreenPhase,
    pub title: &'static str,
    pub card: String,
    pub log: String,
    /// One signed-delta line for the last resolve. Empty when every delta is zero.
    /// Clauses are separate spans so player loss and enemy loss can take
    /// different colours. Concatenate [`DeltaSpan::text`] for the line.
    pub delta: Vec<DeltaSpan>,
    pub actions: Vec<ScreenAction>,
    /// Muted lines under Spare and Take all on a win's Outcome card. `None`
    /// on every other card, including after the choice (#41's receipt is the
    /// post-click truth).
    pub choice_preview: Option<ChoicePreview>,
    /// Always set. The portrait slot is [`PORTRAIT_PLACEHOLDER`].
    pub portrait_placeholder: bool,
}

/// Pre-click copy for the two mercy/greed buttons. Only the deterministic
/// purse and standing are shown. Loot, morale, and departures roll at
/// finalize, so they are never previewed as values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChoicePreview {
    pub spare: String,
    pub take_all: String,
}

impl ChoicePreview {
    /// The line for `action`. Capture has none in P0.
    pub(crate) fn line_for(&self, action: &ScreenAction) -> Option<&str> {
        match action {
            ScreenAction::Spare => Some(&self.spare),
            ScreenAction::TakeAll => Some(&self.take_all),
            _ => None,
        }
    }
}

/// Underworld standing on a won duel when the captain spares
/// (`record_duel_standing` in `session.rs`).
pub(crate) const SPARE_STANDING: i64 = 5;
/// Underworld standing on a won duel when the captain takes all.
pub(crate) const TAKE_ALL_STANDING: i64 = 2;

/// Silver `Session::spare` pays: `20 + strength * 3` (`finalize_victory`).
pub(crate) fn spare_purse(strength: i64) -> i64 {
    strength.saturating_mul(3).saturating_add(20)
}

/// Silver `Session::take_all` pays: `20 + strength * 7` (`finalize_victory`).
pub(crate) fn take_all_purse(strength: i64) -> i64 {
    strength.saturating_mul(7).saturating_add(20)
}

/// `+32 silver - Underworld +5.` for Strength 4. ASCII only.
pub(crate) fn spare_preview(strength: i64) -> String {
    format!(
        "{} silver - Underworld {}.",
        signed_delta(spare_purse(strength)),
        signed_delta(SPARE_STANDING)
    )
}

/// `+48 silver - Underworld +2 - loot unknown.` for Strength 4. ASCII only.
pub(crate) fn take_all_preview(strength: i64) -> String {
    format!(
        "{} silver - Underworld {} - loot unknown.",
        signed_delta(take_all_purse(strength)),
        signed_delta(TAKE_ALL_STANDING)
    )
}

pub(crate) fn choice_preview(strength: i64) -> ChoicePreview {
    ChoicePreview {
        spare: spare_preview(strength),
        take_all: take_all_preview(strength),
    }
}

/// Whose loss a clause names. A gain, a label, or a separator is [`DeltaTone::Neutral`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeltaTone {
    /// Player hull, crew, or HP went down. Chart danger tone.
    PlayerLoss,
    /// Enemy hull, crew, or opponent HP went down. Gold.
    EnemyLoss,
    Neutral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeltaSpan {
    pub text: String,
    pub tone: DeltaTone,
}

pub(crate) fn action_caption(action: &ScreenAction) -> String {
    match action {
        ScreenAction::Choice(choice) => capitalize(choice),
        ScreenAction::Naval(action) | ScreenAction::Combat(action) => capitalize(action),
        ScreenAction::Board => "Board".to_string(),
        ScreenAction::Spare => "Spare".to_string(),
        ScreenAction::Capture => "Capture".to_string(),
        ScreenAction::TakeAll => "Take all".to_string(),
        ScreenAction::Return { at_sea: true } => "Back to the voyage".to_string(),
        ScreenAction::Return { at_sea: false } => "Back to the chart".to_string(),
    }
}

/// Approach card for a catalog captain the next `encounter_choice_with` locks.
/// Name, faction, and strength come from content. Enemy hull is absent until
/// Session returns it.
pub(crate) fn facts_for_catalog_captain(
    id: &str,
    ship: Option<PlayerShip>,
    at_sea: bool,
) -> Option<EncounterFacts> {
    let catalog = portlight_sim::content::content();
    let pirate = catalog.pirate(id)?;
    let mut facts = facts_shell("approach", "approach", ship, at_sea);
    facts.captain_id = pirate.id.clone();
    facts.captain_name = pirate.name.clone();
    facts.faction_id = pirate.faction_id.clone();
    facts.strength = pirate.strength;
    Some(facts)
}

pub(crate) fn facts_from_agency(
    state: &EncounterState,
    ship: Option<PlayerShip>,
    log: &str,
    at_sea: bool,
) -> EncounterFacts {
    let mut facts = facts_shell(&state.phase, "agency", ship, at_sea);
    facts.captain_id = state.enemy_captain_id.clone();
    facts.captain_name = state.enemy_captain_name.clone();
    facts.faction_id = state.enemy_faction_id.clone();
    facts.strength = state.enemy_strength;
    facts.enemy_hull = Some(state.enemy_ship_hull);
    facts.enemy_hull_max = Some(state.enemy_ship_hull_max);
    facts.enemy_crew = Some(state.enemy_ship_crew);
    facts.log = log.to_string();
    facts.on_session = true;
    facts
}

pub(crate) struct StepInput<'a> {
    pub step: &'a EncounterStep,
    pub previous_faction_id: &'a str,
    pub previous_enemy_hull_max: Option<i64>,
    pub pending_victory: bool,
    pub ship: Option<PlayerShip>,
    pub at_sea: bool,
    pub naval_actions: &'a [String],
    pub combat_actions: &'a [String],
}

pub(crate) fn facts_from_step(input: StepInput<'_>) -> EncounterFacts {
    let step = input.step;
    let faction_id = if !input.previous_faction_id.is_empty() {
        input.previous_faction_id.to_string()
    } else {
        portlight_sim::content::content()
            .pirate(&step.enemy_captain_id)
            .map(|pirate| pirate.faction_id.clone())
            .unwrap_or_default()
    };
    let mut facts = facts_shell(&step.phase, &step.kind, input.ship, input.at_sea);
    facts.captain_id = step.enemy_captain_id.clone();
    facts.captain_name = step.enemy_captain_name.clone();
    facts.faction_id = faction_id;
    facts.strength = step.enemy_strength;
    facts.enemy_hull = Some(step.enemy_hull);
    facts.enemy_hull_max = input.previous_enemy_hull_max;
    facts.enemy_crew = Some(step.enemy_crew);
    facts.log = session_text(&step.message, &step.flavor);
    facts.pending_victory = input.pending_victory;
    facts.on_session = step.phase != "resolved" || input.pending_victory;
    facts.player_hull = step.player_hull;
    facts.player_crew = step.player_crew;
    facts.player_hp = step.player_hp;
    facts.opponent_hp = step.opponent_hp;
    facts.player_hull_delta = step.player_hull_delta;
    facts.enemy_hull_delta = step.enemy_hull_delta;
    if step.kind == "board" {
        // `board_step` sets only the `*_crew_lost` amounts and leaves the
        // deltas at 0, so the #40 line was silent on the boarding loss.
        facts.player_crew_delta = -step.player_crew_lost;
        facts.enemy_crew_delta = -step.enemy_crew_lost;
    } else {
        facts.player_crew_delta = step.player_crew_delta;
        facts.enemy_crew_delta = step.enemy_crew_delta;
    }
    // Damage amounts are non-negative. The line shows them as signed HP changes.
    facts.player_hp_delta = -step.damage_to_player;
    facts.opponent_hp_delta = -step.damage_to_opponent;
    facts.naval_actions = input.naval_actions.to_vec();
    facts.combat_actions = input.combat_actions.to_vec();
    facts
}

/// Buttons and copy for one phase. `None` leaves the chart (and any stance
/// duel) in charge.
pub(crate) fn present(facts: &EncounterFacts) -> Option<EncounterView> {
    if facts.pending_victory {
        return Some(view(
            ScreenPhase::Outcome,
            facts,
            vec![
                ScreenAction::Spare,
                ScreenAction::Capture,
                ScreenAction::TakeAll,
            ],
        ));
    }
    let phase = match facts.phase.as_str() {
        "approach" => ScreenPhase::Approach,
        "naval" => ScreenPhase::Naval,
        "boarding" => ScreenPhase::Boarding,
        "duel" => ScreenPhase::Personal,
        "capture_available" => ScreenPhase::Outcome,
        "resolved" => ScreenPhase::Outcome,
        _ => return None,
    };
    let actions = match phase {
        ScreenPhase::Approach => vec![
            ScreenAction::Choice("negotiate"),
            ScreenAction::Choice("flee"),
            ScreenAction::Choice("fight"),
        ],
        ScreenPhase::Naval => facts
            .naval_actions
            .iter()
            .cloned()
            .map(ScreenAction::Naval)
            .collect(),
        ScreenPhase::Boarding => vec![ScreenAction::Board],
        ScreenPhase::Personal => facts
            .combat_actions
            .iter()
            .cloned()
            .map(ScreenAction::Combat)
            .collect(),
        ScreenPhase::Outcome if facts.phase == "capture_available" => {
            vec![ScreenAction::Capture]
        }
        ScreenPhase::Outcome => vec![ScreenAction::Return {
            at_sea: facts.at_sea,
        }],
    };
    Some(view(phase, facts, actions))
}

fn view(phase: ScreenPhase, facts: &EncounterFacts, actions: Vec<ScreenAction>) -> EncounterView {
    EncounterView {
        phase,
        title: phase_title(phase),
        card: card_text(facts, phase),
        log: facts.log.clone(),
        delta: delta_spans(facts),
        choice_preview: (phase == ScreenPhase::Outcome && facts.pending_victory)
            .then(|| choice_preview(facts.strength)),
        actions,
        portrait_placeholder: true,
    }
}

/// ASCII receipt. Flavor and departure text are copied from the receipt.
pub(crate) fn victory_receipt_lines(receipt: &VictoryReceipt) -> Vec<String> {
    let mut lines = Vec::new();
    let header = if receipt.spared {
        format!("Spared {}.", receipt.enemy_captain_name)
    } else {
        format!("Took all from {}.", receipt.enemy_captain_name)
    };
    push_receipt_line(&mut lines, header);
    push_receipt_line(&mut lines, format!("+{} silver.", receipt.silver_delta));
    for message in &receipt.loot_messages {
        push_receipt_line(&mut lines, message.clone());
    }
    if receipt.standing_delta != 0 {
        let signed = if receipt.standing_delta > 0 {
            format!("+{}", receipt.standing_delta)
        } else {
            receipt.standing_delta.to_string()
        };
        push_receipt_line(
            &mut lines,
            format!(
                "Underworld standing {} {signed}.",
                faction_name(&receipt.faction_id)
            ),
        );
    }
    for (_, _, flavor) in &receipt.reactions {
        push_receipt_line(&mut lines, flavor.clone());
    }
    for departure in &receipt.departures {
        push_receipt_line(
            &mut lines,
            format!(
                "{} leaves. {}",
                departure.companion_name, departure.departure_line
            ),
        );
    }
    lines
}

fn push_receipt_line(lines: &mut Vec<String>, line: String) {
    if !line.is_empty() {
        lines.push(line);
    }
}

fn phase_title(phase: ScreenPhase) -> &'static str {
    match phase {
        ScreenPhase::Approach => "Approach",
        ScreenPhase::Naval => "Naval",
        ScreenPhase::Boarding => "Boarding",
        ScreenPhase::Personal => "Personal fight",
        ScreenPhase::Outcome => "Outcome",
    }
}

fn delta_span(text: impl Into<String>, tone: DeltaTone) -> DeltaSpan {
    DeltaSpan {
        text: text.into(),
        tone,
    }
}

/// ASCII `+` / `-` only. Zero is omitted by the caller, so it is not formatted.
fn signed_delta(value: i64) -> String {
    if value > 0 {
        format!("+{value}")
    } else {
        format!("{value}")
    }
}

fn delta_tone(value: i64, player: bool) -> DeltaTone {
    if value >= 0 {
        DeltaTone::Neutral
    } else if player {
        DeltaTone::PlayerLoss
    } else {
        DeltaTone::EnemyLoss
    }
}

fn push_clause(spans: &mut Vec<DeltaSpan>, any: &mut bool) {
    if *any {
        spans.push(delta_span(" | ", DeltaTone::Neutral));
    }
    *any = true;
}

fn push_party(spans: &mut Vec<DeltaSpan>, parts: Vec<DeltaSpan>) {
    for (index, part) in parts.into_iter().enumerate() {
        if index > 0 {
            spans.push(delta_span(", ", DeltaTone::Neutral));
        }
        spans.push(part);
    }
}

/// One ASCII line from the step deltas already stored on `facts`.
/// Zero values are dropped. Hull clauses use ` | `; crew and HP list each side.
fn delta_spans(facts: &EncounterFacts) -> Vec<DeltaSpan> {
    let mut spans = Vec::new();
    let mut any = false;
    if facts.player_hull_delta != 0 {
        push_clause(&mut spans, &mut any);
        spans.push(delta_span(
            format!("Your hull {}", signed_delta(facts.player_hull_delta)),
            delta_tone(facts.player_hull_delta, true),
        ));
    }
    if facts.enemy_hull_delta != 0 {
        push_clause(&mut spans, &mut any);
        spans.push(delta_span(
            format!("Enemy hull {}", signed_delta(facts.enemy_hull_delta)),
            delta_tone(facts.enemy_hull_delta, false),
        ));
    }
    let mut crew = Vec::new();
    if facts.player_crew_delta != 0 {
        crew.push(delta_span(
            format!("you {}", signed_delta(facts.player_crew_delta)),
            delta_tone(facts.player_crew_delta, true),
        ));
    }
    if facts.enemy_crew_delta != 0 {
        crew.push(delta_span(
            format!("enemy {}", signed_delta(facts.enemy_crew_delta)),
            delta_tone(facts.enemy_crew_delta, false),
        ));
    }
    if !crew.is_empty() {
        push_clause(&mut spans, &mut any);
        spans.push(delta_span("Crew: ", DeltaTone::Neutral));
        push_party(&mut spans, crew);
    }
    let mut hp = Vec::new();
    if facts.player_hp_delta != 0 {
        hp.push(delta_span(
            format!("you {}", signed_delta(facts.player_hp_delta)),
            delta_tone(facts.player_hp_delta, true),
        ));
    }
    if facts.opponent_hp_delta != 0 {
        hp.push(delta_span(
            format!("opponent {}", signed_delta(facts.opponent_hp_delta)),
            delta_tone(facts.opponent_hp_delta, false),
        ));
    }
    if !hp.is_empty() {
        push_clause(&mut spans, &mut any);
        spans.push(delta_span("HP: ", DeltaTone::Neutral));
        push_party(&mut spans, hp);
    }
    spans
}

fn card_text(facts: &EncounterFacts, phase: ScreenPhase) -> String {
    let mut lines = Vec::new();
    if !facts.captain_name.is_empty() {
        lines.push(facts.captain_name.clone());
    }
    let faction = faction_name(&facts.faction_id);
    if !faction.is_empty() {
        lines.push(faction);
    }
    if !facts.captain_name.is_empty() {
        lines.push(format!("Strength {}", facts.strength));
    }
    lines.push(match facts.player_hull_max {
        Some(max) => format!(
            "Your hull {}/{max} - crew {}",
            facts.player_hull, facts.player_crew
        ),
        None => format!(
            "Your hull {} - crew {}",
            facts.player_hull, facts.player_crew
        ),
    });
    // A win's outcome card keeps the last fight step's hull, which is not a
    // post-fight reading. Session does not expose one, so the line is omitted.
    if phase != ScreenPhase::Outcome {
        if let Some(hull) = facts.enemy_hull {
            let crew = facts.enemy_crew.unwrap_or(0);
            lines.push(match facts.enemy_hull_max {
                Some(max) => format!("Enemy hull {hull}/{max} - crew {crew}"),
                None => format!("Enemy hull {hull} - crew {crew}"),
            });
        }
    }
    if phase == ScreenPhase::Personal && facts.kind == "fight" {
        lines.push(format!(
            "Your HP {} - opponent {}",
            facts.player_hp, facts.opponent_hp
        ));
    }
    lines.join("\n")
}

/// Player copy for a faction id: the catalog name (`The Iron Wolves`) when
/// the faction is known, else [`humanize_id`]. Every faction shown to the
/// player goes through here so one faction never reads two ways.
pub(crate) fn faction_name(faction_id: &str) -> String {
    if faction_id.is_empty() {
        return String::new();
    }
    portlight_sim::content::content()
        .faction(faction_id)
        .map(|faction| faction.name.clone())
        .unwrap_or_else(|| humanize_id(faction_id))
}

fn facts_shell(phase: &str, kind: &str, ship: Option<PlayerShip>, at_sea: bool) -> EncounterFacts {
    EncounterFacts {
        phase: phase.to_string(),
        kind: kind.to_string(),
        captain_id: String::new(),
        captain_name: String::new(),
        faction_id: String::new(),
        strength: 0,
        player_hull: ship.map_or(0, |ship| ship.hull),
        player_hull_max: ship.map(|ship| ship.hull_max),
        player_crew: ship.map_or(0, |ship| ship.crew),
        enemy_hull: None,
        enemy_hull_max: None,
        enemy_crew: None,
        player_hp: 0,
        opponent_hp: 0,
        player_hull_delta: 0,
        enemy_hull_delta: 0,
        player_crew_delta: 0,
        enemy_crew_delta: 0,
        player_hp_delta: 0,
        opponent_hp_delta: 0,
        log: String::new(),
        pending_victory: false,
        at_sea,
        on_session: false,
        naval_actions: Vec::new(),
        combat_actions: Vec::new(),
        baseline: None,
    }
}

fn capitalize(action: &str) -> String {
    let mut chars = action.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn dominant_color_fraction(samples: &[[u8; 3]]) -> f32 {
    if samples.is_empty() {
        return 1.0;
    }
    let mut counts = std::collections::HashMap::<[u8; 3], usize>::new();
    let mut best = 0usize;
    for sample in samples {
        let count = counts.entry(*sample).or_insert(0);
        *count += 1;
        if *count > best {
            best = *count;
        }
    }
    best as f32 / samples.len() as f32
}

/// Title, captain roster, custom builder, load list, or the in-game save note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NewgamePage {
    Hidden,
    Title,
    Captains,
    Custom,
    Load,
    Saved,
}

/// Which of the four custom point pools a button changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PointPool {
    Trade,
    Sailing,
    Shadow,
    Reputation,
}

/// Player choices for [`Session::new_custom`]. Defaults match the Python
/// builder's prompts: 2/3/3/2, Porto Novo, title "Freelance Captain".
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CustomDraft {
    pub name: String,
    pub title: String,
    pub home_port_id: String,
    pub home_region: String,
    pub trade_points: i64,
    pub sailing_points: i64,
    pub shadow_points: i64,
    pub reputation_points: i64,
    pub bloc_alignment: String,
    pub faction_alignment: String,
    pub mentor_npc_id: String,
    pub backstory: String,
}

impl Default for CustomDraft {
    fn default() -> Self {
        Self {
            name: "Ada".to_string(),
            title: "Freelance Captain".to_string(),
            home_port_id: "porto_novo".to_string(),
            home_region: "Mediterranean".to_string(),
            trade_points: 2,
            sailing_points: 3,
            shadow_points: 3,
            reputation_points: 2,
            bloc_alignment: String::new(),
            faction_alignment: String::new(),
            mentor_npc_id: String::new(),
            backstory: String::new(),
        }
    }
}

impl CustomDraft {
    pub(crate) fn spec(&self) -> portlight_sim::CustomCaptainSpec {
        let name = if self.name.trim().is_empty() {
            "Captain".to_string()
        } else {
            self.name.trim().to_string()
        };
        portlight_sim::CustomCaptainSpec {
            name,
            title: self.title.clone(),
            home_port_id: self.home_port_id.clone(),
            home_region: self.home_region.clone(),
            trade_points: self.trade_points,
            sailing_points: self.sailing_points,
            shadow_points: self.shadow_points,
            reputation_points: self.reputation_points,
            bloc_alignment: self.bloc_alignment.clone(),
            faction_alignment: self.faction_alignment.clone(),
            mentor_npc_id: self.mentor_npc_id.clone(),
            backstory: self.backstory.clone(),
        }
    }

    pub(crate) fn total(&self) -> i64 {
        self.trade_points + self.sailing_points + self.shadow_points + self.reputation_points
    }

    pub(crate) fn points_left(&self) -> i64 {
        portlight_sim::custom_captain::TOTAL_SKILL_POINTS - self.total()
    }

    pub(crate) fn pool(&self, pool: PointPool) -> i64 {
        match pool {
            PointPool::Trade => self.trade_points,
            PointPool::Sailing => self.sailing_points,
            PointPool::Shadow => self.shadow_points,
            PointPool::Reputation => self.reputation_points,
        }
    }

    /// Clamp to 0..=7 and to the points still unspent. The total never
    /// passes [`TOTAL_SKILL_POINTS`]. A short total is what `validate_spec`
    /// rejects when the player presses Begin.
    pub(crate) fn set_pool(&mut self, pool: PointPool, value: i64) {
        let cap = portlight_sim::custom_captain::MAX_POINTS_PER_CATEGORY;
        let other = self.total() - self.pool(pool);
        let room = (portlight_sim::custom_captain::TOTAL_SKILL_POINTS - other).max(0);
        let value = value.clamp(0, cap).min(room);
        match pool {
            PointPool::Trade => self.trade_points = value,
            PointPool::Sailing => self.sailing_points = value,
            PointPool::Shadow => self.shadow_points = value,
            PointPool::Reputation => self.reputation_points = value,
        }
    }
}

pub(crate) fn newgame_copy(page: NewgamePage) -> (&'static str, &'static str) {
    match page {
        NewgamePage::Title => ("Portlight", "New game, or load a saved voyage."),
        NewgamePage::Captains => (
            "Choose a captain",
            "Seed 1. The name is the captain you play.",
        ),
        NewgamePage::Custom => (
            "Custom captain",
            "Distribute 10 points. At most 7 in one category. The home port has to sit in the home region.",
        ),
        NewgamePage::Load => (
            "Load game",
            "Each slot shows the captain and the day saved. Custom captains keep their name, silver and day, but their trade bonuses aren't saved yet, so they trade at merchant rates after loading.",
        ),
        NewgamePage::Saved | NewgamePage::Hidden => ("", ""),
    }
}

pub(crate) fn save_confirm_title(ok: bool) -> &'static str {
    if ok {
        "Game saved."
    } else {
        "Save failed."
    }
}

/// TUI load row: `slot  captain  day N`. An empty name is "Unknown".
pub(crate) fn save_slot_label(slot: &portlight_sim::save::SaveSlotSummary) -> String {
    let captain = if slot.captain.is_empty() {
        "Unknown"
    } else {
        slot.captain.as_str()
    };
    format!("{}  {}  day {}", slot.slot, captain, slot.day)
}

pub(crate) fn captain_button_label(
    captain: &portlight_sim::custom_captain::StartingCaptain,
) -> String {
    format!(
        "{}  --  {}  --  {}  --  {}  --  {} silver",
        captain.name,
        captain.title,
        captain.home_port_name,
        captain.starting_ship_class,
        captain.starting_silver
    )
}

/// Catalog text when it is ASCII. Otherwise the id, which the sim stores in ASCII.
pub(crate) fn ascii_label<'a>(text: &'a str, fallback: &'a str) -> &'a str {
    if !text.is_empty() && text.is_ascii() {
        text
    } else {
        fallback
    }
}

/// Player copy for an id that has no display name: underscores become
/// spaces and each word is title-cased (`monsoon_syndicate` reads
/// `Monsoon Syndicate`). Callers that have a real display name use it first.
pub(crate) fn humanize_id(id: &str) -> String {
    id.split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn cycle_index(len: usize, index: usize, delta: i32) -> usize {
    if len == 0 {
        return 0;
    }
    let len = len as i32;
    let index = index as i32;
    (index + delta).rem_euclid(len) as usize
}

/// Shipyard overlay copy. Built from `Session::world` and the embedded catalog.
/// `None` when the captain is not docked or has no flagship.
pub(crate) const NO_FLEET_HERE: &str = "No other ships docked at this port.";
pub(crate) const NO_SHIPYARD_BODY: &str =
    "This port has no shipyard. Repair, rename, dock, and board still work.";
pub(crate) const FLEET_FULL_BUY: &str =
    "Fleet is full. Buying will sell your current flagship for 40% of list price.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShipyardModel {
    pub port_name: String,
    pub has_shipyard: bool,
    pub gate_notice: String,
    pub silver: i64,
    pub fleet_label: String,
    pub buying_sells_flagship: bool,
    pub flagship: FlagshipCard,
    pub offers: Vec<HullOffer>,
    pub upgrades: Vec<UpgradeOffer>,
    pub slots_notice: String,
    pub fleet: Vec<FleetCard>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FlagshipCard {
    pub template_id: String,
    pub name: String,
    pub class_label: String,
    pub hull: i64,
    pub hull_max: i64,
    pub upgrades_used: i64,
    pub upgrade_slots: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HullOffer {
    pub id: String,
    pub name: String,
    pub class_label: String,
    pub price: i64,
    pub cargo: i64,
    pub speed: String,
    pub hull: i64,
    pub cannons: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UpgradeOffer {
    pub id: String,
    pub name: String,
    pub price: i64,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FleetCard {
    pub name: String,
    pub template_id: String,
    pub class_label: String,
    pub hull: i64,
    pub hull_max: i64,
    pub cargo: bool,
}

pub(crate) fn shipyard_model(session: &Session) -> Option<ShipyardModel> {
    let world = session.world();
    if world.voyage.status != portlight_sim::model::VoyageStatus::InPort {
        return None;
    }
    let port = world.port(&world.voyage.destination_id)?;
    let ship = world.captain.ship.as_ref()?;
    let port_name = display_or_humanized(Some(&port.name), &port.id);
    let has_shipyard = port.has_feature("shipyard");
    let trust = world.captain.standing.commercial_trust;
    let fleet_len = world.captain.fleet.len();
    let catalog = portlight_sim::content::content();
    let class_name = catalog
        .ship(&ship.template_id)
        .map(|template| template.ship_class.as_str())
        .unwrap_or("");
    let installed: Vec<&str> = ship
        .upgrades
        .iter()
        .map(|upgrade| upgrade.upgrade_id.as_str())
        .collect();
    let slots_full = ship.upgrades.len() as i64 >= ship.upgrade_slots;
    let fleet = world
        .captain
        .fleet
        .iter()
        .filter(|owned| owned.docked_port_id == port.id)
        .map(|owned| {
            let class_name = catalog
                .ship(&owned.ship.template_id)
                .map(|template| template.ship_class.as_str())
                .unwrap_or("");
            FleetCard {
                name: ship_display_name(&owned.ship.name, &owned.ship.template_id),
                template_id: owned.ship.template_id.clone(),
                class_label: class_label(class_name),
                hull: owned.ship.hull,
                hull_max: owned.ship.hull_max,
                cargo: !owned.cargo.is_empty(),
            }
        })
        .collect();
    Some(ShipyardModel {
        gate_notice: if has_shipyard {
            String::new()
        } else {
            shipyard_gate_sentence(&port_name)
        },
        port_name,
        has_shipyard,
        silver: world.captain.silver,
        fleet_label: fleet_label(fleet_len, trust),
        buying_sells_flagship: buying_sells_flagship(fleet_len, trust),
        flagship: FlagshipCard {
            template_id: ship.template_id.clone(),
            name: ship_display_name(&ship.name, &ship.template_id),
            class_label: class_label(class_name),
            hull: ship.hull,
            hull_max: ship.hull_max,
            upgrades_used: ship.upgrades.len() as i64,
            upgrade_slots: ship.upgrade_slots,
        },
        offers: catalog
            .ships
            .iter()
            .filter(|template| template.id != ship.template_id)
            .map(|template| HullOffer {
                id: template.id.clone(),
                name: display_or_humanized(Some(&template.name), &template.id),
                class_label: class_label(&template.ship_class),
                price: template.price,
                cargo: template.cargo_capacity,
                speed: format_amount(template.speed),
                hull: template.hull_max,
                cannons: template.cannons,
            })
            .collect(),
        upgrades: catalog
            .upgrades
            .iter()
            .filter(|upgrade| !installed.contains(&upgrade.id.as_str()))
            .map(|upgrade| UpgradeOffer {
                id: upgrade.id.clone(),
                name: display_or_humanized(Some(&upgrade.name), &upgrade.id),
                price: upgrade.price,
                summary: upgrade_summary(upgrade),
            })
            .collect(),
        slots_notice: if slots_full {
            slots_full_sentence(ship.upgrade_slots)
        } else {
            String::new()
        },
        fleet,
    })
}

/// Flagship counts as one. Trust bands match `naval::max_fleet_size`.
pub(crate) fn fleet_label(fleet_len: usize, commercial_trust: i64) -> String {
    let now = fleet_len as i64 + 1;
    let cap = portlight_sim::naval::max_fleet_size(commercial_trust);
    format!("Fleet {now}/{cap}")
}

/// `buy_ship` parks the old hull while `fleet.len() + 1` is under the cap.
pub(crate) fn buying_sells_flagship(fleet_len: usize, commercial_trust: i64) -> bool {
    let count = fleet_len as i64 + 1;
    count >= portlight_sim::naval::max_fleet_size(commercial_trust)
}

pub(crate) fn shipyard_gate_sentence(port_name: &str) -> String {
    format!("{port_name} has no shipyard")
}

/// Same sentence `Session::install_upgrade` returns when every slot is taken.
pub(crate) fn slots_full_sentence(slots: i64) -> String {
    format!("No upgrade slots remaining ({slots}/{slots} used)")
}

pub(crate) fn class_label(class_name: &str) -> String {
    match class_name {
        "sloop" => "Sloop".to_string(),
        "cutter" => "Cutter".to_string(),
        "brigantine" => "Brigantine".to_string(),
        "galleon" => "Galleon".to_string(),
        "man_of_war" => "Man-of-war".to_string(),
        "" => "Ship".to_string(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => "Ship".to_string(),
            }
        }
    }
}

pub(crate) fn upgrade_summary(def: &portlight_sim::content::UpgradeDef) -> String {
    let mut parts = Vec::new();
    if def.speed_bonus != 0.0 {
        parts.push(format!("Speed +{}", format_amount(def.speed_bonus)));
    }
    if def.speed_penalty != 0.0 {
        parts.push(format!("Speed -{}", format_amount(def.speed_penalty)));
    }
    if def.hull_max_bonus != 0 {
        parts.push(format!("Hull +{}", def.hull_max_bonus));
    }
    if def.cargo_bonus != 0 {
        parts.push(format!("Cargo +{}", def.cargo_bonus));
    }
    if def.cannon_bonus != 0 {
        parts.push(format!("Cannons +{}", def.cannon_bonus));
    }
    if def.maneuver_bonus != 0.0 {
        parts.push(format!("Maneuver +{}", format_amount(def.maneuver_bonus)));
    }
    if def.storm_resist_bonus != 0.0 {
        parts.push(format!("Storm +{}", format_amount(def.storm_resist_bonus)));
    }
    if def.crew_max_bonus != 0 {
        parts.push(format!("Crew +{}", def.crew_max_bonus));
    }
    // Every term is title case: stat names (`Hull +15`) and the special's
    // name (`Chain Shot`, from the id `chain_shot`).
    if !def.special.is_empty() && def.special.is_ascii() {
        parts.push(humanize_id(&def.special));
    }
    if parts.is_empty() {
        "Fitted upgrade".to_string()
    } else {
        parts.join(", ")
    }
}

fn format_amount(value: f64) -> String {
    if (value - value.round()).abs() < 0.000_001 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

impl FlagshipCard {
    pub(crate) fn lines(&self, silver: i64, fleet_label: &str) -> Vec<String> {
        vec![
            self.name.clone(),
            self.class_label.clone(),
            format!("Hull {}/{}", self.hull, self.hull_max),
            format!("Upgrades {}/{}", self.upgrades_used, self.upgrade_slots),
            format!("Silver {silver}"),
            fleet_label.to_string(),
        ]
    }
}

impl HullOffer {
    pub(crate) fn line(&self) -> String {
        format!(
            "{}  {}  {} silver  cargo {}  speed {}  hull {}  cannons {}",
            self.name,
            self.class_label,
            self.price,
            self.cargo,
            self.speed,
            self.hull,
            self.cannons
        )
    }
}

impl UpgradeOffer {
    pub(crate) fn line(&self) -> String {
        format!("{}  {} silver  {}", self.name, self.price, self.summary)
    }
}

impl FleetCard {
    pub(crate) fn line(&self) -> String {
        format!(
            "{}  {}  hull {}/{}  cargo {}",
            self.name,
            self.class_label,
            self.hull,
            self.hull_max,
            if self.cargo { "yes" } else { "no" }
        )
    }
}

pub(crate) fn buy_confirm_line(offer: &HullOffer, sells_flagship: bool) -> String {
    let ask = format!("Buy {} for {} silver?", offer.name, offer.price);
    if sells_flagship {
        format!("{FLEET_FULL_BUY} {ask}")
    } else {
        ask
    }
}

pub(crate) fn install_confirm_line(name: &str, price: i64) -> String {
    format!("Install {name} for {price} silver?")
}

pub(crate) fn sell_confirm_line(name: &str) -> String {
    format!("Sell {name}? A shipyard pays 30% of list price, scaled by hull.")
}

pub(crate) fn dock_confirm_line() -> &'static str {
    "Dock the flagship here. The first hull already docked at this port becomes the flagship."
}

pub(crate) fn board_confirm_line(name: &str) -> String {
    format!("Board {name}? It becomes the flagship, and the current flagship docks here.")
}

/// `fleet_grew` is the fleet length after `buy_ship` compared with before.
pub(crate) fn buy_result_line(bought: &str, previous_name: &str, fleet_grew: bool) -> String {
    if fleet_grew {
        format!("Bought {bought}. {previous_name} is docked here.")
    } else {
        format!("Bought {bought}. {previous_name} was sold.")
    }
}

/// Session sentences stay intact when they are ASCII. An em dash becomes `-`.
pub(crate) fn ui_sentence(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '\u{2014}' | '\u{2013}' => '-',
            '\u{2018}' | '\u{2019}' => '\'',
            '\u{201c}' | '\u{201d}' => '"',
            other if other.is_ascii() => other,
            _ => ' ',
        })
        .collect()
}

/// Ink overlay at the chart window size. Same plate and button fills as encounter.
pub(crate) fn shipyard_frame_rejected(width: i32, height: i32, samples: &[[u8; 3]]) -> bool {
    encounter_frame_rejected(width, height, WINDOW_W as i32, WINDOW_H as i32, samples)
}

/// Hire / fire rows. Wages and caps match `session.rs` `role_spec`.
/// Sailors have no role cap. Their hire price is the port `crew_cost`.
struct CrewRoleSpec {
    id: &'static str,
    name: &'static str,
    wage: i64,
    max_per_ship: Option<i64>,
}

const CREW_ROLES: &[CrewRoleSpec] = &[
    CrewRoleSpec {
        id: "sailor",
        name: "Sailor",
        wage: 1,
        max_per_ship: None,
    },
    CrewRoleSpec {
        id: "gunner",
        name: "Gunner",
        wage: 2,
        max_per_ship: Some(3),
    },
    CrewRoleSpec {
        id: "navigator",
        name: "Navigator",
        wage: 3,
        max_per_ship: Some(1),
    },
    CrewRoleSpec {
        id: "surgeon",
        name: "Surgeon",
        wage: 3,
        max_per_ship: Some(1),
    },
    CrewRoleSpec {
        id: "marine",
        name: "Marine",
        wage: 2,
        max_per_ship: Some(4),
    },
    CrewRoleSpec {
        id: "quartermaster",
        name: "Quartermaster",
        wage: 2,
        max_per_ship: Some(1),
    },
];

pub(crate) struct CrewRoleLine {
    pub id: &'static str,
    pub name: &'static str,
    /// Read by the desk tests. The row text already shows the number.
    #[allow(dead_code)]
    pub count: i64,
    /// Read by the desk tests. Sailors store `None` and the row prints `-`.
    #[allow(dead_code)]
    pub max: Option<i64>,
    pub hire_cost: i64,
    pub hire_enabled: bool,
    /// Sailors only, and only when five free berths remain.
    pub hire_five: bool,
    pub fire_enabled: bool,
    pub text: String,
}

pub(crate) struct CrewStyleLine {
    pub id: String,
    pub name: String,
    pub cost: i64,
    pub days: i64,
    pub known: bool,
    pub text: String,
}

pub(crate) struct CrewSkillLine {
    pub id: String,
    pub skill_name: String,
    pub level_name: String,
    pub cost: i64,
    pub days: i64,
    pub can_train: bool,
    pub text: String,
    pub next_text: String,
    /// Set when this port has no trainer for the skill.
    pub empty_trainer: Option<String>,
}

pub(crate) struct CrewOfferLine {
    pub id: String,
    pub name: String,
    pub cost: i64,
    pub text: String,
}

/// Read-only desk. `None` when the captain is not in port.
pub(crate) struct CrewDesk {
    pub port_name: String,
    pub status: String,
    /// Read by the desk tests. The provision line already shows the price.
    #[allow(dead_code)]
    pub provision_per_day: i64,
    pub provision_text: String,
    pub roles: Vec<CrewRoleLine>,
    pub officers: Vec<String>,
    pub known_styles: String,
    pub styles: Vec<CrewStyleLine>,
    pub skills: Vec<CrewSkillLine>,
    pub party_line: String,
    pub party: Vec<String>,
    pub offers: Vec<CrewOfferLine>,
}

pub(crate) const NO_FIGHTING_MASTER: &str = "No fighting master at this port.";
pub(crate) const NO_COMPANIONS_FOR_HIRE: &str = "No companions for hire at this port.";

/// Hire of 50 silver or more asks before `Session::hire_crew`.
pub(crate) fn hire_needs_confirm(cost: i64) -> bool {
    cost >= 50
}

pub(crate) fn hire_confirm_line(count: i64, name: &str, cost: i64) -> String {
    let label = if count == 1 {
        name.to_string()
    } else {
        format!("{name}s")
    };
    format!("Hire {count} {label} for {cost} silver.")
}

/// Silver and the day advance. `train_crew` calls `Session::advance` once per day.
/// Session ignores injury gates on this path, so the line does not mention them.
pub(crate) fn train_confirm_line(name: &str, cost: i64, days: i64) -> String {
    format!("Train {name} for {cost} silver. The calendar will advance {days} days.")
}

/// Silver and the day advance. The button is Learn. The method stays
/// `spend_skill_point`, which calls `Session::advance` once per day.
pub(crate) fn skill_confirm_line(level: &str, skill: &str, cost: i64, days: i64) -> String {
    format!("Learn {level} {skill} for {cost} silver. The calendar will advance {days} days.")
}

/// What `Session::provision` charges. Chart copy keeps the listed port price.
pub(crate) fn effective_provision_per_day(
    listed: i64,
    standing: &portlight_sim::model::Standing,
    port_id: &str,
) -> i64 {
    let mult = portlight_sim::reputation::service_modifier(standing, port_id);
    1.max(portlight_sim::util::py_trunc(listed as f64 * mult))
}

pub(crate) fn recruit_confirm_line(name: &str, cost: i64) -> String {
    format!("Recruit {name} for {cost} silver.")
}

fn ascii_owned(text: &str, fallback: &str) -> String {
    ascii_label(text, fallback).to_string()
}

fn role_count(ship: &portlight_sim::model::Ship, role: &str) -> i64 {
    match role {
        "sailor" => ship.sailors,
        "gunner" => ship.gunners,
        "navigator" => ship.navigators,
        "surgeon" => ship.surgeons,
        "marine" => ship.marines,
        "quartermaster" => ship.quartermasters,
        _ => 0,
    }
}

fn role_name(role: &str) -> String {
    CREW_ROLES
        .iter()
        .find(|spec| spec.id == role)
        .map(|spec| spec.name.to_string())
        .unwrap_or_else(|| humanize_id(role))
}

/// Crew desk stores block: the subhead and the two buy buttons.
pub(crate) const CREW_STORES_SUBHEAD: &str = "Stores";
pub(crate) const CREW_STORES_PLUS_FIVE: &str = "Stores +5";
pub(crate) const CREW_STORES_PLUS_ONE: &str = "Stores +1";

/// Docked crew desk from `session.world()` and the embedded catalogs.
/// Does not call hire, fire, provision, train, recruit, or skill.
pub(crate) fn crew_desk(session: &Session) -> Option<CrewDesk> {
    let world = session.world();
    if world.voyage.status != portlight_sim::model::VoyageStatus::InPort {
        return None;
    }
    let port_id = world.voyage.destination_id.as_str();
    let port = world.port(port_id)?;
    let port_name = display_or_humanized(Some(&port.name), port_id);
    let catalog = portlight_sim::content::content();
    let ship = world.captain.ship.as_ref();
    let (crew, crew_max, space) = ship
        .map(|ship| {
            let max = portlight_sim::ship::resolve_crew_max(ship);
            (ship.crew, max, max - ship.crew)
        })
        .unwrap_or((0, 0, 0));
    let silver = world.captain.silver;
    let provisions = world.captain.provisions;
    let status = if ship.is_none() {
        format!("Silver {silver}. Stores {provisions}. No ship.")
    } else {
        format!("Silver {silver}. Stores {provisions}. Crew {crew}/{crew_max}.")
    };
    let provision_per_day =
        effective_provision_per_day(port.provision_cost, &world.captain.standing, port_id);
    let provision_text = format!("Stores {provisions}. {provision_per_day} silver a day.");
    let roles = CREW_ROLES
        .iter()
        .map(|spec| {
            let count = ship.map(|ship| role_count(ship, spec.id)).unwrap_or(0);
            let hire_cost = if spec.id == "sailor" {
                port.crew_cost
            } else {
                spec.wage * 10
            };
            let at_max = spec.max_per_ship.is_some_and(|max| count >= max);
            let hire_enabled = ship.is_some() && space > 0 && !at_max;
            let max_text = spec
                .max_per_ship
                .map(|max| format!("max {max}"))
                .unwrap_or_else(|| "no cap".to_string());
            CrewRoleLine {
                id: spec.id,
                name: spec.name,
                count,
                max: spec.max_per_ship,
                hire_cost,
                hire_enabled,
                hire_five: spec.id == "sailor" && space >= 5,
                fire_enabled: count > 0,
                text: format!(
                    "{name}  {count}  {max_text}  hire {hire_cost}",
                    name = spec.name
                ),
            }
        })
        .collect();
    let officers = ship
        .map(|ship| {
            ship.officers
                .iter()
                .map(|officer| {
                    let name = ascii_owned(&officer.name, "Officer");
                    let role = role_name(&officer.role);
                    let trait_name = ascii_owned(&officer.trait_name, "steady");
                    format!("{name} - {role} - {trait_name}")
                })
                .collect()
        })
        .unwrap_or_default();
    let learned = &world.captain.learned_styles;
    let styles: Vec<CrewStyleLine> = portlight_sim::training::available_training(port_id)
        .into_iter()
        .filter_map(|id| {
            let style = catalog.fighting_style(&id)?;
            let name = display_or_humanized(Some(&style.name), &style.id);
            let known = learned.iter().any(|learned_id| learned_id == &id);
            let known_word = if known { "yes" } else { "no" };
            Some(CrewStyleLine {
                id: style.id.clone(),
                name: name.clone(),
                cost: style.silver_cost,
                days: style.training_days,
                known,
                text: format!(
                    "{name}  {} silver  {} days  needs {} styles  known {known_word}",
                    style.silver_cost, style.training_days, style.prerequisite_styles
                ),
            })
        })
        .collect();
    let known_styles = if learned.is_empty() {
        "Known styles: none".to_string()
    } else {
        let names: Vec<String> = learned
            .iter()
            .map(|id| {
                display_or_humanized(
                    catalog.fighting_style(id).map(|style| style.name.as_str()),
                    id,
                )
            })
            .collect();
        format!("Known styles: {}", names.join(", "))
    };
    let skills = catalog
        .skills
        .skills
        .iter()
        .map(|skill| {
            let skill_name = display_or_humanized(Some(&skill.name), &skill.id);
            let display = portlight_sim::skills::skill_display(&world.captain.skills, &skill.id);
            let display = ascii_owned(&display, "Untrained");
            let current = portlight_sim::skills::skill_level(&world.captain.skills, &skill.id);
            let trainers = catalog.trainers_at(port_id, Some(&skill.id));
            if trainers.is_empty() {
                let text = format!("{skill_name}  {display}.");
                return CrewSkillLine {
                    id: skill.id.clone(),
                    skill_name,
                    level_name: String::new(),
                    cost: 0,
                    days: 0,
                    can_train: false,
                    text,
                    next_text: String::new(),
                    empty_trainer: Some(format!("No {} trainer at this port.", skill.id)),
                };
            }
            let max_teach = trainers
                .iter()
                .map(|trainer| trainer.max_teach_level)
                .max()
                .unwrap_or(0);
            let trainer_names: Vec<String> = trainers
                .iter()
                .map(|trainer| display_or_humanized(Some(&trainer.name), &trainer.id))
                .collect();
            let next = skill.levels.get(current as usize);
            let can_train = next.is_some() && current < skill.max_level && current < max_teach;
            let (level_name, cost, days, next_text) = if let Some(next) = next {
                let level_name = ascii_owned(&next.name, "level");
                (
                    level_name.clone(),
                    next.silver_cost,
                    next.training_days,
                    format!(
                        "Next: {level_name}  {} silver  {} days.",
                        next.silver_cost, next.training_days
                    ),
                )
            } else {
                (String::new(), 0, 0, "Already at maximum.".to_string())
            };
            let text = format!(
                "{skill_name}  {display}. {} (teaches to {max_teach}).",
                trainer_names.join(", ")
            );
            CrewSkillLine {
                id: skill.id.clone(),
                skill_name,
                level_name,
                cost,
                days,
                can_train,
                text,
                next_text,
                empty_trainer: None,
            }
        })
        .collect();
    let party = &world.captain.party;
    let party_line = format!("Party {}/{}.", party.companions.len(), party.max_size);
    let party_rows = party
        .companions
        .iter()
        .map(|member| {
            let name = display_or_humanized(
                catalog
                    .companion(&member.companion_id)
                    .map(|comp| comp.name.as_str()),
                &member.companion_id,
            );
            let role = display_or_humanized(
                catalog
                    .companion_role(&member.role_id)
                    .map(|role| role.name.as_str()),
                &member.role_id,
            );
            format!("{name}  {role}  morale {}", member.morale)
        })
        .collect();
    let offers = catalog
        .companions
        .companions
        .iter()
        .filter(|comp| comp.home_port_id == port_id)
        .filter(|comp| {
            !party
                .companions
                .iter()
                .any(|member| member.companion_id == comp.id)
        })
        .filter(|comp| !party.departed.iter().any(|id| id == &comp.id))
        .map(|comp| {
            let name = display_or_humanized(Some(&comp.name), &comp.id);
            let role = display_or_humanized(
                catalog
                    .companion_role(&comp.role_id)
                    .map(|role| role.name.as_str()),
                &comp.role_id,
            );
            let region = ascii_owned(&comp.region, "region");
            CrewOfferLine {
                id: comp.id.clone(),
                name: name.clone(),
                cost: comp.hire_cost,
                text: format!(
                    "{name}  {role}  {} silver  standing {} in {region}",
                    comp.hire_cost, comp.required_standing
                ),
            }
        })
        .collect();
    Some(CrewDesk {
        port_name,
        status,
        provision_per_day,
        provision_text,
        roles,
        officers,
        known_styles,
        styles,
        skills,
        party_line,
        party: party_rows,
        offers,
    })
}

/// Chart duel prompt for `World.pending_duel`. Faction by catalog name,
/// ASCII ` - ` separators.
/// Empty facts are dropped with their separator, so an empty region never
/// leaves `strength 3 - .`.
pub(crate) fn duel_prompt(duel: &portlight_sim::model::PendingDuel) -> String {
    let facts = [
        faction_name(&duel.faction_id),
        humanize_id(&duel.personality),
        format!("strength {}", duel.strength),
        duel.region.trim().to_string(),
    ]
    .into_iter()
    .filter(|fact| !fact.is_empty())
    .collect::<Vec<_>>()
    .join(" - ");
    format!(
        "Duel: {}.\nFaction {facts}.\nNext day ticks reputation and does not move the day. Pick stances, or auto-resolve.",
        duel.captain_name,
    )
}

/// R11. One log line for a finished duel, zero terms dropped (R4).
/// `result` is `Won`, `Drew` or `Lost`.
pub(crate) fn duel_result_line(result: &str, opponent: &str, silver: i64, standing: i64) -> String {
    let mut line = if opponent.is_empty() {
        format!("{result} the duel.")
    } else {
        format!("{result} the duel with {opponent}.")
    };
    if silver != 0 {
        line.push_str(&format!(" Silver {silver:+}."));
    }
    if standing != 0 {
        line.push_str(&format!(" Standing {standing:+}, shown only."));
    }
    line
}

/// R12. Drops sim rich-text tags (`[bold]`, `[/dim]`, `[bold yellow]`, `[/]`)
/// and keeps the text inside. Only the style words in `MARKUP_WORDS` make a
/// tag, so any other bracket (`[3/5]`, `[x] done`, `[port]`) stays. A Rich
/// escape `\[` is a literal bracket: the backslash goes, the `[` stays.
pub(crate) fn strip_markup(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        let after = &rest[open + 1..];
        if rest[..open].ends_with('\\') {
            out.push_str(&rest[..open - 1]);
            out.push('[');
            rest = after;
            continue;
        }
        out.push_str(&rest[..open]);
        match after.find(']') {
            Some(close) if is_markup_tag(&after[..close]) => {
                rest = &after[close + 1..];
            }
            _ => {
                out.push('[');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// The Rich style words the sim emits (`bold`, `dim`, `bold yellow` in
/// `port_arrival_engine.rs`), plus the plain styles and colours of the same
/// family. A tag is one or more of these, space-separated, or the bare `[/]`.
const MARKUP_WORDS: [&str; 16] = [
    "bold",
    "dim",
    "italic",
    "underline",
    "strike",
    "reverse",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "white",
    "black",
    "grey",
    "gray",
];

fn is_markup_tag(inner: &str) -> bool {
    let body = inner.strip_prefix('/').unwrap_or(inner);
    if body.is_empty() {
        return inner == "/";
    }
    body.split(' ').all(|word| MARKUP_WORDS.contains(&word))
}

#[cfg(test)]
mod tests {
    use portlight_chart::{chart_to_screen_f, chart_to_uv, facing_from_uv, Facing};
    use portlight_sim::session::VictoryReceipt;

    use super::*;

    /// Copy batch: an empty region leaves no `- .` tail in the duel prompt.
    #[test]
    fn duel_prompt_drops_an_empty_region() {
        let duel = portlight_sim::model::PendingDuel {
            captain_id: "c".into(),
            captain_name: "Bram".into(),
            faction_id: "iron_wolves".into(),
            personality: "aggressive".into(),
            strength: 3,
            region: " ".into(),
        };
        let text = duel_prompt(&duel);
        assert!(
            text.contains("Faction The Iron Wolves - Aggressive - strength 3.\n"),
            "{text}"
        );
        assert!(!text.contains("- ."), "{text}");
    }

    /// Copy batch raw-id fallbacks: live name, else catalog, else humanized.
    #[test]
    fn display_names_never_fall_back_to_raw_ids() {
        assert_eq!(captain_display_name("Old Coral", "old_coral"), "Old Coral");
        assert_eq!(captain_display_name("", "raj_the_quiet"), "Raj the Quiet");
        assert_eq!(
            captain_display_name("", "salt_widow_kell"),
            "Salt Widow Kell"
        );
        assert_eq!(
            captain_display_name("Ra\u{e9}l", "salt_widow_kell"),
            "Salt Widow Kell"
        );
        assert_eq!(ship_display_name("Gull", "swift_cutter"), "Gull");
        let catalog = portlight_sim::content::content()
            .ship("swift_cutter")
            .map(|t| t.name.clone())
            .unwrap();
        assert_eq!(ship_display_name("", "swift_cutter"), catalog);
        assert_eq!(ship_display_name("", "river_barge_x"), "River Barge X");
        for name in [
            captain_display_name("", "salt_widow_kell"),
            ship_display_name("", "river_barge_x"),
        ] {
            assert!(!name.contains('_') && name.is_ascii(), "{name}");
        }
    }

    #[test]
    fn humanize_id_title_cases_snake_case_ids() {
        assert_eq!(humanize_id("monsoon_syndicate"), "Monsoon Syndicate");
        assert_eq!(humanize_id("merchant_line"), "Merchant Line");
        assert_eq!(humanize_id("iron_wolves"), "Iron Wolves");
        assert_eq!(humanize_id("deep_reef"), "Deep Reef");
        assert_eq!(humanize_id("crimson_tide"), "Crimson Tide");
        assert_eq!(humanize_id("gnaw"), "Gnaw");
        assert_eq!(humanize_id("Porto Novo"), "Porto Novo");
        assert_eq!(humanize_id("_odd__id_"), "Odd Id");
        assert_eq!(humanize_id(""), "");
    }

    #[test]
    fn faction_name_keeps_the_catalog_name_and_humanizes_the_rest() {
        assert_eq!(
            faction_name("iron_wolves"),
            portlight_sim::content::content()
                .faction("iron_wolves")
                .unwrap()
                .name
        );
        assert_eq!(faction_name("no_such_faction"), "No Such Faction");
        assert_eq!(faction_name(""), "");
    }

    #[test]
    fn porto_novo_uses_the_rotated_projection() {
        let (x, y) = chart_to_screen_f(18.0, 8.0);
        assert!((x - 1629.2).abs() < 0.2, "{x}");
        assert!((y - 362.0).abs() < 0.2, "{y}");
    }

    #[test]
    fn grain_road_selects_facing_f7() {
        let (u0, v0) = chart_to_uv(18.0, 8.0);
        let (u1, v1) = chart_to_uv(24.0, 6.0);
        assert_eq!(facing_from_uv(u1 - u0, v1 - v0, None), Facing::F7);
    }

    #[test]
    fn facing_holds_through_the_five_degree_edge() {
        let hold = 26.0_f64.to_radians();
        let switch = 28.0_f64.to_radians();
        assert_eq!(
            facing_from_uv(hold.cos(), hold.sin(), Some(Facing::F0)),
            Facing::F0
        );
        assert_eq!(
            facing_from_uv(switch.cos(), switch.sin(), Some(Facing::F0)),
            Facing::F1
        );
    }

    #[test]
    fn duel_button_ignores_stance_count() {
        assert!(!duel_button_enabled(false));
        assert!(duel_button_enabled(true));
    }

    #[test]
    fn day_log_keeps_turn_notes_with_the_existing_lines() {
        let lines = day_log_lines(
            &["Wind on the bow.".to_string()],
            &["Grain prices jumped.".to_string()],
            &[
                "Warehouse at porto_novo closed for non-payment. Goods seized: 2x grain"
                    .to_string(),
            ],
            Some("Docked at Porto Novo."),
        );
        assert_eq!(
            lines,
            vec![
                "Wind on the bow.".to_string(),
                "Grain prices jumped.".to_string(),
                "Warehouse at porto_novo closed for non-payment. Goods seized: 2x grain"
                    .to_string(),
                "Docked at Porto Novo.".to_string(),
            ]
        );
    }

    #[test]
    fn day_log_drops_empty_notes_and_keeps_a_quiet_day() {
        let lines = day_log_lines(
            &["Wind on the bow.".to_string(), String::new()],
            &[String::new()],
            &[String::new()],
            Some("Docked at Porto Novo."),
        );
        assert_eq!(
            lines,
            vec![
                "Wind on the bow.".to_string(),
                "Docked at Porto Novo.".to_string(),
            ]
        );
        assert!(day_log_lines(&[], &[], &[], None).is_empty());
    }

    #[test]
    fn the_panel_and_chart_fit_the_window() {
        let project = include_str!("../../../godot/project.godot");
        assert!(project.contains("window/size/viewport_width=1280"));
        assert!(project.contains("window/size/viewport_height=720"));
        assert!(layout_fits_window());
        assert_eq!(portlight_chart::CHART_VIEW_W, chart_host_width());
        assert_eq!(portlight_chart::CHART_VIEW_H, WINDOW_H);
    }

    #[test]
    fn a_flat_capture_fails_and_a_chart_frame_does_not() {
        let clear = [13, 25, 41];
        assert!(frame_mostly_flat(&[clear; 100]));
        // The committed chart-1280.png was clear colour under a thin strip.
        let mut strip = vec![clear; 96];
        strip.extend([[70, 120, 150]; 4]);
        assert!(frame_mostly_flat(&strip));
        let mut chart = vec![clear; 40];
        chart.extend([[32, 78, 112]; 30]);
        chart.extend([[232, 196, 120]; 30]);
        assert!(!frame_mostly_flat(&chart));
    }

    #[test]
    fn an_illegal_layout_and_a_flat_seam_frame_exit_nonzero() {
        // The old capture wrote the clear colour and exited 0 when the
        // layout panic was swallowed. Both of those outcomes are failures.
        assert_eq!(seam_exit_code(false, true), 1);
        assert_eq!(seam_exit_code(true, false), 1);
        assert_eq!(seam_exit_code(false, false), 1);
        assert_eq!(seam_exit_code(true, true), 0);

        let clear = [13, 25, 41];
        assert!(capture_frame_rejected(1280, 720, 1280, 720, &[clear; 100]));
        assert!(capture_frame_rejected(0, 0, 1280, 720, &[]));
        let mut chart = vec![clear; 40];
        chart.extend([[32, 78, 112]; 30]);
        chart.extend([[232, 196, 120]; 30]);
        assert!(!capture_frame_rejected(1280, 720, 1280, 720, &chart));
        // A crop uses its own size. A flat crop is still rejected.
        assert!(capture_frame_rejected(192, 128, 192, 128, &[clear; 40]));
        assert!(!capture_frame_rejected(192, 128, 192, 128, &chart));
    }

    #[test]
    fn an_ink_encounter_frame_passes_only_with_the_plate_and_a_button() {
        let ink = [20, 28, 41];
        let plate = [184, 148, 92];
        let button = [140, 107, 61];
        // The signed-off layout is mostly ink. The chart's 80% rule rejects it.
        let mut signed = vec![ink; 930];
        signed.extend([plate; 40]);
        signed.extend([button; 10]);
        signed.extend([[240, 232, 214]; 20]);
        assert!(frame_mostly_flat(&signed));
        assert!(capture_frame_rejected(1280, 720, 1280, 720, &signed));
        assert!(!encounter_frame_rejected(1280, 720, 1280, 720, &signed));
        assert!(encounter_frame_rejected(1280, 720, 1280, 720, &[ink; 100]));
        assert!(encounter_frame_rejected(0, 0, 1280, 720, &[]));
        // A few plate pixels and no button is still a failed capture.
        let mut plate_only = vec![ink; 100];
        plate_only.extend([plate; 5]);
        assert!(encounter_frame_rejected(1280, 720, 1280, 720, &plate_only));
    }

    use portlight_sim::session::Session;
    use portlight_sim::SimError;

    use super::{
        action_list_from_error, at_sea, encounter_plate, facts_for_catalog_captain,
        facts_from_agency, facts_from_step, player_ship, present, signed_delta,
        stance_duel_visible, template_player_ship, ui_plate_panel, DeltaSpan, DeltaTone,
        EncounterFacts, ScreenAction, ScreenPhase, StepInput, PORTRAIT_PLACEHOLDER,
        SCRIPTED_CAPTAIN, SCRIPTED_CAPTAIN_TYPE, SCRIPTED_DEPART, SCRIPTED_FIGHT, SCRIPTED_NAME,
        SCRIPTED_NAVAL, SCRIPTED_SEED,
    };

    fn scripted_session() -> Session {
        let mut session =
            Session::new(SCRIPTED_NAME, SCRIPTED_CAPTAIN_TYPE, SCRIPTED_SEED, None).unwrap();
        session.depart(SCRIPTED_DEPART).unwrap();
        assert!(at_sea(&session));
        session
    }

    fn probe_naval(session: &mut Session) -> Vec<String> {
        match session.naval_round("") {
            Err(err) => action_list_from_error(&err).unwrap(),
            Ok(_) => panic!("empty naval action resolved a round"),
        }
    }

    fn probe_fight(session: &mut Session) -> Vec<String> {
        match session.fight("") {
            Err(err) => action_list_from_error(&err).unwrap(),
            Ok(_) => panic!("empty fight action resolved a round"),
        }
    }

    fn adopt(
        session: &Session,
        step: &portlight_sim::session::EncounterStep,
        previous: Option<&EncounterFacts>,
        naval_actions: &[String],
        combat_actions: &[String],
    ) -> EncounterFacts {
        facts_from_step(StepInput {
            step,
            previous_faction_id: previous
                .map(|facts| facts.faction_id.as_str())
                .unwrap_or(""),
            previous_enemy_hull_max: previous.and_then(|facts| facts.enemy_hull_max),
            pending_victory: session.pending_victory(),
            ship: player_ship(session),
            at_sea: at_sea(session),
            naval_actions,
            combat_actions,
        })
    }

    #[test]
    fn a_sloop_plate_is_168_and_a_cutter_uses_its_own_canvas() {
        assert_eq!(ui_plate_panel(64, 64), (168, 168));
        let sloop = encounter_plate("coastal_sloop");
        assert_eq!(sloop.class_name, "sloop");
        assert_eq!(sloop.hull.id, "ship_sloop_f3");
        assert_eq!(
            ui_plate_panel(sloop.hull.canvas_w, sloop.hull.canvas_h),
            (168, 168)
        );
        let cutter = encounter_plate("swift_cutter");
        assert_eq!(cutter.class_name, "cutter");
        assert_eq!(cutter.hull.id, "ship_cutter_f3");
        assert_eq!(
            ui_plate_panel(cutter.hull.canvas_w, cutter.hull.canvas_h),
            (184, 200)
        );
    }

    #[test]
    fn man_of_war_maps_to_the_galleon_plate_and_keeps_its_class_name() {
        let drawn = encounter_plate("royal_man_of_war");
        assert_eq!(drawn.hull.id, "ship_galleon_f3");
        assert_eq!(drawn.class_name, "man_of_war");
        assert_eq!((drawn.hull.canvas_w, drawn.hull.canvas_h), (112, 112));
        assert_eq!(
            ui_plate_panel(drawn.hull.canvas_w, drawn.hull.canvas_h),
            (264, 264)
        );
        let again = encounter_plate("royal_man_of_war");
        assert_eq!(again.class_name, "man_of_war");
        assert_eq!(again.hull.id, drawn.hull.id);
    }

    #[test]
    fn a_forced_man_of_war_card_uses_the_template_hull_and_crew() {
        let template = portlight_sim::content::content()
            .ship("royal_man_of_war")
            .expect("catalog");
        let ship = template_player_ship("royal_man_of_war").expect("template");
        assert_eq!(ship.hull, template.hull_max);
        assert_eq!(ship.hull_max, template.hull_max);
        assert_eq!(ship.crew, template.crew_min);
        assert!(template_player_ship("not_a_ship").is_none());
    }

    #[test]
    fn an_unknown_template_uses_the_sloop_plate() {
        // The once-only warning counter stays inside portlight-chart tests.
        let drawn = encounter_plate("not_a_ship");
        assert_eq!(drawn.hull.id, "ship_sloop_f3");
        assert_eq!(drawn.class_name, "not_a_ship");
        assert_eq!(encounter_plate("not_a_ship").hull.id, drawn.hull.id);
        let empty = encounter_plate("");
        assert_eq!(empty.hull.id, "ship_sloop_f3");
        assert_eq!(empty.class_name, "sloop");
    }

    #[test]
    fn an_open_encounter_hides_the_stance_duel_until_it_closes() {
        assert!(stance_duel_visible(false, true));
        assert!(!stance_duel_visible(true, true));
        assert!(!stance_duel_visible(false, false));
        assert!(!stance_duel_visible(true, false));
    }

    #[test]
    fn approach_card_uses_the_catalog_and_marks_the_portrait_placeholder() {
        assert_eq!(PORTRAIT_PLACEHOLDER, "Placeholder");
        let session = scripted_session();
        let facts =
            facts_for_catalog_captain(SCRIPTED_CAPTAIN, player_ship(&session), at_sea(&session))
                .unwrap();
        let view = present(&facts).unwrap();
        assert_eq!(view.phase, ScreenPhase::Approach);
        assert!(view.portrait_placeholder);
        assert!(view.card.contains("Raj the Quiet"));
        assert!(view.card.contains("The Monsoon Syndicate"));
        assert!(view.card.contains("Strength 5"));
        assert!(view.card.contains("Your hull"));
        assert!(!view.card.contains("Enemy hull"));
        assert!(view.delta.is_empty());
        assert_eq!(
            view.actions,
            vec![
                ScreenAction::Choice("negotiate"),
                ScreenAction::Choice("flee"),
                ScreenAction::Choice("fight"),
            ]
        );
    }

    fn delta_text(spans: &[DeltaSpan]) -> String {
        spans.iter().map(|span| span.text.as_str()).collect()
    }

    fn cleared_deltas(
        step: &portlight_sim::session::EncounterStep,
    ) -> portlight_sim::session::EncounterStep {
        let mut step = step.clone();
        step.player_hull_delta = 0;
        step.enemy_hull_delta = 0;
        step.player_crew_delta = 0;
        step.enemy_crew_delta = 0;
        step.damage_to_player = 0;
        step.damage_to_opponent = 0;
        step.hull_damage = 0;
        step
    }

    fn line_for(
        session: &Session,
        step: &portlight_sim::session::EncounterStep,
    ) -> (String, Vec<DeltaSpan>) {
        let facts = adopt(session, step, None, &[], &[]);
        let view = present(&facts).unwrap();
        let text = delta_text(&view.delta);
        (text, view.delta)
    }

    fn assert_line_matches_step(step: &portlight_sim::session::EncounterStep, text: &str) {
        assert!(text.is_ascii(), "{text}");
        assert!(
            text.chars().all(|c| {
                c.is_ascii_alphanumeric() || matches!(c, ' ' | '+' | '-' | '|' | ':' | ',')
            }),
            "{text}"
        );
        assert!(!text.contains('/'), "{text}");
        if step.player_hull_delta != 0 {
            assert!(text.contains(&format!(
                "Your hull {}",
                signed_delta(step.player_hull_delta)
            )));
        } else {
            assert!(!text.contains("Your hull"), "{text}");
        }
        if step.enemy_hull_delta != 0 {
            assert!(text.contains(&format!(
                "Enemy hull {}",
                signed_delta(step.enemy_hull_delta)
            )));
        } else {
            assert!(!text.contains("Enemy hull"), "{text}");
        }
        if step.player_crew_delta != 0 || step.enemy_crew_delta != 0 {
            assert!(text.contains("Crew:"), "{text}");
        } else {
            assert!(!text.contains("Crew:"), "{text}");
        }
        if step.damage_to_player != 0 || step.damage_to_opponent != 0 {
            assert!(text.contains("HP:"), "{text}");
        } else {
            assert!(!text.contains("HP:"), "{text}");
        }
        if step.damage_to_player != 0 {
            assert!(text.contains(&format!("you {}", signed_delta(-step.damage_to_player))));
        }
        if step.damage_to_opponent != 0 {
            assert!(text.contains(&format!(
                "opponent {}",
                signed_delta(-step.damage_to_opponent)
            )));
        }
    }

    #[test]
    fn board_step_shows_its_crew_loss_on_the_delta_line() {
        let mut session = scripted_session();
        let base = session
            .encounter_choice_with("fight", Some(SCRIPTED_CAPTAIN), None)
            .unwrap();
        let mut step = cleared_deltas(&base);
        step.kind = "board".to_string();
        step.phase = "duel".to_string();
        step.player_crew_lost = 3;
        step.enemy_crew_lost = 1;
        let (text, spans) = line_for(&session, &step);
        assert_eq!(text, "Crew: you -3, enemy -1");
        let parts: Vec<(&str, DeltaTone)> = spans
            .iter()
            .map(|span| (span.text.as_str(), span.tone))
            .collect();
        assert_eq!(
            parts,
            vec![
                ("Crew: ", DeltaTone::Neutral),
                ("you -3", DeltaTone::PlayerLoss),
                (", ", DeltaTone::Neutral),
                ("enemy -1", DeltaTone::EnemyLoss),
            ]
        );
        // Only the board step maps `*_crew_lost`; other kinds keep the deltas.
        step.kind = "naval".to_string();
        step.phase = "naval".to_string();
        assert_eq!(line_for(&session, &step).0, "");
    }

    #[test]
    fn encounter_end_line_heads_terms_and_clause() {
        assert_eq!(
            encounter_end_line(
                EncounterEnd::LostFight,
                "Raj the Quiet",
                [-30, -3, 0, 0],
                true
            ),
            "Lost the fight with Raj the Quiet - Silver -30 - Crew -3 - the bounty stays open"
        );
        assert_eq!(
            encounter_end_line(
                EncounterEnd::DrewFight,
                "Raj the Quiet",
                [0, -3, 0, 0],
                false
            ),
            "Drew the fight with Raj the Quiet - Crew -3"
        );
        assert_eq!(
            encounter_end_line(
                EncounterEnd::LostAtSea,
                "The Butcher",
                [-55, -4, -20, -12],
                false
            ),
            "Lost the sea fight with The Butcher - Silver -55 - Crew -4 - Hull -20 - Cargo -12"
        );
        assert_eq!(
            encounter_end_line(EncounterEnd::BrokeAway, "Typhoon Mei", [0, 0, -4, 0], false),
            "Broke away from Typhoon Mei - Hull -4"
        );
        // All zero, no clause: just the head.
        assert_eq!(
            encounter_end_line(EncounterEnd::BrokeAway, "Typhoon Mei", [0; 4], false),
            "Broke away from Typhoon Mei"
        );
        // The clause only when the bounty is open.
        assert!(
            encounter_end_line(EncounterEnd::BrokeAway, "Typhoon Mei", [0; 4], true)
                .ends_with(" - the bounty stays open")
        );
        // Empty name drops ` with {Name}` / ` from {Name}`.
        assert_eq!(
            encounter_end_line(EncounterEnd::LostFight, "", [-30, 0, 0, 0], false),
            "Lost the fight - Silver -30"
        );
        assert_eq!(
            encounter_end_line(EncounterEnd::BrokeAway, "", [0, 0, -4, 0], false),
            "Broke away - Hull -4"
        );
        // A gain keeps its sign; zero never prints.
        assert_eq!(
            encounter_end_line(EncounterEnd::LostAtSea, "X", [0, 0, 2, 0], false),
            "Lost the sea fight with X - Hull +2"
        );
        for end in [
            EncounterEnd::LostFight,
            EncounterEnd::DrewFight,
            EncounterEnd::LostAtSea,
            EncounterEnd::BrokeAway,
        ] {
            for deltas in [[0; 4], [-30, -3, -4, -12], [5, 0, 0, 0]] {
                let line = encounter_end_line(end, "Raj the Quiet", deltas, true);
                assert!(line.is_ascii(), "{line}");
                assert!(
                    !line.contains('\u{2014}') && !line.contains('\u{2013}'),
                    "{line}"
                );
                assert!(!line.contains("->"), "{line}");
                assert!(!line.contains("+0") && !line.contains("-0"), "{line}");
                assert!(
                    !line.contains("Trust") && !line.contains("Standing"),
                    "{line}"
                );
                assert!(!line.contains('%'), "{line}");
            }
        }
    }

    #[test]
    fn encounter_end_name_prefers_catalog_then_humanizes_never_raw_id() {
        // Catalog ASCII display name wins.
        assert_eq!(encounter_end_name("raj_the_quiet"), "Raj the Quiet");
        // Missing from the catalog: humanized id.
        assert_eq!(encounter_end_name("salt_widow_kell"), "Salt Widow Kell");
        assert_eq!(
            display_or_humanized(None, "salt_widow_kell"),
            "Salt Widow Kell"
        );
        assert_eq!(
            display_or_humanized(Some(""), "salt_widow_kell"),
            "Salt Widow Kell"
        );
        // Non-ASCII display name: humanized id.
        assert_eq!(
            display_or_humanized(Some("Ra\u{e9}l the Quiet"), "rael_the_quiet"),
            "Rael The Quiet"
        );
        assert_eq!(
            display_or_humanized(Some("Raj the Quiet"), "raj_the_quiet"),
            "Raj the Quiet"
        );
        // Never the raw snake_case id, and always ASCII.
        for id in [
            "raj_the_quiet",
            "salt_widow_kell",
            "the_butcher",
            "typhoon_mei",
        ] {
            let name = encounter_end_name(id);
            assert_ne!(name, id);
            assert!(!name.contains('_'), "{name}");
            assert!(name.is_ascii(), "{name}");
        }
        for (display, id) in [(Some("Ra\u{e9}l"), "rael_x"), (None, "no_such_captain")] {
            let name = display_or_humanized(display, id);
            assert_ne!(name, id);
            assert!(!name.contains('_') && name.is_ascii(), "{name}");
        }
        // Empty id and no name: empty, so the head drops ` with {Name}`.
        assert_eq!(encounter_end_name(""), "");
        assert_eq!(
            encounter_end_line(
                EncounterEnd::LostFight,
                &encounter_end_name(""),
                [-30, 0, 0, 0],
                false
            ),
            "Lost the fight - Silver -30"
        );
    }

    #[test]
    fn encounter_end_classifies_resolved_non_wins() {
        let mut session = scripted_session();
        let base = session
            .encounter_choice_with("fight", Some(SCRIPTED_CAPTAIN), None)
            .unwrap();
        let synth = |kind: &str, edit: &dyn Fn(&mut portlight_sim::session::EncounterStep)| {
            let mut step = cleared_deltas(&base);
            step.kind = kind.to_string();
            step.phase = "resolved".to_string();
            step.escaped = false;
            step.enemy_sunk = false;
            step.player_sunk = false;
            step.player_won = false;
            step.draw = false;
            edit(&mut step);
            step
        };
        let fight_loss = synth("fight", &|_| {});
        let draw = synth("fight", &|step| step.draw = true);
        let naval_sunk = synth("naval", &|step| step.player_sunk = true);
        let naval_crew_gone = synth("naval", &|step| step.player_crew = 0);
        let approach_flee = synth("choice", &|step| {
            step.choice = "flee".to_string();
            step.escaped = true;
        });
        let naval_flee = synth("naval", &|step| {
            step.choice = "flee".to_string();
            step.escaped = true;
        });
        let fight_win = synth("fight", &|step| step.player_won = true);
        let enemy_sunk = synth("naval", &|step| step.enemy_sunk = true);
        let peace = synth("choice", &|step| step.choice = "negotiate".to_string());
        let capture = synth("capture", &|_| {});
        let running = synth("fight", &|step| step.phase = "duel".to_string());
        assert_eq!(
            encounter_end(&fight_loss, false),
            Some(EncounterEnd::LostFight)
        );
        assert_eq!(encounter_end(&draw, false), Some(EncounterEnd::DrewFight));
        assert_eq!(
            encounter_end(&naval_sunk, false),
            Some(EncounterEnd::LostAtSea)
        );
        assert_eq!(
            encounter_end(&naval_crew_gone, false),
            Some(EncounterEnd::LostAtSea)
        );
        assert_eq!(
            encounter_end(&approach_flee, false),
            Some(EncounterEnd::BrokeAway)
        );
        assert_eq!(
            encounter_end(&naval_flee, false),
            Some(EncounterEnd::BrokeAway)
        );
        assert_eq!(encounter_end(&fight_win, false), None);
        assert_eq!(encounter_end(&enemy_sunk, false), None);
        assert_eq!(encounter_end(&peace, false), None);
        assert_eq!(encounter_end(&capture, false), None);
        assert_eq!(encounter_end(&running, false), None);
        // A pending win is never a non-win end.
        assert_eq!(encounter_end(&fight_loss, true), None);
    }

    #[test]
    fn encounter_baseline_diffs_the_world() {
        let session = scripted_session();
        let before = EncounterBaseline::read(session.world());
        let ship = session.world().captain.ship.as_ref().unwrap();
        assert_eq!(before.silver, session.world().captain.silver);
        assert_eq!(before.crew, ship.crew);
        assert_eq!(before.hull, ship.hull);
        let after = EncounterBaseline {
            silver: before.silver - 30,
            crew: before.crew - 3,
            hull: before.hull,
            cargo_units: before.cargo_units - 12,
        };
        assert_eq!(before.deltas(&after), [-30, -3, 0, -12]);
        assert_eq!(before.deltas(&before), [0; 4]);
    }

    #[test]
    fn approach_flee_receipt_matches_the_world() {
        // Real Session: an approach flee that escapes ends resolved and
        // classifies as Broke away; the hull term is the real hull change.
        for seed in 0..40 {
            let mut session = Session::new("Low", "merchant", 1, None).unwrap();
            let baseline = EncounterBaseline::read(session.world());
            let step = session
                .encounter_choice_with("flee", None, Some(seed))
                .unwrap();
            if step.phase != "resolved" || !step.escaped {
                continue;
            }
            assert_eq!(
                encounter_end(&step, session.pending_victory()),
                Some(EncounterEnd::BrokeAway)
            );
            let deltas = baseline.deltas(&EncounterBaseline::read(session.world()));
            assert_eq!(deltas[2], -step.hull_damage);
            let line = encounter_end_line(
                EncounterEnd::BrokeAway,
                &encounter_end_name(&step.enemy_captain_id),
                deltas,
                false,
            );
            assert!(line.starts_with("Broke away from "), "{line}");
            return;
        }
        panic!("no escaping approach flee in 40 seeds");
    }

    #[test]
    fn one_ascii_signed_delta_line_drops_zeros() {
        let mut session = scripted_session();
        let base = session
            .encounter_choice_with("fight", Some(SCRIPTED_CAPTAIN), None)
            .unwrap();
        assert_eq!(line_for(&session, &base).0, "");

        let mut step = cleared_deltas(&base);
        step.player_hull_delta = -4;
        step.enemy_hull_delta = -12;
        let (hull, spans) = line_for(&session, &step);
        assert_eq!(hull, "Your hull -4 | Enemy hull -12");
        assert_eq!(spans[0].tone, DeltaTone::PlayerLoss);
        assert_eq!(spans[1].tone, DeltaTone::Neutral);
        assert_eq!(spans[2].tone, DeltaTone::EnemyLoss);

        step = cleared_deltas(&base);
        step.player_crew_delta = -1;
        step.enemy_crew_delta = -2;
        let (crew, spans) = line_for(&session, &step);
        assert_eq!(crew, "Crew: you -1, enemy -2");
        assert_eq!(spans[0].tone, DeltaTone::Neutral);
        assert_eq!(spans[1].tone, DeltaTone::PlayerLoss);
        assert_eq!(spans[3].tone, DeltaTone::EnemyLoss);

        step = cleared_deltas(&base);
        step.damage_to_player = 3;
        step.damage_to_opponent = 5;
        let (hp, spans) = line_for(&session, &step);
        assert_eq!(hp, "HP: you -3, opponent -5");
        assert_eq!(spans[1].text, "you -3");
        assert_eq!(spans[1].tone, DeltaTone::PlayerLoss);
        assert_eq!(spans[3].text, "opponent -5");
        assert_eq!(spans[3].tone, DeltaTone::EnemyLoss);

        step = cleared_deltas(&base);
        step.player_hull_delta = -4;
        step.enemy_hull_delta = -12;
        step.player_crew_delta = -1;
        step.enemy_crew_delta = -2;
        step.damage_to_player = 3;
        step.damage_to_opponent = 5;
        assert_eq!(
            line_for(&session, &step).0,
            "Your hull -4 | Enemy hull -12 | Crew: you -1, enemy -2 | HP: you -3, opponent -5"
        );

        step = cleared_deltas(&base);
        step.player_hull_delta = 2;
        step.enemy_crew_delta = -2;
        let (gain, spans) = line_for(&session, &step);
        assert_eq!(gain, "Your hull +2 | Crew: enemy -2");
        assert_eq!(spans[0].tone, DeltaTone::Neutral);
        assert_eq!(spans[3].tone, DeltaTone::EnemyLoss);

        step = cleared_deltas(&base);
        step.enemy_hull_delta = -12;
        assert_eq!(line_for(&session, &step).0, "Enemy hull -12");

        // Flee reports hull_damage in the log. It is not a delta field.
        step = cleared_deltas(&base);
        step.hull_damage = 4;
        step.message = "A parting shot catches your hull for 4 damage.".to_string();
        let facts = adopt(&session, &step, None, &[], &[]);
        let view = present(&facts).unwrap();
        assert!(view.delta.is_empty());
        assert!(view.log.contains('4'));
    }

    #[test]
    fn scripted_and_bounty_resolves_share_the_delta_line() {
        let mut session = scripted_session();
        let approach =
            facts_for_catalog_captain(SCRIPTED_CAPTAIN, player_ship(&session), true).unwrap();
        assert!(present(&approach).unwrap().delta.is_empty());

        let step = session
            .encounter_choice_with("fight", Some(SCRIPTED_CAPTAIN), None)
            .unwrap();
        let naval_actions = probe_naval(&mut session);
        let mut facts = adopt(&session, &step, Some(&approach), &naval_actions, &[]);
        assert!(
            present(&facts).unwrap().delta.is_empty(),
            "opening the naval phase is not a hull resolve"
        );

        let mut saw_naval_delta = false;
        for action in SCRIPTED_NAVAL {
            let step = session.naval_round(action).unwrap();
            let actions = if step.phase == "naval" {
                probe_naval(&mut session)
            } else {
                Vec::new()
            };
            facts = adopt(&session, &step, Some(&facts), &actions, &[]);
            let text = delta_text(&present(&facts).unwrap().delta);
            assert_line_matches_step(&step, &text);
            if !text.is_empty() {
                saw_naval_delta = true;
            }
            if step.phase == "boarding" {
                break;
            }
        }
        assert!(
            saw_naval_delta,
            "a scripted naval resolve should show a signed hull or crew delta"
        );

        let step = session.resolve_boarding().unwrap();
        let combat_actions = probe_fight(&mut session);
        facts = adopt(&session, &step, Some(&facts), &[], &combat_actions);
        let boarded = delta_text(&present(&facts).unwrap().delta);
        // Section 10.2: the board step's `*_crew_lost` reads as crew deltas.
        let mut mapped = step.clone();
        mapped.player_crew_delta = -step.player_crew_lost;
        mapped.enemy_crew_delta = -step.enemy_crew_lost;
        assert_line_matches_step(&mapped, &boarded);
        if step.player_crew_lost != 0 {
            assert!(
                boarded.contains(&format!("you -{}", step.player_crew_lost)),
                "{boarded}"
            );
        }
        if step.enemy_crew_lost != 0 {
            assert!(
                boarded.contains(&format!("enemy -{}", step.enemy_crew_lost)),
                "{boarded}"
            );
        }

        let mut last_fight = String::new();
        for action in SCRIPTED_FIGHT {
            let step = session.fight(action).unwrap();
            let actions = if step.phase == "duel" {
                probe_fight(&mut session)
            } else {
                Vec::new()
            };
            facts = adopt(&session, &step, Some(&facts), &[], &actions);
            last_fight = delta_text(&present(&facts).unwrap().delta);
            assert_line_matches_step(&step, &last_fight);
        }
        assert!(
            last_fight.contains("HP:"),
            "the last personal-fight resolve should show HP: {last_fight}"
        );

        let mut bounty =
            Session::new(SCRIPTED_NAME, SCRIPTED_CAPTAIN_TYPE, SCRIPTED_SEED, None).unwrap();
        bounty.accept_bounty(SCRIPTED_CAPTAIN).unwrap();
        let state = bounty.hunt_bounty(SCRIPTED_CAPTAIN).unwrap();
        let agency = facts_from_agency(&state, player_ship(&bounty), "", at_sea(&bounty));
        assert!(present(&agency).unwrap().delta.is_empty());
        assert!(agency.enemy_hull_max.is_some());
        let step = bounty.encounter_choice("fight").unwrap();
        let naval_actions = probe_naval(&mut bounty);
        let facts = adopt(&bounty, &step, Some(&agency), &naval_actions, &[]);
        assert!(present(&facts).unwrap().delta.is_empty());
        let step = bounty.naval_round("broadside").unwrap();
        let facts = adopt(&bounty, &step, Some(&facts), &[], &[]);
        let text = delta_text(&present(&facts).unwrap().delta);
        assert_line_matches_step(&step, &text);
        assert!(
            !text.is_empty(),
            "bounty broadside should show the same delta line"
        );
        assert_eq!(text, line_for(&bounty, &step).0);
    }

    #[test]
    fn rejected_probe_is_the_action_list_and_does_not_play_the_round() {
        let mut session = scripted_session();
        let step = session
            .encounter_choice_with("fight", Some(SCRIPTED_CAPTAIN), None)
            .unwrap();
        assert_eq!(step.phase, "naval");
        let hull = session.world().captain.ship.as_ref().unwrap().hull;
        let actions = probe_naval(&mut session);
        assert_eq!(session.world().captain.ship.as_ref().unwrap().hull, hull);
        assert!(actions.iter().any(|action| action == "broadside"));
        assert!(actions.iter().any(|action| action == "flee"));
        let facts = adopt(&session, &step, None, &actions, &[]);
        let view = present(&facts).unwrap();
        assert_eq!(view.phase, ScreenPhase::Naval);
        assert_eq!(view.log, step.message);
        assert!(view.card.contains("Enemy hull"));
        let shown: Vec<_> = view
            .actions
            .iter()
            .map(|action| match action {
                ScreenAction::Naval(name) => name.as_str(),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            shown,
            actions.iter().map(String::as_str).collect::<Vec<_>>()
        );
    }

    #[test]
    fn escape_and_loss_keep_sessions_sentence() {
        let mut session = Session::new("Low", "merchant", 1, None).unwrap();
        let step = session
            .encounter_choice_with("flee", None, Some(2))
            .unwrap();
        let facts = adopt(&session, &step, None, &[], &[]);
        let view = present(&facts).unwrap();
        assert_eq!(view.log, super::session_text(&step.message, &step.flavor));
        assert!(!view.log.is_empty());
        if step.phase == "resolved" {
            assert!(view
                .actions
                .iter()
                .any(|action| matches!(action, ScreenAction::Return { .. })));
            assert!(!view
                .actions
                .iter()
                .any(|action| matches!(action, ScreenAction::Spare | ScreenAction::Choice(_))));
        }
    }

    #[test]
    fn a_win_offers_spare_capture_and_take_all() {
        let mut facts = facts_for_catalog_captain("raj_the_quiet", None, true).unwrap();
        facts.phase = "resolved".to_string();
        facts.pending_victory = true;
        facts.log = "Blade to blade.".to_string();
        facts.enemy_hull = Some(40);
        facts.enemy_crew = Some(8);
        let view = present(&facts).unwrap();
        assert_eq!(view.phase, ScreenPhase::Outcome);
        assert_eq!(view.log, "Blade to blade.");
        assert!(!view.card.contains("Enemy hull"));
        assert_eq!(
            view.actions,
            vec![
                ScreenAction::Spare,
                ScreenAction::Capture,
                ScreenAction::TakeAll,
            ]
        );
    }

    #[test]
    fn card_text_is_ascii_in_every_phase() {
        let mut facts = facts_for_catalog_captain(
            "raj_the_quiet",
            Some(PlayerShip {
                hull: 18,
                hull_max: 20,
                crew: 4,
            }),
            true,
        )
        .unwrap();
        facts.kind = "fight".to_string();
        facts.enemy_hull = Some(40);
        facts.enemy_hull_max = Some(50);
        facts.enemy_crew = Some(8);
        facts.player_hp = 12;
        facts.opponent_hp = 9;
        for phase in [
            ScreenPhase::Approach,
            ScreenPhase::Naval,
            ScreenPhase::Boarding,
            ScreenPhase::Personal,
            ScreenPhase::Outcome,
        ] {
            let card = card_text(&facts, phase);
            assert!(card.is_ascii(), "{phase:?}: {card}");
            assert!(!card.contains('\u{b7}'), "{phase:?}: {card}");
        }
        assert!(card_text(&facts, ScreenPhase::Naval).contains("Your hull 18/20 - crew 4"));
        assert!(card_text(&facts, ScreenPhase::Naval).contains("Enemy hull 40/50 - crew 8"));
        assert!(card_text(&facts, ScreenPhase::Personal).contains("Your HP 12 - opponent 9"));
        facts.player_hull_max = None;
        facts.enemy_hull_max = None;
        let card = card_text(&facts, ScreenPhase::Naval);
        assert!(card.contains("Your hull 18 - crew 4"), "{card}");
        assert!(card.contains("Enemy hull 40 - crew 8"), "{card}");
    }

    #[test]
    fn purse_formulas_match_finalize_victory() {
        for strength in [0, 1, 4, 9, 12, 100] {
            assert_eq!(spare_purse(strength), 20 + strength * 3, "{strength}");
            assert_eq!(take_all_purse(strength), 20 + strength * 7, "{strength}");
            assert!(take_all_purse(strength) >= spare_purse(strength));
        }
        assert_eq!(SPARE_STANDING, 5);
        assert_eq!(TAKE_ALL_STANDING, 2);
        // A huge strength saturates rather than overflowing the UI.
        assert_eq!(spare_purse(i64::MAX), i64::MAX);
        assert_eq!(take_all_purse(i64::MAX), i64::MAX);
    }

    #[test]
    fn choice_preview_copy_is_the_binding_text() {
        // Brief example: Strength 4.
        assert_eq!(spare_preview(4), "+32 silver - Underworld +5.");
        assert_eq!(
            take_all_preview(4),
            "+48 silver - Underworld +2 - loot unknown."
        );
        // Strength 0 still pays the base 20.
        assert_eq!(spare_preview(0), "+20 silver - Underworld +5.");
        assert_eq!(
            take_all_preview(0),
            "+20 silver - Underworld +2 - loot unknown."
        );
        // Plain integers, no thousands separator (matches the receipt).
        assert_eq!(spare_preview(1000), "+3020 silver - Underworld +5.");
        assert_eq!(
            take_all_preview(1000),
            "+7020 silver - Underworld +2 - loot unknown."
        );
        let preview = choice_preview(4);
        assert_eq!(preview.spare, spare_preview(4));
        assert_eq!(preview.take_all, take_all_preview(4));
    }

    #[test]
    fn choice_preview_copy_is_ascii_with_no_promises() {
        for strength in [0, 1, 4, 12, 1000] {
            for line in [spare_preview(strength), take_all_preview(strength)] {
                assert!(line.is_ascii(), "{line}");
                assert!(
                    !line.contains('\u{2014}') && !line.contains('\u{2013}'),
                    "{line}"
                );
                assert!(!line.contains("--"), "{line}");
                assert!(line.contains(" - "), "{line}");
                assert!(line.ends_with('.'), "{line}");
                assert!(!line.contains("Purse"), "{line}");
                assert!(!line.to_ascii_lowercase().contains("win"), "{line}");
                assert!(!line.contains('_'), "{line}");
            }
        }
        assert!(!spare_preview(4).contains("loot"));
    }

    #[test]
    fn choice_preview_lines_sit_under_spare_and_take_all_only() {
        let preview = choice_preview(4);
        assert_eq!(
            preview.line_for(&ScreenAction::Spare),
            Some("+32 silver - Underworld +5.")
        );
        assert_eq!(
            preview.line_for(&ScreenAction::TakeAll),
            Some("+48 silver - Underworld +2 - loot unknown.")
        );
        assert_eq!(preview.line_for(&ScreenAction::Capture), None);
        assert_eq!(
            preview.line_for(&ScreenAction::Return { at_sea: true }),
            None
        );
    }

    #[test]
    fn choice_preview_shows_only_while_the_win_is_pending() {
        let mut facts = facts_for_catalog_captain("raj_the_quiet", None, true).unwrap();
        let strength = facts.strength;
        assert!(present(&facts).unwrap().choice_preview.is_none());
        facts.phase = "resolved".to_string();
        facts.pending_victory = true;
        let outcome = present(&facts).unwrap();
        assert_eq!(outcome.choice_preview, Some(choice_preview(strength)));
        // Strength stays on the card; the preview reads the same number.
        assert!(outcome.card.contains(&format!("Strength {strength}")));
        // After the choice the card is still Outcome, but the preview is gone.
        facts.pending_victory = false;
        let done = present(&facts).unwrap();
        assert_eq!(done.phase, ScreenPhase::Outcome);
        assert!(done.choice_preview.is_none());
        // Capture-available is Outcome without a pending win: no preview.
        facts.phase = "capture_available".to_string();
        assert!(present(&facts).unwrap().choice_preview.is_none());
        for phase in ["naval", "boarding", "duel"] {
            facts.phase = phase.to_string();
            assert!(present(&facts).unwrap().choice_preview.is_none(), "{phase}");
        }
    }

    #[test]
    fn scripted_boarding_walks_approach_naval_boarding_and_outcome() {
        let mut session = scripted_session();
        let approach =
            facts_for_catalog_captain(SCRIPTED_CAPTAIN, player_ship(&session), true).unwrap();
        assert_eq!(present(&approach).unwrap().phase, ScreenPhase::Approach);

        let step = session
            .encounter_choice_with("fight", Some(SCRIPTED_CAPTAIN), None)
            .unwrap();
        let naval_actions = probe_naval(&mut session);
        let mut facts = adopt(&session, &step, Some(&approach), &naval_actions, &[]);
        assert_eq!(present(&facts).unwrap().phase, ScreenPhase::Naval);

        let mut boarding = false;
        for action in SCRIPTED_NAVAL {
            let step = session.naval_round(action).unwrap();
            let actions = if step.phase == "naval" {
                probe_naval(&mut session)
            } else {
                Vec::new()
            };
            facts = adopt(&session, &step, Some(&facts), &actions, &[]);
            if step.phase == "boarding" {
                boarding = true;
                break;
            }
        }
        assert!(boarding, "script did not reach boarding");
        assert_eq!(present(&facts).unwrap().phase, ScreenPhase::Boarding);
        assert_eq!(present(&facts).unwrap().actions, vec![ScreenAction::Board]);

        let step = session.resolve_boarding().unwrap();
        let combat_actions = probe_fight(&mut session);
        facts = adopt(&session, &step, Some(&facts), &[], &combat_actions);
        let personal = present(&facts).unwrap();
        assert_eq!(personal.phase, ScreenPhase::Personal);
        assert_eq!(personal.log, step.message);
        assert!(personal
            .actions
            .iter()
            .any(|action| action == &ScreenAction::Combat("thrust".to_string())));
        assert!(combat_actions.iter().any(|action| action == "dodge"));

        for action in SCRIPTED_FIGHT {
            let step = session.fight(action).unwrap();
            let actions = if step.phase == "duel" {
                probe_fight(&mut session)
            } else {
                Vec::new()
            };
            facts = adopt(&session, &step, Some(&facts), &[], &actions);
        }
        assert!(session.pending_victory());
        let outcome = present(&facts).unwrap();
        assert_eq!(outcome.phase, ScreenPhase::Outcome);
        assert_eq!(
            outcome.actions,
            vec![
                ScreenAction::Spare,
                ScreenAction::Capture,
                ScreenAction::TakeAll,
            ]
        );
        assert_eq!(outcome.log, facts.log);
        assert!(!outcome.log.is_empty());
        let preview = outcome.choice_preview.clone().unwrap();
        let silver = session.world().captain.silver;
        let receipt = session.spare().unwrap();
        // The pre-click line is the receipt's purse and standing.
        assert_eq!(
            preview.spare,
            format!(
                "+{} silver - Underworld +{}.",
                receipt.silver_delta, receipt.standing_delta
            )
        );
        let lines = victory_receipt_lines(&receipt);
        assert!(lines.iter().any(|line| line.starts_with("Spared ")));
        assert!(lines
            .iter()
            .any(|line| line == &format!("+{} silver.", receipt.silver_delta)));
        assert!(!session.pending_victory());
        assert!(session.world().captain.silver > silver);
        facts.pending_victory = false;
        facts.phase = "resolved".to_string();
        facts.on_session = false;
        let done = present(&facts).unwrap();
        assert!(done
            .actions
            .iter()
            .any(|action| matches!(action, ScreenAction::Return { at_sea: true })));
        assert!(present(&EncounterFacts {
            phase: String::new(),
            ..facts
        })
        .is_none());
    }

    #[test]
    fn first_playable_agency_does_not_open_an_encounter() {
        let mut with_agency = Session::new("Ada", "merchant", 1, None).unwrap();
        let mut bare = Session::new("Ada", "merchant", 1, None).unwrap();
        with_agency.buy("grain", 1).unwrap();
        bare.buy("grain", 1).unwrap();
        with_agency.depart("al_manar").unwrap();
        bare.depart("al_manar").unwrap();
        let mut docked = false;
        for _ in 0..12 {
            let day = with_agency.world().day;
            let at_sea = at_sea(&with_agency);
            with_agency.advance().unwrap();
            bare.advance().unwrap();
            if at_sea && with_agency.world().day != day {
                let (encounter, _, _) = with_agency.tick_sea_captain_agency();
                assert!(encounter.is_none());
            }
            if with_agency.world().voyage.status == portlight_sim::model::VoyageStatus::InPort
                && with_agency.world().voyage.destination_id == "al_manar"
            {
                docked = true;
                break;
            }
        }
        assert!(docked);
        assert!(with_agency.world().pending_duel.is_none());
        assert_eq!(
            format!("{:?}", with_agency.world()),
            format!("{:?}", bare.world())
        );
    }

    #[test]
    fn bounty_hunt_clears_pending_duel_when_the_encounter_ends() {
        // `parity/scripts/bounty_claim.txt` through the fight. The screen does
        // not clear `pending_duel`. `Session::clear_encounter` does, when the
        // encounter ends.
        let mut session =
            Session::new(SCRIPTED_NAME, SCRIPTED_CAPTAIN_TYPE, SCRIPTED_SEED, None).unwrap();
        session.accept_bounty(SCRIPTED_CAPTAIN).unwrap();
        let state = session.hunt_bounty(SCRIPTED_CAPTAIN).unwrap();
        assert_eq!(state.phase, "approach");
        assert!(session.world().pending_duel.is_some());

        let step = session.encounter_choice("fight").unwrap();
        let naval_actions = probe_naval(&mut session);
        let mut facts = adopt(&session, &step, None, &naval_actions, &[]);
        assert_eq!(present(&facts).unwrap().phase, ScreenPhase::Naval);

        let mut boarding = false;
        for action in SCRIPTED_NAVAL {
            let step = session.naval_round(action).unwrap();
            let actions = if step.phase == "naval" {
                probe_naval(&mut session)
            } else {
                Vec::new()
            };
            facts = adopt(&session, &step, Some(&facts), &actions, &[]);
            if step.phase == "boarding" {
                boarding = true;
                break;
            }
        }
        assert!(boarding, "bounty hunt did not reach boarding");
        let step = session.resolve_boarding().unwrap();
        let combat_actions = probe_fight(&mut session);
        facts = adopt(&session, &step, Some(&facts), &[], &combat_actions);
        for action in SCRIPTED_FIGHT {
            let step = session.fight(action).unwrap();
            let actions = if step.phase == "duel" {
                probe_fight(&mut session)
            } else {
                Vec::new()
            };
            facts = adopt(&session, &step, Some(&facts), &[], &actions);
        }
        assert!(session.pending_victory());
        let outcome = present(&facts).unwrap();
        assert_eq!(outcome.phase, ScreenPhase::Outcome);
        assert!(session.world().pending_duel.is_some());
        let preview = outcome.choice_preview.unwrap();

        let receipt = session.take_all().unwrap();
        assert_eq!(
            preview.take_all,
            format!(
                "+{} silver - Underworld +{} - loot unknown.",
                receipt.silver_delta, receipt.standing_delta
            )
        );
        assert!(!session.pending_victory());
        assert!(
            session.world().pending_duel.is_none(),
            "pending_duel still set after the encounter ended"
        );
    }

    #[test]
    fn unknown_phase_stays_on_the_chart() {
        let mut facts = facts_for_catalog_captain("raj_the_quiet", None, true).unwrap();
        facts.phase.clear();
        assert!(present(&facts).is_none());
        let err = SimError::NotInNavalCombat;
        assert!(action_list_from_error(&err).is_none());
    }

    #[test]
    fn newgame_copy_and_the_save_line_stay_ascii() {
        for page in [
            NewgamePage::Title,
            NewgamePage::Captains,
            NewgamePage::Custom,
            NewgamePage::Load,
        ] {
            let (title, card) = newgame_copy(page);
            assert!(title.is_ascii() && card.is_ascii(), "{title} / {card}");
        }
        assert_eq!(
            newgame_copy(NewgamePage::Load).1,
            "Each slot shows the captain and the day saved. Custom captains keep their name, silver and day, but their trade bonuses aren't saved yet, so they trade at merchant rates after loading."
        );
        assert_eq!(save_confirm_title(true), "Game saved.");
        assert!(save_confirm_title(false).is_ascii());
        let row = portlight_sim::save::SaveSlotSummary {
            slot: "default".to_string(),
            captain: String::new(),
            day: 3,
        };
        assert_eq!(save_slot_label(&row), "default  Unknown  day 3");
        let merchant = Session::starting_captains()
            .into_iter()
            .find(|captain| captain.id == "merchant")
            .unwrap();
        let label = captain_button_label(&merchant);
        assert!(label.is_ascii());
        assert!(label.contains("The Merchant"));
        assert!(label.contains("Porto Novo"));
        assert!(label.contains("sloop"));
        assert!(label.contains("550 silver"));
        assert_eq!(ascii_label("Tomas", "id"), "Tomas");
        assert_eq!(ascii_label("Tom\u{00e1}s", "pn_tomas"), "pn_tomas");
    }

    #[test]
    fn point_buttons_stay_inside_the_sim_budget() {
        let mut draft = CustomDraft::default();
        assert_eq!(draft.points_left(), 0);
        assert_eq!(draft.total(), 10);
        draft.set_pool(PointPool::Trade, 9);
        assert_eq!(draft.trade_points, 2);
        draft.set_pool(PointPool::Sailing, 0);
        assert_eq!(draft.points_left(), 3);
        draft.set_pool(PointPool::Trade, 7);
        assert_eq!(draft.trade_points, 5);
        assert_eq!(draft.total(), 10);
        let spec = draft.spec();
        assert_eq!(spec.name, "Ada");
        assert!(portlight_sim::validate_spec(&spec).is_empty());
        draft.name = "  ".to_string();
        assert_eq!(draft.spec().name, "Captain");
    }

    #[test]
    fn shipyard_at_porto_novo_lists_other_hulls_and_hides_the_flagship() {
        let session = Session::new("Ada", "merchant", 1, None).unwrap();
        let model = shipyard_model(&session).unwrap();
        assert_eq!(model.port_name, "Porto Novo");
        assert!(model.has_shipyard);
        assert!(model.gate_notice.is_empty());
        assert_eq!(model.flagship.template_id, "coastal_sloop");
        assert_eq!(model.flagship.class_label, "Sloop");
        assert_eq!(model.fleet_label, "Fleet 1/3");
        assert!(!model.buying_sells_flagship);
        assert!(model.fleet.is_empty());
        assert!(model.slots_notice.is_empty());
        assert_eq!(model.offers.len(), 4);
        assert!(model.offers.iter().all(|offer| offer.id != "coastal_sloop"));
        assert_eq!(model.upgrades.len(), 18);
        let cutter = model
            .offers
            .iter()
            .find(|offer| offer.id == "swift_cutter")
            .unwrap();
        assert_eq!(
            cutter.line(),
            "Swift Cutter  Cutter  450 silver  cargo 50  speed 9  hull 70  cannons 2"
        );
        let man = model
            .offers
            .iter()
            .find(|offer| offer.id == "royal_man_of_war")
            .unwrap();
        assert_eq!(man.class_label, "Man-of-war");
        let strapping = model
            .upgrades
            .iter()
            .find(|upgrade| upgrade.id == "iron_strapping")
            .unwrap();
        assert_eq!(strapping.summary, "Hull +15");
        let nest = model
            .upgrades
            .iter()
            .find(|upgrade| upgrade.id == "crows_nest")
            .unwrap();
        assert_eq!(nest.summary, "Maneuver +0.05, Danger Reduction");
        // One casing for every upgrade summary: each term starts with a capital.
        for upgrade in &model.upgrades {
            for term in upgrade.summary.split(", ") {
                assert!(
                    term.chars().next().is_some_and(|c| c.is_ascii_uppercase()),
                    "{}: {term}",
                    upgrade.id
                );
            }
        }
        for line in model.flagship.lines(model.silver, &model.fleet_label) {
            assert!(line.is_ascii(), "{line}");
        }
        assert!(model.offers.iter().all(|offer| offer.line().is_ascii()));
        assert!(model
            .upgrades
            .iter()
            .all(|upgrade| upgrade.line().is_ascii()));
        assert_eq!(
            buy_confirm_line(cutter, false),
            "Buy Swift Cutter for 450 silver?"
        );
        assert!(buy_confirm_line(cutter, true).starts_with(FLEET_FULL_BUY));
        assert_eq!(
            buy_result_line("Swift Cutter", "Coastal Sloop", true),
            "Bought Swift Cutter. Coastal Sloop is docked here."
        );
        assert_eq!(
            buy_result_line("Swift Cutter", "Coastal Sloop", false),
            "Bought Swift Cutter. Coastal Sloop was sold."
        );
        assert_eq!(NO_FLEET_HERE, "No other ships docked at this port.");
        assert!(NO_SHIPYARD_BODY.is_ascii());
        assert_eq!(
            shipyard_gate_sentence("Al-Manar"),
            "Al-Manar has no shipyard"
        );
        assert_eq!(
            slots_full_sentence(2),
            "No upgrade slots remaining (2/2 used)"
        );
        assert_eq!(fleet_label(0, 10), "Fleet 1/2");
        assert_eq!(fleet_label(0, 11), "Fleet 1/3");
        assert_eq!(fleet_label(0, 26), "Fleet 1/5");
        assert!(!buying_sells_flagship(0, 15));
        assert!(buying_sells_flagship(2, 15));
        assert_eq!(
            ui_sentence("Ship has cargo \u{2014} transfer it first"),
            "Ship has cargo - transfer it first"
        );
    }

    #[test]
    fn shipyard_model_is_absent_at_sea() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        session.depart("al_manar").unwrap();
        assert!(shipyard_model(&session).is_none());
    }

    #[test]
    fn a_shipyard_frame_needs_the_plate_and_a_button_at_1280x720() {
        let mut samples = vec![[20, 28, 41]; 200];
        assert!(shipyard_frame_rejected(1280, 720, &samples));
        samples.extend(std::iter::repeat_n([184, 148, 92], 4));
        samples.extend(std::iter::repeat_n([140, 107, 61], 2));
        assert!(!shipyard_frame_rejected(1280, 720, &samples));
        assert!(shipyard_frame_rejected(1280, 800, &samples));
    }

    #[test]
    fn a_newgame_frame_needs_the_plate_and_a_button_at_1280x800() {
        let mut samples = vec![[20, 28, 41]; 200];
        assert!(newgame_frame_rejected(1280, 800, &samples));
        samples.extend(std::iter::repeat_n([184, 148, 92], 4));
        samples.extend(std::iter::repeat_n([140, 107, 61], 2));
        assert!(!newgame_frame_rejected(1280, 800, &samples));
        assert!(newgame_frame_rejected(1280, 720, &samples));
        assert!(newgame_frame_rejected(1280, 800, &[]));
    }

    #[test]
    fn receipt_lines_copy_loot_flavor_and_departure_text() {
        let receipt = VictoryReceipt {
            spared: false,
            silver_delta: 55,
            loot: Vec::new(),
            loot_messages: vec!["+12 silver".to_string(), "Found: Cutlass".to_string()],
            standing_delta: 2,
            faction_id: "iron_wolves".to_string(),
            enemy_captain_id: "raj_the_quiet".to_string(),
            enemy_captain_name: "Raj the Quiet".to_string(),
            reactions: vec![(
                "red_tomas".to_string(),
                1,
                "Red Tomas approves. (+1 morale)".to_string(),
            )],
            departures: vec![portlight_sim::companion::DepartureEvent {
                companion_id: "dr_amara".to_string(),
                companion_name: "Dr. Amara".to_string(),
                reason: "Morale collapsed".to_string(),
                departure_line: "I cannot stay.".to_string(),
            }],
        };
        assert_eq!(
            victory_receipt_lines(&receipt),
            vec![
                "Took all from Raj the Quiet.".to_string(),
                "+55 silver.".to_string(),
                "+12 silver".to_string(),
                "Found: Cutlass".to_string(),
                "Underworld standing The Iron Wolves +2.".to_string(),
                "Red Tomas approves. (+1 morale)".to_string(),
                "Dr. Amara leaves. I cannot stay.".to_string(),
            ]
        );
        let mut quiet = receipt;
        quiet.standing_delta = 0;
        quiet.reactions.clear();
        quiet.departures.clear();
        quiet.loot_messages.clear();
        quiet.spared = true;
        assert_eq!(
            victory_receipt_lines(&quiet),
            vec![
                "Spared Raj the Quiet.".to_string(),
                "+55 silver.".to_string(),
            ]
        );
        let shocks = vec!["Shortage: silk scarce at Coral Throne".to_string()];
        let notes = vec![
            "Healing: Gunshot Wound (29 days left).".to_string(),
            String::new(),
        ];
        assert_eq!(
            day_log_lines(&[], &shocks, &notes, None),
            vec![
                "Shortage: silk scarce at Coral Throne".to_string(),
                "Healing: Gunshot Wound (29 days left).".to_string(),
            ]
        );
    }

    fn desk_strings_are_ascii(desk: &CrewDesk) {
        assert!(desk.port_name.is_ascii());
        assert!(desk.status.is_ascii());
        assert!(desk.provision_text.is_ascii());
        assert!(desk.known_styles.is_ascii());
        assert!(desk.party_line.is_ascii());
        for role in &desk.roles {
            assert!(role.text.is_ascii(), "{}", role.text);
        }
        for officer in &desk.officers {
            assert!(officer.is_ascii(), "{officer}");
        }
        for style in &desk.styles {
            assert!(style.text.is_ascii(), "{}", style.text);
        }
        for skill in &desk.skills {
            assert!(skill.text.is_ascii(), "{}", skill.text);
            assert!(skill.next_text.is_ascii(), "{}", skill.next_text);
            if let Some(empty) = &skill.empty_trainer {
                assert!(empty.is_ascii(), "{empty}");
            }
        }
        for member in &desk.party {
            assert!(member.is_ascii(), "{member}");
        }
        for offer in &desk.offers {
            assert!(offer.text.is_ascii(), "{}", offer.text);
        }
    }

    #[test]
    fn porto_novo_desk_lists_roster_training_and_no_companions() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        let desk = crew_desk(&session).unwrap();
        assert_eq!(desk.port_name, "Porto Novo");
        assert!(desk.status.contains("Crew "));
        assert!(desk.status.contains('/'));
        let sailor = desk.roles.iter().find(|role| role.id == "sailor").unwrap();
        assert_eq!(sailor.count, 3);
        assert!(sailor.max.is_none());
        assert!(sailor.text.contains("  no cap  "));
        assert!(!sailor.text.contains("max -"));
        assert_eq!(sailor.hire_cost, 4);
        assert!(sailor.hire_five);
        assert!(sailor.fire_enabled);
        let gunner = desk.roles.iter().find(|role| role.id == "gunner").unwrap();
        assert_eq!(gunner.max, Some(3));
        assert_eq!(gunner.hire_cost, 20);
        assert!(!gunner.hire_five);
        assert!(!gunner.fire_enabled);
        let silver = session.world().captain.silver;
        session.hire_crew(1, "sailor").unwrap();
        assert_eq!(session.world().captain.silver, silver - sailor.hire_cost);
        let per_day = desk.provision_per_day;
        let before = session.world().captain.silver;
        session.provision(1).unwrap();
        assert_eq!(before - session.world().captain.silver, per_day);
        assert!(desk.provision_text.contains(&format!("{per_day} silver")));
        assert!(desk.styles.iter().any(|style| {
            style.id == "la_destreza" && style.cost == 80 && style.days == 5 && !style.known
        }));
        assert_eq!(desk.known_styles, "Known styles: none");
        let skill = desk
            .skills
            .iter()
            .find(|skill| skill.id == "blacksmith")
            .unwrap();
        assert!(skill.text.contains("Old Vasquez"));
        assert!(skill.can_train);
        assert_eq!(skill.cost, 50);
        assert_eq!(skill.days, 3);
        assert!(skill.empty_trainer.is_none());
        assert!(desk.offers.is_empty());
        assert_eq!(desk.party.len(), 0);
        let train = train_confirm_line("La Destreza", 80, 5);
        assert!(train.contains("80"));
        assert!(train.contains("advance 5 days"));
        let skill_line =
            skill_confirm_line(&skill.level_name, &skill.skill_name, skill.cost, skill.days);
        assert!(skill_line.starts_with("Learn "));
        assert!(skill_line.contains("advance 3 days"));
        assert!(skill_line.contains("50"));
        assert!(!train.contains("injur"));
        assert!(!recruit_confirm_line("Iron Marta", 80).contains("advance"));
        assert!(hire_needs_confirm(50));
        assert!(!hire_needs_confirm(20));
        desk_strings_are_ascii(&desk);
        assert!(NO_COMPANIONS_FOR_HIRE.is_ascii());
        assert!(NO_FIGHTING_MASTER.is_ascii());
    }

    #[test]
    fn companions_are_offered_only_at_their_home_port() {
        let mut session = Session::new("Ada", "corsair", 1, None).unwrap();
        let desk = crew_desk(&session).unwrap();
        let ids: Vec<&str> = desk.offers.iter().map(|offer| offer.id.as_str()).collect();
        assert!(ids.contains(&"red_tomas"));
        assert!(ids.contains(&"rosa_the_fence"));
        assert!(!ids.contains(&"iron_marta"));
        assert!(desk
            .offers
            .iter()
            .all(|offer| offer.text.contains("standing")));
        let rosa = desk
            .offers
            .iter()
            .find(|offer| offer.id == "rosa_the_fence")
            .unwrap();
        assert!(rosa.text.contains("Smuggler"), "{}", rosa.text);
        assert!(desk.roles.iter().all(|role| role.id != "smuggler"));
        assert!(matches!(
            session.hire_crew(1, "smuggler"),
            Err(portlight_sim::SimError::UnknownRole(_))
        ));
        let skill = desk
            .skills
            .iter()
            .find(|skill| skill.id == "blacksmith")
            .unwrap();
        assert_eq!(
            skill.empty_trainer.as_deref(),
            Some("No blacksmith trainer at this port.")
        );
        desk_strings_are_ascii(&desk);
    }

    #[test]
    fn the_crew_desk_is_closed_at_sea() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        session.depart("al_manar").unwrap();
        assert!(crew_desk(&session).is_none());
    }

    #[test]
    fn hired_officers_are_listed_in_ascii() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        session.hire_crew(1, "gunner").unwrap();
        let desk = crew_desk(&session).unwrap();
        assert_eq!(desk.officers.len(), 1);
        assert!(desk.officers[0].contains("Gunner"));
        assert!(desk.officers[0].contains(" - "));
        desk_strings_are_ascii(&desk);
    }

    #[test]
    fn crew_desk_text_says_stores_everywhere() {
        let session =
            Session::new(SCRIPTED_NAME, SCRIPTED_CAPTAIN_TYPE, SCRIPTED_SEED, None).unwrap();
        let desk = crew_desk(&session).unwrap();
        for text in [
            desk.status.as_str(),
            desk.provision_text.as_str(),
            CREW_STORES_SUBHEAD,
            CREW_STORES_PLUS_FIVE,
            CREW_STORES_PLUS_ONE,
        ] {
            assert!(!text.to_lowercase().contains("provision"), "{text}");
            assert!(text.is_ascii(), "{text}");
        }
        assert!(desk.status.contains("Stores "), "{}", desk.status);
        assert_eq!(CREW_STORES_PLUS_FIVE, "Stores +5");
        assert_eq!(CREW_STORES_PLUS_ONE, "Stores +1");
    }

    #[test]
    fn the_desk_prices_provisions_after_standing() {
        let mut session = Session::new("Ada", "merchant", 1, Some("silva_bay")).unwrap();
        let desk = crew_desk(&session).unwrap();
        let listed = session.world().port("silva_bay").unwrap().provision_cost;
        assert!(listed > 1, "{listed}");
        assert_eq!(desk.provision_per_day, listed);
        assert!(!desk.provision_text.contains("listed"));
        assert!(desk
            .provision_text
            .contains(&format!("{listed} silver a day")));
        let before = session.world().captain.silver;
        session.provision(1).unwrap();
        assert_eq!(
            before - session.world().captain.silver,
            desk.provision_per_day
        );
        let mut standing = portlight_sim::model::Standing {
            regional: [0; 5],
            heat: [0; 5],
            commercial_trust: 0,
            port_standing: Vec::new(),
            underworld: Vec::new(),
            incidents: Vec::new(),
        };
        standing.set_port("silva_bay", 30);
        let discounted = effective_provision_per_day(listed, &standing, "silva_bay");
        assert!(discounted < listed, "{discounted} vs {listed}");
        assert_eq!(
            discounted,
            1.max(portlight_sim::util::py_trunc(listed as f64 * 0.8))
        );
    }

    #[test]
    fn blacksmith_levels_follow_the_catalog_and_vasquez_stops_at_two() {
        let catalog = portlight_sim::content::content();
        let skill = catalog
            .skills
            .skills
            .iter()
            .find(|skill| skill.id == "blacksmith")
            .unwrap();
        assert_eq!(
            skill
                .levels
                .iter()
                .map(|level| (level.name.as_str(), level.silver_cost, level.training_days))
                .collect::<Vec<_>>(),
            vec![
                ("Apprentice", 50, 3),
                ("Journeyman", 150, 5),
                ("Master", 400, 8),
            ]
        );
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        let apprentice = crew_desk(&session)
            .unwrap()
            .skills
            .into_iter()
            .find(|skill| skill.id == "blacksmith")
            .unwrap();
        assert_eq!(apprentice.level_name, "Apprentice");
        assert_eq!((apprentice.cost, apprentice.days), (50, 3));
        assert!(apprentice.can_train);
        assert!(apprentice.text.contains("teaches to 2"));
        session.spend_skill_point("blacksmith").unwrap();
        let journeyman = crew_desk(&session)
            .unwrap()
            .skills
            .into_iter()
            .find(|skill| skill.id == "blacksmith")
            .unwrap();
        assert_eq!(journeyman.level_name, "Journeyman");
        assert_eq!((journeyman.cost, journeyman.days), (150, 5));
        assert!(journeyman.can_train);
        session.spend_skill_point("blacksmith").unwrap();
        let master = crew_desk(&session)
            .unwrap()
            .skills
            .into_iter()
            .find(|skill| skill.id == "blacksmith")
            .unwrap();
        assert_eq!(master.level_name, "Master");
        assert_eq!((master.cost, master.days), (400, 8));
        assert!(!master.can_train);
        assert!(master.next_text.contains("Master"));
    }

    #[test]
    fn ascii_punctuation_maps_dashes_and_minus() {
        assert_eq!(
            ascii_punctuation("A gunshot cracks \u{2014} 6 damage!"),
            "A gunshot cracks - 6 damage!"
        );
        assert_eq!(ascii_punctuation("Hull\u{2013}crew"), "Hull - crew");
        assert_eq!(ascii_punctuation("Crew \u{2212}"), "Crew -");
        assert_eq!(
            ascii_punctuation("\u{201c}Fair winds\u{201d} \u{2026} it\u{2019}s"),
            "\"Fair winds\" ... it's"
        );
        assert_eq!(
            ascii_punctuation("Your hull 18/20 \u{00b7} crew 4"),
            "Your hull 18/20 - crew 4"
        );
        assert_eq!(ascii_punctuation("Plain text - 3"), "Plain text - 3");
        assert_eq!(ascii_punctuation("Tom\u{00e1}s"), "Tom\u{00e1}s");
        for text in [
            "a \u{2014} b",
            "a\u{2014}b",
            "\u{2212}5",
            "x \u{2013} y \u{2026}",
        ] {
            assert!(ascii_punctuation(text).is_ascii(), "{text}");
            assert!(!ascii_punctuation(text).contains("  "), "{text}");
        }
        assert_eq!(
            session_text("", "Shot \u{2014} 6 damage!"),
            "Shot - 6 damage!"
        );
    }

    #[test]
    fn strip_markup_keeps_the_text_inside_tags() {
        assert_eq!(
            strip_markup("[dim]Landmark: The Cliff Tavern[/dim]"),
            "Landmark: The Cliff Tavern"
        );
        assert_eq!(
            strip_markup("[bold]You arrive at Al-Manar.[/bold]"),
            "You arrive at Al-Manar."
        );
        assert_eq!(
            strip_markup("[bold]Harbor:[/bold] busy quay"),
            "Harbor: busy quay"
        );
        assert_eq!(
            strip_markup("Use [bold]portlight duel <stance>[/bold] to fight."),
            "Use portlight duel <stance> to fight."
        );
        assert_eq!(strip_markup("[bold red]Hot[/] day"), "Hot day");
        assert_eq!(
            strip_markup("[bold yellow]Storm warning[/bold yellow] at sea"),
            "Storm warning at sea"
        );
        // A Rich escape is a literal bracket, never a tag.
        assert_eq!(strip_markup(r"\[dim]x"), "[dim]x");
        assert_eq!(
            strip_markup(r"keep \[x] and [dim]y[/dim]"),
            "keep [x] and y"
        );
        // Not markup: kept as written.
        for plain in [
            "Bounty [3/5] posted",
            "[contracts.accept.x]",
            "a [ b",
            "x ] y",
            "[]",
            "[Upper]",
            "Grain x5",
            // Lowercase words that are not style words stay too.
            "[x] done",
            "[see note]",
            "Use [port] here",
            "[weather] calm",
            "[bold x]",
            "[color=red]Hot",
        ] {
            assert_eq!(strip_markup(plain), plain);
        }
    }

    #[test]
    fn duel_result_line_drops_zero_terms() {
        assert_eq!(
            duel_result_line("Won", "Bram", 40, 1),
            "Won the duel with Bram. Silver +40. Standing +1, shown only."
        );
        assert_eq!(
            duel_result_line("Lost", "Bram", -36, 0),
            "Lost the duel with Bram. Silver -36."
        );
        assert_eq!(
            duel_result_line("Drew", "Bram", 0, 0),
            "Drew the duel with Bram."
        );
        assert_eq!(
            duel_result_line("Won", "", 0, 2),
            "Won the duel. Standing +2, shown only."
        );
        for line in [
            duel_result_line("Lost", "Bram", -36, 0),
            duel_result_line("Won", "Bram", 0, 0),
        ] {
            assert!(!line.contains("+0"), "{line}");
        }
    }

    #[test]
    fn duel_prompt_names_the_faction_in_ascii() {
        let duel = portlight_sim::model::PendingDuel {
            captain_id: "c".into(),
            captain_name: "Bram".into(),
            faction_id: "iron_wolves".into(),
            personality: "aggressive".into(),
            strength: 3,
            region: "Mediterranean".into(),
        };
        let text = duel_prompt(&duel);
        assert!(text.is_ascii(), "{text}");
        assert!(!text.contains('\u{00b7}'), "{text}");
        assert!(
            text.contains("Faction The Iron Wolves - Aggressive - strength 3 - Mediterranean."),
            "{text}"
        );
        assert!(!text.contains("aggressive"), "{text}");
        assert!(!text.contains("iron_wolves"), "{text}");
    }

    /// Crew desk names: a catalog miss reads as humanized copy, never the
    /// raw id; a known id keeps its catalog name.
    #[test]
    fn catalog_misses_never_print_raw_ids() {
        assert_eq!(role_name("salt_spit_cove"), "Salt Spit Cove");
        assert_eq!(
            display_or_humanized(None, "salt_spit_cove"),
            "Salt Spit Cove"
        );
        assert_eq!(
            display_or_humanized(Some("Caf\u{e9}"), "salt_spit_cove"),
            "Salt Spit Cove"
        );
        assert_eq!(display_or_humanized(Some("Raj"), "raj_the_quiet"), "Raj");
        let session = Session::new(SCRIPTED_NAME, SCRIPTED_CAPTAIN_TYPE, SCRIPTED_SEED, None)
            .expect("scripted session");
        let desk = crew_desk(&session).expect("docked desk");
        let port = session
            .world()
            .port(&session.world().voyage.destination_id)
            .expect("port");
        assert_eq!(desk.port_name, port.name);
        for text in desk
            .party
            .iter()
            .chain(desk.offers.iter().map(|offer| &offer.text))
            .chain(std::iter::once(&desk.known_styles))
        {
            assert!(!text.contains('_'), "{text}");
        }
    }

    #[test]
    fn at_sea_lane_note_is_the_binding_text() {
        assert_eq!(AT_SEA_LANE_NOTE, "At sea. Use Next day to continue.");
        assert!(AT_SEA_LANE_NOTE.is_ascii());
        assert!(!AT_SEA_LANE_NOTE.contains("Advance"));
        assert_eq!(empty_lane_note(true), AT_SEA_LANE_NOTE);
        assert_eq!(empty_lane_note(false), "No lanes from here.");
        assert!(NO_LANES_NOTE.is_ascii());
    }

    #[test]
    fn status_header_says_stores_not_provisions() {
        let line = status_header(2, "spring", "Ada", 89, 9);
        assert_eq!(line, "Day 2   spring   Ada   89 silver   Stores 9");
        assert!(line.is_ascii());
        assert!(line.contains("Stores 9"));
        assert!(!line.contains("provisions"));
    }

    #[test]
    fn price_line_says_stores_listed() {
        let line = services_line(4, 1);
        assert_eq!(
            line,
            "Sailor listed 4 silver. Stores listed 1 silver a day."
        );
        assert!(line.is_ascii());
        assert!(!line.to_lowercase().contains("provisions"));
    }

    #[test]
    fn dimmed_frame_gate_accepts_a_mostly_ink_frame_with_a_card() {
        let ink = [20, 28, 41];
        let text = [94, 82, 64];
        let mut dimmed = vec![ink; 90];
        dimmed.extend(vec![text; 10]);
        assert!(frame_mostly_flat(&dimmed));
        assert!(!dimmed_frame_rejected(1280, 720, 1280, 720, &dimmed));
        assert!(dimmed_frame_rejected(1280, 720, 1280, 720, &[ink; 100]));
        assert!(dimmed_frame_rejected(1280, 800, 1280, 720, &dimmed));
        assert!(dimmed_frame_rejected(0, 0, 1280, 720, &[]));
    }
}
