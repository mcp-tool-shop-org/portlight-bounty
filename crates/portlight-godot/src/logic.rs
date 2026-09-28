//! Decisions the Godot view can make without a display server.
//!
//! Projection and facing stay in `portlight-chart`. This module is the view's
//! use of that math, plus the button rules that sit beside `Session`.
//! Sampling a captured image only reads pixels; it does not draw.

use godot::classes::Image;
use godot::prelude::*;
use portlight_chart::{ship_asset, ship_class_plate, Asset, Facing};
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
    if samples.is_empty() || width != expect_w || height != expect_h {
        return true;
    }
    let plate = sample_fraction(samples, ENCOUNTER_PLATE_RGB);
    let button = sample_fraction(samples, ENCOUNTER_BUTTON_RGB);
    // A 64×64 plate at 2× plus padding is ~3% of the frame. One stray pixel
    // must not pass a blank ink overlay.
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

/// Side-on plate for the ship's content class. `ship_draw` replaces this
/// lookup when that helper is available. Missing classes use the sloop.
pub(crate) fn encounter_plate(class: &str) -> &'static Asset {
    ship_class_plate(class, Facing::F3).unwrap_or_else(|| ship_asset(Facing::F3))
}

/// Hull and crew the screen can read off [`Session::world`].
#[derive(Debug, Clone, Copy)]
pub(crate) struct PlayerShip {
    pub hull: i64,
    pub hull_max: i64,
    pub crew: i64,
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
    pub actions: Vec<ScreenAction>,
    /// Always set. The portrait slot is [`PORTRAIT_PLACEHOLDER`].
    pub portrait_placeholder: bool,
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
        facts_from_step, player_ship, present, stance_duel_visible, ui_plate_panel, EncounterFacts,
        ScreenAction, ScreenPhase, StepInput, PORTRAIT_PLACEHOLDER, SCRIPTED_CAPTAIN,
        SCRIPTED_CAPTAIN_TYPE, SCRIPTED_DEPART, SCRIPTED_FIGHT, SCRIPTED_NAME, SCRIPTED_NAVAL,
        SCRIPTED_SEED,
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
        let sloop = encounter_plate("sloop");
        assert_eq!(sloop.id, "ship_sloop_f3");
        assert_eq!(ui_plate_panel(sloop.canvas_w, sloop.canvas_h), (168, 168));
        let cutter = encounter_plate("cutter");
        assert_eq!(cutter.id, "ship_cutter_f3");
        assert_eq!(ui_plate_panel(cutter.canvas_w, cutter.canvas_h), (184, 200));
        assert_eq!(encounter_plate("man_of_war").id, "ship_sloop_f3");
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
        assert_eq!(
            view.actions,
            vec![
                ScreenAction::Choice("negotiate"),
                ScreenAction::Choice("flee"),
                ScreenAction::Choice("fight"),
            ]
        );
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
}
