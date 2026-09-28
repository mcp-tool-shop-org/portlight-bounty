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
//! not written onto reputation. Negotiate, flee, naval rounds, and boarding
//! are not on `Session` and are not offered. Hire and provisions call
//! `hire_crew` and `provision`; a `SimError` is shown with its `Display`.

use godot::classes::control::{LayoutPreset, SizeFlags};
use godot::classes::scroll_container::ScrollMode;
use godot::classes::text_server::{AutowrapMode, OverrunBehavior};
use godot::classes::{
    Button, Control, HBoxContainer, IControl, Label, Node, Os, PanelContainer, ScrollContainer,
    StyleBoxFlat, SubViewport, SubViewportContainer, VBoxContainer,
};
use godot::global::Error;
use godot::obj::InstanceId;
use godot::prelude::*;
use portlight_chart::{
    docked_sloop_marker, frame_to_view, lane_inspect, press_port, project_chart, ChartModel,
    Facing, PortPress, CHART_VIEW_H, CHART_VIEW_W, FIRST_PLAYABLE_CAPTAIN, FIRST_PLAYABLE_NAME,
    FIRST_PLAYABLE_SEED,
};
use portlight_sim::model::VoyageStatus;
use portlight_sim::{content, DuelOutcome, LaneSuitability, Session};

use crate::chart_canvas::{connect_port_pressed, ChartCanvas};
use crate::logic::{
    chart_host_width, duel_button_enabled, frame_mostly_flat, layout_fits_window, PANEL_MIN_W,
    ROW_SEPARATION, WINDOW_H, WINDOW_W,
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
    /// Art-director frame: a docked sloop plus the sailing sloop. Not play.
    art: bool,
    smoke_ok: bool,
    shot_path: Option<String>,
    capture_frames: i32,
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
            capture_frames: 0,
        }
    }

    fn ready(&mut self) {
        self.smoke = flag_set("PORTLIGHT_SMOKE") || user_arg("--smoke");
        // A shot is opt-in. Headless `--smoke` checks the session and does not
        // read the viewport: the dummy renderer has no texture, and asking for
        // one logs `Parameter "t" is null` while still exiting 0.
        self.shot_path = std::env::var("PORTLIGHT_SHOT")
            .ok()
            .filter(|path| !path.is_empty());
        self.build_ui();
        self.start_game();
        if user_arg("--encounter") {
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
                self.shot_path = Some("/tmp/portlight-art-sloop.png".to_string());
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
        // Measure after layout. Headless `--smoke` has no shot and skips this:
        // the Xvfb capture is the run that has to see real label sizes.
        if self.smoke && self.shot_path.is_some() && self.market_open {
            self.assert_panel_labels();
        }
        if let Some(path) = self.shot_path.clone() {
            if !self.save_shot(&path) {
                self.smoke_ok = false;
            }
        }
        if !self.smoke {
            return;
        }
        let code = if self.smoke_ok { 0 } else { 1 };
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
        // The viewport stays CHART_VIEW_* and stretches into the space left
        // beside the panel, so 1280 does not clip the Sail column.
        view_host.set_custom_minimum_size(Vector2::new(chart_host_width(), WINDOW_H));
        view_host.set_h_size_flags(SizeFlags::EXPAND_FILL);
        view_host.set_v_size_flags(SizeFlags::EXPAND_FILL);
        view_host.set_stretch(true);
        let mut viewport = SubViewport::new_alloc();
        viewport.set_size(Vector2i::new(CHART_VIEW_W as i32, CHART_VIEW_H as i32));
        viewport.set_disable_3d(true);
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

    /// One chart frame for the art gate: docked sloop at Porto Novo, and the
    /// seed-1 ship one day along the Grain Road at facing f7 with its wake.
    fn run_art(&mut self) {
        self.art = true;
        self.perform(Action::Sail("al_manar".into()));
        self.perform(Action::NextDay);
        let chart = self.chart_now();
        let ok = chart.as_ref().is_some_and(|chart| {
            chart.ship.facing == Facing::F7
                && !chart.ship.docked
                && chart.ship.asset_id == "ship_sloop_f7"
                && chart.gallery.len() == 1
                && chart.gallery[0].docked
                && chart.gallery[0].asset_id == "ship_sloop_f1"
        });
        if ok {
            self.push_log(
                "Art check: docked sloop at Porto Novo, sailing sloop at f7 with wake, on the approved chart water."
                    .to_string(),
            );
        } else {
            self.smoke_ok = false;
            self.push_log("Art: expected a docked f1 sloop and a sailing f7 sloop.".to_string());
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
        if failed && self.smoke {
            self.smoke_ok = false;
        }
        for note in notes {
            self.push_log(note);
        }
        self.refresh();
    }

    fn trade(&mut self, buy: bool, good: &str) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            if buy {
                session.buy(good, 1)
            } else {
                session.sell(good, 1)
            }
        };
        match result {
            Ok(receipt) => self.push_log(format!(
                "{} {} {} for {} silver.",
                receipt.action, receipt.quantity, receipt.good_id, receipt.total_price
            )),
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
        let can_duel = duel_button_enabled(pending);
        if let Some(row) = self.port_row.as_mut() {
            row.set_visible(docked);
        }
        if let Some(label) = self.port_note.as_mut() {
            label.set_visible(docked);
            label.set_text(&services);
        }
        if let Some(box_node) = self.encounter_box.as_mut() {
            box_node.set_visible(pending);
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
                "At sea  {} → {}  {}/{}",
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
                format!(
                    "{}   hull {}/{}   crew {}",
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
            // Keep the Mediterranean in frame so the docked ship and the
            // sailing ship are both on screen. Play still follows the ship.
            chart.frame = frame_to_view(chart.focus, CHART_VIEW_W, CHART_VIEW_H);
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
    fn save_shot(&self, path: &str) -> bool {
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
        godot_print!(
            "screenshot {path} {width}x{height} samples={} flat={flat} error={err:?}",
            samples.len()
        );
        if err != Error::OK {
            godot_print!("screenshot save failed");
            return false;
        }
        if width != WINDOW_W as i32 || height != WINDOW_H as i32 || flat {
            godot_print!(
                "screenshot rejected: expected a full 1280x720 frame that is not one flat colour"
            );
            return false;
        }
        true
    }
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

fn frame_samples(image: &Gd<godot::classes::Image>) -> Vec<[u8; 3]> {
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

fn flag_set(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| value != "0" && !value.is_empty())
}

fn user_arg(flag: &str) -> bool {
    let args = Os::singleton().get_cmdline_user_args();
    (0..args.len()).any(|index| args.get(index).is_some_and(|value| value == flag))
}
