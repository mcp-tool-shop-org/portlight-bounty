//! Decisions the Godot view can make without a display server.
//!
//! Projection and facing stay in `portlight-chart`. This module is the view's
//! use of that math, plus the button rules that sit beside `Session`.
//! Sampling a captured image only reads pixels; it does not draw.

use godot::classes::Image;
use godot::prelude::*;
use portlight_chart::{plates_for_class, ship_draw, Facing, ShipDraw};
use portlight_sim::encounter::EncounterState;
use portlight_sim::session::{EncounterStep, Session};
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

/// The Duel button is available whenever a duel is pending. Stance count is
/// the sim's check (`Session::duel`), not a second gate in the view.
pub(crate) fn duel_button_enabled(pending: bool) -> bool {
    pending
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

/// Plate-panel fill, `Color::from_rgb(0.72, 0.58, 0.36)`.
const ENCOUNTER_PLATE_RGB: [u8; 3] = [184, 148, 92];
/// Action-button fill, `Color::from_rgb(0.55, 0.42, 0.24)`.
const ENCOUNTER_BUTTON_RGB: [u8; 3] = [140, 107, 61];

/// Encounter captures do not use [`capture_frame_rejected`].
///
/// Addendum M.1 shrink-wraps the plate, so the ink ground `(20, 28, 41)`
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

/// Exact integer scale for a plate in a UI panel. Addendum M.1.
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
        message.to_string()
    } else {
        flavor.to_string()
    }
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
    /// Always set. The portrait slot is [`PORTRAIT_PLACEHOLDER`].
    pub portrait_placeholder: bool,
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
    facts.player_crew_delta = step.player_crew_delta;
    facts.enemy_crew_delta = step.enemy_crew_delta;
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
        actions,
        portrait_placeholder: true,
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
            "Your hull {}/{max} · crew {}",
            facts.player_hull, facts.player_crew
        ),
        None => format!(
            "Your hull {} · crew {}",
            facts.player_hull, facts.player_crew
        ),
    });
    // A win's outcome card keeps the last fight step's hull, which is not a
    // post-fight reading. Session does not expose one, so the line is omitted.
    if phase != ScreenPhase::Outcome {
        if let Some(hull) = facts.enemy_hull {
            let crew = facts.enemy_crew.unwrap_or(0);
            lines.push(match facts.enemy_hull_max {
                Some(max) => format!("Enemy hull {hull}/{max} · crew {crew}"),
                None => format!("Enemy hull {hull} · crew {crew}"),
            });
        }
    }
    if phase == ScreenPhase::Personal && facts.kind == "fight" {
        lines.push(format!(
            "Your HP {} · opponent {}",
            facts.player_hp, facts.opponent_hp
        ));
    }
    lines.join("\n")
}

fn faction_name(faction_id: &str) -> String {
    if faction_id.is_empty() {
        return String::new();
    }
    portlight_sim::content::content()
        .faction(faction_id)
        .map(|faction| faction.name.clone())
        .unwrap_or_else(|| faction_id.to_string())
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
    let port_name = ascii_label(&port.name, &port.id).to_string();
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
                name: ascii_label(&owned.ship.name, &owned.ship.template_id).to_string(),
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
            name: ascii_label(&ship.name, &ship.template_id).to_string(),
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
                name: ascii_label(&template.name, &template.id).to_string(),
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
                name: ascii_label(&upgrade.name, &upgrade.id).to_string(),
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
        parts.push(format!("speed +{}", format_amount(def.speed_bonus)));
    }
    if def.speed_penalty != 0.0 {
        parts.push(format!("speed -{}", format_amount(def.speed_penalty)));
    }
    if def.hull_max_bonus != 0 {
        parts.push(format!("hull +{}", def.hull_max_bonus));
    }
    if def.cargo_bonus != 0 {
        parts.push(format!("cargo +{}", def.cargo_bonus));
    }
    if def.cannon_bonus != 0 {
        parts.push(format!("cannons +{}", def.cannon_bonus));
    }
    if def.maneuver_bonus != 0.0 {
        parts.push(format!("maneuver +{}", format_amount(def.maneuver_bonus)));
    }
    if def.storm_resist_bonus != 0.0 {
        parts.push(format!("storm +{}", format_amount(def.storm_resist_bonus)));
    }
    if def.crew_max_bonus != 0 {
        parts.push(format!("crew +{}", def.crew_max_bonus));
    }
    if !def.special.is_empty() {
        let words = def.special.replace('_', " ");
        if words.is_ascii() {
            parts.push(words);
        }
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

#[cfg(test)]
mod tests {
    use portlight_chart::{chart_to_screen_f, chart_to_uv, facing_from_uv, Facing};

    use super::*;

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
        assert_line_matches_step(&step, &boarded);
        assert!(
            boarded.is_empty(),
            "boarding crew loss is not an EncounterStep delta: {boarded}"
        );

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
        let silver = session.world().captain.silver;
        session.spare().unwrap();
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
        assert_eq!(present(&facts).unwrap().phase, ScreenPhase::Outcome);
        assert!(session.world().pending_duel.is_some());

        session.take_all().unwrap();
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
        assert_eq!(strapping.summary, "hull +15");
        let nest = model
            .upgrades
            .iter()
            .find(|upgrade| upgrade.id == "crows_nest")
            .unwrap();
        assert_eq!(nest.summary, "maneuver +0.05, danger reduction");
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
}
