//! First playable: Mediterranean chart, sail Porto Novo to Al-Manar, trade.
//!
//! Buttons call [`portlight_sim::Session`]. Labels repeat fields those queries
//! already computed. Good names and the season name are catalog strings.
//!
//! A pending duel is named on the panel. Next day still calls `Session::advance`,
//! which ticks reputation and does not move the day. A `SimError` is shown if
//! advance fails for another reason. The stance fight is `Session::duel`
//! (the sim requires at least three of thrust, slash, and parry) or
//! `Session::resolve_pending_duel`. `DuelOutcome.standing_delta` is shown and
//! not written onto reputation. `Session::sell` returns `Sale`: the log shows
//! the receipt and any contract summaries. `Session::board` is the contract
//! board. Deck melee is `Session::resolve_boarding`. Hire and provisions
//! call `hire_crew` and `provision`; a `SimError` is shown with its `Display`.
//!
//! An encounter that opens on a sea day (`tick_sea_captain_agency`) or a
//! scripted approach uses the encounter screen: `encounter_choice` /
//! `encounter_choice_with`, `naval_round`, `resolve_boarding`, `fight`,
//! `spare`, `capture`, and `take_all`. The voyage stance duel stays
//! `Session::duel` and `Session::resolve_pending_duel` on the chart panel.
//! That panel is hidden while the encounter screen is open and shown again
//! when the screen closes if a duel is still pending. The screen does not
//! clear `pending_duel`.

use godot::classes::canvas_item::TextureFilter;
use godot::classes::control::{LayoutPreset, SizeFlags};
use godot::classes::scroll_container::ScrollMode;
use godot::classes::text_server::{AutowrapMode, OverrunBehavior};
use godot::classes::viewport::DefaultCanvasItemTextureFilter;
use godot::classes::{
    Button, Control, DisplayServer, HBoxContainer, IControl, Label, Node, Os, PanelContainer,
    ScrollContainer, StyleBoxFlat, SubViewport, SubViewportContainer, VBoxContainer,
};
use godot::global::Error;
use godot::obj::InstanceId;
use godot::prelude::*;
use portlight_chart::{
    art_frame, docked_sloop_marker, lane_inspect, press_port, project_chart, ChartModel, Facing,
    PortPress, FIRST_PLAYABLE_CAPTAIN, FIRST_PLAYABLE_NAME, FIRST_PLAYABLE_SEED,
};
use portlight_sim::economy::TradeReceipt;
use portlight_sim::encounter::EncounterState;
use portlight_sim::model::VoyageStatus;
use portlight_sim::session::{EncounterStep, Sale};
use portlight_sim::{content, DuelOutcome, LaneSuitability, Session, SimError};

use crate::chart_canvas::{connect_port_pressed, ChartCanvas};
use crate::encounter_screen::{self, set_ship_plate, EncounterNodes};
use crate::logic::{
    action_caption, action_list_from_error, at_sea, capture_frame_rejected, chart_host_width,
    duel_button_enabled, encounter_frame_rejected, facts_for_catalog_captain, facts_from_agency,
    facts_from_step, frame_mostly_flat, frame_samples, layout_fits_window, player_ship, present,
    session_text, stance_duel_visible, template_player_ship, EncounterFacts, ScreenAction,
    ScreenPhase, StepInput, PANEL_MIN_W, ROW_SEPARATION, SCRIPTED_CAPTAIN, SCRIPTED_CAPTAIN_TYPE,
    SCRIPTED_DEPART, SCRIPTED_FIGHT, SCRIPTED_NAME, SCRIPTED_NAVAL, SCRIPTED_SEED, WINDOW_H,
    WINDOW_W,
};

const CREAM: Color = Color::from_rgb(0.94, 0.91, 0.84);
const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);

struct MarketRow {
    id: String,
    name: String,
    stock: i64,
    buy: i64,
    sell: i64,
    held: i64,
}

#[derive(Clone)]
enum Action {
    NewGame,
    NextDay,
    Work,
    ToggleMarket,
    HireSailor,
    Provision,
    Stance(Stance),
    ClearStances,
    Duel,
    AutoResolve,
    Sail(String),
    Buy(String),
    Sell(String),
    EncounterChoice(String),
    Naval(String),
    Board,
    Combat(String),
    Spare,
    Capture,
    TakeAll,
    CaptureCrew(i64),
    LeaveEncounter,
}

#[derive(Clone, Copy)]
enum ShotPhase {
    Approach,
    Naval,
    Boarding,
    Personal,
    Outcome,
}

impl ShotPhase {
    fn file_name(self) -> &'static str {
        match self {
            Self::Approach => "encounter-approach.png",
            Self::Naval => "encounter-naval.png",
            Self::Boarding => "encounter-boarding.png",
            Self::Personal => "encounter-fight.png",
            Self::Outcome => "encounter-outcome.png",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Approach => "approach",
            Self::Naval => "naval",
            Self::Boarding => "boarding",
            Self::Personal => "personal fight",
            Self::Outcome => "outcome",
        }
    }

    fn screen(self) -> ScreenPhase {
        match self {
            Self::Approach => ScreenPhase::Approach,
            Self::Naval => ScreenPhase::Naval,
            Self::Boarding => ScreenPhase::Boarding,
            Self::Personal => ScreenPhase::Personal,
            Self::Outcome => ScreenPhase::Outcome,
        }
    }
}

#[derive(Clone, Copy)]
enum Stance {
    Thrust,
    Slash,
    Parry,
}

impl Stance {
    fn as_str(self) -> &'static str {
        match self {
            Self::Thrust => "thrust",
            Self::Slash => "slash",
            Self::Parry => "parry",
        }
    }
}

#[derive(GodotClass)]
#[class(base = Control)]
struct PortlightGame {
    base: Base<Control>,
    session: Option<Session>,
    canvas: Option<Gd<ChartCanvas>>,
    status: Option<Gd<Label>>,
    lane_box: Option<Gd<VBoxContainer>>,
    market_box: Option<Gd<VBoxContainer>>,
    market_scroll: Option<Gd<ScrollContainer>>,
    log_label: Option<Gd<Label>>,
    market_button: Option<Gd<Button>>,
    work_button: Option<Gd<Button>>,
    port_row: Option<Gd<HBoxContainer>>,
    port_note: Option<Gd<Label>>,
    encounter_box: Option<Gd<VBoxContainer>>,
    encounter_label: Option<Gd<Label>>,
    stance_label: Option<Gd<Label>>,
    duel_button: Option<Gd<Button>>,
    stances: Vec<String>,
    log_lines: Vec<String>,
    market_open: bool,
    armed_sail: Option<String>,
    smoke: bool,
    /// Art-director frame: a docked sloop plus a sailing cutter. Not play.
    art: bool,
    smoke_ok: bool,
    shot_path: Option<String>,
    /// The path was filled by a default, not `PORTLIGHT_SHOT`. Headless skips
    /// that save. An explicit shot still runs and fails when it cannot.
    shot_implicit: bool,
    /// The encounter script has finished, including the bounty pass. The ok
    /// line waits until every capture has been judged.
    encounter_checked: bool,
    capture_frames: i32,
    encounter_nodes: Option<EncounterNodes>,
    encounter: Option<EncounterFacts>,
    /// Catalog captain locked into the next `encounter_choice_with`.
    scripted_captain: Option<String>,
    capture_crew: i64,
    encounter_shot_dir: Option<String>,
    encounter_shot: Option<ShotPhase>,
    /// Capture path only. Draws `royal_man_of_war` in the player plate slot
    /// and shows that template's hull and crew on the card.
    galleon_frame: bool,
    /// A rejected screenshot. Kept off [`Self::smoke_ok`] so a failed frame
    /// does not stop the boarding script.
    capture_failed: bool,
}

#[godot_api]
impl IControl for PortlightGame {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            session: None,
            canvas: None,
            status: None,
            lane_box: None,
            market_box: None,
            market_scroll: None,
            log_label: None,
            market_button: None,
            work_button: None,
            port_row: None,
            port_note: None,
            encounter_box: None,
            encounter_label: None,
            stance_label: None,
            duel_button: None,
            stances: Vec::new(),
            log_lines: Vec::new(),
            market_open: false,
            armed_sail: None,
            smoke: false,
            art: false,
            smoke_ok: true,
            shot_path: None,
            shot_implicit: false,
            encounter_checked: false,
            capture_frames: 0,
            encounter_nodes: None,
            encounter: None,
            scripted_captain: None,
            capture_crew: 0,
            encounter_shot_dir: None,
            encounter_shot: None,
            galleon_frame: false,
            capture_failed: false,
        }
    }

    fn ready(&mut self) {
        self.smoke = flag_set("PORTLIGHT_SMOKE") || user_arg("--smoke");
        // A shot is opt-in. Headless `--smoke` checks the session and does not
        // read the viewport: the dummy renderer has no texture, and asking for
        // one logs `Parameter "t" is null` while still exiting 0.
        self.shot_path = std::env::var("PORTLIGHT_SHOT")
            .ok()
            .filter(|path| !path.is_empty())
            .map(|path| resolve_repo_path(&path));
        self.build_ui();
        self.start_game();
        if user_arg("--encounter-screen") {
            self.smoke = true;
            // The five frames are opt-in. A default `/tmp` path is not a
            // request. An explicit directory still captures on headless, and
            // the empty viewport fails the run.
            if encounter_frames_requested() {
                self.encounter_shot_dir = Some(encounter_shot_dir());
                self.begin_encounter_shots();
                self.capture_frames = 4;
            } else {
                self.run_encounter_screen();
                self.capture_frames = 2;
            }
        } else if user_arg("--encounter-galleon") {
            self.smoke = true;
            self.galleon_frame = true;
            if self.shot_path.is_none() {
                self.shot_path = Some(art_shot_path("encounter-galleon.png"));
                self.shot_implicit = true;
            }
            self.prepare_scripted_voyage();
            self.open_scripted_approach();
            self.apply_template_ship_card("royal_man_of_war");
            self.refresh();
            self.capture_frames = 4;
        } else if user_arg("--encounter") {
            self.smoke = true;
            self.run_encounter();
            self.capture_frames = 4;
        } else if user_arg("--duel") {
            self.smoke = true;
            self.run_duel_resolution(false);
            self.capture_frames = 2;
        } else if user_arg("--resolve") {
            self.smoke = true;
            self.run_duel_resolution(true);
            self.capture_frames = 2;
        } else if user_arg("--work") {
            self.smoke = true;
            self.run_work();
            self.capture_frames = 2;
        } else if user_arg("--art") {
            self.smoke = true;
            if self.shot_path.is_none() {
                self.shot_path = Some(art_shot_path("chart-cutter-f7.png"));
                self.shot_implicit = true;
            }
            self.run_art();
            self.capture_frames = 4;
        } else if self.smoke {
            self.run_smoke();
            self.capture_frames = 4;
        }
    }

    fn process(&mut self, _delta: f64) {
        if self.capture_frames <= 0 {
            return;
        }
        self.capture_frames -= 1;
        if self.capture_frames > 0 {
            return;
        }
        if self.advance_encounter_shot() {
            return;
        }
        // Measure after layout. Headless `--smoke` has no shot and skips this:
        // the Xvfb capture is the run that has to see real label sizes.
        if self.smoke && self.shot_path.is_some() && self.market_open {
            self.assert_panel_labels();
        }
        if let Some(path) = self.shot_path.clone() {
            // `--encounter-galleon` is still on the encounter screen. The
            // multi-frame shot saves its own files before this, then the
            // encounter has closed, so a trailing PORTLIGHT_SHOT is a chart.
            // Headless skips only an implicit default. `PORTLIGHT_SHOT` still
            // reads the viewport and fails when that image is empty.
            let skip = headless_runtime() && self.shot_implicit;
            if !skip && !self.save_shot(&path, self.galleon_frame) {
                self.smoke_ok = false;
            }
        }
        if self.capture_failed {
            self.smoke_ok = false;
        }
        if !self.smoke {
            return;
        }
        let code = if self.smoke_ok { 0 } else { 1 };
        if self.encounter_checked {
            godot_print!(
                "portlight encounter smoke {}",
                if self.smoke_ok { "ok" } else { "FAILED" }
            );
        }
        godot_print!(
            "portlight smoke {}",
            if self.smoke_ok { "ok" } else { "FAILED" }
        );
        let mut tree = self.base().get_tree();
        tree.quit_ex().exit_code(code).done();
    }
}

impl PortlightGame {
    fn build_ui(&mut self) {
        assert!(
            layout_fits_window(),
            "panel and chart must fit the 1280x720 window"
        );
        // `set_anchors_preset` keeps the control's current size (it rewrites
        // offsets). On a new node that size is the minimum, which here was the
        // panel's 14+14 content margin: a 28 px strip across 1280, clear
        // colour underneath. Offsets have to be the full-rect preset too.
        self.base_mut()
            .set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);

        let mut row = HBoxContainer::new_alloc();
        row.set_h_size_flags(SizeFlags::EXPAND_FILL);
        row.set_v_size_flags(SizeFlags::EXPAND_FILL);
        row.add_theme_constant_override("separation", ROW_SEPARATION);
        row.set_custom_minimum_size(Vector2::new(WINDOW_W, WINDOW_H));
        self.base_mut().add_child(&row);
        row.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);

        let mut view_host = SubViewportContainer::new_alloc();
        // Stretch resizes this viewport to the control, so the chart is 1:1
        // with the pixels beside the panel. The project Nearest setting does
        // not apply inside a SubViewport; Linear is the viewport default.
        view_host.set_custom_minimum_size(Vector2::new(chart_host_width(), WINDOW_H));
        view_host.set_h_size_flags(SizeFlags::EXPAND_FILL);
        view_host.set_v_size_flags(SizeFlags::EXPAND_FILL);
        view_host.set_stretch(true);
        view_host.set_texture_filter(TextureFilter::NEAREST);
        let mut viewport = SubViewport::new_alloc();
        viewport.set_size(Vector2i::new(chart_host_width() as i32, WINDOW_H as i32));
        viewport.set_disable_3d(true);
        viewport.set_default_canvas_item_texture_filter(DefaultCanvasItemTextureFilter::NEAREST);
        let mut canvas = ChartCanvas::new_alloc();
        let game_for_chart = self.instance_id();
        connect_port_pressed(&mut canvas, move |port_id: GString| {
            let Ok(mut gd) = Gd::<PortlightGame>::try_from_instance_id(game_for_chart) else {
                return;
            };
            gd.bind_mut().on_port_pressed(&port_id.to_string());
        });
        viewport.add_child(&canvas);
        view_host.add_child(&viewport);
        row.add_child(&view_host);
        self.canvas = Some(canvas);

        let mut panel = PanelContainer::new_alloc();
        panel.set_custom_minimum_size(Vector2::new(PANEL_MIN_W, WINDOW_H));
        panel.set_h_size_flags(SizeFlags::SHRINK_END);
        panel.set_v_size_flags(SizeFlags::EXPAND_FILL);
        let mut style = StyleBoxFlat::new_gd();
        style.set_bg_color(Color::from_rgb(0.1, 0.14, 0.19));
        style.set_content_margin_all(14.0);
        panel.add_theme_stylebox_override("panel", &style);
        row.add_child(&panel);

        let mut panel_scroll = ScrollContainer::new_alloc();
        panel_scroll.set_h_size_flags(SizeFlags::EXPAND_FILL);
        panel_scroll.set_v_size_flags(SizeFlags::EXPAND_FILL);
        panel_scroll.set_horizontal_scroll_mode(ScrollMode::DISABLED);
        panel.add_child(&panel_scroll);

        let mut column = VBoxContainer::new_alloc();
        column.set_h_size_flags(SizeFlags::EXPAND_FILL);
        column.set_v_size_flags(SizeFlags::EXPAND_FILL);
        panel_scroll.add_child(&column);

        let game_id = self.instance_id();
        column.add_child(&title_label("Portlight", 22, GOLD));
        column.add_child(&body_label(
            "Chart water and port markers are the approved plates.",
            13,
            MUTED,
        ));

        let mut status = body_label("", 15, CREAM);
        status.set_autowrap_mode(AutowrapMode::WORD_SMART);
        column.add_child(&status);
        self.status = Some(status);

        let mut buttons = HBoxContainer::new_alloc();
        buttons.add_child(&action_button("New game", game_id, Action::NewGame));
        buttons.add_child(&action_button("Next day", game_id, Action::NextDay));
        column.add_child(&buttons);

        let mut port_row = HBoxContainer::new_alloc();
        let market = action_button("Market", game_id, Action::ToggleMarket);
        port_row.add_child(&market);
        self.market_button = Some(market);
        port_row.add_child(&action_button("Hire sailor", game_id, Action::HireSailor));
        port_row.add_child(&action_button("Provisions +5", game_id, Action::Provision));
        let work = action_button("Work", game_id, Action::Work);
        port_row.add_child(&work);
        self.work_button = Some(work);
        port_row.set_visible(false);
        column.add_child(&port_row);
        self.port_row = Some(port_row);
        let mut port_note = body_label("", 12, MUTED);
        port_note.set_autowrap_mode(AutowrapMode::WORD_SMART);
        port_note.set_visible(false);
        column.add_child(&port_note);
        self.port_note = Some(port_note);

        let mut encounter = VBoxContainer::new_alloc();
        encounter.set_visible(false);
        let mut encounter_label = body_label("", 14, Color::from_rgb(0.93, 0.55, 0.42));
        encounter_label.set_autowrap_mode(AutowrapMode::WORD_SMART);
        encounter.add_child(&encounter_label);
        let mut stance_label = body_label("Stances: none yet. Pick at least 3.", 13, CREAM);
        stance_label.set_autowrap_mode(AutowrapMode::WORD_SMART);
        encounter.add_child(&stance_label);
        let mut stance_buttons = HBoxContainer::new_alloc();
        stance_buttons.add_child(&action_button(
            "Thrust",
            game_id,
            Action::Stance(Stance::Thrust),
        ));
        stance_buttons.add_child(&action_button(
            "Slash",
            game_id,
            Action::Stance(Stance::Slash),
        ));
        stance_buttons.add_child(&action_button(
            "Parry",
            game_id,
            Action::Stance(Stance::Parry),
        ));
        stance_buttons.add_child(&action_button("Clear", game_id, Action::ClearStances));
        encounter.add_child(&stance_buttons);
        let mut resolve_buttons = HBoxContainer::new_alloc();
        let duel = action_button("Duel", game_id, Action::Duel);
        resolve_buttons.add_child(&duel);
        resolve_buttons.add_child(&action_button("Auto-resolve", game_id, Action::AutoResolve));
        encounter.add_child(&resolve_buttons);
        column.add_child(&encounter);
        self.encounter_box = Some(encounter);
        self.encounter_label = Some(encounter_label);
        self.stance_label = Some(stance_label);
        self.duel_button = Some(duel);

        column.add_child(&body_label(
            "Lanes from the sail picker. Days use raw ship speed.",
            13,
            MUTED,
        ));
        let (lane_scroll, lane_box) = scrolling(220.0);
        column.add_child(&lane_scroll);
        self.lane_box = Some(lane_box);

        let (market_scroll, market_box) = scrolling(180.0);
        market_scroll.clone().set_visible(false);
        column.add_child(&market_scroll);
        self.market_scroll = Some(market_scroll);
        self.market_box = Some(market_box);

        column.add_child(&body_label("Log", 14, GOLD));
        let mut log = body_label("", 13, CREAM);
        log.set_autowrap_mode(AutowrapMode::WORD_SMART);
        column.add_child(&log);
        self.log_label = Some(log);

        let mut encounter_screen = encounter_screen::build_encounter_screen();
        self.base_mut().add_child(&encounter_screen.root);
        encounter_screen::fill_parent(&mut encounter_screen.root);
        self.encounter_nodes = Some(encounter_screen);
    }

    fn instance_id(&self) -> InstanceId {
        self.to_gd().instance_id()
    }

    fn start_game(&mut self) {
        self.log_lines.clear();
        self.market_open = false;
        self.armed_sail = None;
        self.stances.clear();
        match Session::new(
            FIRST_PLAYABLE_NAME,
            FIRST_PLAYABLE_CAPTAIN,
            FIRST_PLAYABLE_SEED,
            None,
        ) {
            Ok(session) => {
                let place = docked_name(&session).unwrap_or_else(|| "a port".to_string());
                self.session = Some(session);
                self.push_log(format!("New game. Docked at {place}."));
            }
            Err(err) => {
                self.session = None;
                self.smoke_ok = false;
                self.push_log(err.to_string());
            }
        }
        self.refresh();
    }

    fn run_smoke(&mut self) {
        let chart = self.chart_now();
        if chart.as_ref().is_none_or(|chart| {
            chart.ports.len() != 4
                || !chart
                    .lanes
                    .iter()
                    .any(|lane| lane.suitability == LaneSuitability::Warning)
        }) {
            self.smoke_ok = false;
            self.push_log("Smoke: expected four ports and a warning lane.".to_string());
        }
        self.perform(Action::Buy("grain".into()));
        self.perform(Action::Sail("al_manar".into()));
        for _ in 0..12 {
            if self.docked_id() == Some("al_manar") {
                break;
            }
            self.perform(Action::NextDay);
        }
        if self.docked_id() != Some("al_manar") {
            self.smoke_ok = false;
            let duel = self
                .session
                .as_ref()
                .is_some_and(|session| session.world().pending_duel.is_some());
            self.push_log(format!(
                "Smoke: did not dock at Al-Manar{}.",
                if duel { " (pending duel)" } else { "" }
            ));
        }
        self.market_open = true;
        if self.held("grain") > 0 {
            self.perform(Action::Sell("grain".into()));
        }
        self.perform(Action::Buy("spice".into()));
        self.perform(Action::Sell("spice".into()));
        let chart = self.chart_now();
        if chart.as_ref().is_none_or(|chart| {
            !chart
                .lanes
                .iter()
                .any(|lane| lane.suitability == LaneSuitability::Blocked)
        }) {
            self.smoke_ok = false;
            self.push_log("Smoke: Al-Manar did not list a blocked lane.".to_string());
        }
        self.refresh();
    }

    /// One chart frame for the art gate: docked sloop at Porto Novo, and a
    /// cutter bought through Session, one day along the Grain Road at facing
    /// f7 with its own wake.
    fn run_art(&mut self) {
        self.art = true;
        let bought = {
            let Some(session) = self.session.as_mut() else {
                self.smoke_ok = false;
                self.push_log("Art: no session.".to_string());
                self.refresh();
                return;
            };
            session.buy_ship("swift_cutter")
        };
        if let Err(err) = bought {
            self.smoke_ok = false;
            self.push_log(format!("Art: could not buy a cutter: {err}"));
        }
        self.perform(Action::Sail("al_manar".into()));
        self.perform(Action::NextDay);
        let chart = self.chart_now();
        let ok = chart.as_ref().is_some_and(|chart| {
            chart.ship.facing == Facing::F7
                && !chart.ship.docked
                && chart.ship.class_name == "cutter"
                && chart.ship.asset_id == "ship_cutter_f7"
                && chart.ship.wake_id == "ship_cutter_wake"
                && chart.gallery.len() == 1
                && chart.gallery[0].docked
                && chart.gallery[0].asset_id == "ship_sloop_f1"
                && chart.gallery[0].class_name == "sloop"
        });
        if ok {
            self.push_log(
                "Art check: docked sloop at Porto Novo, sailing cutter at f7 with wake, on the approved chart water."
                    .to_string(),
            );
        } else {
            self.smoke_ok = false;
            self.push_log("Art: expected a docked f1 sloop and a sailing f7 cutter.".to_string());
        }
        self.refresh();
    }

    /// Seed 42, the `duel_block` script: sail until a pirate freezes the day.
    fn run_encounter(&mut self) {
        if let Some(name) = self.sail_until_duel() {
            self.push_log(format!(
                "Duel with {name}. Next day calls Session::advance, which ticks reputation and does not move the day while this duel is pending. Pick at least 3 stances, or auto-resolve."
            ));
        }
        self.refresh();
    }

    /// Docked at the start port. `Session::work` pays 3 to 5 silver.
    /// Markets, provisions, wages, and reputation do not tick. The captain's
    /// day is copied onto the world, which is how the sim records the day of work.
    fn run_work(&mut self) {
        let before = match self.session.as_ref() {
            Some(session) => {
                let world = session.world();
                (world.captain.silver, world.day, world.captain.provisions)
            }
            None => {
                self.smoke_ok = false;
                self.push_log("Work: no session.".to_string());
                self.refresh();
                return;
            }
        };
        if self.docked_id().is_none() {
            self.smoke_ok = false;
            self.push_log("Work: expected to be docked.".to_string());
        }
        self.work_docks();
        let after = self.session.as_ref().map(|session| {
            let world = session.world();
            (world.captain.silver, world.day, world.captain.provisions)
        });
        let (silver, day, provisions) = before;
        match after {
            Some((next_silver, next_day, next_provisions))
                if (3..=5).contains(&(next_silver - silver))
                    && next_day == day + 1
                    && next_provisions == provisions => {}
            _ => {
                self.smoke_ok = false;
                self.push_log(
                    "Work: expected 3 to 5 silver, the day copied forward, and provisions unchanged."
                        .to_string(),
                );
            }
        }
        self.refresh();
    }

    /// Same voyage as `--encounter`, then `Session::duel` or `resolve_pending_duel`.
    fn run_duel_resolution(&mut self, auto: bool) {
        if self.sail_until_duel().is_none() {
            self.refresh();
            return;
        }
        let day_before = self.session.as_ref().map(|session| session.world().day);
        let standing_before = self
            .session
            .as_ref()
            .map(|session| format!("{:?}", session.world().captain.standing));
        if auto {
            self.auto_resolve();
        } else {
            self.stances = vec!["thrust".into(), "slash".into(), "parry".into()];
            self.fight_duel();
        }
        let pending = self
            .session
            .as_ref()
            .is_some_and(|session| session.world().pending_duel.is_some());
        if pending {
            self.smoke_ok = false;
            self.push_log("Duel: the challenge is still pending.".to_string());
            self.refresh();
            return;
        }
        let standing_after = self
            .session
            .as_ref()
            .map(|session| format!("{:?}", session.world().captain.standing));
        if standing_before != standing_after {
            self.smoke_ok = false;
            self.push_log(
                "Duel: standing changed. standing_delta is information only.".to_string(),
            );
        }
        self.next_day();
        let day_after = self.session.as_ref().map(|session| session.world().day);
        if day_after <= day_before {
            self.smoke_ok = false;
            self.push_log(
                "Duel: the day did not move after the challenge was cleared.".to_string(),
            );
        }
        self.refresh();
    }

    /// Buy grain, sail Silva Bay, advance until `pending_duel` is set.
    fn sail_until_duel(&mut self) -> Option<String> {
        self.log_lines.clear();
        self.market_open = false;
        self.armed_sail = None;
        self.stances.clear();
        let session = match Session::new("Ada", "merchant", 42, None) {
            Ok(session) => session,
            Err(err) => {
                self.smoke_ok = false;
                self.push_log(err.to_string());
                return None;
            }
        };
        let mut session = session;
        if let Err(err) = session.buy("grain", 8) {
            self.smoke_ok = false;
            self.push_log(err.to_string());
        }
        if let Err(err) = session.depart("silva_bay") {
            self.smoke_ok = false;
            self.push_log(err.to_string());
        }
        for _ in 0..6 {
            if session.world().pending_duel.is_some() {
                break;
            }
            if let Err(err) = session.advance() {
                self.smoke_ok = false;
                self.push_log(err.to_string());
                break;
            }
        }
        let pending = session
            .world()
            .pending_duel
            .as_ref()
            .map(|duel| duel.captain_name.clone());
        self.session = Some(session);
        if pending.is_none() {
            self.smoke_ok = false;
            self.push_log("Encounter preview: no pending duel.".to_string());
        }
        pending
    }

    fn perform(&mut self, action: Action) {
        match action {
            Action::NewGame => self.start_game(),
            Action::NextDay => self.next_day(),
            Action::Work => self.work_docks(),
            Action::HireSailor => self.hire_sailor(),
            Action::Provision => self.buy_provisions(),
            Action::Stance(stance) => self.push_stance(stance),
            Action::ClearStances => {
                self.stances.clear();
                self.refresh();
            }
            Action::Duel => self.fight_duel(),
            Action::AutoResolve => self.auto_resolve(),
            Action::ToggleMarket => {
                if self.docked_id().is_some() {
                    self.market_open = !self.market_open;
                }
                self.refresh();
            }
            Action::Sail(dest) => self.sail(&dest),
            Action::Buy(good) => self.trade(true, &good),
            Action::Sell(good) => self.trade(false, &good),
            Action::EncounterChoice(choice) => self.choose_encounter(&choice),
            Action::Naval(action) => self.play_naval(&action),
            Action::Board => self.resolve_board(),
            Action::Combat(action) => self.play_fight(&action),
            Action::Spare => self.spare_enemy(),
            Action::Capture => self.capture_prize(),
            Action::TakeAll => self.take_prize(),
            Action::CaptureCrew(delta) => {
                self.capture_crew = 0.max(self.capture_crew + delta);
                self.refresh();
            }
            Action::LeaveEncounter => self.leave_encounter(),
        }
    }

    fn sail(&mut self, dest: &str) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.depart(dest).map(|_| {
                session
                    .world()
                    .port(dest)
                    .map(|port| port.name.clone())
                    .unwrap_or_else(|| dest.to_string())
            })
        };
        match result {
            Ok(name) => self.push_log(format!("Departed for {name}.")),
            Err(err) => self.push_log(err.to_string()),
        }
        self.refresh();
    }

    fn on_port_pressed(&mut self, port_id: &str) {
        let decision = {
            let Some(session) = self.session.as_ref() else {
                return;
            };
            press_port(session, port_id)
        };
        match decision {
            PortPress::AtSea => {
                self.push_log("At sea. The chart does not change course.".to_string());
            }
            PortPress::NoLane => {
                self.armed_sail = None;
                self.push_log("No lane from this port. Nothing was sent to the sim.".to_string());
            }
            PortPress::OpenHere => {
                self.armed_sail = None;
                self.market_open = true;
            }
            PortPress::Depart(id) => {
                if self.armed_sail.as_deref() == Some(id.as_str()) {
                    self.armed_sail = None;
                    self.sail(&id);
                    return;
                }
                self.armed_sail = Some(id.clone());
                let text = self
                    .chart_now()
                    .and_then(|chart| {
                        chart
                            .lanes
                            .iter()
                            .find(|lane| lane.destination_id == id)
                            .map(lane_inspect)
                    })
                    .unwrap_or(id);
                self.push_log(format!("Selected {text}. Click the port again to sail."));
            }
        }
        self.refresh();
    }

    fn next_day(&mut self) {
        let mut failed = false;
        let screen_open = self.encounter.is_some();
        let (day_before, sailed) = self
            .session
            .as_ref()
            .map(|session| (session.world().day, at_sea(session)))
            .unwrap_or((0, false));
        let notes = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            match session.advance() {
                Err(err) => {
                    failed = true;
                    vec![err.to_string()]
                }
                Ok(turn) => {
                    let mut notes: Vec<String> = turn
                        .events
                        .iter()
                        .map(|event| event.message.clone())
                        .chain(turn.shocks.iter().cloned())
                        .filter(|line| !line.is_empty())
                        .collect();
                    if session.world().voyage.status == VoyageStatus::InPort {
                        let place = docked_name(session).unwrap_or_else(|| "port".to_string());
                        notes.push(format!("Docked at {place}."));
                    } else if session.world().voyage.status == VoyageStatus::AtSea {
                        notes.push(format!(
                            "At sea. Progress {}/{}.",
                            session.world().voyage.progress,
                            session.world().voyage.distance
                        ));
                    }
                    notes
                }
            }
        };
        // The CLI and the TUI call this after a sea day. It is not part of
        // `advance`. A frozen pending duel does not move the day, so it is
        // not asked again. An encounter screen that is already up is left
        // alone.
        let mut opened = None;
        if !failed && !screen_open && sailed {
            if let Some(session) = self.session.as_mut() {
                if session.world().day != day_before {
                    opened = sea_agency(session);
                }
            }
        }
        if failed && self.smoke {
            self.smoke_ok = false;
        }
        for note in notes {
            self.push_log(note);
        }
        if let Some((state, log)) = opened {
            if !log.is_empty() {
                self.push_log(log.clone());
            }
            self.open_agency(state, log);
        }
        self.refresh();
    }

    fn trade(&mut self, buy: bool, good: &str) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            if buy {
                match session.buy(good, 1) {
                    Ok(receipt) => Ok(vec![receipt_line(&receipt)]),
                    Err(err) => Err(err),
                }
            } else {
                match session.sell(good, 1) {
                    Ok(sale) => Ok(sale_lines(&sale)),
                    Err(err) => Err(err),
                }
            }
        };
        match result {
            Ok(lines) => {
                for line in lines {
                    self.push_log(line);
                }
            }
            Err(err) => {
                if self.smoke {
                    self.smoke_ok = false;
                }
                self.push_log(err.to_string());
            }
        }
        self.refresh();
    }

    fn work_docks(&mut self) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.work()
        };
        match result {
            Ok(earned) => self.push_log(format!("Worked the docks for {earned} silver.")),
            Err(err) => {
                if self.smoke {
                    self.smoke_ok = false;
                }
                self.push_log(err.to_string());
            }
        }
        self.refresh();
    }

    fn hire_sailor(&mut self) {
        let before = self
            .session
            .as_ref()
            .map(|session| session.world().captain.silver);
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.hire_crew(1, "sailor")
        };
        match result {
            Ok(()) => {
                let (silver, crew) = self
                    .session
                    .as_ref()
                    .map(|session| {
                        let world = session.world();
                        let crew = world
                            .captain
                            .ship
                            .as_ref()
                            .map(|ship| ship.crew)
                            .unwrap_or(0);
                        (world.captain.silver, crew)
                    })
                    .unwrap_or((0, 0));
                self.push_log(format!(
                    "Hired 1 sailor. Crew {crew}. Silver {} → {silver}.",
                    before.unwrap_or(silver)
                ));
            }
            Err(err) => {
                if self.smoke {
                    self.smoke_ok = false;
                }
                self.push_log(err.to_string());
            }
        }
        self.refresh();
    }

    fn buy_provisions(&mut self) {
        let before = self
            .session
            .as_ref()
            .map(|session| session.world().captain.provisions);
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.provision(5)
        };
        match result {
            Ok(()) => {
                let (silver, days) = self
                    .session
                    .as_ref()
                    .map(|session| {
                        (
                            session.world().captain.silver,
                            session.world().captain.provisions,
                        )
                    })
                    .unwrap_or((0, 0));
                self.push_log(format!(
                    "Bought provisions. Days {} → {days}. Silver {silver}.",
                    before.unwrap_or(days)
                ));
            }
            Err(err) => {
                if self.smoke {
                    self.smoke_ok = false;
                }
                self.push_log(err.to_string());
            }
        }
        self.refresh();
    }

    fn push_stance(&mut self, stance: Stance) {
        if self
            .session
            .as_ref()
            .is_none_or(|session| session.world().pending_duel.is_none())
        {
            return;
        }
        self.stances.push(stance.as_str().to_string());
        self.refresh();
    }

    fn fight_duel(&mut self) {
        let stances = self.stances.clone();
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.duel(&stances)
        };
        self.finish_duel_result(result);
    }

    fn auto_resolve(&mut self) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.resolve_pending_duel()
        };
        self.finish_duel_result(result);
    }

    /// `standing_delta` is reported and not applied. `Session` already leaves
    /// reputation alone; the view does not write it either.
    fn finish_duel_result(&mut self, result: Result<DuelOutcome, portlight_sim::SimError>) {
        match result {
            Ok(outcome) => {
                self.stances.clear();
                self.push_log(duel_outcome_line(&outcome));
            }
            Err(err) => {
                if self.smoke {
                    self.smoke_ok = false;
                }
                self.push_log(err.to_string());
            }
        }
        self.refresh();
    }

    fn refresh(&mut self) {
        let docked = self.docked_id().is_some();
        let pending = self
            .session
            .as_ref()
            .is_some_and(|session| session.world().pending_duel.is_some());
        if !docked {
            self.market_open = false;
        }
        if let Some(button) = self.work_button.as_mut() {
            button.set_disabled(!docked);
        }
        if let Some(button) = self.market_button.as_mut() {
            button.set_disabled(!docked);
            button.set_text(if self.market_open {
                "Hide market"
            } else {
                "Market"
            });
        }
        if let Some(scroll) = self.market_scroll.as_mut() {
            scroll.set_visible(self.market_open);
        }
        let services = self.port_services_text();
        let encounter = self.encounter_text();
        let stance_line = self.stance_line();
        let screen_open = self
            .encounter
            .as_ref()
            .is_some_and(|facts| present(facts).is_some());
        let can_duel = duel_button_enabled(pending);
        let show_stance = stance_duel_visible(screen_open, pending);
        if let Some(row) = self.port_row.as_mut() {
            row.set_visible(docked);
        }
        if let Some(label) = self.port_note.as_mut() {
            label.set_visible(docked);
            label.set_text(&services);
        }
        if let Some(box_node) = self.encounter_box.as_mut() {
            box_node.set_visible(show_stance);
        }
        if let Some(label) = self.encounter_label.as_mut() {
            label.set_text(&encounter);
        }
        if let Some(label) = self.stance_label.as_mut() {
            label.set_text(&stance_line);
        }
        if let Some(button) = self.duel_button.as_mut() {
            button.set_disabled(!can_duel);
        }
        let status_text = self.status_text();
        if let Some(label) = self.status.as_mut() {
            label.set_text(&status_text);
        }
        if let Some(label) = self.log_label.as_mut() {
            label.set_text(&self.log_lines.join("\n"));
        }
        self.rebuild_lanes();
        self.rebuild_market();
        if let Some(chart) = self.chart_now() {
            if let Some(mut canvas) = self.canvas.clone() {
                canvas.bind_mut().show(chart, self.smoke);
            }
        }
        self.sync_encounter_screen();
    }

    fn rebuild_lanes(&mut self) {
        let Some(mut box_node) = self.lane_box.clone() else {
            return;
        };
        clear_children(&mut box_node);
        let Some(chart) = self.chart_now() else {
            return;
        };
        if chart.lanes.is_empty() {
            box_node.add_child(&body_label("At sea. Advance the day to sail.", 14, CREAM));
            return;
        }
        let game_id = self.instance_id();
        for lane in &chart.lanes {
            let mut block = VBoxContainer::new_alloc();
            let mut row = HBoxContainer::new_alloc();
            let mut label = body_label(
                &format!(
                    "{}   {}d   {}   {}",
                    lane.destination_name, lane.estimated_days, lane.distance, lane.min_ship_class
                ),
                14,
                Color::from_rgba8(lane.color.r, lane.color.g, lane.color.b, 255),
            );
            shrink_label(&mut label);
            row.add_child(&label);
            let mut sail =
                action_button("Sail", game_id, Action::Sail(lane.destination_id.clone()));
            sail.set_h_size_flags(SizeFlags::SHRINK_END);
            row.add_child(&sail);
            block.add_child(&row);
            if let Some(note) = &lane.suitability_note {
                let mut note_label = body_label(note, 12, MUTED);
                note_label.set_autowrap_mode(AutowrapMode::WORD_SMART);
                block.add_child(&note_label);
            }
            box_node.add_child(&block);
        }
    }

    fn rebuild_market(&mut self) {
        let Some(mut box_node) = self.market_box.clone() else {
            return;
        };
        clear_children(&mut box_node);
        if !self.market_open {
            return;
        }
        let Some((port_name, rows)) = self.market_rows() else {
            return;
        };
        box_node.add_child(&body_label(&format!("Market at {port_name}"), 15, GOLD));
        let game_id = self.instance_id();
        for row in rows {
            let MarketRow {
                id,
                name,
                stock,
                buy,
                sell,
                held,
            } = row;
            let mut row = HBoxContainer::new_alloc();
            let mut label = body_label(
                &format!("{name}  buy {buy}  sell {sell}  stock {stock}  held {held}"),
                13,
                CREAM,
            );
            shrink_label(&mut label);
            row.add_child(&label);
            let mut buy = action_button("Buy", game_id, Action::Buy(id.clone()));
            buy.set_h_size_flags(SizeFlags::SHRINK_END);
            let mut sell = action_button("Sell", game_id, Action::Sell(id));
            sell.set_h_size_flags(SizeFlags::SHRINK_END);
            row.add_child(&buy);
            row.add_child(&sell);
            box_node.add_child(&row);
        }
    }

    fn market_rows(&self) -> Option<(String, Vec<MarketRow>)> {
        let session = self.session.as_ref()?;
        let port_id = docked_port_id(session)?;
        let world = session.world();
        let port = world.port(port_id)?;
        let port_name = port.name.clone();
        let slots: Vec<(String, i64, i64, i64)> = port
            .market
            .iter()
            .map(|slot| {
                (
                    slot.good_id.clone(),
                    slot.stock_current,
                    slot.buy_price,
                    slot.sell_price,
                )
            })
            .collect();
        let rows = slots
            .into_iter()
            .map(|(id, stock, buy, sell)| {
                let name = good_name(&id);
                let held = cargo_held(&world.captain.cargo, &id);
                MarketRow {
                    id,
                    name,
                    stock,
                    buy,
                    sell,
                    held,
                }
            })
            .collect();
        Some((port_name, rows))
    }

    fn status_text(&self) -> String {
        let Some(session) = self.session.as_ref() else {
            return "No game".to_string();
        };
        let world = session.world();
        let ship = world.captain.ship.as_ref();
        let place = match world.voyage.status {
            VoyageStatus::AtSea => format!(
                "At sea  {} -> {}  {}/{}",
                port_name(world, &world.voyage.origin_id),
                port_name(world, &world.voyage.destination_id),
                world.voyage.progress,
                world.voyage.distance
            ),
            VoyageStatus::Arrived => {
                format!(
                    "Arrived at {}",
                    port_name(world, &world.voyage.destination_id)
                )
            }
            VoyageStatus::InPort => {
                format!(
                    "Docked at {}",
                    port_name(world, &world.voyage.destination_id)
                )
            }
        };
        let ship_line = ship
            .map(|ship| {
                let class = content::content()
                    .ship(&ship.template_id)
                    .map(|template| template.ship_class.as_str())
                    .unwrap_or(ship.template_id.as_str());
                format!(
                    "{}   {class}   hull {}/{}   crew {}",
                    ship.name, ship.hull, ship.hull_max, ship.crew
                )
            })
            .unwrap_or_default();
        let ledger = ledger_line(session);
        let paths = victory_line(session);
        format!(
            "Day {}   {}   {}   {} silver   {} provisions\n{place}\n{ship_line}\n{ledger}\n{paths}",
            world.day,
            content::season_name(world.day),
            world.captain.name,
            world.captain.silver,
            world.captain.provisions
        )
    }

    fn chart_now(&self) -> Option<ChartModel> {
        let session = self.session.as_ref()?;
        let mut chart = project_chart(session);
        if self.art {
            if let Some(port) = session.world().port("porto_novo") {
                chart
                    .gallery
                    .push(docked_sloop_marker(port.map_x, port.map_y));
            }
            // 1.0 when both plates fit in the chart area of the 1280×720
            // window, otherwise 0.72. Play still follows the ship.
            chart.frame = art_frame(&chart, chart_host_width(), WINDOW_H);
        }
        Some(chart)
    }

    fn docked_id(&self) -> Option<&str> {
        self.session.as_ref().and_then(docked_port_id)
    }

    fn held(&self, good: &str) -> i64 {
        self.session
            .as_ref()
            .map(|session| cargo_held(&session.world().captain.cargo, good))
            .unwrap_or(0)
    }

    /// The Xvfb chart capture opens the market. Every lane and market label
    /// must have text and a real line height. Autowrap plus clip-text used to
    /// collapse these to 1×1, which the flat-frame check does not see.
    fn assert_panel_labels(&mut self) {
        let lanes = self
            .lane_box
            .as_ref()
            .map(labels_under_box)
            .unwrap_or_default();
        let market = self
            .market_box
            .as_ref()
            .map(labels_under_box)
            .unwrap_or_default();
        let lanes_ok = self.labels_have_a_line("lane", &lanes);
        let market_ok = self.labels_have_a_line("market", &market);
        let text_ok = self.labels_name_the_rows(&lanes, &market);
        if !lanes_ok || !market_ok || !text_ok {
            self.smoke_ok = false;
        }
    }

    fn labels_have_a_line(&mut self, kind: &str, labels: &[Gd<Label>]) -> bool {
        if labels.is_empty() {
            let line = format!("Smoke: no {kind} labels.");
            godot_print!("{line}");
            self.push_log(line);
            return false;
        }
        let mut ok = true;
        for label in labels {
            let text = label.get_text().to_string();
            let lines = label.get_line_count();
            let height = label.get_size().y;
            let line_h = label.get_line_height();
            if text.trim().is_empty() || lines < 1 || line_h <= 0 || height < line_h as f32 {
                ok = false;
                let line = format!(
                    "Smoke: {kind} label {text:?} lines={lines} height={height} line_h={line_h}"
                );
                godot_print!("{line}");
                self.push_log(line);
            }
        }
        godot_print!("panel {kind} labels {} ok={ok}", labels.len());
        ok
    }

    fn labels_name_the_rows(&mut self, lanes: &[Gd<Label>], market: &[Gd<Label>]) -> bool {
        let mut ok = true;
        if let Some(chart) = self.chart_now() {
            for lane in &chart.lanes {
                let days = format!("{}d", lane.estimated_days);
                let shown = lanes.iter().any(|label| {
                    let text = label.get_text().to_string();
                    text.contains(&lane.destination_name) && text.contains(&days)
                });
                if !shown {
                    ok = false;
                    let line = format!(
                        "Smoke: lane {} missing destination or days.",
                        lane.destination_name
                    );
                    godot_print!("{line}");
                    self.push_log(line);
                }
                if let Some(note) = &lane.suitability_note {
                    let shown = lanes
                        .iter()
                        .any(|label| label.get_text().to_string().contains(note.as_str()));
                    if !shown {
                        ok = false;
                        let line =
                            format!("Smoke: lane {} missing lock reason.", lane.destination_name);
                        godot_print!("{line}");
                        self.push_log(line);
                    }
                }
            }
        }
        if let Some((_, rows)) = self.market_rows() {
            for row in rows {
                let price = format!("buy {}", row.buy);
                let stock = format!("stock {}", row.stock);
                let shown = market.iter().any(|label| {
                    let text = label.get_text().to_string();
                    text.contains(&row.name) && text.contains(&price) && text.contains(&stock)
                });
                if !shown {
                    ok = false;
                    let line = format!("Smoke: market {} missing good, price, or stock.", row.name);
                    godot_print!("{line}");
                    self.push_log(line);
                }
            }
        } else {
            ok = false;
            godot_print!("Smoke: market rows missing.");
            self.push_log("Smoke: market rows missing.".to_string());
        }
        ok
    }

    fn push_log(&mut self, line: impl Into<String>) {
        self.log_lines.push(line.into());
        if self.log_lines.len() > 8 {
            let extra = self.log_lines.len() - 8;
            self.log_lines.drain(0..extra);
        }
        if let Some(label) = self.log_label.as_mut() {
            label.set_text(&self.log_lines.join("\n"));
        }
    }

    /// Saves the window. An empty image or a frame that is mostly one colour
    /// is a failed capture. Headless Godot cannot produce this image; call
    /// this only from a real GL context (`PORTLIGHT_SHOT` set).
    fn save_shot(&self, path: &str, encounter: bool) -> bool {
        let image = self.base().get_viewport().and_then(|viewport| {
            viewport
                .get_texture()
                .and_then(|texture| texture.get_image())
        });
        let Some(image) = image else {
            godot_print!("viewport image was empty");
            return false;
        };
        if image.is_empty() {
            godot_print!("viewport image was empty");
            return false;
        }
        let width = image.get_width();
        let height = image.get_height();
        let err = image.save_png(path);
        let samples = frame_samples(&image);
        let flat = frame_mostly_flat(&samples);
        // The encounter ground is ink. Chart and harbour shots keep the 80% rule.
        // The mode is the capture, not the filename: `/tmp/galleon1.png` is
        // still an encounter frame.
        let rejected = if encounter {
            encounter_frame_rejected(width, height, WINDOW_W as i32, WINDOW_H as i32, &samples)
        } else {
            capture_frame_rejected(width, height, WINDOW_W as i32, WINDOW_H as i32, &samples)
        };
        godot_print!(
            "screenshot {path} {width}x{height} samples={} flat={flat} error={err:?}",
            samples.len()
        );
        if err != Error::OK {
            godot_print!("screenshot save failed");
            return false;
        }
        if rejected {
            godot_print!(
                "screenshot rejected: expected a full 1280x720 frame that is not one flat colour"
            );
            return false;
        }
        true
    }

    fn open_agency(&mut self, state: EncounterState, log: String) {
        let (ship, sailing) = self
            .session
            .as_ref()
            .map(|session| (player_ship(session), at_sea(session)))
            .unwrap_or((None, false));
        let mut facts = facts_from_agency(&state, ship, &log, sailing);
        if facts.phase == "naval" {
            facts.naval_actions = self.probe_naval();
        }
        self.scripted_captain = None;
        self.encounter = Some(facts);
    }

    fn open_scripted_approach(&mut self) {
        self.open_named_approach(SCRIPTED_CAPTAIN);
    }

    fn open_named_approach(&mut self, captain_id: &str) {
        let (ship, sailing) = self
            .session
            .as_ref()
            .map(|session| (player_ship(session), at_sea(session)))
            .unwrap_or((None, true));
        match facts_for_catalog_captain(captain_id, ship, sailing) {
            Some(facts) => {
                self.scripted_captain = Some(captain_id.to_string());
                self.encounter = Some(facts);
            }
            None => {
                self.smoke_ok = false;
                self.push_log(format!("Encounter: unknown captain {captain_id}."));
            }
        }
    }

    /// Capture only. The plate is forced to `template_id`, so the card uses
    /// that template's new-ship hull and crew (`Ship::from_template`) instead
    /// of the scripted cutter.
    fn apply_template_ship_card(&mut self, template_id: &str) {
        let Some(ship) = template_player_ship(template_id) else {
            self.smoke_ok = false;
            self.push_log(format!("Encounter: unknown ship {template_id}."));
            return;
        };
        let Some(facts) = self.encounter.as_mut() else {
            return;
        };
        facts.player_hull = ship.hull;
        facts.player_hull_max = Some(ship.hull_max);
        facts.player_crew = ship.crew;
    }

    fn choose_encounter(&mut self, choice: &str) {
        let locked = self.scripted_captain.clone();
        let on_session = self
            .encounter
            .as_ref()
            .is_some_and(|facts| facts.on_session);
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            if on_session {
                session.encounter_choice(choice)
            } else {
                session.encounter_choice_with(choice, locked.as_deref(), None)
            }
        };
        if result.is_ok() {
            self.scripted_captain = None;
        }
        self.ingest(result);
    }

    fn play_naval(&mut self, action: &str) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.naval_round(action)
        };
        self.ingest(result);
    }

    fn resolve_board(&mut self) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.resolve_boarding()
        };
        self.ingest(result);
    }

    fn play_fight(&mut self, action: &str) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.fight(action)
        };
        self.ingest(result);
    }

    fn spare_enemy(&mut self) {
        self.finish_victory(true);
    }

    fn take_prize(&mut self) {
        self.finish_victory(false);
    }

    fn finish_victory(&mut self, spare: bool) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            if spare {
                session.spare()
            } else {
                session.take_all()
            }
        };
        match result {
            Ok(()) => {
                if let Some(facts) = self.encounter.as_mut() {
                    facts.pending_victory = false;
                    facts.phase = "resolved".to_string();
                    facts.on_session = false;
                    facts.kind = "resolved".to_string();
                }
            }
            Err(err) => self.note_session_error(err),
        }
        self.refresh();
    }

    fn capture_prize(&mut self) {
        let crew = self.capture_crew;
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.capture(crew)
        };
        self.ingest(result);
    }

    fn leave_encounter(&mut self) {
        self.encounter = None;
        self.scripted_captain = None;
        self.refresh();
    }

    /// `Session::hunt_bounty` opens the encounter. The screen only reads it.
    fn open_bounty_hunt(&mut self, target_id: &str) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.hunt_bounty(target_id)
        };
        match result {
            Ok(state) => self.open_agency(state, String::new()),
            Err(err) => self.note_session_error(err),
        }
    }

    fn ingest(&mut self, result: Result<EncounterStep, SimError>) {
        match result {
            Ok(step) => {
                let text = session_text(&step.message, &step.flavor);
                if !text.is_empty() {
                    self.push_log(text);
                }
                self.adopt_step(step);
            }
            Err(err) => self.note_session_error(err),
        }
        self.refresh();
    }

    fn note_session_error(&mut self, err: SimError) {
        if self.smoke {
            self.smoke_ok = false;
        }
        let text = err.to_string();
        self.push_log(text.clone());
        if let Some(facts) = self.encounter.as_mut() {
            facts.log = text;
        }
    }

    fn adopt_step(&mut self, step: EncounterStep) {
        let previous_faction = self
            .encounter
            .as_ref()
            .map(|facts| facts.faction_id.clone())
            .unwrap_or_default();
        let previous_max = self
            .encounter
            .as_ref()
            .and_then(|facts| facts.enemy_hull_max);
        let (pending, ship, sailing, naval_actions, combat_actions) = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            let pending = session.pending_victory();
            let ship = player_ship(session);
            let sailing = at_sea(session);
            let naval_actions = if step.phase == "naval" {
                match session.naval_round("") {
                    Err(err) => action_list_from_error(&err).unwrap_or_default(),
                    Ok(_) => Vec::new(),
                }
            } else {
                Vec::new()
            };
            let combat_actions = if step.phase == "duel" {
                match session.fight("") {
                    Err(err) => action_list_from_error(&err).unwrap_or_default(),
                    Ok(_) => Vec::new(),
                }
            } else {
                Vec::new()
            };
            (pending, ship, sailing, naval_actions, combat_actions)
        };
        self.encounter = Some(facts_from_step(StepInput {
            step: &step,
            previous_faction_id: &previous_faction,
            previous_enemy_hull_max: previous_max,
            pending_victory: pending,
            ship,
            at_sea: sailing,
            naval_actions: &naval_actions,
            combat_actions: &combat_actions,
        }));
    }

    fn probe_naval(&mut self) -> Vec<String> {
        let Some(session) = self.session.as_mut() else {
            return Vec::new();
        };
        match session.naval_round("") {
            Err(err) => action_list_from_error(&err).unwrap_or_default(),
            Ok(_) => Vec::new(),
        }
    }

    fn sync_encounter_screen(&mut self) {
        let game_id = self.instance_id();
        let crew_count = self.capture_crew;
        let template_id = if self.galleon_frame {
            "royal_man_of_war".to_string()
        } else {
            self.session
                .as_ref()
                .and_then(|session| session.world().captain.ship.as_ref())
                .map(|ship| ship.template_id.clone())
                .unwrap_or_default()
        };
        let view = self.encounter.as_ref().and_then(present);
        let Some(nodes) = self.encounter_nodes.as_mut() else {
            return;
        };
        set_ship_plate(
            &mut nodes.plate,
            &mut nodes.plate_panel,
            &mut nodes.placeholder,
            &mut nodes.plate_caption,
            &template_id,
        );
        let open = view.is_some();
        nodes.root.set_visible(open);
        nodes.root.set_mouse_filter(if open {
            godot::classes::control::MouseFilter::STOP
        } else {
            godot::classes::control::MouseFilter::IGNORE
        });
        let Some(view) = view else {
            return;
        };
        nodes.title.set_text(view.title);
        nodes.card.set_text(&view.card);
        nodes.log.set_text(&view.log);
        let show_crew = view
            .actions
            .iter()
            .any(|action| matches!(action, ScreenAction::Capture));
        nodes.crew.set_visible(show_crew);
        nodes
            .crew
            .set_text(&format!("Crew to the prize  {crew_count}"));
        let actions = view.actions.clone();
        let mut box_node = nodes.actions.clone();
        clear_children(&mut box_node);
        let mut row = HBoxContainer::new_alloc();
        row.add_theme_constant_override("separation", 8);
        let mut count = 0;
        for action in actions {
            if count == 4 {
                box_node.add_child(&row);
                row = HBoxContainer::new_alloc();
                row.add_theme_constant_override("separation", 8);
                count = 0;
            }
            let caption = action_caption(&action);
            let command = match action {
                ScreenAction::Choice(choice) => Action::EncounterChoice(choice.to_string()),
                ScreenAction::Naval(action) => Action::Naval(action),
                ScreenAction::Board => Action::Board,
                ScreenAction::Combat(action) => Action::Combat(action),
                ScreenAction::Spare => Action::Spare,
                ScreenAction::Capture => Action::Capture,
                ScreenAction::TakeAll => Action::TakeAll,
                ScreenAction::Return { .. } => Action::LeaveEncounter,
            };
            row.add_child(&encounter_button(&caption, game_id, command));
            count += 1;
        }
        if count > 0 {
            box_node.add_child(&row);
        }
        if show_crew {
            let mut crew_row = HBoxContainer::new_alloc();
            crew_row.add_theme_constant_override("separation", 8);
            crew_row.add_child(&encounter_button(
                "Crew −",
                game_id,
                Action::CaptureCrew(-1),
            ));
            crew_row.add_child(&encounter_button("Crew +", game_id, Action::CaptureCrew(1)));
            box_node.add_child(&crew_row);
        }
    }

    fn run_encounter_screen(&mut self) {
        self.prepare_scripted_voyage();
        self.open_scripted_approach();
        self.expect_phase(ScreenPhase::Approach, "approach");
        self.choose_encounter("fight");
        self.expect_phase(ScreenPhase::Naval, "naval");
        self.play_scripted_naval();
        self.expect_phase(ScreenPhase::Boarding, "boarding");
        self.resolve_board();
        self.expect_phase(ScreenPhase::Personal, "personal fight");
        self.play_scripted_fight();
        self.expect_phase(ScreenPhase::Outcome, "outcome");
        self.expect_outcome_actions();
        self.spare_enemy();
        self.expect_returned();
        self.leave_encounter();
        self.report_encounter_smoke();
    }

    fn begin_encounter_shots(&mut self) {
        self.prepare_scripted_voyage();
        self.open_scripted_approach();
        self.encounter_shot = Some(ShotPhase::Approach);
        self.refresh();
    }

    /// Saves the phase that is on screen, then advances the script.
    /// Returns true when another frame is needed before the process quits.
    fn advance_encounter_shot(&mut self) -> bool {
        let Some(phase) = self.encounter_shot else {
            return false;
        };
        let Some(dir) = self.encounter_shot_dir.clone() else {
            return false;
        };
        self.expect_phase(phase.screen(), phase.label());
        let path = format!("{dir}/{}", phase.file_name());
        if !self.save_shot(&path, true) {
            self.capture_failed = true;
        }
        match phase {
            ShotPhase::Approach => {
                self.choose_encounter("fight");
                self.encounter_shot = Some(ShotPhase::Naval);
            }
            ShotPhase::Naval => {
                self.play_scripted_naval();
                self.encounter_shot = Some(ShotPhase::Boarding);
            }
            ShotPhase::Boarding => {
                self.resolve_board();
                self.encounter_shot = Some(ShotPhase::Personal);
            }
            ShotPhase::Personal => {
                self.play_scripted_fight();
                self.encounter_shot = Some(ShotPhase::Outcome);
            }
            ShotPhase::Outcome => {
                self.expect_outcome_actions();
                self.spare_enemy();
                self.expect_returned();
                self.leave_encounter();
                self.report_encounter_smoke();
                self.encounter_shot = None;
                return false;
            }
        }
        self.capture_frames = 4;
        true
    }

    fn prepare_scripted_voyage(&mut self) {
        self.log_lines.clear();
        self.market_open = false;
        self.armed_sail = None;
        self.stances.clear();
        self.encounter = None;
        self.scripted_captain = None;
        match Session::new(SCRIPTED_NAME, SCRIPTED_CAPTAIN_TYPE, SCRIPTED_SEED, None) {
            Ok(mut session) => match session.depart(SCRIPTED_DEPART) {
                Ok(()) => {
                    let place = session
                        .world()
                        .port(SCRIPTED_DEPART)
                        .map(|port| port.name.clone())
                        .unwrap_or_else(|| SCRIPTED_DEPART.to_string());
                    self.session = Some(session);
                    self.push_log(format!("Departed for {place}."));
                }
                Err(err) => {
                    self.smoke_ok = false;
                    self.session = Some(session);
                    self.push_log(err.to_string());
                }
            },
            Err(err) => {
                self.smoke_ok = false;
                self.session = None;
                self.push_log(err.to_string());
            }
        }
    }

    fn play_scripted_naval(&mut self) {
        for action in SCRIPTED_NAVAL {
            if !self.smoke_ok {
                return;
            }
            if self.phase_is(ScreenPhase::Boarding) {
                return;
            }
            self.play_naval(action);
        }
    }

    fn play_scripted_fight(&mut self) {
        for action in SCRIPTED_FIGHT {
            if !self.smoke_ok {
                return;
            }
            if self.phase_is(ScreenPhase::Outcome) {
                return;
            }
            self.play_fight(action);
        }
    }

    fn phase_is(&self, phase: ScreenPhase) -> bool {
        self.encounter
            .as_ref()
            .and_then(present)
            .is_some_and(|view| view.phase == phase)
    }

    fn expect_phase(&mut self, phase: ScreenPhase, label: &str) {
        if self.phase_is(phase) {
            godot_print!("encounter phase {label}");
            return;
        }
        self.smoke_ok = false;
        let found = self
            .encounter
            .as_ref()
            .map(|facts| facts.phase.clone())
            .unwrap_or_else(|| "none".to_string());
        let line = format!("Encounter smoke: expected {label}, phase was {found}.");
        godot_print!("{line}");
        self.push_log(line);
    }

    fn expect_outcome_actions(&mut self) {
        let actions = self
            .encounter
            .as_ref()
            .and_then(present)
            .map(|view| view.actions)
            .unwrap_or_default();
        let ok = actions.contains(&ScreenAction::Spare)
            && actions.contains(&ScreenAction::Capture)
            && actions.contains(&ScreenAction::TakeAll);
        if !ok {
            self.smoke_ok = false;
            self.push_log(
                "Encounter smoke: outcome did not offer spare, capture, and take all.".to_string(),
            );
        }
        let pending = self
            .session
            .as_ref()
            .is_some_and(|session| session.pending_victory());
        if !pending {
            self.smoke_ok = false;
            self.push_log("Encounter smoke: pending_victory was not set.".to_string());
        }
    }

    fn expect_returned(&mut self) {
        let pending = self
            .session
            .as_ref()
            .is_some_and(|session| session.pending_victory());
        if pending {
            self.smoke_ok = false;
            self.push_log("Encounter smoke: spare left pending_victory set.".to_string());
        }
        if !self.phase_is(ScreenPhase::Outcome) {
            self.smoke_ok = false;
            self.push_log("Encounter smoke: spare did not leave the outcome card.".to_string());
        }
    }

    fn report_encounter_smoke(&mut self) {
        self.run_bounty_encounter();
        self.encounter_checked = true;
    }

    /// `parity/scripts/bounty_claim.txt` through the fight. After `take_all`,
    /// `pending_duel` is whatever `Session` left. The screen does not clear it.
    fn run_bounty_encounter(&mut self) {
        let prior_ok = self.smoke_ok;
        self.smoke_ok = true;
        self.log_lines.clear();
        self.encounter = None;
        self.scripted_captain = None;
        match Session::new(SCRIPTED_NAME, SCRIPTED_CAPTAIN_TYPE, SCRIPTED_SEED, None) {
            Ok(session) => self.session = Some(session),
            Err(err) => {
                self.smoke_ok = false;
                self.session = None;
                self.push_log(err.to_string());
                self.smoke_ok = prior_ok && self.smoke_ok;
                return;
            }
        }
        let accepted = {
            let Some(session) = self.session.as_mut() else {
                self.smoke_ok = prior_ok && false;
                return;
            };
            session.accept_bounty(SCRIPTED_CAPTAIN)
        };
        if let Err(err) = accepted {
            self.note_session_error(err);
            self.smoke_ok = prior_ok && self.smoke_ok;
            return;
        }
        self.open_bounty_hunt(SCRIPTED_CAPTAIN);
        self.expect_phase(ScreenPhase::Approach, "bounty approach");
        let pending = self
            .session
            .as_ref()
            .is_some_and(|session| session.world().pending_duel.is_some());
        if !pending {
            self.smoke_ok = false;
            self.push_log("Bounty smoke: hunt_bounty did not set pending_duel.".to_string());
        }
        self.choose_encounter("fight");
        self.expect_phase(ScreenPhase::Naval, "bounty naval");
        self.play_scripted_naval();
        self.expect_phase(ScreenPhase::Boarding, "bounty boarding");
        self.resolve_board();
        self.expect_phase(ScreenPhase::Personal, "bounty personal fight");
        self.play_scripted_fight();
        self.expect_phase(ScreenPhase::Outcome, "bounty outcome");
        let still_pending = self
            .session
            .as_ref()
            .is_some_and(|session| session.world().pending_duel.is_some());
        if !still_pending {
            self.smoke_ok = false;
            self.push_log(
                "Bounty smoke: pending_duel cleared before the encounter ended.".to_string(),
            );
        }
        self.take_prize();
        let cleared = self
            .session
            .as_ref()
            .is_some_and(|session| session.world().pending_duel.is_none());
        if !cleared {
            self.smoke_ok = false;
            self.push_log(
                "Bounty smoke: pending_duel was still set after the encounter.".to_string(),
            );
        }
        self.leave_encounter();
        let bounty_ok = self.smoke_ok;
        self.smoke_ok = prior_ok && bounty_ok;
        if bounty_ok {
            godot_print!("bounty pending duel cleared");
        }
    }
}

/// `tick_sea_captain_agency` after a sea day that moved.
/// Notices are Session text. `None` when no encounter opened.
fn sea_agency(session: &mut Session) -> Option<(EncounterState, String)> {
    let (encounter, _ambush, notices) = session.tick_sea_captain_agency();
    let state = encounter?;
    let log = notices
        .into_iter()
        .map(|(_, message)| message)
        .filter(|message| !message.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    Some((state, log))
}

fn cargo_held(cargo: &[portlight_sim::model::CargoItem], good: &str) -> i64 {
    cargo
        .iter()
        .filter(|item| item.good_id == good)
        .map(|item| item.quantity)
        .sum()
}

fn good_name(id: &str) -> String {
    content::content()
        .good(id)
        .map(|good| good.name.clone())
        .unwrap_or_else(|| id.to_string())
}

impl PortlightGame {
    fn encounter_text(&self) -> String {
        let Some(session) = self.session.as_ref() else {
            return String::new();
        };
        let Some(duel) = session.world().pending_duel.as_ref() else {
            return String::new();
        };
        format!(
            "Duel: {}.\nFaction {} · {} · strength {} · {}.\nNext day ticks reputation and does not move the day. Pick stances, or auto-resolve.",
            duel.captain_name, duel.faction_id, duel.personality, duel.strength, duel.region
        )
    }

    fn stance_line(&self) -> String {
        if self.stances.is_empty() {
            "Stances: none yet. Pick at least 3.".to_string()
        } else {
            format!("Stances: {}.", self.stances.join(", "))
        }
    }

    fn port_services_text(&self) -> String {
        let Some(session) = self.session.as_ref() else {
            return String::new();
        };
        let Some(id) = docked_port_id(session) else {
            return String::new();
        };
        let Some(port) = session.world().port(id) else {
            return String::new();
        };
        format!(
            "Sailor listed {} silver. Provisions listed {} silver a day.",
            port.crew_cost, port.provision_cost
        )
    }
}

fn receipt_line(receipt: &TradeReceipt) -> String {
    format!(
        "{} {} {} for {} silver.",
        receipt.action, receipt.quantity, receipt.good_id, receipt.total_price
    )
}

fn sale_lines(sale: &Sale) -> Vec<String> {
    let mut lines = vec![receipt_line(&sale.receipt)];
    for contract in &sale.contracts {
        if !contract.summary.is_empty() {
            lines.push(contract.summary.clone());
        }
    }
    lines
}

fn duel_outcome_line(outcome: &DuelOutcome) -> String {
    let result = if outcome.player_won {
        "Won"
    } else if outcome.draw {
        "Draw"
    } else {
        "Lost"
    };
    format!(
        "{result} the duel with {}. Silver {:+}. Standing {:+}, shown only.",
        outcome.opponent_name, outcome.silver_delta, outcome.standing_delta
    )
}

fn ledger_line(session: &Session) -> String {
    let books = session.books();
    format!(
        "Ledger: {} trades, net {} silver",
        books.trade_count, books.net_profit
    )
}

fn victory_line(session: &Session) -> String {
    let paths = session.victory();
    if paths.is_empty() {
        return String::new();
    }
    let names: Vec<&str> = paths.iter().map(|path| path.name.as_str()).collect();
    format!("Victory paths: {}", names.join(", "))
}

fn docs_capture() -> bool {
    flag_set("PORTLIGHT_ART_DOCS") || user_arg("--art-docs")
}

/// `/tmp/<file>`. Encounter captures use this. A docs path is only an
/// explicit `PORTLIGHT_SHOT`, which is applied before the default.
fn tmp_shot_path(file: &str) -> String {
    format!("/tmp/{file}")
}

/// `--art` writes `/tmp/<file>`. `PORTLIGHT_ART_DOCS` or `--art-docs` writes
/// `docs/screenshots/<file>`. `PORTLIGHT_SHOT` is applied first and wins.
/// Godot changes into the project directory, so a relative docs path is
/// taken from the repo root.
fn art_shot_path(file: &str) -> String {
    if docs_capture() {
        resolve_repo_path(&format!("docs/screenshots/{file}"))
    } else {
        tmp_shot_path(file)
    }
}

fn encounter_dir_set() -> bool {
    std::env::var("PORTLIGHT_ENCOUNTER_DIR")
        .ok()
        .is_some_and(|path| !path.is_empty())
}

/// The five encounter frames are written only when this is set. The `/tmp`
/// default is the directory those frames use, not a reason to capture.
fn encounter_frames_requested() -> bool {
    encounter_dir_set() || docs_capture()
}

/// Directory for the five encounter frames, once a capture was requested.
/// `PORTLIGHT_ENCOUNTER_DIR` wins. `--art-docs` or `PORTLIGHT_ART_DOCS`
/// writes `docs/screenshots`. Otherwise `/tmp`.
fn encounter_shot_dir() -> String {
    if let Some(dir) = std::env::var("PORTLIGHT_ENCOUNTER_DIR")
        .ok()
        .filter(|path| !path.is_empty())
    {
        return resolve_repo_path(&dir);
    }
    if docs_capture() {
        resolve_repo_path("docs/screenshots")
    } else {
        "/tmp".to_string()
    }
}

fn headless_runtime() -> bool {
    // `--headless` is an engine argument, so it is not in the user-arg list.
    // The dummy display server has no viewport texture. A capture there logs
    // `Parameter "t" is null` and must not run.
    let display = DisplayServer::singleton().get_name();
    if display == "headless" {
        return true;
    }
    let args = Os::singleton().get_cmdline_args();
    (0..args.len()).any(|index| args.get(index).is_some_and(|value| value == "--headless"))
}

/// Absolute paths stay as given. A relative `PORTLIGHT_SHOT` is from the repo
/// root, not Godot's project directory.
fn resolve_repo_path(path: &str) -> String {
    let path_buf = std::path::Path::new(path);
    if path_buf.is_absolute() {
        return path.to_string();
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let root = if cwd.file_name().and_then(|name| name.to_str()) == Some("godot") {
        cwd.parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or(cwd)
    } else {
        cwd
    };
    root.join(path_buf).to_string_lossy().into_owned()
}

fn docked_port_id(session: &Session) -> Option<&str> {
    let world = session.world();
    if world.voyage.status == VoyageStatus::InPort {
        Some(world.voyage.destination_id.as_str())
    } else {
        None
    }
}

fn docked_name(session: &Session) -> Option<String> {
    let id = docked_port_id(session)?;
    Some(session.world().port(id)?.name.clone())
}

fn port_name(world: &portlight_sim::model::World, id: &str) -> String {
    world
        .port(id)
        .map(|port| port.name.clone())
        .unwrap_or_else(|| id.to_string())
}

fn title_label(text: &str, size: i32, color: Color) -> Gd<Label> {
    let mut label = Label::new_alloc();
    label.set_text(text);
    label.add_theme_font_size_override("font_size", size);
    label.add_theme_color_override("font_color", color);
    label
}

fn body_label(text: &str, size: i32, color: Color) -> Gd<Label> {
    title_label(text, size, color)
}

fn action_button(text: &str, game: InstanceId, action: Action) -> Gd<Button> {
    let mut button = Button::new_alloc();
    button.set_text(text);
    button.add_theme_color_override("font_color", CREAM);
    button.add_theme_color_override("font_hover_color", GOLD);
    let action_for_click = action;
    button.signals().pressed().connect(move || {
        let Ok(mut gd) = Gd::<PortlightGame>::try_from_instance_id(game) else {
            return;
        };
        gd.bind_mut().perform(action_for_click.clone());
    });
    button
}

/// Encounter actions only. The chart side panel keeps [`action_button`].
fn encounter_button(text: &str, game: InstanceId, action: Action) -> Gd<Button> {
    let mut button = Button::new_alloc();
    button.set_text(text);
    encounter_screen::style_encounter_button(&mut button);
    let action_for_click = action;
    button.signals().pressed().connect(move || {
        let Ok(mut gd) = Gd::<PortlightGame>::try_from_instance_id(game) else {
            return;
        };
        gd.bind_mut().perform(action_for_click.clone());
    });
    button
}

fn labels_under_box(node: &Gd<VBoxContainer>) -> Vec<Gd<Label>> {
    let mut found = Vec::new();
    for child in node.get_children().iter_shared() {
        found.extend(labels_from_node(&child));
    }
    found
}

fn labels_from_node(node: &Gd<Node>) -> Vec<Gd<Label>> {
    let mut found = Vec::new();
    if let Ok(label) = node.clone().try_cast::<Label>() {
        found.push(label);
    }
    for child in node.get_children().iter_shared() {
        found.extend(labels_from_node(&child));
    }
    found
}

/// Fit a row label beside its button.
///
/// Autowrap and clip-text together make Godot 4.7.2 report a 1×1 minimum
/// size (`label.cpp` around the autowrap clip branch), so the text never
/// draws. Ellipsis trimming keeps one line and does not collapse it.
fn shrink_label(label: &mut Gd<Label>) {
    label.set_h_size_flags(SizeFlags::EXPAND_FILL);
    label.set_autowrap_mode(AutowrapMode::OFF);
    label.set_clip_text(false);
    label.set_text_overrun_behavior(OverrunBehavior::TRIM_ELLIPSIS);
}

fn scrolling(height: f32) -> (Gd<ScrollContainer>, Gd<VBoxContainer>) {
    let mut scroll = ScrollContainer::new_alloc();
    scroll.set_custom_minimum_size(Vector2::new(0.0, height));
    scroll.set_horizontal_scroll_mode(ScrollMode::DISABLED);
    scroll.set_v_size_flags(SizeFlags::EXPAND_FILL);
    let mut inner = VBoxContainer::new_alloc();
    inner.set_h_size_flags(SizeFlags::EXPAND_FILL);
    scroll.add_child(&inner);
    (scroll, inner)
}

fn clear_children(node: &mut Gd<VBoxContainer>) {
    let children = node.get_children();
    for mut child in children.iter_shared() {
        node.remove_child(&child);
        child.queue_free();
    }
}

fn flag_set(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| value != "0" && !value.is_empty())
}

fn user_arg(flag: &str) -> bool {
    let args = Os::singleton().get_cmdline_user_args();
    (0..args.len()).any(|index| args.get(index).is_some_and(|value| value == flag))
}
