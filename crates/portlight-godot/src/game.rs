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
//! board. The docked Contracts screen reads that board through
//! `available_contracts`, `accept_contract`, `complete_contract`, and
//! `abandon_contract` (no captain, so no breach). Deck melee is
//! `Session::resolve_boarding`. Hire and provisions
//! call `hire_crew` and `provision`; a `SimError` is shown with its `Display`.
//! The docked Crew desk calls `hire_crew`, `fire_crew`, `provision`,
//! `train_crew`, `recruit_companion`, and `spend_skill_point`. Train and
//! Learn show the day advance before the call. Session ignores injury gates
//! on train, so the desk does not. The provision line is the effective price.
//! Chart Hire (one sailor) and Stores +5 stay one-shot shortcuts.
//!
//! An encounter that opens on a sea day (`tick_sea_captain_agency`) or a
//! scripted approach uses the encounter screen: `encounter_choice` /
//! `encounter_choice_with`, `naval_round`, `resolve_boarding`, `fight`,
//! `spare`, `capture`, and `take_all`. Spare and take-all log the
//! [`portlight_sim::VictoryReceipt`] on the chart and on the outcome card.
//! Next day logs `Turn` events, shocks, and notes, including injury healing.
//! The voyage stance duel stays
//! `Session::duel` and `Session::resolve_pending_duel` on the chart panel.
//! That panel is hidden while the encounter screen is open and shown again
//! when the screen closes if a duel is still pending. The screen does not
//! clear `pending_duel`.
//!
//! The docked shipyard screen calls `repair`, `rename_ship`, `dock_current_ship`,
//! `board_fleet_ship`, `sell_fleet_ship`, `buy_ship`, and `install_upgrade`.
//! Buy, sell, dock, board, and install wait for a confirm. Opening the screen
//! does not advance the day.
//!
//! The Hunt overlay is forage (`Session::hunt`) plus the bounty desk
//! (`bounty_board`, `accept_bounty`, `hunt_bounty`, `claim_bounty`).
//! `hunt_bounty` opens this same encounter screen. A runner `bounty` verb
//! is a follow-on; this view does not add one.

use godot::classes::canvas_item::TextureFilter;
use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::classes::scroll_container::ScrollMode;
use godot::classes::text_server::{AutowrapMode, OverrunBehavior};
use godot::classes::viewport::DefaultCanvasItemTextureFilter;
use godot::classes::{
    AudioStream, AudioStreamPlayer, Button, Control, DisplayServer, HBoxContainer, IControl,
    InputEvent, Label, LineEdit, Node, Os, PanelContainer, ScrollContainer, StyleBoxFlat,
    SubViewport, SubViewportContainer, VBoxContainer,
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
use std::collections::HashMap;

use crate::chart_canvas::{connect_port_pressed, ChartCanvas};
use crate::contract_strip::{self, ContractStripNodes};
use crate::contracts_screen::{self, ContractsNodes};
use crate::crew_screen::{self, CrewNodes};
use crate::day_report::{self, DayReportDocument, DayReportMemory, DayReportNodes};
use crate::encounter_screen::{self, set_ship_plate, EncounterNodes};
use crate::harbour_screen::{self, HarbourIntent, HarbourModel, HarbourNodes};
use crate::hunt_screen::{self, HuntAction, HuntConfirm, HuntDesk};
use crate::journal_screen::{self, JournalNodes};
use crate::logic::{
    action_caption, action_list_from_error, ascii_label, at_sea, board_confirm_line,
    buy_confirm_line, buy_result_line, captain_button_label, capture_frame_rejected,
    chart_host_width, crew_desk, cycle_index, day_log_lines, dock_confirm_line,
    duel_button_enabled, encounter_frame_rejected, facts_for_catalog_captain, facts_from_agency,
    facts_from_step, frame_mostly_flat, frame_samples, hire_confirm_line, hire_needs_confirm,
    install_confirm_line, layout_fits_window, newgame_copy, newgame_frame_rejected, player_ship,
    present, recruit_confirm_line, save_confirm_title, save_slot_label, sell_confirm_line,
    session_text, shipyard_frame_rejected, shipyard_model, skill_confirm_line, stance_duel_visible,
    template_player_ship, train_confirm_line, ui_sentence, victory_receipt_lines, CrewDesk,
    CustomDraft, EncounterFacts, NewgamePage, PointPool, ScreenAction, ScreenPhase, ShipyardModel,
    StepInput, NEWGAME_SHOT_H, NEWGAME_SHOT_W, NO_COMPANIONS_FOR_HIRE, NO_FIGHTING_MASTER,
    NO_FLEET_HERE, NO_SHIPYARD_BODY, PANEL_MIN_W, ROW_SEPARATION, SCRIPTED_CAPTAIN,
    SCRIPTED_CAPTAIN_TYPE, SCRIPTED_DEPART, SCRIPTED_FIGHT, SCRIPTED_NAME, SCRIPTED_NAVAL,
    SCRIPTED_SEED, WINDOW_H, WINDOW_W,
};
use crate::market::{self, BuyRoom};
use crate::newgame_screen::{self, NewgameNodes};
use crate::playtest::{action_playtest_id, hunt_playtest_id, parse_playtest_id, PlaytestCommand};
use crate::shipyard_screen::{self, ShipyardNodes};

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
pub(crate) enum Action {
    NewGame,
    SaveGame,
    OpenCaptains,
    OpenCustom,
    OpenLoad,
    NewgameBack,
    StartCaptain(String),
    BeginCustom,
    AdjustPoints(PointPool, i64),
    CycleRegion(i32),
    CyclePort(i32),
    CycleBloc(i32),
    CycleFaction(i32),
    CycleMentor(i32),
    LoadSlot(String),
    NextDay,
    Work,
    OpenHunt,
    ToggleMarket,
    CycleTradeQty,
    HireSailor,
    Provision,
    OpenCrew,
    CloseCrew,
    CrewHire { role: String, count: i64 },
    CrewFire { role: String, count: i64 },
    CrewProvision(i64),
    CrewTrain(String),
    CrewSkill(String),
    CrewRecruit(String),
    CrewConfirm,
    CrewCancel,
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
    OpenContracts,
    CloseContracts,
    RefreshContracts,
    AcceptContract(String),
    CompleteContract(String),
    ArmAbandon(String),
    ConfirmAbandon,
    CancelAbandon,
    OpenShipyard,
    CloseShipyard,
    ShipyardRepair,
    ShipyardRename,
    ShipyardArm(ShipyardArm),
    ShipyardConfirm,
    ShipyardCancel,
    OpenJournal,
    CloseJournal,
    CloseDayReport,
    ToggleBeat(String),
    OpenHarbour,
    CloseHarbour,
    HarbourPrepare(HarbourIntent),
    HarbourConfirm,
    HarbourCancel,
    HarbourDeposit(String),
    HarbourWithdraw(String),
    HarbourRepayField,
    HarbourRepayAll,
    HarbourDraw,
    HarbourEmergency,
}

#[derive(Clone)]
pub(crate) enum ShipyardArm {
    Buy(String),
    Install(String),
    Sell(String),
    Dock,
    Board(String),
}

#[derive(Clone, Copy)]
enum ShipyardShot {
    Flagship,
    Yard,
    Fleet,
}

impl ShipyardShot {
    fn file_name(self) -> &'static str {
        match self {
            Self::Flagship => "shipyard-flagship.png",
            Self::Yard => "shipyard-yard.png",
            Self::Fleet => "shipyard-fleet.png",
        }
    }
}

#[derive(Clone, Copy)]
enum ShipyardSection {
    Yard,
    Fleet,
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
enum DraftEdit {
    Name,
    Title,
    Story,
}

#[derive(Clone, Copy)]
enum JournalShot {
    Chronicle,
    Victory,
    Memories,
}

impl JournalShot {
    fn file_name(self) -> &'static str {
        match self {
            Self::Chronicle => "journal-chronicle.png",
            Self::Victory => "journal-victory.png",
            Self::Memories => "journal-memories.png",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Chronicle => "chronicle",
            Self::Victory => "victory",
            Self::Memories => "memories",
        }
    }
}

#[derive(Clone, Copy)]
enum NewgameShot {
    Title,
    Captains,
    Custom,
    Load,
    Save,
}

impl NewgameShot {
    fn file_name(self) -> &'static str {
        match self {
            Self::Title => "newgame-title.png",
            Self::Captains => "newgame-captains.png",
            Self::Custom => "newgame-custom.png",
            Self::Load => "newgame-load.png",
            Self::Save => "newgame-save.png",
        }
    }

    fn page(self) -> NewgamePage {
        match self {
            Self::Title => NewgamePage::Title,
            Self::Captains => NewgamePage::Captains,
            Self::Custom => NewgamePage::Custom,
            Self::Load => NewgamePage::Load,
            Self::Save => NewgamePage::Saved,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ContractsShot {
    Board,
    Active,
    Empty,
}

impl ContractsShot {
    fn file_name(self) -> &'static str {
        match self {
            Self::Board => "contracts-board.png",
            Self::Active => "contracts-active.png",
            Self::Empty => "contracts-empty.png",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CrewShot {
    Roster,
    Provisions,
    /// Layout pass after the train confirm is armed. Not a saved frame.
    PrepareTraining,
    Training,
    /// Layout pass after the confirm is cleared. Not a saved frame.
    PrepareCompanions,
    Companions,
}

impl CrewShot {
    fn file_name(self) -> &'static str {
        match self {
            Self::Roster => "crew-roster.png",
            Self::Provisions => "crew-provisions.png",
            Self::PrepareTraining | Self::Training => "crew-training.png",
            Self::PrepareCompanions | Self::Companions => "crew-companions.png",
        }
    }

    fn saves(self) -> bool {
        matches!(
            self,
            Self::Roster | Self::Provisions | Self::Training | Self::Companions
        )
    }

    fn section(self) -> &'static str {
        match self {
            Self::Roster => crew_screen::SECTION_ROSTER,
            Self::Provisions => crew_screen::SECTION_PROVISIONS,
            Self::PrepareTraining | Self::Training => crew_screen::SECTION_TRAINING,
            Self::PrepareCompanions | Self::Companions => crew_screen::SECTION_COMPANIONS,
        }
    }
}

#[derive(Clone)]
enum CrewPending {
    Hire { role: String, count: i64 },
    Train { id: String },
    Skill { id: String },
    Recruit { id: String },
}

#[derive(Clone, Copy)]
enum HuntShot {
    Forage,
    Board,
    Active,
}

impl HuntShot {
    fn file_name(self) -> &'static str {
        match self {
            Self::Forage => "hunt-forage.png",
            Self::Board => "hunt-board.png",
            Self::Active => "hunt-active.png",
        }
    }

    fn section(self) -> &'static str {
        match self {
            Self::Forage => "HuntForage",
            Self::Board => "HuntBoard",
            Self::Active => "HuntActive",
        }
    }
}

#[derive(Clone, Copy)]
enum DayReportShot {
    Full,
    Deadline,
    Week,
    Arrival,
}

impl DayReportShot {
    fn file_name(self) -> &'static str {
        match self {
            Self::Full => "day-report-full.png",
            Self::Deadline => "day-report-deadline.png",
            Self::Week => "day-report-week.png",
            Self::Arrival => "day-report-arrival.png",
        }
    }
}

#[derive(Clone, Copy)]
enum ContractStripShot {
    Active,
    Urgent,
}

impl ContractStripShot {
    fn file_name(self) -> &'static str {
        match self {
            Self::Active => "contract-strip-active.png",
            Self::Urgent => "contract-strip-urgent.png",
        }
    }
}

/// T-N frames: the GOLD paid notice at the top of the Market box.
#[derive(Clone, Copy)]
enum MarketPaidShot {
    /// The real settle from the trade smoke: one line.
    Single,
    /// Staged on top of that settle: two lines and `+1 more`.
    More,
}

impl MarketPaidShot {
    fn file_name(self) -> &'static str {
        match self {
            Self::Single => "market-contract-paid.png",
            Self::More => "market-contract-paid-more.png",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Stance {
    Thrust,
    Slash,
    Parry,
}

impl Stance {
    pub(crate) fn as_str(self) -> &'static str {
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
    /// At-sea Hunt control, on the Next day row. Hidden while docked.
    sea_hunt_button: Option<Gd<Button>>,
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
    /// The path is the implicit `/tmp` default. Headless skips that save.
    /// `PORTLIGHT_SHOT`, `PORTLIGHT_ENCOUNTER_DIR`, `--art-docs`, and
    /// `PORTLIGHT_ART_DOCS` are explicit and still fail when they cannot save.
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
    save_button: Option<Gd<Button>>,
    newgame_nodes: Option<NewgameNodes>,
    newgame_page: NewgamePage,
    newgame_built: Option<NewgamePage>,
    draft: CustomDraft,
    name_edit: Option<Gd<LineEdit>>,
    title_edit: Option<Gd<LineEdit>>,
    story_edit: Option<Gd<LineEdit>>,
    save_base: std::path::PathBuf,
    newgame_notice: String,
    save_ok: bool,
    newgame_checked: bool,
    newgame_shot_dir: Option<String>,
    newgame_shot: Option<NewgameShot>,
    contracts_nodes: Option<ContractsNodes>,
    contracts_open: bool,
    contracts_notice: String,
    /// Offer id waiting on Confirm. Abandon is not sent until then.
    contracts_confirm: Option<String>,
    contracts_checked: bool,
    contracts_shot_dir: Option<String>,
    contracts_shot: Option<ContractsShot>,
    shipyard_nodes: Option<ShipyardNodes>,
    shipyard_open: bool,
    shipyard_notice: String,
    shipyard_confirm: Option<ShipyardArm>,
    shipyard_shown: Option<ShipyardModel>,
    rename_edit: Option<Gd<LineEdit>>,
    rename_draft: String,
    yard_mark: Option<Gd<Control>>,
    fleet_mark: Option<Gd<Control>>,
    shipyard_button: Option<Gd<Button>>,
    shipyard_checked: bool,
    /// Four voices so a repeated trade click does not cut the coin off.
    /// Empty means [`Self::play_sfx`] stays silent.
    sfx_players: Vec<Gd<AudioStreamPlayer>>,
    sfx_cache: HashMap<String, Gd<AudioStream>>,
    sfx_rr: usize,
    shipyard_shot_dir: Option<String>,
    shipyard_shot: Option<ShipyardShot>,
    shipyard_scroll: Option<ShipyardSection>,
    /// Day when `--shipyard-screen` opened. The script must not move it.
    shipyard_day: i64,
    journal_nodes: Option<JournalNodes>,
    journal_open: bool,
    /// Body was filled for the current open. A refresh does not rebuild it,
    /// so a scroll position survives. Opening, or expanding a beat, clears it.
    journal_filled: bool,
    journal_expanded: Vec<String>,
    journal_button: Option<Gd<Button>>,
    journal_checked: bool,
    journal_shot_dir: Option<String>,
    journal_shot: Option<JournalShot>,
    /// At-sea button check runs after the journal frames, so the shots stay
    /// on the docked early voyage.
    journal_sea_pending: bool,
    harbour_nodes: Option<HarbourNodes>,
    harbour_open: bool,
    harbour_notice: String,
    harbour_pending: Option<HarbourIntent>,
    harbour_qty: Option<Gd<LineEdit>>,
    harbour_draw: Option<Gd<LineEdit>>,
    harbour_repay: Option<Gd<LineEdit>>,
    harbour_emergency: Option<Gd<LineEdit>>,
    harbour_checked: bool,
    harbour_shot_dir: Option<String>,
    harbour_shot: Option<HarbourShot>,
    crew_nodes: Option<CrewNodes>,
    crew_open: bool,
    crew_notice: String,
    crew_pending: Option<CrewPending>,
    crew_checked: bool,
    crew_shot_dir: Option<String>,
    crew_shot: Option<CrewShot>,
    hunt_nodes: Option<hunt_screen::HuntNodes>,
    hunt_open: bool,
    hunt_desk: HuntDesk,
    hunt_checked: bool,
    hunt_shot_dir: Option<String>,
    hunt_shot: Option<HuntShot>,
    /// The section is scrolled after layout, then the frame waits one draw.
    hunt_scrolled: bool,
    day_report_nodes: Option<DayReportNodes>,
    day_report_open: bool,
    day_report_doc: Option<DayReportDocument>,
    day_report_memory: DayReportMemory,
    day_report_checked: bool,
    day_report_shot_dir: Option<String>,
    day_report_shot: Option<DayReportShot>,
    contract_strip_nodes: Option<ContractStripNodes>,
    contract_strip_checked: bool,
    contract_strip_shot_dir: Option<String>,
    contract_strip_shot: Option<ContractStripShot>,
    /// T-Q. Units per Buy/Sell press. Godot memory only; never saved.
    trade_qty: i64,
    /// T-N. GOLD paid lines at the top of the Market box until the next
    /// trade, Market close, undock or Next day.
    market_notice: Vec<String>,
    /// `--trade-smoke`: Qty 10 buy, sail, Qty 10 sell that settles a contract.
    trade_checked: bool,
    market_paid_shot_dir: Option<String>,
    market_paid_shot: Option<MarketPaidShot>,
}

#[derive(Clone, Copy)]
enum HarbourShot {
    Warehouse,
    Broker,
    Finance,
}

impl HarbourShot {
    fn file_name(self) -> &'static str {
        match self {
            Self::Warehouse => "harbour-warehouse.png",
            Self::Broker => "harbour-broker.png",
            Self::Finance => "harbour-finance.png",
        }
    }

    fn anchor(self) -> &'static str {
        match self {
            Self::Warehouse => harbour_screen::ANCHOR_WAREHOUSE,
            Self::Broker => harbour_screen::ANCHOR_BROKER,
            Self::Finance => harbour_screen::ANCHOR_FINANCE,
        }
    }
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
            sea_hunt_button: None,
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
            save_button: None,
            newgame_nodes: None,
            newgame_page: NewgamePage::Hidden,
            newgame_built: None,
            draft: CustomDraft::default(),
            name_edit: None,
            title_edit: None,
            story_edit: None,
            save_base: std::path::PathBuf::new(),
            newgame_notice: String::new(),
            save_ok: false,
            newgame_checked: false,
            newgame_shot_dir: None,
            newgame_shot: None,
            contracts_nodes: None,
            contracts_open: false,
            contracts_notice: String::new(),
            contracts_confirm: None,
            contracts_checked: false,
            contracts_shot_dir: None,
            contracts_shot: None,
            shipyard_nodes: None,
            shipyard_open: false,
            shipyard_notice: String::new(),
            shipyard_confirm: None,
            shipyard_shown: None,
            rename_edit: None,
            rename_draft: String::new(),
            yard_mark: None,
            fleet_mark: None,
            shipyard_button: None,
            shipyard_checked: false,
            sfx_players: Vec::new(),
            sfx_cache: HashMap::new(),
            sfx_rr: 0,
            shipyard_shot_dir: None,
            shipyard_shot: None,
            shipyard_scroll: None,
            shipyard_day: 0,
            journal_nodes: None,
            journal_open: false,
            journal_filled: false,
            journal_expanded: Vec::new(),
            journal_button: None,
            journal_checked: false,
            journal_shot_dir: None,
            journal_shot: None,
            journal_sea_pending: false,
            harbour_nodes: None,
            harbour_open: false,
            harbour_notice: String::new(),
            harbour_pending: None,
            harbour_qty: None,
            harbour_draw: None,
            harbour_repay: None,
            harbour_emergency: None,
            harbour_checked: false,
            harbour_shot_dir: None,
            harbour_shot: None,
            crew_nodes: None,
            crew_open: false,
            crew_notice: String::new(),
            crew_pending: None,
            crew_checked: false,
            crew_shot_dir: None,
            crew_shot: None,
            hunt_nodes: None,
            hunt_open: false,
            hunt_desk: HuntDesk::default(),
            hunt_checked: false,
            hunt_shot_dir: None,
            hunt_shot: None,
            hunt_scrolled: false,
            day_report_nodes: None,
            day_report_open: false,
            day_report_doc: None,
            day_report_memory: DayReportMemory::default(),
            day_report_checked: false,
            day_report_shot_dir: None,
            day_report_shot: None,
            contract_strip_nodes: None,
            contract_strip_checked: false,
            contract_strip_shot_dir: None,
            contract_strip_shot: None,
            trade_qty: market::TRADE_QTYS[0],
            market_notice: Vec::new(),
            trade_checked: false,
            market_paid_shot_dir: None,
            market_paid_shot: None,
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
        self.save_base = std::path::PathBuf::from(resolve_repo_path("."));
        self.build_ui();
        if user_arg("--newgame-screen") {
            self.smoke = true;
            let capture = newgame_frames_requested(self.shot_path.is_some());
            self.run_newgame_smoke();
            if capture {
                self.begin_newgame_shots();
                self.capture_frames = 4;
            } else {
                self.capture_frames = 2;
            }
        } else if user_arg("--shipyard-screen") {
            self.smoke = true;
            self.start_game();
            self.shipyard_checked = true;
            self.shipyard_day = self
                .session
                .as_ref()
                .map(|session| session.world().day)
                .unwrap_or(0);
            let capture = newgame_frames_requested(self.shot_path.is_some());
            self.open_shipyard();
            if capture {
                self.begin_shipyard_shots();
                self.capture_frames = 4;
            } else {
                self.run_shipyard_actions();
                self.capture_frames = 2;
            }
        } else if user_arg("--journal-screen") {
            self.smoke = true;
            self.start_game();
            let capture = newgame_frames_requested(self.shot_path.is_some());
            self.run_journal_checks();
            if capture {
                self.begin_journal_shots();
                self.journal_sea_pending = true;
                self.capture_frames = 4;
            } else {
                self.journal_sea_check();
                self.capture_frames = 2;
            }
        } else if user_arg("--hunt-screen") {
            self.smoke = true;
            let capture = hunt_frames_requested(self.shot_path.is_some());
            self.run_hunt_smoke();
            if capture {
                self.begin_hunt_shots();
                self.capture_frames = 4;
            } else {
                self.capture_frames = 2;
            }
        } else if user_arg("--day-report-screen") {
            self.smoke = true;
            self.day_report_checked = true;
            let capture = day_report_frames_requested(self.shot_path.is_some());
            if capture {
                self.begin_day_report_shots();
                self.capture_frames = 4;
            } else {
                self.run_day_report_smoke();
                self.capture_frames = 2;
            }
        } else if user_arg("--day-report-week") {
            // Captain's week footer on a real notable day (Session verbs only).
            self.smoke = true;
            self.day_report_checked = true;
            let capture = day_report_frames_requested(self.shot_path.is_some());
            if self.run_day_report_week_smoke() && capture {
                self.day_report_shot_dir = Some(day_report_shot_dir(self.shot_path.as_deref()));
                self.day_report_shot = Some(DayReportShot::Week);
                self.capture_frames = 4;
            } else {
                self.capture_frames = 2;
            }
        } else if user_arg("--day-report-arrival") {
            // Arrival variant: real Session sail→dock smoke, then forced frame.
            self.smoke = true;
            self.day_report_checked = true;
            let capture = day_report_frames_requested(self.shot_path.is_some());
            if self.run_day_report_arrival_smoke() && capture {
                let day = self
                    .session
                    .as_ref()
                    .map(|session| session.world().day)
                    .unwrap_or(1);
                self.open_day_report_doc(day_report::smoke_arrival_document(day));
                self.day_report_shot_dir = Some(day_report_shot_dir(self.shot_path.as_deref()));
                self.day_report_shot = Some(DayReportShot::Arrival);
                self.capture_frames = 4;
            } else {
                self.capture_frames = 2;
            }
        } else if user_arg("--contract-strip-screen") {
            self.smoke = true;
            self.contract_strip_checked = true;
            let capture = contract_strip_frames_requested(self.shot_path.is_some());
            if capture {
                self.begin_contract_strip_shots();
                self.capture_frames = 4;
            } else {
                self.run_contract_strip_smoke();
                self.capture_frames = 2;
            }
        } else if user_arg("--trade-smoke") {
            // T-Q / T-N: real Session trades only, no frames.
            self.smoke = true;
            self.trade_checked = true;
            self.run_trade_smoke();
            self.capture_frames = 2;
        } else if user_arg("--market-paid-screen") {
            // T-N frames: the real trade-smoke settle, then a staged
            // two-contract notice on the same Market.
            self.smoke = true;
            self.trade_checked = true;
            let capture = market_paid_frames_requested(self.shot_path.is_some());
            if self.run_trade_settle() {
                self.check_market_notice_leads("single");
                if capture {
                    self.market_paid_shot_dir =
                        Some(market_paid_shot_dir(self.shot_path.as_deref()));
                    self.market_paid_shot = Some(MarketPaidShot::Single);
                    self.capture_frames = 4;
                } else {
                    self.stage_market_paid_more();
                    self.capture_frames = 2;
                }
            } else {
                self.capture_frames = 2;
            }
        } else if scripted_launch() {
            self.start_game();
            self.launch_scripted();
        } else {
            self.open_newgame(NewgamePage::Title);
        }
    }

    fn process(&mut self, _delta: f64) {
        if self.capture_frames <= 0 {
            return;
        }
        // Scroll one frame before the capture so the active row has a position.
        if self.capture_frames == 2 {
            self.pin_contracts_scroll();
        }
        self.capture_frames -= 1;
        if self.capture_frames == 2 {
            self.apply_shipyard_scroll();
        }
        if self.capture_frames > 0 {
            return;
        }
        if self.advance_shipyard_shot() {
            return;
        }
        if self.advance_newgame_shot() {
            return;
        }
        if self.advance_harbour_shot() {
            return;
        }
        if self.advance_crew_shot() {
            return;
        }
        if self.advance_hunt_shot() {
            return;
        }
        if self.advance_day_report_shot() {
            return;
        }
        if self.advance_contract_strip_shot() {
            return;
        }
        if self.advance_market_paid_shot() {
            return;
        }
        if self.advance_encounter_shot() {
            return;
        }
        if self.advance_contracts_shot() {
            return;
        }
        if self.advance_journal_shot() {
            return;
        }
        if self.journal_sea_pending {
            self.journal_sea_pending = false;
            self.journal_sea_check();
        }
        // Measure after layout. Headless `--smoke` has no shot and skips this:
        // the Xvfb capture is the run that has to see real label sizes.
        if self.smoke && self.shot_path.is_some() && self.market_open {
            self.assert_panel_labels();
            self.assert_port_row_fits();
        }
        // A new-game, contracts, shipyard, journal, harbour, crew, or hunt sequence already wrote its frames.
        if self.newgame_shot_dir.is_none()
            && self.contracts_shot_dir.is_none()
            && self.shipyard_shot_dir.is_none()
            && self.journal_shot_dir.is_none()
            && self.harbour_shot_dir.is_none()
            && self.crew_shot_dir.is_none()
            && self.hunt_shot_dir.is_none()
            && self.day_report_shot_dir.is_none()
            && self.contract_strip_shot_dir.is_none()
            && self.market_paid_shot_dir.is_none()
        {
            if let Some(path) = self.shot_path.clone() {
                // `--encounter-galleon` is still on the encounter screen. The
                // multi-frame shot saves its own files before this, then the
                // encounter has closed, so a trailing PORTLIGHT_SHOT is a chart.
                // Headless skips only an implicit `/tmp` default. `PORTLIGHT_SHOT`
                // and `--art-docs` still read the viewport and fail when it is empty.
                // A new-game sequence already wrote its own frames.
                let skip = headless_runtime() && self.shot_implicit;
                if !skip && !self.save_shot(&path, self.galleon_frame) {
                    self.smoke_ok = false;
                }
            }
        }
        if self.capture_failed {
            self.smoke_ok = false;
        }
        if !self.smoke {
            return;
        }
        let code = if self.smoke_ok { 0 } else { 1 };
        if self.journal_checked {
            godot_print!(
                "portlight journal smoke {}",
                if self.smoke_ok { "ok" } else { "FAILED" }
            );
        } else if self.shipyard_checked {
            godot_print!(
                "portlight shipyard smoke {}",
                if self.smoke_ok { "ok" } else { "FAILED" }
            );
        } else if self.newgame_checked {
            godot_print!(
                "portlight newgame smoke {}",
                if self.smoke_ok { "ok" } else { "FAILED" }
            );
        } else if self.contracts_checked {
            godot_print!(
                "portlight contracts smoke {}",
                if self.smoke_ok { "ok" } else { "FAILED" }
            );
        } else if self.harbour_checked {
            godot_print!(
                "portlight harbour smoke {}",
                if self.smoke_ok { "ok" } else { "FAILED" }
            );
        } else if self.crew_checked {
            godot_print!(
                "portlight crew smoke {}",
                if self.smoke_ok { "ok" } else { "FAILED" }
            );
        } else if self.hunt_checked {
            godot_print!(
                "portlight hunt smoke {}",
                if self.smoke_ok { "ok" } else { "FAILED" }
            );
        } else if self.day_report_checked {
            godot_print!(
                "portlight day-report smoke {}",
                if self.smoke_ok { "ok" } else { "FAILED" }
            );
        } else if self.contract_strip_checked {
            godot_print!(
                "portlight contract-strip smoke {}",
                if self.smoke_ok { "ok" } else { "FAILED" }
            );
        } else if self.trade_checked {
            godot_print!(
                "portlight trade smoke {}",
                if self.smoke_ok { "ok" } else { "FAILED" }
            );
        } else {
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
        }
        let mut tree = self.base().get_tree();
        tree.quit_ex().exit_code(code).done();
    }

    fn unhandled_key_input(&mut self, event: Gd<InputEvent>) {
        // Docked-consistency: dismiss via InputMap action ui_cancel (Escape), not a raw keycode.
        // Today that closes Day's report only. The playtest `key` path calls the same function,
        // so Escape stays the known-fail until docked-consistency changes this one place.
        if !event.is_action_pressed("ui_cancel") {
            return;
        }
        self.dismiss_cancel();
    }
}

impl PortlightGame {
    fn launch_scripted(&mut self) {
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
                // `--art-docs` names docs/screenshots. Only the bare `/tmp`
                // default is implicit.
                self.shot_implicit = !docs_capture();
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
        } else if user_arg("--crew-screen") {
            self.smoke = true;
            self.crew_checked = true;
            self.open_crew();
            if self.docked_id().is_none() || !self.crew_open {
                self.smoke_ok = false;
                self.push_log("Crew smoke: expected a docked crew desk.".to_string());
            }
            if crew_frames_requested(self.shot_path.is_some()) {
                self.begin_crew_shots();
                self.capture_frames = 4;
            } else {
                self.run_crew_actions();
                self.capture_frames = 2;
            }
        } else if user_arg("--art") {
            self.smoke = true;
            if self.shot_path.is_none() {
                self.shot_path = Some(art_shot_path("chart-cutter-f7.png"));
                self.shot_implicit = !docs_capture();
            }
            self.run_art();
            self.capture_frames = 4;
        } else if user_arg("--contracts-screen") {
            self.smoke = true;
            self.contracts_checked = true;
            let capture = newgame_frames_requested(self.shot_path.is_some());
            if capture {
                self.begin_contracts_shots();
                self.capture_frames = 4;
            } else {
                self.run_contracts_smoke();
                self.capture_frames = 2;
            }
        } else if user_arg("--harbour-screen") {
            self.smoke = true;
            let capture = harbour_frames_requested(self.shot_path.is_some());
            self.run_harbour_smoke(capture);
            if capture {
                self.begin_harbour_shots();
                self.capture_frames = 4;
            } else {
                self.capture_frames = 2;
            }
        } else if self.smoke {
            self.run_smoke();
            self.capture_frames = 4;
        }
    }

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

        // Children of PortlightGame, not of the chart SubViewport, so a frame
        // capture never reads them. Playback itself is gated in `play_sfx`.
        self.sfx_players.clear();
        for _ in 0..4 {
            let mut player = AudioStreamPlayer::new_alloc();
            player.set_bus("SFX");
            self.base_mut().add_child(&player);
            self.sfx_players.push(player);
        }

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
        // Same control as New game and Next day. The docked port row is already
        // full, and it is hidden at sea, so Save stays on this row.
        let save = action_button("Save", game_id, Action::SaveGame);
        buttons.add_child(&save);
        self.save_button = Some(save);
        let journal = action_button("Journal", game_id, Action::OpenJournal);
        buttons.add_child(&journal);
        self.journal_button = Some(journal);
        // At sea this sits beside Journal, same control as New game and Save.
        // Hidden in port; docked Hunt is on the port row.
        let mut sea_hunt = action_button("Hunt", game_id, Action::OpenHunt);
        sea_hunt.set_visible(false);
        buttons.add_child(&sea_hunt);
        self.sea_hunt_button = Some(sea_hunt);
        column.add_child(&buttons);

        // One 31 px line. A wrap would push the lanes, the market, and the log.
        let mut port_row = HBoxContainer::new_alloc();
        port_row.set_name("PortRow");
        // 2 px gaps plus the 1 px button borders mark where each label
        // ends. The market toggle always reads Market, so the row keeps the
        // same width open or shut and stays one line inside the panel.
        port_row.add_theme_constant_override("separation", 2);
        let mut market = port_row_button("Market", game_id, Action::ToggleMarket);
        // Open shows as the pressed style with a gold face, not a new label.
        market.set_toggle_mode(true);
        market.add_theme_color_override("font_pressed_color", GOLD);
        market.add_theme_color_override("font_hover_pressed_color", GOLD);
        port_row.add_child(&market);
        self.market_button = Some(market);
        let mut contracts = port_row_button("Contracts", game_id, Action::OpenContracts);
        contracts.set_name("ContractsButton");
        port_row.add_child(&contracts);
        // `Hire` is one sailor (HireSailor). The short label keeps the row on
        // one line at 1280.
        port_row.add_child(&port_row_button("Hire", game_id, Action::HireSailor));
        port_row.add_child(&port_row_button("Stores +5", game_id, Action::Provision));
        let work = port_row_button("Work", game_id, Action::Work);
        port_row.add_child(&work);
        self.work_button = Some(work);
        let shipyard = port_row_button("Shipyard", game_id, Action::OpenShipyard);
        port_row.add_child(&shipyard);
        self.shipyard_button = Some(shipyard);
        port_row.add_child(&port_row_button("Harbour", game_id, Action::OpenHarbour));
        port_row.add_child(&port_row_button("Crew", game_id, Action::OpenCrew));
        // Same port-row control as Harbour and Crew. On this line, so the
        // lanes below do not move.
        port_row.add_child(&port_row_button("Hunt", game_id, Action::OpenHunt));
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

        let mut newgame = newgame_screen::build_newgame_screen();
        self.base_mut().add_child(&newgame.root);
        newgame_screen::fill_parent(&mut newgame.root);
        self.newgame_nodes = Some(newgame);

        let mut contracts = contracts_screen::build_contracts_screen();
        self.base_mut().add_child(&contracts.root);
        encounter_screen::fill_parent(&mut contracts.root);
        self.wire_contracts_chrome(&mut contracts);
        self.contracts_nodes = Some(contracts);

        let mut shipyard = shipyard_screen::build_shipyard_screen();
        self.base_mut().add_child(&shipyard.root);
        encounter_screen::fill_parent(&mut shipyard.root);
        stamp_playtest_id(
            &mut shipyard.close,
            &action_playtest_id(&Action::CloseShipyard),
        );
        let close = shipyard.close.clone();
        let close_game = game_id;
        close.signals().pressed().connect(move || {
            let Ok(mut gd) = Gd::<PortlightGame>::try_from_instance_id(close_game) else {
                return;
            };
            gd.bind_mut().perform(Action::CloseShipyard);
        });
        self.shipyard_nodes = Some(shipyard);

        let mut journal = journal_screen::build_journal_screen();
        stamp_playtest_id(
            &mut journal.close,
            &action_playtest_id(&Action::CloseJournal),
        );
        let close_id = game_id;
        journal.close.signals().pressed().connect(move || {
            let Ok(mut gd) = Gd::<PortlightGame>::try_from_instance_id(close_id) else {
                return;
            };
            gd.bind_mut().perform(Action::CloseJournal);
        });
        self.base_mut().add_child(&journal.root);
        journal_screen::fill_parent(&mut journal.root);
        self.journal_nodes = Some(journal);

        let mut harbour = harbour_screen::build_harbour_screen();
        self.base_mut().add_child(&harbour.root);
        encounter_screen::fill_parent(&mut harbour.root);
        self.harbour_nodes = Some(harbour);

        let mut crew = crew_screen::build_crew_screen();
        self.base_mut().add_child(&crew.root);
        crew_screen::fill_parent(&mut crew.root);
        let mut confirm_row = HBoxContainer::new_alloc();
        confirm_row.add_theme_constant_override("separation", 8);
        confirm_row.add_child(&crew_button("Confirm", game_id, Action::CrewConfirm));
        confirm_row.add_child(&crew_button("Cancel", game_id, Action::CrewCancel));
        crew.confirm.add_child(&confirm_row);
        let mut close = crew_button("Close", game_id, Action::CloseCrew);
        close.set_h_size_flags(SizeFlags::SHRINK_BEGIN);
        crew.column.add_child(&close);
        self.crew_nodes = Some(crew);
        let mut hunt = hunt_screen::build_hunt_screen();
        self.base_mut().add_child(&hunt.root);
        hunt_screen::fill_parent(&mut hunt.root);
        self.hunt_nodes = Some(hunt);

        let mut day_report = day_report::build_day_report_screen();
        stamp_playtest_id(
            &mut day_report.close,
            &action_playtest_id(&Action::CloseDayReport),
        );
        let close_day = game_id;
        day_report.close.signals().pressed().connect(move || {
            let Ok(mut gd) = Gd::<PortlightGame>::try_from_instance_id(close_day) else {
                return;
            };
            gd.bind_mut().perform(Action::CloseDayReport);
        });
        self.base_mut().add_child(&day_report.root);
        day_report::place_card(&mut day_report.root);
        self.day_report_nodes = Some(day_report);

        let mut contract_strip = contract_strip::build_contract_strip();
        let strip_click = game_id;
        contract_strip.hit.signals().pressed().connect(move || {
            let Ok(mut gd) = Gd::<PortlightGame>::try_from_instance_id(strip_click) else {
                return;
            };
            // Docked: open Contracts (closes day report). At sea: open_contracts no-ops.
            gd.bind_mut().perform(Action::OpenContracts);
        });
        self.base_mut().add_child(&contract_strip.root);
        contract_strip::place_strip(&mut contract_strip.root);
        self.contract_strip_nodes = Some(contract_strip);
    }

    fn instance_id(&self) -> InstanceId {
        self.to_gd().instance_id()
    }

    fn start_game(&mut self) {
        self.log_lines.clear();
        self.market_open = false;
        self.armed_sail = None;
        self.stances.clear();
        self.hunt_open = false;
        self.hunt_desk = HuntDesk::default();
        self.reset_day_report();
        self.reset_trade();
        self.encounter = None;
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

    fn open_newgame(&mut self, page: NewgamePage) {
        self.read_draft_fields();
        if page != self.newgame_page {
            self.newgame_notice.clear();
        }
        self.newgame_page = page;
        self.newgame_built = None;
        self.refresh();
    }

    fn newgame_back(&mut self) {
        match self.newgame_page {
            NewgamePage::Captains | NewgamePage::Load => self.open_newgame(NewgamePage::Title),
            NewgamePage::Custom => self.open_newgame(NewgamePage::Captains),
            NewgamePage::Title | NewgamePage::Saved if self.session.is_some() => {
                self.open_newgame(NewgamePage::Hidden);
            }
            _ => {}
        }
    }

    fn read_draft_fields(&mut self) {
        if let Some(edit) = &self.name_edit {
            self.draft.name = edit.get_text().to_string();
        }
        if let Some(edit) = &self.title_edit {
            self.draft.title = edit.get_text().to_string();
        }
        if let Some(edit) = &self.story_edit {
            self.draft.backstory = edit.get_text().to_string();
        }
    }

    fn adjust_points(&mut self, pool: PointPool, delta: i64) {
        self.read_draft_fields();
        let next = self.draft.pool(pool) + delta;
        self.draft.set_pool(pool, next);
        self.newgame_notice.clear();
        self.newgame_built = None;
        self.refresh();
    }

    fn cycle_region(&mut self, delta: i32) {
        self.read_draft_fields();
        let options = Session::custom_captain_options();
        let index = options
            .regions
            .iter()
            .position(|region| region == &self.draft.home_region)
            .unwrap_or(0);
        let next = cycle_index(options.regions.len(), index, delta);
        if let Some(region) = options.regions.get(next) {
            self.draft.home_region = region.clone();
            if let Some(port) = options.ports.iter().find(|port| port.region == *region) {
                self.draft.home_port_id = port.id.clone();
            }
        }
        self.keep_mentor();
        self.newgame_notice.clear();
        self.newgame_built = None;
        self.refresh();
    }

    fn cycle_port(&mut self, delta: i32) {
        self.read_draft_fields();
        let options = Session::custom_captain_options();
        let ports: Vec<_> = options
            .ports
            .iter()
            .filter(|port| port.region == self.draft.home_region)
            .map(|port| port.id.clone())
            .collect();
        let index = ports
            .iter()
            .position(|id| id == &self.draft.home_port_id)
            .unwrap_or(0);
        if let Some(id) = ports.get(cycle_index(ports.len(), index, delta)) {
            self.draft.home_port_id = id.clone();
        }
        self.keep_mentor();
        self.newgame_notice.clear();
        self.newgame_built = None;
        self.refresh();
    }

    fn cycle_bloc(&mut self, delta: i32) {
        self.read_draft_fields();
        let options = Session::custom_captain_options();
        let ids = leading_none(options.blocs.iter().map(|bloc| bloc.id.clone()));
        self.draft.bloc_alignment = cycled(&ids, &self.draft.bloc_alignment, delta);
        self.newgame_notice.clear();
        self.newgame_built = None;
        self.refresh();
    }

    fn cycle_faction(&mut self, delta: i32) {
        self.read_draft_fields();
        let options = Session::custom_captain_options();
        let ids = leading_none(options.factions.iter().map(|faction| faction.id.clone()));
        self.draft.faction_alignment = cycled(&ids, &self.draft.faction_alignment, delta);
        self.newgame_notice.clear();
        self.newgame_built = None;
        self.refresh();
    }

    fn cycle_mentor(&mut self, delta: i32) {
        self.read_draft_fields();
        let ids = self.mentor_ids();
        self.draft.mentor_npc_id = cycled(&ids, &self.draft.mentor_npc_id, delta);
        self.newgame_notice.clear();
        self.newgame_built = None;
        self.refresh();
    }

    fn keep_mentor(&mut self) {
        let ids = self.mentor_ids();
        if !ids.iter().any(|id| id == &self.draft.mentor_npc_id) {
            self.draft.mentor_npc_id.clear();
        }
    }

    fn mentor_ids(&self) -> Vec<String> {
        let options = Session::custom_captain_options();
        leading_none(
            options
                .mentors
                .iter()
                .filter(|mentor| mentor.port_id == self.draft.home_port_id)
                .map(|mentor| mentor.id.clone()),
        )
    }

    fn start_captain(&mut self, id: &str) {
        self.read_draft_fields();
        let spec_name = self.draft.spec().name;
        match Session::new(&spec_name, id, FIRST_PLAYABLE_SEED, None) {
            Ok(session) => self.begin_session(session),
            Err(err) => self.note_newgame(err.to_string()),
        }
    }

    fn begin_custom(&mut self) {
        self.read_draft_fields();
        let spec = self.draft.spec();
        match Session::new_custom(&spec, FIRST_PLAYABLE_SEED, None) {
            Ok(session) => self.begin_session(session),
            Err(err) => self.note_newgame(err.to_string()),
        }
    }

    fn begin_session(&mut self, session: Session) {
        let place = docked_name(&session).unwrap_or_else(|| "a port".to_string());
        self.log_lines.clear();
        self.market_open = false;
        self.armed_sail = None;
        self.stances.clear();
        self.reset_day_report();
        self.reset_trade();
        self.session = Some(session);
        self.newgame_notice.clear();
        self.push_log(format!("A new voyage begins. Docked at {place}."));
        self.play_sfx("sfx_ui_newgame_start");
        self.newgame_page = NewgamePage::Hidden;
        self.newgame_built = None;
        self.refresh();
    }

    fn note_newgame(&mut self, notice: String) {
        self.newgame_notice = notice;
        self.newgame_built = None;
        if self.smoke {
            self.smoke_ok = false;
        }
        self.refresh();
    }

    fn load_slot(&mut self, slot: &str) {
        match Session::load(&self.save_base, slot) {
            Ok(Some(session)) => {
                let place = docked_name(&session).unwrap_or_else(|| "a port".to_string());
                self.log_lines.clear();
                self.market_open = false;
                self.armed_sail = None;
                self.stances.clear();
                self.reset_day_report();
                self.reset_trade();
                self.session = Some(session);
                self.push_log(format!("Loaded slot {slot}. Docked at {place}."));
                self.play_sfx("sfx_ui_newgame_start");
                self.newgame_page = NewgamePage::Hidden;
                self.newgame_built = None;
                self.refresh();
            }
            Ok(None) => self.note_newgame(format!("Could not load slot '{slot}'.")),
            Err(err) => self.note_newgame(err.to_string()),
        }
    }

    fn save_current_game(&mut self) {
        let saved = {
            let Some(session) = self.session.as_mut() else {
                self.save_ok = false;
                self.newgame_notice = "No game to save.".to_string();
                self.newgame_page = NewgamePage::Saved;
                self.newgame_built = None;
                self.refresh();
                return;
            };
            session.save(&self.save_base, "default")
        };
        match saved {
            Ok(_) => {
                self.save_ok = true;
                self.newgame_notice = Session::list_saves(&self.save_base)
                    .into_iter()
                    .find(|row| row.slot == "default")
                    .map(|row| save_slot_label(&row))
                    .unwrap_or_else(|| "default  saved".to_string());
            }
            Err(err) => {
                self.save_ok = false;
                self.newgame_notice = err.to_string();
                if self.smoke {
                    self.smoke_ok = false;
                }
            }
        }
        self.newgame_page = NewgamePage::Saved;
        self.newgame_built = None;
        self.refresh();
    }

    fn run_newgame_smoke(&mut self) {
        self.newgame_checked = true;
        self.save_base = fresh_dir("portlight-newgame-smoke");
        let draft = CustomDraft {
            name: "Mara".to_string(),
            ..CustomDraft::default()
        };
        let spec = draft.spec();
        let mut session = match Session::new_custom(&spec, FIRST_PLAYABLE_SEED, None) {
            Ok(session) => session,
            Err(err) => {
                self.fail_newgame(format!("New game smoke: custom captain refused: {err}"));
                return;
            }
        };
        let silver = session.world().captain.silver;
        let day = session.world().day;
        if let Err(err) = session.save(&self.save_base, "voyage") {
            self.fail_newgame(format!("New game smoke: save failed: {err}"));
            return;
        }
        let slots = Session::list_saves(&self.save_base);
        let Some(row) = slots.iter().find(|row| row.slot == "voyage") else {
            self.fail_newgame("New game smoke: list_saves missed the voyage slot.");
            return;
        };
        if row.captain != "Mara" || row.day != day {
            self.fail_newgame(format!(
                "New game smoke: slot read {} day {}, expected Mara day {day}.",
                row.captain, row.day
            ));
            return;
        }
        match Session::load(&self.save_base, "voyage") {
            Ok(Some(loaded)) => {
                let world = loaded.world();
                if world.captain.name != "Mara"
                    || world.captain.captain_type != "custom"
                    || world.captain.silver != silver
                    || world.day != day
                    || world.voyage.destination_id != "porto_novo"
                {
                    self.fail_newgame(
                        "New game smoke: loaded game did not match the custom captain.".to_string(),
                    );
                    return;
                }
            }
            Ok(None) => {
                self.fail_newgame("New game smoke: voyage slot was missing on load.".to_string());
                return;
            }
            Err(err) => {
                self.fail_newgame(format!("New game smoke: load failed: {err}"));
                return;
            }
        }
        let text =
            std::fs::read_to_string(self.save_base.join("saves/voyage.json")).unwrap_or_default();
        if !text.contains("\"version\": 12") {
            self.fail_newgame("New game smoke: save was not version 12.".to_string());
            return;
        }
        match Session::new("Ada", "merchant", FIRST_PLAYABLE_SEED, None) {
            Ok(mut merchant) => {
                if let Err(err) = merchant.save(&self.save_base, "ada") {
                    self.fail_newgame(format!("New game smoke: merchant save failed: {err}"));
                    return;
                }
            }
            Err(err) => {
                self.fail_newgame(format!("New game smoke: merchant start failed: {err}"));
                return;
            }
        }
        if Session::list_saves(&self.save_base).len() < 2 {
            self.fail_newgame("New game smoke: expected two save slots.".to_string());
            return;
        }
        if Session::starting_captains().len() != 9 {
            self.fail_newgame("New game smoke: expected nine catalog captains.".to_string());
            return;
        }
        self.session = Some(session);
        self.push_log("New game smoke: custom captain saved and loaded.".to_string());
        self.refresh();
    }

    fn fail_newgame(&mut self, line: impl Into<String>) {
        let line = line.into();
        godot_print!("{line}");
        self.push_log(line);
        self.smoke_ok = false;
        self.newgame_checked = true;
    }

    fn begin_newgame_shots(&mut self) {
        self.resize_newgame_window();
        self.newgame_shot_dir = Some(newgame_shot_dir(self.shot_path.as_deref()));
        self.newgame_shot = Some(NewgameShot::Title);
        self.newgame_page = NewgamePage::Title;
        self.newgame_built = None;
        self.newgame_notice.clear();
        self.refresh();
    }

    fn advance_newgame_shot(&mut self) -> bool {
        let Some(phase) = self.newgame_shot else {
            return false;
        };
        let Some(dir) = self.newgame_shot_dir.clone() else {
            return false;
        };
        if self.newgame_page != phase.page() {
            self.smoke_ok = false;
            godot_print!("New game smoke: screen was not {}", phase.file_name());
        }
        let path = format!("{dir}/{}", phase.file_name());
        if !self.save_newgame_shot(&path) {
            self.capture_failed = true;
        }
        match phase {
            NewgameShot::Title => self.open_newgame(NewgamePage::Captains),
            NewgameShot::Captains => self.open_newgame(NewgamePage::Custom),
            NewgameShot::Custom => self.open_newgame(NewgamePage::Load),
            NewgameShot::Load => self.save_current_game(),
            NewgameShot::Save => {
                self.newgame_shot = None;
                return false;
            }
        }
        self.newgame_shot = Some(match phase {
            NewgameShot::Title => NewgameShot::Captains,
            NewgameShot::Captains => NewgameShot::Custom,
            NewgameShot::Custom => NewgameShot::Load,
            NewgameShot::Load => NewgameShot::Save,
            NewgameShot::Save => NewgameShot::Save,
        });
        self.capture_frames = 4;
        true
    }

    fn open_journal(&mut self) {
        if self.session.is_none() {
            return;
        }
        self.close_day_report();
        self.journal_open = true;
        self.journal_filled = false;
        self.refresh();
        self.scroll_journal_to(0);
    }

    fn close_journal(&mut self) {
        self.journal_open = false;
        self.refresh();
    }

    fn toggle_beat(&mut self, id: &str) {
        if !self.journal_open {
            return;
        }
        if let Some(index) = self.journal_expanded.iter().position(|have| have == id) {
            self.journal_expanded.remove(index);
        } else {
            self.journal_expanded.push(id.to_string());
        }
        self.journal_filled = false;
        self.refresh();
    }

    fn sync_journal(&mut self) {
        let open = self.journal_open;
        let Some(mut nodes) = self.journal_nodes.clone() else {
            return;
        };
        journal_screen::set_open(&mut nodes, open);
        if !open {
            return;
        }
        let template_id = self
            .session
            .as_ref()
            .and_then(|session| session.world().captain.ship.as_ref())
            .map(|ship| ship.template_id.clone())
            .unwrap_or_default();
        set_ship_plate(
            &mut nodes.plate,
            &mut nodes.plate_panel,
            &mut nodes.placeholder,
            &mut nodes.plate_caption,
            &template_id,
        );
        if self.journal_filled {
            return;
        }
        let game_id = self.instance_id();
        let kept_scroll = nodes.scroll.get_v_scroll();
        let Some(session) = self.session.as_ref() else {
            return;
        };
        let doc =
            journal_screen::journal_document(session, &self.log_lines, &self.journal_expanded);
        journal_screen::apply_document(&mut nodes, &doc, |id, label| {
            encounter_button(label, game_id, Action::ToggleBeat(id.to_string()))
        });
        nodes.scroll.set_v_scroll(kept_scroll);
        self.journal_filled = true;
    }

    fn scroll_journal_to(&mut self, y: i32) {
        if let Some(nodes) = self.journal_nodes.as_mut() {
            nodes.scroll.set_v_scroll(y.max(0));
        }
    }

    fn scroll_journal_anchor(&mut self, anchor: &str) {
        let Some(mut nodes) = self.journal_nodes.clone() else {
            return;
        };
        let section = match anchor {
            "victory" => nodes.victory.clone(),
            "memories" => nodes.memories.clone(),
            _ => nodes.chronicle.clone(),
        };
        journal_screen::scroll_to(&mut nodes.scroll, &section);
    }

    fn voyage_stamp(&self) -> Option<(i64, i64, VoyageStatus)> {
        self.session.as_ref().map(|session| {
            let world = session.world();
            (world.day, world.voyage.progress, world.voyage.status)
        })
    }

    fn run_journal_checks(&mut self) {
        self.journal_checked = true;
        let enabled = self
            .journal_button
            .as_ref()
            .is_some_and(|button| !button.is_disabled());
        if !enabled {
            self.fail_journal("Journal smoke: Journal button unavailable in port.");
        }
        let before = self.voyage_stamp();
        self.open_journal();
        if self.voyage_stamp() != before {
            self.fail_journal("Journal smoke: opening the journal changed the voyage.");
        }
        let expected = self
            .session
            .as_ref()
            .map(|session| {
                let doc = journal_screen::journal_document(
                    session,
                    &self.log_lines,
                    &self.journal_expanded,
                );
                journal_screen::document_lines(&doc)
            })
            .unwrap_or_default();
        let shown = self
            .journal_nodes
            .as_ref()
            .map(|nodes| journal_screen::overlay_text(&nodes.root))
            .unwrap_or_default();
        if !shown.is_ascii() {
            self.fail_journal("Journal smoke: overlay text is not ASCII.");
        }
        for line in &expected {
            if !shown.contains(line.as_str()) {
                self.fail_journal(format!("Journal smoke: missing {line}"));
            }
        }
        if shown.contains("Ledger:") {
            self.fail_journal("Journal smoke: ledger was merged into the journal.");
        }
        if !shown.contains("Close") {
            self.fail_journal("Journal smoke: Close is missing.");
        }
        self.close_journal();
    }

    fn journal_sea_check(&mut self) {
        let before_day = self.session.as_ref().map(|session| session.world().day);
        let departed = {
            let Some(session) = self.session.as_mut() else {
                self.fail_journal("Journal smoke: no session.");
                return;
            };
            session.depart("al_manar")
        };
        if let Err(err) = departed {
            self.fail_journal(format!("Journal smoke: could not leave port: {err}"));
            return;
        }
        self.refresh();
        let enabled = self
            .journal_button
            .as_ref()
            .is_some_and(|button| !button.is_disabled() && button.is_visible());
        if !enabled {
            self.fail_journal("Journal smoke: Journal button unavailable at sea.");
        }
        let at_sea = self
            .session
            .as_ref()
            .is_some_and(|session| session.world().voyage.status == VoyageStatus::AtSea);
        if !at_sea {
            self.fail_journal("Journal smoke: expected to be at sea.");
        }
        let underway = self.voyage_stamp();
        self.open_journal();
        let notice = self
            .journal_nodes
            .as_ref()
            .map(|nodes| nodes.notice.get_text().to_string())
            .unwrap_or_default();
        if !notice.contains("At sea") {
            self.fail_journal(format!("Journal smoke: notice was not at sea: {notice}"));
        }
        if self.voyage_stamp() != underway {
            self.fail_journal("Journal smoke: opening the journal changed the voyage.");
        }
        if before_day != self.session.as_ref().map(|session| session.world().day) {
            self.fail_journal("Journal smoke: the day moved.");
        }
        self.close_journal();
    }

    fn fail_journal(&mut self, line: impl Into<String>) {
        let line = line.into();
        godot_print!("{line}");
        self.push_log(line);
        self.smoke_ok = false;
        self.journal_checked = true;
    }

    fn begin_journal_shots(&mut self) {
        self.journal_shot_dir = Some(newgame_shot_dir(self.shot_path.as_deref()));
        self.journal_shot = Some(JournalShot::Chronicle);
        self.journal_open = true;
        self.journal_filled = false;
        self.journal_expanded.clear();
        self.refresh();
        self.scroll_journal_to(0);
    }

    fn advance_journal_shot(&mut self) -> bool {
        let Some(phase) = self.journal_shot else {
            return false;
        };
        let Some(dir) = self.journal_shot_dir.clone() else {
            return false;
        };
        if !self.journal_open {
            self.fail_journal("Journal smoke: overlay was closed before the frame.");
        }
        if !headless_runtime() && !self.journal_frame_ready(phase) {
            self.fail_journal(format!("Journal smoke: {} was not in view.", phase.label()));
        }
        let path = format!("{dir}/{}", phase.file_name());
        if !self.save_shot(&path, true) {
            self.capture_failed = true;
        }
        match phase {
            JournalShot::Chronicle => {
                self.scroll_journal_anchor("victory");
                self.journal_shot = Some(JournalShot::Victory);
            }
            JournalShot::Victory => {
                self.scroll_journal_anchor("memories");
                self.journal_shot = Some(JournalShot::Memories);
            }
            JournalShot::Memories => {
                self.close_journal();
                self.journal_shot = None;
                if self.journal_sea_pending {
                    self.capture_frames = 2;
                    return true;
                }
                return false;
            }
        }
        self.capture_frames = 4;
        true
    }

    fn journal_frame_ready(&mut self, phase: JournalShot) -> bool {
        let Some(nodes) = self.journal_nodes.as_ref() else {
            return false;
        };
        let (anchor, needle) = match phase {
            JournalShot::Chronicle => (&nodes.chronicle, "No chronicle entries yet."),
            JournalShot::Victory => (&nodes.victory, "["),
            JournalShot::Memories => (&nodes.memories, "No captains remembered yet."),
        };
        journal_screen::section_visible(&nodes.scroll, anchor)
            && journal_screen::line_visible(&nodes.scroll, anchor, needle)
    }

    fn begin_harbour_shots(&mut self) {
        self.harbour_shot_dir = Some(newgame_shot_dir(self.shot_path.as_deref()));
        self.harbour_shot = Some(HarbourShot::Warehouse);
        self.open_harbour();
    }

    fn advance_harbour_shot(&mut self) -> bool {
        let Some(phase) = self.harbour_shot else {
            return false;
        };
        let Some(dir) = self.harbour_shot_dir.clone() else {
            return false;
        };
        if !self.harbour_open {
            self.smoke_ok = false;
            godot_print!(
                "Harbour smoke: screen was not open for {}",
                phase.file_name()
            );
        }
        self.scroll_harbour(phase.anchor());
        let path = format!("{dir}/{}", phase.file_name());
        if !self.save_shot(&path, true) {
            self.capture_failed = true;
        }
        let next = match phase {
            HarbourShot::Warehouse => Some(HarbourShot::Broker),
            HarbourShot::Broker => Some(HarbourShot::Finance),
            HarbourShot::Finance => None,
        };
        if let Some(next) = next {
            self.scroll_harbour(next.anchor());
            self.harbour_shot = Some(next);
            self.capture_frames = 4;
            true
        } else {
            self.harbour_shot = None;
            false
        }
    }

    fn scroll_harbour(&self, anchor: &str) {
        let Some(nodes) = self.harbour_nodes.as_ref() else {
            return;
        };
        let Some(control) = named_child(&nodes.body, anchor) else {
            return;
        };
        // Put the section at the top. `ensure_control_visible` only scrolls
        // until the header peeks in, which left Broker and Credit on the
        // bottom edge of the frame.
        let y = control.get_position().y.round().max(0.0) as i32;
        let mut scroll = nodes.scroll.clone();
        scroll.set_v_scroll(y);
    }

    fn resize_newgame_window(&mut self) {
        let Some(mut window) = self.base().get_window() else {
            return;
        };
        let size = Vector2i::new(NEWGAME_SHOT_W, NEWGAME_SHOT_H);
        window.set_content_scale_size(size);
        window.set_size(size);
    }

    fn sync_newgame(&mut self) {
        let Some(mut nodes) = self.newgame_nodes.clone() else {
            return;
        };
        newgame_screen::show_page(&mut nodes, self.newgame_page);
        if self.newgame_page == NewgamePage::Hidden {
            return;
        }
        let ship = self.newgame_ship_id();
        if self.newgame_page == NewgamePage::Saved {
            nodes
                .confirm_title
                .set_text(save_confirm_title(self.save_ok));
            nodes.confirm_detail.set_text(&self.newgame_notice);
            set_ship_plate(
                &mut nodes.confirm_plate,
                &mut nodes.confirm_plate_panel,
                &mut nodes.confirm_placeholder,
                &mut nodes.confirm_caption,
                &ship,
            );
            if self.newgame_built != Some(NewgamePage::Saved) {
                self.fill_saved_actions();
                self.newgame_built = Some(NewgamePage::Saved);
            }
            return;
        }
        let (title, card) = newgame_copy(self.newgame_page);
        nodes.title.set_text(title);
        nodes.card.set_text(card);
        nodes.detail.set_text(&self.menu_detail());
        set_ship_plate(
            &mut nodes.plate,
            &mut nodes.plate_panel,
            &mut nodes.placeholder,
            &mut nodes.plate_caption,
            &ship,
        );
        if self.newgame_built != Some(self.newgame_page) {
            self.fill_newgame_actions();
            self.newgame_built = Some(self.newgame_page);
        }
    }

    fn menu_detail(&self) -> String {
        let mut lines = Vec::new();
        if self.newgame_page == NewgamePage::Custom {
            lines.push(format!("Points left: {}", self.draft.points_left()));
        }
        if !self.newgame_notice.is_empty() {
            lines.push(self.newgame_notice.clone());
        }
        lines.join("\n")
    }

    fn newgame_ship_id(&self) -> String {
        if self.newgame_page == NewgamePage::Saved {
            if let Some(id) = self.session.as_ref().and_then(|session| {
                session
                    .world()
                    .captain
                    .ship
                    .as_ref()
                    .map(|ship| ship.template_id.clone())
            }) {
                return id;
            }
        }
        "coastal_sloop".to_string()
    }

    fn fill_saved_actions(&mut self) {
        let Some(mut actions) = self
            .newgame_nodes
            .as_ref()
            .map(|nodes| nodes.confirm_actions.clone())
        else {
            return;
        };
        clear_children(&mut actions);
        let game_id = self.instance_id();
        actions.add_child(&encounter_button(
            "Back to the chart",
            game_id,
            Action::NewgameBack,
        ));
    }

    fn fill_newgame_actions(&mut self) {
        self.name_edit = None;
        self.title_edit = None;
        self.story_edit = None;
        let Some(mut actions) = self
            .newgame_nodes
            .as_ref()
            .map(|nodes| nodes.actions.clone())
        else {
            return;
        };
        clear_children(&mut actions);
        let game_id = self.instance_id();
        match self.newgame_page {
            NewgamePage::Title => {
                actions.add_child(&encounter_button("New game", game_id, Action::OpenCaptains));
                actions.add_child(&encounter_button("Load game", game_id, Action::OpenLoad));
                if self.session.is_some() {
                    actions.add_child(&encounter_button(
                        "Back to the chart",
                        game_id,
                        Action::NewgameBack,
                    ));
                }
            }
            NewgamePage::Captains => {
                self.add_draft_edit(
                    &mut actions,
                    "Name",
                    &self.draft.name.clone(),
                    DraftEdit::Name,
                );
                for captain in Session::starting_captains() {
                    let label = captain_button_label(&captain);
                    actions.add_child(&encounter_button(
                        &label,
                        game_id,
                        Action::StartCaptain(captain.id),
                    ));
                }
                actions.add_child(&encounter_button(
                    "Custom captain",
                    game_id,
                    Action::OpenCustom,
                ));
                actions.add_child(&encounter_button("Back", game_id, Action::NewgameBack));
            }
            NewgamePage::Custom => self.fill_custom(&mut actions, game_id),
            NewgamePage::Load => {
                let slots = Session::list_saves(&self.save_base);
                if slots.is_empty() {
                    actions.add_child(&body_label("No saves found.", 16, CREAM));
                }
                for slot in slots {
                    let label = save_slot_label(&slot);
                    actions.add_child(&encounter_button(
                        &label,
                        game_id,
                        Action::LoadSlot(slot.slot),
                    ));
                }
                actions.add_child(&encounter_button("Back", game_id, Action::NewgameBack));
            }
            NewgamePage::Hidden | NewgamePage::Saved => {}
        }
    }

    fn fill_custom(&mut self, actions: &mut Gd<VBoxContainer>, game_id: InstanceId) {
        let name = self.draft.name.clone();
        let title = self.draft.title.clone();
        let story = self.draft.backstory.clone();
        self.add_draft_edit(actions, "Name", &name, DraftEdit::Name);
        self.add_draft_edit(actions, "Title", &title, DraftEdit::Title);
        for pool in [
            PointPool::Trade,
            PointPool::Sailing,
            PointPool::Shadow,
            PointPool::Reputation,
        ] {
            let mut row = HBoxContainer::new_alloc();
            row.add_theme_constant_override("separation", 8);
            row.add_child(&body_label(
                &format!("{}  {}", pool_name(pool), self.draft.pool(pool)),
                16,
                CREAM,
            ));
            row.add_child(&encounter_button(
                &format!("{} -", pool_name(pool)),
                game_id,
                Action::AdjustPoints(pool, -1),
            ));
            row.add_child(&encounter_button(
                &format!("{} +", pool_name(pool)),
                game_id,
                Action::AdjustPoints(pool, 1),
            ));
            actions.add_child(&row);
        }
        let options = Session::custom_captain_options();
        self.choice_row(
            actions,
            game_id,
            &format!("Region: {}", self.draft.home_region),
            Action::CycleRegion(-1),
            Action::CycleRegion(1),
            "Region",
        );
        let port_name = options
            .ports
            .iter()
            .find(|port| port.id == self.draft.home_port_id)
            .map(|port| ascii_label(&port.name, &port.id).to_string())
            .unwrap_or_else(|| self.draft.home_port_id.clone());
        self.choice_row(
            actions,
            game_id,
            &format!("Home port: {port_name}"),
            Action::CyclePort(-1),
            Action::CyclePort(1),
            "Port",
        );
        let bloc = named_or_none(&options.blocs, &self.draft.bloc_alignment);
        self.choice_row(
            actions,
            game_id,
            &format!("Trade bloc: {bloc}"),
            Action::CycleBloc(-1),
            Action::CycleBloc(1),
            "Bloc",
        );
        let faction = named_or_none(&options.factions, &self.draft.faction_alignment);
        self.choice_row(
            actions,
            game_id,
            &format!("Pirate faction: {faction}"),
            Action::CycleFaction(-1),
            Action::CycleFaction(1),
            "Faction",
        );
        let mentor = mentor_or_none(&options.mentors, &self.draft.mentor_npc_id);
        self.choice_row(
            actions,
            game_id,
            &format!("Mentor: {mentor}"),
            Action::CycleMentor(-1),
            Action::CycleMentor(1),
            "Mentor",
        );
        self.add_draft_edit(actions, "Backstory", &story, DraftEdit::Story);
        actions.add_child(&encounter_button(
            "Begin voyage",
            game_id,
            Action::BeginCustom,
        ));
        actions.add_child(&encounter_button("Back", game_id, Action::NewgameBack));
    }

    fn choice_row(
        &self,
        actions: &mut Gd<VBoxContainer>,
        game_id: InstanceId,
        label: &str,
        down: Action,
        up: Action,
        name: &str,
    ) {
        let mut row = HBoxContainer::new_alloc();
        row.add_theme_constant_override("separation", 8);
        let mut text = body_label(label, 16, CREAM);
        text.set_h_size_flags(SizeFlags::EXPAND_FILL);
        row.add_child(&text);
        row.add_child(&encounter_button(&format!("{name} -"), game_id, down));
        row.add_child(&encounter_button(&format!("{name} +"), game_id, up));
        actions.add_child(&row);
    }

    fn add_draft_edit(
        &mut self,
        actions: &mut Gd<VBoxContainer>,
        label: &str,
        value: &str,
        kind: DraftEdit,
    ) {
        actions.add_child(&body_label(label, 14, MUTED));
        let mut edit = LineEdit::new_alloc();
        edit.set_text(value);
        newgame_screen::style_field(&mut edit);
        actions.add_child(&edit);
        match kind {
            DraftEdit::Name => self.name_edit = Some(edit),
            DraftEdit::Title => self.title_edit = Some(edit),
            DraftEdit::Story => self.story_edit = Some(edit),
        }
    }

    fn open_shipyard(&mut self) {
        if self.docked_id().is_none() {
            self.push_log("Shipyard opens from a dock.".to_string());
            return;
        }
        self.close_day_report();
        self.shipyard_open = true;
        self.play_sfx("sfx_ui_port_open");
        self.shipyard_confirm = None;
        self.shipyard_notice.clear();
        self.shipyard_shown = None;
        self.rename_draft.clear();
        self.refresh();
    }

    fn close_shipyard(&mut self) {
        self.read_rename_field();
        self.shipyard_open = false;
        self.shipyard_confirm = None;
        self.refresh();
    }

    fn shipyard_repair(&mut self) {
        self.shipyard_confirm = None;
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.repair(None)
        };
        self.shipyard_notice = match result {
            Ok((points, cost)) => format!("Restored {points} hull for {cost} silver."),
            Err(err) => ui_sentence(&err.to_string()),
        };
        self.refresh();
    }

    fn shipyard_rename(&mut self) {
        self.read_rename_field();
        self.shipyard_confirm = None;
        let draft = self.rename_draft.clone();
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.rename_ship(&draft, None)
        };
        self.shipyard_notice = match result {
            Ok(()) => {
                let stored = self
                    .session
                    .as_ref()
                    .and_then(|session| session.world().captain.ship.as_ref())
                    .map(|ship| ship.name.clone())
                    .unwrap_or(draft);
                format!("Renamed to {stored}.")
            }
            Err(err) => ui_sentence(&err.to_string()),
        };
        self.refresh();
    }

    fn arm_shipyard(&mut self, arm: ShipyardArm) {
        let Some(model) = self.session.as_ref().and_then(shipyard_model) else {
            return;
        };
        self.shipyard_notice = match &arm {
            ShipyardArm::Buy(id) => model
                .offers
                .iter()
                .find(|offer| offer.id == *id)
                .map(|offer| buy_confirm_line(offer, model.buying_sells_flagship))
                .unwrap_or_else(|| format!("Buy {id}?")),
            ShipyardArm::Install(id) => model
                .upgrades
                .iter()
                .find(|upgrade| upgrade.id == *id)
                .map(|upgrade| install_confirm_line(&upgrade.name, upgrade.price))
                .unwrap_or_else(|| format!("Install {id}?")),
            ShipyardArm::Sell(name) => sell_confirm_line(name),
            ShipyardArm::Dock => dock_confirm_line().to_string(),
            ShipyardArm::Board(name) => board_confirm_line(name),
        };
        self.shipyard_confirm = Some(arm);
        self.refresh();
    }

    fn cancel_shipyard(&mut self) {
        self.shipyard_confirm = None;
        self.shipyard_notice.clear();
        self.refresh();
    }

    fn confirm_shipyard(&mut self) {
        let Some(arm) = self.shipyard_confirm.clone() else {
            return;
        };
        self.shipyard_notice = match arm {
            ShipyardArm::Buy(id) => self.buy_hull(&id),
            ShipyardArm::Install(id) => self.install_hull_upgrade(&id),
            ShipyardArm::Sell(name) => self.sell_docked_hull(&name),
            ShipyardArm::Dock => self.dock_flagship(),
            ShipyardArm::Board(name) => self.board_docked_hull(&name),
        };
        self.shipyard_confirm = None;
        self.refresh();
    }

    fn buy_hull(&mut self, ship_id: &str) -> String {
        let (previous_name, fleet_before) = {
            let Some(session) = self.session.as_ref() else {
                return "No game".to_string();
            };
            let world = session.world();
            (
                world
                    .captain
                    .ship
                    .as_ref()
                    .map(|ship| ship.name.clone())
                    .unwrap_or_default(),
                world.captain.fleet.len(),
            )
        };
        let result = {
            let Some(session) = self.session.as_mut() else {
                return "No game".to_string();
            };
            session.buy_ship(ship_id)
        };
        match result {
            Ok(()) => {
                let Some(session) = self.session.as_ref() else {
                    return "No game".to_string();
                };
                let world = session.world();
                let bought = world
                    .captain
                    .ship
                    .as_ref()
                    .map(|ship| ship.name.clone())
                    .unwrap_or_else(|| ship_id.to_string());
                buy_result_line(
                    &bought,
                    &previous_name,
                    world.captain.fleet.len() > fleet_before,
                )
            }
            Err(err) => ui_sentence(&err.to_string()),
        }
    }

    fn install_hull_upgrade(&mut self, upgrade_id: &str) -> String {
        let name = portlight_sim::content::content()
            .upgrade(upgrade_id)
            .map(|upgrade| ascii_label(&upgrade.name, upgrade_id).to_string())
            .unwrap_or_else(|| upgrade_id.to_string());
        let result = {
            let Some(session) = self.session.as_mut() else {
                return "No game".to_string();
            };
            session.install_upgrade(upgrade_id)
        };
        match result {
            Ok(()) => format!("Installed {name}."),
            Err(err) => ui_sentence(&err.to_string()),
        }
    }

    fn sell_docked_hull(&mut self, name: &str) -> String {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return "No game".to_string();
            };
            session.sell_fleet_ship(name)
        };
        match result {
            Ok((silver, sold)) => format!("Sold {sold} for {silver} silver."),
            Err(err) => ui_sentence(&err.to_string()),
        }
    }

    fn dock_flagship(&mut self) -> String {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return "No game".to_string();
            };
            session.dock_current_ship()
        };
        match result {
            Ok(()) => self.sailing_notice(),
            Err(err) => ui_sentence(&err.to_string()),
        }
    }

    fn board_docked_hull(&mut self, name: &str) -> String {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return "No game".to_string();
            };
            session.board_fleet_ship(name)
        };
        match result {
            Ok(()) => self.sailing_notice(),
            Err(err) => ui_sentence(&err.to_string()),
        }
    }

    fn sailing_notice(&self) -> String {
        self.session
            .as_ref()
            .and_then(|session| session.world().captain.ship.as_ref())
            .map(|ship| format!("Now sailing {}.", ship.name))
            .unwrap_or_else(|| "Now sailing.".to_string())
    }

    fn read_rename_field(&mut self) {
        if let Some(edit) = self.rename_edit.as_ref() {
            self.rename_draft = edit.get_text().to_string();
        }
    }

    fn set_rename_text(&mut self, text: &str) {
        self.rename_draft = text.to_string();
        if let Some(mut edit) = self.rename_edit.clone() {
            edit.set_text(text);
        }
    }

    fn sync_shipyard(&mut self) {
        let Some(mut nodes) = self.shipyard_nodes.clone() else {
            return;
        };
        let model = self.session.as_ref().and_then(shipyard_model);
        let open = self.shipyard_open && model.is_some();
        nodes.root.set_visible(open);
        if !open {
            if self.shipyard_open {
                self.shipyard_open = false;
            }
            return;
        }
        let model = model.expect("open shipyard has a model");
        if self.shipyard_notice.is_empty() && self.shipyard_confirm.is_none() {
            self.shipyard_notice.clone_from(&model.gate_notice);
        }
        nodes.title.set_text(&model.port_name);
        nodes.notice.set_text(&self.shipyard_notice);
        set_ship_plate(
            &mut nodes.plate,
            &mut nodes.plate_panel,
            &mut nodes.placeholder,
            &mut nodes.plate_caption,
            &model.flagship.template_id,
        );
        self.read_rename_field();
        if self.shipyard_shown.as_ref() != Some(&model) {
            self.fill_shipyard_body(&model);
            self.shipyard_shown = Some(model);
        }
        self.sync_shipyard_confirm();
    }

    fn sync_shipyard_confirm(&mut self) {
        let Some(mut confirm) = self
            .shipyard_nodes
            .as_ref()
            .map(|nodes| nodes.confirm.clone())
        else {
            return;
        };
        clear_hbox(&mut confirm);
        if self.shipyard_confirm.is_none() {
            confirm.set_visible(false);
            return;
        }
        confirm.set_visible(true);
        let game_id = self.instance_id();
        confirm.add_child(&encounter_button(
            "Confirm",
            game_id,
            Action::ShipyardConfirm,
        ));
        confirm.add_child(&encounter_button("Cancel", game_id, Action::ShipyardCancel));
    }

    fn fill_shipyard_body(&mut self, model: &ShipyardModel) {
        self.rename_edit = None;
        self.yard_mark = None;
        self.fleet_mark = None;
        let Some(mut body) = self.shipyard_nodes.as_ref().map(|nodes| nodes.body.clone()) else {
            return;
        };
        clear_children(&mut body);
        let game_id = self.instance_id();
        body.add_child(&body_label("Flagship", 16, GOLD));
        for line in model.flagship.lines(model.silver, &model.fleet_label) {
            let mut label = body_label(&line, 16, CREAM);
            label.set_autowrap_mode(AutowrapMode::WORD_SMART);
            body.add_child(&label);
        }
        body.add_child(&encounter_button("Repair", game_id, Action::ShipyardRepair));
        body.add_child(&body_label("Rename", 14, MUTED));
        let mut edit = LineEdit::new_alloc();
        edit.set_text(&self.rename_draft);
        edit.set_placeholder("New name");
        edit.set_max_length(30);
        newgame_screen::style_field(&mut edit);
        body.add_child(&edit);
        self.rename_edit = Some(edit);
        body.add_child(&encounter_button(
            "Confirm rename",
            game_id,
            Action::ShipyardRename,
        ));

        let yard = body_label("Yard", 16, GOLD);
        self.yard_mark = Some(yard.clone().upcast());
        body.add_child(&yard);
        if !model.has_shipyard {
            let mut gated = body_label(NO_SHIPYARD_BODY, 16, CREAM);
            gated.set_autowrap_mode(AutowrapMode::WORD_SMART);
            body.add_child(&gated);
        }
        body.add_child(&body_label("Buy hull", 14, GOLD));
        for offer in &model.offers {
            let mut block = VBoxContainer::new_alloc();
            block.add_theme_constant_override("separation", 4);
            let mut label = body_label(&offer.line(), 16, CREAM);
            label.set_autowrap_mode(AutowrapMode::WORD_SMART);
            block.add_child(&label);
            let mut buy = encounter_button(
                "Buy",
                game_id,
                Action::ShipyardArm(ShipyardArm::Buy(offer.id.clone())),
            );
            buy.set_disabled(!model.has_shipyard);
            block.add_child(&buy);
            body.add_child(&block);
        }
        body.add_child(&body_label("Install upgrade", 14, GOLD));
        if !model.slots_notice.is_empty() {
            let mut full = body_label(&model.slots_notice, 16, CREAM);
            full.set_autowrap_mode(AutowrapMode::WORD_SMART);
            body.add_child(&full);
        }
        for upgrade in &model.upgrades {
            let mut block = VBoxContainer::new_alloc();
            block.add_theme_constant_override("separation", 4);
            let mut label = body_label(&upgrade.line(), 16, CREAM);
            label.set_autowrap_mode(AutowrapMode::WORD_SMART);
            block.add_child(&label);
            let mut install = encounter_button(
                "Install",
                game_id,
                Action::ShipyardArm(ShipyardArm::Install(upgrade.id.clone())),
            );
            install.set_disabled(!model.has_shipyard || !model.slots_notice.is_empty());
            block.add_child(&install);
            body.add_child(&block);
        }

        let fleet = body_label("Fleet here", 16, GOLD);
        self.fleet_mark = Some(fleet.clone().upcast());
        body.add_child(&fleet);
        body.add_child(&body_label(&model.fleet_label, 16, CREAM));
        if model.fleet.is_empty() {
            body.add_child(&body_label(NO_FLEET_HERE, 16, CREAM));
        }
        for owned in &model.fleet {
            let mut block = VBoxContainer::new_alloc();
            block.add_theme_constant_override("separation", 4);
            let mut label = body_label(&owned.line(), 16, CREAM);
            label.set_autowrap_mode(AutowrapMode::WORD_SMART);
            block.add_child(&label);
            if owned.cargo {
                block.add_child(&body_label("Ship has cargo - transfer it first", 14, MUTED));
            }
            let mut buttons = HBoxContainer::new_alloc();
            buttons.add_theme_constant_override("separation", 8);
            buttons.add_child(&encounter_button(
                "Board",
                game_id,
                Action::ShipyardArm(ShipyardArm::Board(owned.name.clone())),
            ));
            let mut sell = encounter_button(
                "Sell",
                game_id,
                Action::ShipyardArm(ShipyardArm::Sell(owned.name.clone())),
            );
            sell.set_disabled(!model.has_shipyard || owned.cargo);
            buttons.add_child(&sell);
            block.add_child(&buttons);
            body.add_child(&block);
        }
        body.add_child(&encounter_button(
            "Dock flagship",
            game_id,
            Action::ShipyardArm(ShipyardArm::Dock),
        ));
        // Lets the fleet header scroll to the top of the viewport. Without it
        // the last rows stay pinned to the bottom under the upgrade list.
        let mut tail = Control::new_alloc();
        tail.set_mouse_filter(godot::classes::control::MouseFilter::IGNORE);
        tail.set_custom_minimum_size(Vector2::new(0.0, 480.0));
        body.add_child(&tail);
    }

    fn run_shipyard_actions(&mut self) {
        self.shipyard_repair_and_rename();
        self.shipyard_buy_and_install();
        self.shipyard_dock_board_sell();
        self.close_shipyard();
        self.finish_shipyard_smoke();
    }

    fn begin_shipyard_shots(&mut self) {
        self.shipyard_shot_dir = Some(newgame_shot_dir(self.shot_path.as_deref()));
        self.shipyard_shot = Some(ShipyardShot::Flagship);
        self.shipyard_scroll = None;
    }

    fn advance_shipyard_shot(&mut self) -> bool {
        let Some(phase) = self.shipyard_shot else {
            return false;
        };
        let Some(dir) = self.shipyard_shot_dir.clone() else {
            return false;
        };
        if !self.shipyard_open {
            self.fail_shipyard(format!(
                "Shipyard smoke: screen was closed before {}.",
                phase.file_name()
            ));
        }
        let path = format!("{dir}/{}", phase.file_name());
        if !self.save_shipyard_shot(&path) {
            self.capture_failed = true;
        }
        match phase {
            ShipyardShot::Flagship => {
                self.shipyard_repair_and_rename();
                self.shipyard_scroll = Some(ShipyardSection::Yard);
                self.shipyard_shot = Some(ShipyardShot::Yard);
            }
            ShipyardShot::Yard => {
                self.shipyard_buy_and_install();
                self.shipyard_scroll = Some(ShipyardSection::Fleet);
                self.shipyard_shot = Some(ShipyardShot::Fleet);
            }
            ShipyardShot::Fleet => {
                self.shipyard_dock_board_sell();
                self.close_shipyard();
                self.finish_shipyard_smoke();
                self.shipyard_shot = None;
                return false;
            }
        }
        self.capture_frames = 4;
        true
    }

    fn apply_shipyard_scroll(&mut self) {
        let Some(section) = self.shipyard_scroll.take() else {
            return;
        };
        let Some(mut scroll) = self
            .shipyard_nodes
            .as_ref()
            .map(|nodes| nodes.scroll.clone())
        else {
            return;
        };
        let mark = match section {
            ShipyardSection::Yard => self.yard_mark.clone(),
            ShipyardSection::Fleet => self.fleet_mark.clone(),
        };
        let Some(mark) = mark else {
            return;
        };
        let y = mark.get_position().y.round().max(0.0) as i32;
        scroll.set_v_scroll(y);
    }

    fn shipyard_repair_and_rename(&mut self) {
        let original = self
            .session
            .as_ref()
            .and_then(|session| session.world().captain.ship.as_ref())
            .map(|ship| ship.name.clone())
            .unwrap_or_default();
        self.perform(Action::ShipyardRepair);
        let notice = self.shipyard_notice.clone();
        if notice != "Ship is already in perfect condition" && !notice.starts_with("Restored ") {
            self.fail_shipyard(format!("Shipyard smoke: repair returned `{notice}`."));
        }
        if self.rename_edit.is_none() {
            self.fail_shipyard("Shipyard smoke: rename field missing.".to_string());
            return;
        }
        self.set_rename_text("Harbor Sloop");
        self.perform(Action::ShipyardRename);
        let renamed = self.flagship_name();
        if renamed != "Harbor Sloop" {
            self.fail_shipyard(format!("Shipyard smoke: rename landed on `{renamed}`."));
        }
        self.set_rename_text(&original);
        self.perform(Action::ShipyardRename);
        let restored = self.flagship_name();
        if restored != original {
            self.fail_shipyard(format!(
                "Shipyard smoke: rename did not restore `{original}` (got `{restored}`)."
            ));
        }
    }

    fn shipyard_buy_and_install(&mut self) {
        self.perform(Action::ShipyardArm(ShipyardArm::Buy(
            "swift_cutter".to_string(),
        )));
        let ask = self.shipyard_notice.clone();
        if !ask.contains("Buy Swift Cutter for 450 silver?") || ask.contains("Fleet is full") {
            self.fail_shipyard(format!("Shipyard smoke: buy confirm was `{ask}`."));
        }
        self.perform(Action::ShipyardConfirm);
        let bought = self.shipyard_notice.clone();
        if !bought.contains("Coastal Sloop is docked here") {
            self.fail_shipyard(format!("Shipyard smoke: buy result was `{bought}`."));
        }
        if self.flagship_template() != "swift_cutter" {
            self.fail_shipyard("Shipyard smoke: flagship was not the cutter.".to_string());
        }
        self.perform(Action::ShipyardArm(ShipyardArm::Install(
            "iron_strapping".to_string(),
        )));
        self.perform(Action::ShipyardConfirm);
        let installed = self.shipyard_notice.clone();
        if installed != "Installed Iron Strapping." {
            self.fail_shipyard(format!("Shipyard smoke: install result was `{installed}`."));
        }
    }

    fn shipyard_dock_board_sell(&mut self) {
        self.perform(Action::ShipyardArm(ShipyardArm::Dock));
        self.perform(Action::ShipyardConfirm);
        if self.flagship_template() != "coastal_sloop" {
            self.fail_shipyard(format!(
                "Shipyard smoke: dock left the flagship as {}.",
                self.flagship_template()
            ));
        }
        self.perform(Action::ShipyardArm(ShipyardArm::Board(
            "Swift Cutter".to_string(),
        )));
        self.perform(Action::ShipyardConfirm);
        if self.flagship_template() != "swift_cutter" {
            self.fail_shipyard(format!(
                "Shipyard smoke: board left the flagship as {}.",
                self.flagship_template()
            ));
        }
        self.perform(Action::ShipyardArm(ShipyardArm::Sell(
            "Coastal Sloop".to_string(),
        )));
        self.perform(Action::ShipyardConfirm);
        let sold = self.shipyard_notice.clone();
        if !sold.starts_with("Sold Coastal Sloop for ") {
            self.fail_shipyard(format!("Shipyard smoke: sell result was `{sold}`."));
        }
    }

    fn finish_shipyard_smoke(&mut self) {
        let visible = self
            .shipyard_button
            .as_ref()
            .is_some_and(|button| button.is_visible_in_tree());
        if !visible {
            self.fail_shipyard("Shipyard smoke: docked chart hid the Shipyard button.".to_string());
        }
        if self.shipyard_open {
            self.fail_shipyard("Shipyard smoke: screen stayed open.".to_string());
        }
        let Some(session) = self.session.as_ref() else {
            self.fail_shipyard("Shipyard smoke: no session.".to_string());
            return;
        };
        let world = session.world();
        let day = world.day;
        let port = if world.voyage.status == VoyageStatus::InPort {
            Some(world.voyage.destination_id.clone())
        } else {
            None
        };
        let ship = world.captain.ship.as_ref();
        let on_cutter = ship.is_some_and(|ship| ship.template_id == "swift_cutter");
        let strapped = ship.is_some_and(|ship| {
            ship.upgrades
                .iter()
                .any(|upgrade| upgrade.upgrade_id == "iron_strapping")
        });
        let fleet_empty = world.captain.fleet.is_empty();
        let silver = world.captain.silver;
        if day != self.shipyard_day {
            self.fail_shipyard(format!(
                "Shipyard smoke: day moved from {} to {day}.",
                self.shipyard_day
            ));
        }
        if port.as_deref() != Some("porto_novo") {
            self.fail_shipyard("Shipyard smoke: left Porto Novo.".to_string());
        }
        if !on_cutter {
            self.fail_shipyard("Shipyard smoke: expected to end on the cutter.".to_string());
        }
        if !strapped {
            self.fail_shipyard("Shipyard smoke: iron strapping was not fitted.".to_string());
        }
        if !fleet_empty {
            self.fail_shipyard("Shipyard smoke: the sold hull is still in the fleet.".to_string());
        }
        if silver != 0 {
            self.fail_shipyard(format!(
                "Shipyard smoke: expected 0 silver after the cutter and the strapping, have {silver}."
            ));
        }
    }

    fn flagship_name(&self) -> String {
        self.session
            .as_ref()
            .and_then(|session| session.world().captain.ship.as_ref())
            .map(|ship| ship.name.clone())
            .unwrap_or_default()
    }

    fn flagship_template(&self) -> String {
        self.session
            .as_ref()
            .and_then(|session| session.world().captain.ship.as_ref())
            .map(|ship| ship.template_id.clone())
            .unwrap_or_default()
    }

    fn fail_shipyard(&mut self, line: impl Into<String>) {
        let line = line.into();
        godot_print!("{line}");
        self.push_log(line);
        self.smoke_ok = false;
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

    /// Docked Porto Novo. Lease, deposit, credit, and insurance stay on Session.
    /// Next day must keep at least one `Turn.notes` line. At sea the Harbour
    /// button goes with the port row. Capture skips the depart so the frames
    /// stay on the desk.
    fn run_harbour_smoke(&mut self, capture: bool) {
        self.harbour_checked = true;
        if self.docked_id() != Some("porto_novo") {
            self.fail_harbour("Harbour smoke: expected to be docked at Porto Novo.");
            return;
        }
        if !self.harbour_button_shown() {
            self.fail_harbour("Harbour smoke: Harbour button is missing while docked.");
            return;
        }
        let day = self.session.as_ref().map(|session| session.world().day);
        self.open_harbour();
        let title = self
            .harbour_nodes
            .as_ref()
            .map(|nodes| nodes.title.get_text().to_string())
            .unwrap_or_default();
        if title != "Porto Novo" {
            self.fail_harbour(format!("Harbour smoke: title was {title}."));
            return;
        }
        if !self
            .harbour_body_text()
            .contains("No warehouse leased here.")
        {
            self.fail_harbour("Harbour smoke: expected no warehouse yet.");
            return;
        }
        self.perform(Action::HarbourPrepare(HarbourIntent::LeaseWarehouse(
            "depot".into(),
        )));
        self.perform(Action::HarbourConfirm);
        if !self.warehouse_active("porto_novo") {
            self.fail_harbour(format!(
                "Harbour smoke: depot lease failed. {}",
                self.harbour_notice
            ));
            return;
        }
        self.perform(Action::Buy("grain".into()));
        if self.held("grain") < 1 {
            self.fail_harbour("Harbour smoke: could not buy grain.");
            return;
        }
        self.perform(Action::HarbourDeposit("grain".into()));
        if self.held("grain") != 0 || !self.warehouse_holds("porto_novo", "grain") {
            self.fail_harbour(format!(
                "Harbour smoke: deposit failed. {}",
                self.harbour_notice
            ));
            return;
        }
        self.perform(Action::HarbourPrepare(HarbourIntent::OpenCredit(
            "merchant_line".into(),
        )));
        self.perform(Action::HarbourConfirm);
        if !self.credit_active() {
            self.fail_harbour(format!(
                "Harbour smoke: credit open failed. {}",
                self.harbour_notice
            ));
            return;
        }
        self.perform(Action::HarbourPrepare(HarbourIntent::Draw {
            tier: "merchant_line".into(),
            amount: 40,
        }));
        self.perform(Action::HarbourConfirm);
        if self.credit_outstanding() < 40 {
            self.fail_harbour(format!(
                "Harbour smoke: draw failed. {}",
                self.harbour_notice
            ));
            return;
        }
        self.perform(Action::HarbourPrepare(HarbourIntent::BuyInsurance {
            policy_id: "hull_basic".into(),
            target_id: String::new(),
            origin: String::new(),
            destination: String::new(),
        }));
        self.perform(Action::HarbourConfirm);
        if !self.policy_active("hull_basic") {
            self.fail_harbour(format!(
                "Harbour smoke: insurance failed. {}",
                self.harbour_notice
            ));
            return;
        }
        let license = {
            let Some(session) = self.session.as_mut() else {
                self.fail_harbour("Harbour smoke: no session for the license.");
                return;
            };
            session.buy_infrastructure("license", &["med_trade_charter"])
        };
        match license {
            Err(err) => {
                let text = harbour_screen::ascii_copy(&err.to_string());
                if !text.to_lowercase().contains("broker") {
                    self.fail_harbour(format!("Harbour smoke: license error was {text}."));
                    return;
                }
                self.harbour_notice = text;
            }
            Ok(()) => {
                self.fail_harbour("Harbour smoke: license should be rejected without a broker.");
                return;
            }
        }
        if self.session.as_ref().map(|session| session.world().day) != day {
            self.fail_harbour("Harbour smoke: the desk advanced the day.");
            return;
        }
        self.close_harbour();
        if self.harbour_open || !self.warehouse_active("porto_novo") {
            self.fail_harbour("Harbour smoke: close dropped the desk or the lease.");
            return;
        }
        let mut saw_note = false;
        for _ in 0..15 {
            self.next_day();
            if self.log_lines.iter().any(|line| {
                line.contains("Interest accrued")
                    || line.contains("Credit payment")
                    || line.contains("DEFAULT")
                    || line.contains("seized")
                    || line.contains("closed for non-payment")
            }) {
                saw_note = true;
                break;
            }
        }
        if !saw_note {
            self.fail_harbour(format!(
                "Harbour smoke: Next day log missed Turn.notes. {}",
                self.log_lines.join(" | ")
            ));
            return;
        }
        if capture {
            return;
        }
        self.perform(Action::Sail("al_manar".to_string()));
        if self.docked_id().is_some() || self.port_row_visible() || self.harbour_open {
            self.fail_harbour("Harbour smoke: Harbour stayed available at sea.");
        }
    }

    fn fail_harbour(&mut self, line: impl Into<String>) {
        let line = line.into();
        godot_print!("{line}");
        self.push_log(line);
        self.smoke_ok = false;
        self.harbour_checked = true;
    }

    fn harbour_button_shown(&self) -> bool {
        let Some(row) = self.port_row.as_ref() else {
            return false;
        };
        if !row.is_visible() {
            return false;
        }
        row.get_children().iter_shared().any(|child| {
            child
                .try_cast::<Button>()
                .ok()
                .is_some_and(|button| button.get_text() == "Harbour")
        })
    }

    fn port_row_visible(&self) -> bool {
        self.port_row.as_ref().is_some_and(|row| row.is_visible())
    }

    fn harbour_body_text(&self) -> String {
        let Some(nodes) = self.harbour_nodes.as_ref() else {
            return String::new();
        };
        labels_under_box(&nodes.body)
            .into_iter()
            .map(|label| label.get_text().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn warehouse_active(&self, port_id: &str) -> bool {
        self.session.as_ref().is_some_and(|session| {
            session
                .infrastructure()
                .warehouses
                .iter()
                .any(|lease| lease.port_id == port_id && lease.active && lease.tier == "depot")
        })
    }

    fn warehouse_holds(&self, port_id: &str, good: &str) -> bool {
        self.session.as_ref().is_some_and(|session| {
            session.infrastructure().warehouses.iter().any(|lease| {
                lease.port_id == port_id
                    && lease.active
                    && lease
                        .inventory
                        .iter()
                        .any(|lot| lot.good_id == good && lot.quantity > 0)
            })
        })
    }

    fn credit_active(&self) -> bool {
        self.session.as_ref().is_some_and(|session| {
            session
                .infrastructure()
                .credit
                .as_ref()
                .is_some_and(|credit| credit.active && credit.tier == "merchant_line")
        })
    }

    fn credit_outstanding(&self) -> i64 {
        self.session
            .as_ref()
            .and_then(|session| session.infrastructure().credit.as_ref())
            .map(|credit| credit.outstanding)
            .unwrap_or(0)
    }

    fn policy_active(&self, spec_id: &str) -> bool {
        self.session.as_ref().is_some_and(|session| {
            session
                .infrastructure()
                .policies
                .iter()
                .any(|policy| policy.active && policy.spec_id == spec_id)
        })
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
            Action::NewGame => self.open_newgame(NewgamePage::Title),
            Action::SaveGame => self.save_current_game(),
            Action::OpenCaptains => self.open_newgame(NewgamePage::Captains),
            Action::OpenCustom => self.open_newgame(NewgamePage::Custom),
            Action::OpenLoad => self.open_newgame(NewgamePage::Load),
            Action::NewgameBack => self.newgame_back(),
            Action::StartCaptain(id) => self.start_captain(&id),
            Action::BeginCustom => self.begin_custom(),
            Action::AdjustPoints(pool, delta) => self.adjust_points(pool, delta),
            Action::CycleRegion(delta) => self.cycle_region(delta),
            Action::CyclePort(delta) => self.cycle_port(delta),
            Action::CycleBloc(delta) => self.cycle_bloc(delta),
            Action::CycleFaction(delta) => self.cycle_faction(delta),
            Action::CycleMentor(delta) => self.cycle_mentor(delta),
            Action::LoadSlot(slot) => self.load_slot(&slot),
            Action::NextDay => self.next_day(),
            Action::Work => self.work_docks(),
            Action::OpenHunt => self.open_hunt(),
            Action::HireSailor => self.hire_sailor(),
            Action::Provision => self.buy_provisions(),
            Action::OpenCrew => self.open_crew(),
            Action::CloseCrew => self.close_crew(),
            Action::CrewHire { role, count } => self.crew_hire(&role, count),
            Action::CrewFire { role, count } => self.crew_fire(&role, count),
            Action::CrewProvision(days) => self.crew_provision(days),
            Action::CrewTrain(id) => self.arm_train(&id),
            Action::CrewSkill(id) => self.arm_skill(&id),
            Action::CrewRecruit(id) => self.arm_recruit(&id),
            Action::CrewConfirm => self.confirm_crew(),
            Action::CrewCancel => self.cancel_crew(),
            Action::Stance(stance) => self.push_stance(stance),
            Action::ClearStances => {
                self.stances.clear();
                self.refresh();
            }
            Action::Duel => self.fight_duel(),
            Action::AutoResolve => self.auto_resolve(),
            Action::ToggleMarket => {
                if self.docked_id().is_some() {
                    let opening = !self.market_open;
                    self.market_open = opening;
                    if opening {
                        self.play_sfx("sfx_ui_port_open");
                    } else {
                        self.market_notice.clear();
                    }
                }
                self.refresh();
            }
            Action::CycleTradeQty => {
                self.trade_qty = market::next_trade_qty(self.trade_qty);
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
            Action::OpenContracts => self.open_contracts(),
            Action::CloseContracts => self.close_contracts(),
            Action::RefreshContracts => self.refresh_contracts(),
            Action::AcceptContract(id) => self.accept_contract_offer(&id),
            Action::CompleteContract(id) => self.complete_contract_offer(&id),
            Action::ArmAbandon(id) => self.arm_abandon(&id),
            Action::ConfirmAbandon => self.confirm_abandon(),
            Action::CancelAbandon => self.cancel_abandon(),
            Action::OpenShipyard => self.open_shipyard(),
            Action::CloseShipyard => self.close_shipyard(),
            Action::ShipyardRepair => self.shipyard_repair(),
            Action::ShipyardRename => self.shipyard_rename(),
            Action::ShipyardArm(arm) => self.arm_shipyard(arm),
            Action::ShipyardConfirm => self.confirm_shipyard(),
            Action::ShipyardCancel => self.cancel_shipyard(),
            Action::OpenJournal => self.open_journal(),
            Action::CloseJournal => self.close_journal(),
            Action::CloseDayReport => self.close_day_report(),
            Action::ToggleBeat(id) => self.toggle_beat(&id),
            Action::OpenHarbour => self.open_harbour(),
            Action::CloseHarbour => self.close_harbour(),
            Action::HarbourPrepare(intent) => self.prepare_harbour(intent),
            Action::HarbourConfirm => self.confirm_harbour(),
            Action::HarbourCancel => {
                self.harbour_pending = None;
                self.harbour_notice.clear();
                self.refresh();
            }
            Action::HarbourDeposit(good) => self.deposit_cargo(&good),
            Action::HarbourWithdraw(good) => self.withdraw_cargo(&good),
            Action::HarbourRepayField => self.repay_from_field(),
            Action::HarbourRepayAll => self.repay_all(),
            Action::HarbourDraw => self.draw_from_field(),
            Action::HarbourEmergency => self.emergency_from_field(),
        }
    }

    fn sail(&mut self, dest: &str) {
        // Arrival visit memory: snapshot docked sell prices before depart (never saved).
        if let Some(session) = self.session.as_ref() {
            if session.world().voyage.status == VoyageStatus::InPort {
                let port_id = session.world().voyage.destination_id.clone();
                let prices = day_report::snapshot_docked_prices(session);
                self.day_report_memory
                    .visit_price_memory
                    .insert(port_id, prices);
            }
        }
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
            Ok(name) => {
                self.market_notice.clear();
                self.play_sfx("sfx_chart_sail_depart");
                self.push_log(format!("Departed for {name}."));
            }
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
                self.play_sfx("sfx_ui_port_open");
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
        self.market_notice.clear();
        let mut failed = false;
        let screen_open = self.encounter.is_some();
        let (day_before, sailed) = self
            .session
            .as_ref()
            .map(|session| (session.world().day, at_sea(session)))
            .unwrap_or((0, false));
        let prices_before = self
            .session
            .as_ref()
            .map(day_report::snapshot_docked_prices)
            .unwrap_or_default();
        let injuries_before = self
            .session
            .as_ref()
            .map(|session| session.injuries().to_vec())
            .unwrap_or_default();
        let mut turn_contracts = Vec::new();
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
                    turn_contracts = turn.contracts.clone();
                    let events: Vec<String> = turn
                        .events
                        .iter()
                        .map(|event| event.message.clone())
                        .collect();
                    let voyage_line = if session.world().voyage.status == VoyageStatus::InPort {
                        let place = docked_name(session).unwrap_or_else(|| "port".to_string());
                        Some(format!("Docked at {place}."))
                    } else if session.world().voyage.status == VoyageStatus::AtSea {
                        Some(format!(
                            "At sea. Progress {}/{}.",
                            session.world().voyage.progress,
                            session.world().voyage.distance
                        ))
                    } else {
                        None
                    };
                    day_log_lines(&events, &turn.shocks, &turn.notes, voyage_line.as_deref())
                }
            }
        };
        // Cue 7 plays on a successful advance even if a sea encounter opens
        // on this same tick. The Err arm stays silent.
        if !failed {
            self.play_sfx("sfx_chart_next_day");
        }
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
        if !failed {
            let arrival_day = sailed
                && self
                    .session
                    .as_ref()
                    .is_some_and(|session| session.world().voyage.status == VoyageStatus::InPort);
            self.show_day_report_after_advance(
                &turn_contracts,
                &injuries_before,
                &prices_before,
                arrival_day,
            );
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
        self.market_notice.clear();
        let qty = market::wire_qty(self.clamped_trade_qty(buy, good));
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            if buy {
                match session.buy(good, qty) {
                    Ok(receipt) => Ok((vec![receipt_line(&receipt)], Vec::new())),
                    Err(err) => Err(err),
                }
            } else {
                match session.sell(good, qty) {
                    Ok(sale) => Ok((
                        sale_lines(&sale),
                        market::paid_notice_lines(&sale.contracts),
                    )),
                    Err(err) => Err(err),
                }
            }
        };
        match result {
            Ok((lines, paid)) => {
                self.play_sfx("sfx_ui_trade_coin");
                for line in lines {
                    self.push_log(line);
                }
                self.market_notice = paid;
            }
            Err(err) => {
                if self.smoke {
                    self.smoke_ok = false;
                }
                self.push_log(err.to_string());
            }
        }
        self.refresh();
        if !self.market_notice.is_empty() {
            self.scroll_market_to_top();
        }
    }

    /// T-Q. The current qty clamped to what this press can do: buy to stock,
    /// silver and free hold, sell to held. 0 when nothing would succeed.
    fn clamped_trade_qty(&self, buy: bool, good: &str) -> i64 {
        let Some(session) = self.session.as_ref() else {
            return 0;
        };
        let world = session.world();
        if !buy {
            return market::clamp_sell(self.trade_qty, cargo_held(&world.captain.cargo, good));
        }
        let slot = docked_port_id(session)
            .and_then(|id| world.port(id))
            .and_then(|port| port.slot(good));
        let (Some(slot), Some(ship)) = (slot, world.captain.ship.as_ref()) else {
            return 0;
        };
        let capacity = portlight_sim::ship::resolve_cargo_capacity(ship) as f64;
        let room = BuyRoom {
            stock: slot.stock_current,
            unit_price: slot.buy_price,
            silver: world.captain.silver,
            capacity,
            current_weight: portlight_sim::economy::cargo_weight(&world.captain.cargo),
            weight_per_unit: content::content()
                .good(good)
                .map(|def| def.weight_per_unit)
                .unwrap_or(1.0),
        };
        market::clamp_buy(self.trade_qty, &room)
    }

    /// AV.1: a new paid notice sits at the top of the Market pane, so the pane
    /// scrolls there and the notice is never above the view.
    fn scroll_market_to_top(&mut self) {
        if let Some(scroll) = self.market_scroll.as_mut() {
            scroll.set_v_scroll(0);
        }
    }

    /// T-Q / T-N state is Godot memory only: a new game or load starts here.
    fn reset_trade(&mut self) {
        self.trade_qty = market::TRADE_QTYS[0];
        self.market_notice.clear();
    }

    fn work_docks(&mut self) {
        // T-N: Work is a port action like a trade, so the paid notice goes.
        self.market_notice.clear();
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
                    "Hired 1 sailor. Crew {crew}. Silver {} -> {silver}.",
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
                    "Bought provisions. Days {} -> {days}. Silver {silver}.",
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

    fn open_harbour(&mut self) {
        if self.docked_id().is_none() {
            return;
        }
        self.close_day_report();
        self.harbour_open = true;
        self.harbour_pending = None;
        self.harbour_notice.clear();
        self.refresh();
    }

    fn close_harbour(&mut self) {
        self.harbour_open = false;
        self.harbour_pending = None;
        self.harbour_notice.clear();
        self.refresh();
    }

    fn prepare_harbour(&mut self, intent: HarbourIntent) {
        let prompt = self
            .session
            .as_ref()
            .map(|session| harbour_screen::confirm_prompt(session, &intent))
            .unwrap_or_default();
        self.harbour_pending = Some(intent);
        self.harbour_notice = prompt;
        self.refresh();
    }

    fn confirm_harbour(&mut self) {
        let Some(intent) = self.harbour_pending.clone() else {
            return;
        };
        self.apply_harbour(intent);
    }

    fn apply_harbour(&mut self, intent: HarbourIntent) {
        let outcome = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            match &intent {
                HarbourIntent::LeaseWarehouse(_)
                | HarbourIntent::OpenBroker { .. }
                | HarbourIntent::BuyLicense(_) => {
                    let kind = harbour_screen::infrastructure_kind(&intent)
                        .expect("warehouse, broker, or license");
                    let args = infrastructure_args(&intent);
                    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
                    session
                        .buy_infrastructure(kind, &arg_refs)
                        .map(|()| match &intent {
                            HarbourIntent::LeaseWarehouse(tier) => format!("Leased {tier}."),
                            HarbourIntent::OpenBroker { region, tier } => {
                                format!("Opened {tier} broker in {region}.")
                            }
                            HarbourIntent::BuyLicense(id) => format!("Bought license {id}."),
                            _ => "Done.".to_string(),
                        })
                }
                HarbourIntent::OpenCredit(tier) => session
                    .take_credit(tier, 0)
                    .map(|_| format!("Opened {tier}.")),
                HarbourIntent::Draw { tier, amount } => session
                    .take_credit(tier, *amount)
                    .map(|received| format!("Drew {received} silver.")),
                HarbourIntent::Emergency(amount) => session
                    .take_credit("emergency", *amount)
                    .map(|received| format!("Emergency loan {received} silver.")),
                HarbourIntent::BuyInsurance {
                    policy_id,
                    target_id,
                    origin,
                    destination,
                } => session
                    .buy_insurance(policy_id, target_id, origin, destination)
                    .map(|()| format!("Bought {policy_id}.")),
            }
        };
        match outcome {
            Ok(line) => self.harbour_notice = line,
            Err(err) => self.harbour_notice = harbour_screen::ascii_copy(&err.to_string()),
        }
        self.harbour_pending = None;
        self.refresh();
    }

    fn deposit_cargo(&mut self, good: &str) {
        let text = edit_text(&self.harbour_qty);
        let qty = match harbour_screen::parse_positive(&text) {
            Ok(qty) => qty,
            Err(_) => {
                self.harbour_notice = "Quantity must be positive".to_string();
                self.refresh();
                return;
            }
        };
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.deposit_cargo(good, qty)
        };
        match result {
            Ok(moved) => self.harbour_notice = format!("Deposited {moved} {good}."),
            Err(err) => self.harbour_notice = harbour_screen::ascii_copy(&err.to_string()),
        }
        self.refresh();
    }

    fn withdraw_cargo(&mut self, good: &str) {
        let text = edit_text(&self.harbour_qty);
        let qty = match harbour_screen::parse_positive(&text) {
            Ok(qty) => qty,
            Err(_) => {
                self.harbour_notice = "Quantity must be positive".to_string();
                self.refresh();
                return;
            }
        };
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.withdraw_cargo(good, qty, None)
        };
        match result {
            Ok(moved) => self.harbour_notice = format!("Withdrew {moved} {good}."),
            Err(err) => self.harbour_notice = harbour_screen::ascii_copy(&err.to_string()),
        }
        self.refresh();
    }

    fn repay_from_field(&mut self) {
        let text = edit_text(&self.harbour_repay);
        match harbour_screen::parse_positive(&text) {
            Ok(amount) => self.repay_credit(amount),
            Err(_) => {
                self.harbour_notice = "Amount must be positive".to_string();
                self.refresh();
            }
        }
    }

    fn repay_all(&mut self) {
        let amount = self
            .session
            .as_ref()
            .and_then(|session| session.infrastructure().credit.as_ref())
            .filter(|credit| credit.active)
            .map(|credit| credit.outstanding + credit.interest_accrued)
            .unwrap_or(0);
        self.repay_credit(amount);
    }

    fn repay_credit(&mut self, amount: i64) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.repay_credit(amount)
        };
        match result {
            Ok(()) => self.harbour_notice = format!("Repaid {amount} silver."),
            Err(err) => self.harbour_notice = harbour_screen::ascii_copy(&err.to_string()),
        }
        self.refresh();
    }

    fn draw_from_field(&mut self) {
        let text = edit_text(&self.harbour_draw);
        let amount = match harbour_screen::parse_positive(&text) {
            Ok(amount) => amount,
            Err(_) => {
                self.harbour_notice = "Amount must be positive".to_string();
                self.refresh();
                return;
            }
        };
        let tier = self
            .session
            .as_ref()
            .and_then(|session| session.infrastructure().credit.as_ref())
            .filter(|credit| credit.active)
            .map(|credit| credit.tier.clone())
            .unwrap_or_else(|| "merchant_line".to_string());
        self.prepare_harbour(HarbourIntent::Draw { tier, amount });
    }

    fn emergency_from_field(&mut self) {
        let text = edit_text(&self.harbour_emergency);
        match harbour_screen::parse_positive(&text) {
            Ok(amount) => self.prepare_harbour(HarbourIntent::Emergency(amount)),
            Err(_) => {
                self.harbour_notice = "Amount must be positive".to_string();
                self.refresh();
            }
        }
    }

    fn sync_harbour(&mut self) {
        let Some(nodes) = self.harbour_nodes.clone() else {
            return;
        };
        if self.harbour_open && self.docked_id().is_none() {
            self.harbour_open = false;
            self.harbour_pending = None;
        }
        let mut root = nodes.root.clone();
        root.set_visible(self.harbour_open);
        if !self.harbour_open {
            return;
        }
        let Some(model) = self
            .session
            .as_ref()
            .and_then(harbour_screen::harbour_model)
        else {
            root.set_visible(false);
            self.harbour_open = false;
            return;
        };
        let mut title = nodes.title.clone();
        title.set_text(&model.port_name);
        let mut notice = nodes.notice.clone();
        let text = if self.harbour_notice.is_empty() {
            model.status.clone()
        } else {
            format!("{}\n{}", self.harbour_notice, model.status)
        };
        notice.set_text(&text);
        let mut plate = nodes.plate.clone();
        let mut panel = nodes.plate_panel.clone();
        let mut placeholder = nodes.placeholder.clone();
        let mut caption = nodes.plate_caption.clone();
        set_ship_plate(
            &mut plate,
            &mut panel,
            &mut placeholder,
            &mut caption,
            &model.ship_template,
        );
        self.fill_harbour_body(&model);
        self.fill_harbour_confirm();
    }

    fn fill_harbour_confirm(&mut self) {
        let Some(nodes) = self.harbour_nodes.clone() else {
            return;
        };
        let mut row = nodes.confirm_row.clone();
        clear_hbox(&mut row);
        let game_id = self.instance_id();
        if self.harbour_pending.is_some() {
            row.set_visible(true);
            row.add_child(&encounter_button(
                "Confirm",
                game_id,
                Action::HarbourConfirm,
            ));
            row.add_child(&encounter_button("Cancel", game_id, Action::HarbourCancel));
        } else {
            row.set_visible(false);
        }
        let mut close_host = nodes.close_host.clone();
        if close_host.get_child_count() == 0 {
            close_host.add_child(&encounter_button("Close", game_id, Action::CloseHarbour));
        }
    }

    fn fill_harbour_body(&mut self, model: &HarbourModel) {
        let Some(mut body) = self.harbour_nodes.as_ref().map(|nodes| nodes.body.clone()) else {
            return;
        };
        clear_children(&mut body);
        self.harbour_qty = None;
        self.harbour_draw = None;
        self.harbour_repay = None;
        self.harbour_emergency = None;
        let game_id = self.instance_id();

        let mut warehouse = harbour_screen::section_label("Warehouse");
        warehouse.set_name(harbour_screen::ANCHOR_WAREHOUSE);
        body.add_child(&warehouse);
        for line in &model.warehouse_lines {
            body.add_child(&harbour_screen::cream_label(line));
        }
        body.add_child(&harbour_screen::muted_label("Qty"));
        let mut qty = LineEdit::new_alloc();
        qty.set_text("1");
        newgame_screen::style_field(&mut qty);
        body.add_child(&qty);
        self.harbour_qty = Some(qty);
        if model.deposit_goods.is_empty() {
            body.add_child(&harbour_screen::muted_label("No cargo in the hold."));
        }
        for good in &model.deposit_goods {
            self.harbour_good_row(
                &mut body,
                game_id,
                &good.label,
                "Deposit",
                Action::HarbourDeposit(good.good_id.clone()),
            );
        }
        if model.withdraw_goods.is_empty() {
            body.add_child(&harbour_screen::muted_label(
                "Nothing in the warehouse to withdraw.",
            ));
        }
        for good in &model.withdraw_goods {
            self.harbour_good_row(
                &mut body,
                game_id,
                &good.label,
                "Withdraw",
                Action::HarbourWithdraw(good.good_id.clone()),
            );
        }
        self.harbour_offers(&mut body, game_id, &model.warehouse_offers);

        let mut broker = harbour_screen::section_label("Broker");
        broker.set_name(harbour_screen::ANCHOR_BROKER);
        body.add_child(&broker);
        for line in &model.broker_lines {
            body.add_child(&harbour_screen::cream_label(line));
        }
        self.harbour_offers(&mut body, game_id, &model.broker_offers);

        body.add_child(&harbour_screen::section_label("License"));
        for line in &model.license_lines {
            body.add_child(&harbour_screen::cream_label(line));
        }
        self.harbour_offers(&mut body, game_id, &model.license_offers);

        let mut credit = harbour_screen::section_label("Credit");
        credit.set_name(harbour_screen::ANCHOR_FINANCE);
        body.add_child(&credit);
        for line in &model.credit_lines {
            body.add_child(&harbour_screen::cream_label(line));
        }
        self.harbour_offers(&mut body, game_id, &model.credit_offers);
        body.add_child(&harbour_screen::muted_label(&format!(
            "Draw on {}",
            model.draw_tier
        )));
        let mut draw = LineEdit::new_alloc();
        draw.set_text("40");
        newgame_screen::style_field(&mut draw);
        body.add_child(&draw);
        self.harbour_draw = Some(draw);
        let mut draw_button = encounter_button("Draw", game_id, Action::HarbourDraw);
        if !model.draw_block.is_empty() {
            draw_button.set_disabled(true);
            body.add_child(&harbour_screen::muted_label(&model.draw_block));
        }
        body.add_child(&draw_button);
        body.add_child(&harbour_screen::muted_label("Repay"));
        let mut repay = LineEdit::new_alloc();
        repay.set_text("10");
        newgame_screen::style_field(&mut repay);
        body.add_child(&repay);
        self.harbour_repay = Some(repay);
        let mut repay_button = encounter_button("Repay", game_id, Action::HarbourRepayField);
        let repay_all_label = if model.repay_all > 0 {
            format!("Repay all {}", model.repay_all)
        } else {
            "Repay all".to_string()
        };
        let mut repay_all = encounter_button(&repay_all_label, game_id, Action::HarbourRepayAll);
        if !model.repay_block.is_empty() {
            repay_button.set_disabled(true);
            repay_all.set_disabled(true);
            body.add_child(&harbour_screen::muted_label(&model.repay_block));
        }
        let mut repay_row = HBoxContainer::new_alloc();
        repay_row.add_theme_constant_override("separation", 8);
        repay_row.add_child(&repay_button);
        repay_row.add_child(&repay_all);
        body.add_child(&repay_row);
        body.add_child(&harbour_screen::muted_label("Emergency loan"));
        let mut emergency = LineEdit::new_alloc();
        emergency.set_text("50");
        newgame_screen::style_field(&mut emergency);
        body.add_child(&emergency);
        self.harbour_emergency = Some(emergency);
        let mut emergency_button = Button::new_alloc();
        emergency_button.set_text("Emergency loan");
        harbour_screen::style_danger_button(&mut emergency_button);
        let action = Action::HarbourEmergency;
        stamp_playtest_id(&mut emergency_button, &action_playtest_id(&action));
        emergency_button.signals().pressed().connect(move || {
            let Ok(mut gd) = Gd::<PortlightGame>::try_from_instance_id(game_id) else {
                return;
            };
            gd.bind_mut().perform(action.clone());
        });
        body.add_child(&emergency_button);

        body.add_child(&harbour_screen::section_label("Insurance"));
        for line in &model.policy_lines {
            body.add_child(&harbour_screen::cream_label(line));
        }
        if !model.claim_lines.is_empty() {
            body.add_child(&harbour_screen::muted_label("Claims"));
            for line in &model.claim_lines {
                body.add_child(&harbour_screen::cream_label(line));
            }
        }
        self.harbour_offers(&mut body, game_id, &model.insurance_offers);
    }

    fn harbour_good_row(
        &self,
        body: &mut Gd<VBoxContainer>,
        game_id: InstanceId,
        label: &str,
        button: &str,
        action: Action,
    ) {
        let mut row = HBoxContainer::new_alloc();
        row.add_theme_constant_override("separation", 8);
        let mut text = harbour_screen::cream_label(label);
        text.set_h_size_flags(SizeFlags::EXPAND_FILL);
        row.add_child(&text);
        row.add_child(&encounter_button(button, game_id, action));
        body.add_child(&row);
    }

    fn harbour_offers(
        &self,
        body: &mut Gd<VBoxContainer>,
        game_id: InstanceId,
        rows: &[harbour_screen::ActionRow],
    ) {
        for row in rows {
            body.add_child(&harbour_screen::cream_label(&row.title));
            body.add_child(&harbour_screen::muted_label(&row.detail));
            if !row.block.is_empty() {
                body.add_child(&harbour_screen::muted_label(&row.block));
            }
            let action = row
                .intent
                .clone()
                .map(Action::HarbourPrepare)
                .unwrap_or(Action::HarbourCancel);
            let mut button = encounter_button(&row.button, game_id, action);
            if row.intent.is_none() {
                button.set_disabled(true);
            }
            body.add_child(&button);
        }
    }

    fn open_crew(&mut self) {
        if self.docked_id().is_none() {
            return;
        }
        self.close_day_report();
        self.crew_open = true;
        self.crew_pending = None;
        self.crew_notice.clear();
        self.refresh();
    }

    fn close_crew(&mut self) {
        self.crew_open = false;
        self.crew_pending = None;
        self.crew_notice.clear();
        self.refresh();
    }

    fn crew_hire(&mut self, role: &str, count: i64) {
        if self.crew_pending.is_some() {
            return;
        }
        let quote = self.session.as_ref().and_then(crew_desk).and_then(|desk| {
            desk.roles
                .into_iter()
                .find(|row| row.id == role)
                .map(|row| (row.name, row.hire_cost * count))
        });
        if let Some((name, cost)) = quote {
            if hire_needs_confirm(cost) {
                self.crew_notice = hire_confirm_line(count, name, cost);
                self.crew_pending = Some(CrewPending::Hire {
                    role: role.to_string(),
                    count,
                });
                self.refresh();
                return;
            }
        }
        self.apply_hire(role, count);
    }

    fn apply_hire(&mut self, role: &str, count: i64) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.hire_crew(count, role)
        };
        self.crew_pending = None;
        match result {
            Ok(()) => {
                let (crew, silver) = self.crew_counts();
                self.crew_notice = format!("Hired {count} {role}. Crew {crew}. Silver {silver}.");
                self.push_log(self.crew_notice.clone());
            }
            Err(err) => self.note_crew_error(err),
        }
        self.refresh();
    }

    fn crew_fire(&mut self, role: &str, count: i64) {
        if self.crew_pending.is_some() {
            return;
        }
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.fire_crew(count, role)
        };
        match result {
            Ok(()) => {
                let (crew, silver) = self.crew_counts();
                self.crew_notice = format!("Fired {count} {role}. Crew {crew}. Silver {silver}.");
                self.push_log(self.crew_notice.clone());
            }
            Err(err) => self.note_crew_error(err),
        }
        self.refresh();
    }

    fn crew_provision(&mut self, days: i64) {
        if self.crew_pending.is_some() {
            return;
        }
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.provision(days)
        };
        match result {
            Ok(()) => {
                let (provisions, silver) = self
                    .session
                    .as_ref()
                    .map(|session| {
                        (
                            session.world().captain.provisions,
                            session.world().captain.silver,
                        )
                    })
                    .unwrap_or((0, 0));
                self.crew_notice =
                    format!("Bought provisions. Stores {provisions}. Silver {silver}.");
                self.push_log(self.crew_notice.clone());
            }
            Err(err) => self.note_crew_error(err),
        }
        self.refresh();
    }

    fn arm_train(&mut self, id: &str) {
        if self.crew_pending.is_some() {
            return;
        }
        let Some(style) = self
            .session
            .as_ref()
            .and_then(crew_desk)
            .and_then(|desk| desk.styles.into_iter().find(|style| style.id == id))
        else {
            self.crew_notice = NO_FIGHTING_MASTER.to_string();
            self.refresh();
            return;
        };
        self.crew_notice = train_confirm_line(&style.name, style.cost, style.days);
        self.crew_pending = Some(CrewPending::Train { id: id.to_string() });
        self.refresh();
    }

    fn arm_skill(&mut self, id: &str) {
        if self.crew_pending.is_some() {
            return;
        }
        let Some(skill) = self
            .session
            .as_ref()
            .and_then(crew_desk)
            .and_then(|desk| desk.skills.into_iter().find(|skill| skill.id == id))
        else {
            self.crew_notice = "Unknown skill.".to_string();
            self.refresh();
            return;
        };
        if !skill.can_train {
            self.crew_notice = skill.empty_trainer.unwrap_or(skill.next_text);
            self.refresh();
            return;
        }
        self.crew_notice =
            skill_confirm_line(&skill.level_name, &skill.skill_name, skill.cost, skill.days);
        self.crew_pending = Some(CrewPending::Skill { id: id.to_string() });
        self.refresh();
    }

    fn arm_recruit(&mut self, id: &str) {
        if self.crew_pending.is_some() {
            return;
        }
        let Some(offer) = self
            .session
            .as_ref()
            .and_then(crew_desk)
            .and_then(|desk| desk.offers.into_iter().find(|offer| offer.id == id))
        else {
            self.crew_notice = NO_COMPANIONS_FOR_HIRE.to_string();
            self.refresh();
            return;
        };
        self.crew_notice = recruit_confirm_line(&offer.name, offer.cost);
        self.crew_pending = Some(CrewPending::Recruit { id: id.to_string() });
        self.refresh();
    }

    fn confirm_crew(&mut self) {
        let Some(pending) = self.crew_pending.clone() else {
            return;
        };
        self.crew_pending = None;
        match pending {
            CrewPending::Hire { role, count } => self.apply_hire(&role, count),
            CrewPending::Train { id } => self.apply_train(&id),
            CrewPending::Skill { id } => self.apply_skill(&id),
            CrewPending::Recruit { id } => self.apply_recruit(&id),
        }
    }

    fn cancel_crew(&mut self) {
        if self.crew_pending.is_none() {
            return;
        }
        self.crew_pending = None;
        self.crew_notice = "Cancelled.".to_string();
        self.refresh();
    }

    fn apply_train(&mut self, id: &str) {
        let day_before = self.session.as_ref().map(|session| session.world().day);
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.train_crew(id)
        };
        match result {
            Ok(()) => {
                let day_after = self.session.as_ref().map(|session| session.world().day);
                let advanced = day_before
                    .zip(day_after)
                    .map(|(before, after)| after - before)
                    .unwrap_or(0);
                self.crew_notice = format!("Learned {id}. The calendar advanced {advanced} days.");
                self.push_log(self.crew_notice.clone());
            }
            Err(err) => self.note_crew_error(err),
        }
        self.refresh();
    }

    fn apply_skill(&mut self, id: &str) {
        let day_before = self.session.as_ref().map(|session| session.world().day);
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.spend_skill_point(id)
        };
        match result {
            Ok(()) => {
                let day_after = self.session.as_ref().map(|session| session.world().day);
                let advanced = day_before
                    .zip(day_after)
                    .map(|(before, after)| after - before)
                    .unwrap_or(0);
                self.crew_notice = format!("Trained {id}. The calendar advanced {advanced} days.");
                self.push_log(self.crew_notice.clone());
            }
            Err(err) => self.note_crew_error(err),
        }
        self.refresh();
    }

    fn apply_recruit(&mut self, id: &str) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.recruit_companion(id)
        };
        match result {
            Ok(()) => {
                self.crew_notice = format!("Recruited {id}.");
                self.push_log(self.crew_notice.clone());
            }
            Err(err) => self.note_crew_error(err),
        }
        self.refresh();
    }

    fn note_crew_error(&mut self, err: SimError) {
        self.crew_notice = err.to_string();
        self.push_log(self.crew_notice.clone());
    }

    fn crew_counts(&self) -> (i64, i64) {
        self.session
            .as_ref()
            .map(|session| {
                let world = session.world();
                let crew = world
                    .captain
                    .ship
                    .as_ref()
                    .map(|ship| ship.crew)
                    .unwrap_or(0);
                (crew, world.captain.silver)
            })
            .unwrap_or((0, 0))
    }

    fn sync_crew(&mut self) {
        let Some(mut nodes) = self.crew_nodes.clone() else {
            return;
        };
        if !self.crew_open {
            nodes.root.set_visible(false);
            return;
        }
        let Some(session) = self.session.as_ref() else {
            self.crew_open = false;
            nodes.root.set_visible(false);
            return;
        };
        let Some(desk) = crew_desk(session) else {
            self.crew_open = false;
            nodes.root.set_visible(false);
            return;
        };
        nodes.root.set_visible(true);
        nodes.title.set_text(&desk.port_name);
        nodes.status.set_text(&desk.status);
        nodes.notice.set_text(&self.crew_notice);
        nodes.confirm_label.set_text(&self.crew_notice);
        nodes.confirm.set_visible(self.crew_pending.is_some());
        let template = session
            .world()
            .captain
            .ship
            .as_ref()
            .map(|ship| ship.template_id.clone())
            .unwrap_or_default();
        set_ship_plate(
            &mut nodes.plate,
            &mut nodes.plate_panel,
            &mut nodes.placeholder,
            &mut nodes.plate_caption,
            &template,
        );
        let kept = nodes.scroll.get_v_scroll();
        self.fill_crew_body(&nodes.body, &desk);
        nodes.scroll.set_v_scroll(kept);
    }

    fn fill_crew_body(&mut self, body: &Gd<VBoxContainer>, desk: &CrewDesk) {
        let mut body = body.clone();
        clear_children(&mut body);
        let locked = self.crew_pending.is_some();
        let game_id = self.instance_id();

        let mut roster = crew_section(crew_screen::SECTION_ROSTER);
        roster.add_child(&crew_subhead("Roster"));
        for role in &desk.roles {
            let mut row = HBoxContainer::new_alloc();
            row.add_theme_constant_override("separation", 8);
            let mut label = body_label(&role.text, 15, CREAM);
            shrink_label(&mut label);
            row.add_child(&label);
            let mut hire = crew_button(
                "Hire 1",
                game_id,
                Action::CrewHire {
                    role: role.id.to_string(),
                    count: 1,
                },
            );
            hire.set_disabled(locked || !role.hire_enabled);
            row.add_child(&hire);
            let mut fire = crew_button(
                "Fire 1",
                game_id,
                Action::CrewFire {
                    role: role.id.to_string(),
                    count: 1,
                },
            );
            fire.set_disabled(locked || !role.fire_enabled);
            row.add_child(&fire);
            let mut hire_five = crew_button(
                "Hire 5",
                game_id,
                Action::CrewHire {
                    role: role.id.to_string(),
                    count: 5,
                },
            );
            if role.hire_five {
                hire_five.set_disabled(locked || !role.hire_enabled);
            } else {
                // Same width, not drawn and not clickable, so every role's
                // Hire 1 and Fire 1 share one column with Sailor's.
                hire_five.set_name("HireFiveSlot");
                hire_five.set_disabled(true);
                hire_five.set_focus_mode(godot::classes::control::FocusMode::NONE);
                hire_five.set_mouse_filter(MouseFilter::IGNORE);
                hire_five.set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 0.0));
            }
            row.add_child(&hire_five);
            roster.add_child(&row);
        }
        for officer in &desk.officers {
            roster.add_child(&crew_muted(officer));
        }
        let mut provisions = crew_section(crew_screen::SECTION_PROVISIONS);
        provisions.add_child(&crew_subhead("Provisions"));
        provisions.add_child(&crew_copy(&desk.provision_text));
        let mut provision_buttons = HBoxContainer::new_alloc();
        provision_buttons.add_theme_constant_override("separation", 8);
        let mut plus_five = crew_button("Provisions +5", game_id, Action::CrewProvision(5));
        plus_five.set_disabled(locked);
        let mut plus_one = crew_button("Provisions +1", game_id, Action::CrewProvision(1));
        plus_one.set_disabled(locked);
        provision_buttons.add_child(&plus_five);
        provision_buttons.add_child(&plus_one);
        provisions.add_child(&provision_buttons);
        roster.add_child(&provisions);
        body.add_child(&roster);

        let mut training = crew_section(crew_screen::SECTION_TRAINING);
        training.add_child(&crew_subhead("Training"));
        training.add_child(&crew_copy(&desk.known_styles));
        if desk.styles.is_empty() {
            training.add_child(&crew_copy(NO_FIGHTING_MASTER));
        }
        for style in &desk.styles {
            training.add_child(&crew_copy(&style.text));
            let mut train = crew_button("Train", game_id, Action::CrewTrain(style.id.clone()));
            train.set_disabled(locked || style.known);
            training.add_child(&train);
        }
        training.add_child(&crew_subhead("Skills"));
        for skill in &desk.skills {
            training.add_child(&crew_copy(&skill.text));
            if let Some(empty) = &skill.empty_trainer {
                training.add_child(&crew_copy(empty));
            } else if !skill.next_text.is_empty() {
                training.add_child(&crew_muted(&skill.next_text));
            }
            if skill.can_train {
                let mut train = crew_button("Learn", game_id, Action::CrewSkill(skill.id.clone()));
                train.set_disabled(locked);
                training.add_child(&train);
            }
        }
        body.add_child(&training);

        let mut companions = crew_section(crew_screen::SECTION_COMPANIONS);
        companions.add_child(&crew_subhead("Companions"));
        companions.add_child(&crew_copy(&desk.party_line));
        if desk.party.is_empty() {
            companions.add_child(&crew_copy("In party: none."));
        }
        for member in &desk.party {
            companions.add_child(&crew_copy(member));
        }
        if desk.offers.is_empty() {
            companions.add_child(&crew_copy(NO_COMPANIONS_FOR_HIRE));
        }
        for offer in &desk.offers {
            companions.add_child(&crew_copy(&offer.text));
            let mut recruit =
                crew_button("Recruit", game_id, Action::CrewRecruit(offer.id.clone()));
            recruit.set_disabled(locked);
            companions.add_child(&recruit);
        }
        body.add_child(&companions);

        if self.crew_shot.is_some() {
            let mut pad = Control::new_alloc();
            pad.set_name("CrewShotPad");
            pad.set_mouse_filter(MouseFilter::IGNORE);
            pad.set_custom_minimum_size(Vector2::new(0.0, 640.0));
            body.add_child(&pad);
        }
    }

    fn begin_crew_shots(&mut self) {
        self.crew_shot_dir = Some(newgame_shot_dir(self.shot_path.as_deref()));
        self.crew_shot = Some(CrewShot::Roster);
        self.refresh();
    }

    fn advance_crew_shot(&mut self) -> bool {
        let Some(phase) = self.crew_shot else {
            return false;
        };
        let Some(dir) = self.crew_shot_dir.clone() else {
            return false;
        };
        // Rebuilding the body invalidates section positions until the next
        // layout. Prepare phases only scroll, after that layout has run.
        if !phase.saves() {
            self.scroll_crew_section(phase.section());
            self.crew_shot = Some(match phase {
                CrewShot::PrepareTraining => CrewShot::Training,
                _ => CrewShot::Companions,
            });
            self.capture_frames = 4;
            return true;
        }
        if !self.crew_section_at_top(phase.section()) {
            self.capture_failed = true;
            godot_print!(
                "Crew smoke: {} was not scrolled into view",
                phase.file_name()
            );
        }
        if phase == CrewShot::Training && !self.training_warning_visible() {
            self.capture_failed = true;
        }
        let path = format!("{dir}/{}", phase.file_name());
        if !self.save_shot(&path, true) {
            self.capture_failed = true;
        }
        if phase == CrewShot::Roster {
            // The body is already laid out, so the provisions block can
            // scroll before the train confirm rebuilds it.
            self.scroll_crew_section(crew_screen::SECTION_PROVISIONS);
            self.crew_shot = Some(CrewShot::Provisions);
            self.capture_frames = 4;
            return true;
        }
        if phase == CrewShot::Provisions {
            // Arming rebuilds the body, so the training scroll waits.
            self.arm_train("la_destreza");
            self.crew_shot = Some(CrewShot::PrepareTraining);
            self.capture_frames = 4;
            return true;
        }
        if phase == CrewShot::Training {
            // Drop the confirm without writing "Cancelled." Companions
            // is the idle desk again, and the action pass arms its own.
            self.crew_pending = None;
            self.crew_notice.clear();
            self.refresh();
            self.crew_shot = Some(CrewShot::PrepareCompanions);
            self.capture_frames = 4;
            return true;
        }
        self.crew_shot = None;
        self.run_crew_actions();
        false
    }

    /// The training frame has to show the day-advance line in the notice bar,
    /// fully inside the window, with every wrapped line tall enough to read.
    fn training_warning_visible(&mut self) -> bool {
        let Some(nodes) = self.crew_nodes.clone() else {
            godot_print!("Crew smoke: no crew desk for the training warning.");
            return false;
        };
        let text = nodes.notice.get_text().to_string();
        if !text.contains("advance") || !text.contains("days") {
            godot_print!("Crew smoke: notice bar missing the day-advance warning: {text:?}");
            return false;
        }
        if !nodes.confirm.is_visible() {
            godot_print!("Crew smoke: the train confirm was hidden.");
            return false;
        }
        let notice_ok = self.warning_label_visible("notice", &nodes.notice);
        let confirm_ok = self.warning_label_visible("confirm", &nodes.confirm_label);
        notice_ok && confirm_ok
    }

    fn warning_label_visible(&mut self, kind: &str, label: &Gd<Label>) -> bool {
        let text = label.get_text().to_string();
        let rect = label.get_global_rect();
        let pos = rect.position;
        let size = rect.size;
        let inside = pos.x >= -0.5
            && pos.y >= -0.5
            && pos.x + size.x <= WINDOW_W + 0.5
            && pos.y + size.y <= WINDOW_H + 0.5;
        let lines = label.get_line_count();
        let line_h = label.get_line_height();
        let covered = lines >= 1 && line_h > 0 && size.y + 1.0 >= (lines * line_h) as f32;
        let visible_lines = label.get_visible_line_count();
        if !text.contains("advance") || !inside || !covered || visible_lines < lines {
            godot_print!(
                "Crew smoke: {kind} warning clipped text={text:?} pos=({}, {}) size=({}, {}) lines={lines} visible={visible_lines} line_h={line_h}",
                pos.x,
                pos.y,
                size.x,
                size.y,
                );
            return false;
        }
        true
    }

    fn scroll_crew_section(&mut self, name: &str) {
        let Some(nodes) = self.crew_nodes.clone() else {
            return;
        };
        let mut scroll = nodes.scroll.clone();
        let Some(y) = section_y(&nodes.body, name) else {
            self.capture_failed = true;
            godot_print!("Crew smoke: missing section {name}");
            return;
        };
        scroll.set_v_scroll(y);
    }

    fn crew_section_at_top(&self, name: &str) -> bool {
        let Some(nodes) = &self.crew_nodes else {
            return false;
        };
        let Some(y) = section_y(&nodes.body, name) else {
            return false;
        };
        (y - nodes.scroll.get_v_scroll()).abs() <= 4
    }

    /// Hire, fire, provision, then train and skill through the confirm bar.
    /// The chart Hire and Stores +5 shortcuts still call Session.
    fn run_crew_actions(&mut self) {
        if !self.crew_open {
            self.fail_crew("Crew smoke: the desk was not open.");
            return;
        }
        if self.docked_id().is_none() {
            self.fail_crew("Crew smoke: expected to be docked.");
            return;
        }
        let chart_row = self.port_row.as_ref().is_some_and(|row| row.is_visible());
        if !chart_row {
            self.fail_crew("Crew smoke: the docked Crew control was hidden.");
            return;
        }
        let (crew_before, silver_before, provisions_before, day_before) =
            match self.session.as_ref() {
                Some(session) => {
                    let world = session.world();
                    let crew = world
                        .captain
                        .ship
                        .as_ref()
                        .map(|ship| ship.crew)
                        .unwrap_or(0);
                    (
                        crew,
                        world.captain.silver,
                        world.captain.provisions,
                        world.day,
                    )
                }
                None => {
                    self.fail_crew("Crew smoke: no session.");
                    return;
                }
            };
        self.crew_hire("sailor", 1);
        let hired = self.session.as_ref().is_some_and(|session| {
            let world = session.world();
            world.captain.ship.as_ref().map(|ship| ship.crew) == Some(crew_before + 1)
                && world.captain.silver < silver_before
        });
        if !hired {
            self.fail_crew("Crew smoke: hire 1 sailor did not change crew and silver.");
            return;
        }
        self.crew_fire("sailor", 1);
        let fired = self.session.as_ref().is_some_and(|session| {
            session.world().captain.ship.as_ref().map(|ship| ship.crew) == Some(crew_before)
        });
        if !fired {
            self.fail_crew("Crew smoke: fire 1 sailor did not restore the crew.");
            return;
        }
        self.crew_provision(1);
        let stored = self
            .session
            .as_ref()
            .is_some_and(|session| session.world().captain.provisions == provisions_before + 1);
        if !stored {
            self.fail_crew("Crew smoke: provision +1 did not add a day of stores.");
            return;
        }
        let day_now = self.session.as_ref().map(|session| session.world().day);
        self.arm_train("la_destreza");
        if self.crew_pending.is_none()
            || !self.crew_notice.contains("advance")
            || !self.crew_notice.contains("5 days")
        {
            self.fail_crew("Crew smoke: train confirm did not warn that the calendar advances.");
            return;
        }
        self.cancel_crew();
        if self.session.as_ref().is_some_and(|session| {
            session
                .world()
                .captain
                .learned_styles
                .iter()
                .any(|id| id == "la_destreza")
        }) || self.session.as_ref().map(|session| session.world().day) != day_now
        {
            self.fail_crew("Crew smoke: cancel changed the style or the day.");
            return;
        }
        self.arm_train("la_destreza");
        self.confirm_crew();
        let trained = self.session.as_ref().is_some_and(|session| {
            let world = session.world();
            world
                .captain
                .learned_styles
                .iter()
                .any(|id| id == "la_destreza")
                && world.day == day_before + 5
        });
        if !trained {
            self.fail_crew("Crew smoke: train did not learn La Destreza or advance 5 days.");
            return;
        }
        let day_trained = self.session.as_ref().map(|session| session.world().day);
        self.arm_skill("blacksmith");
        if self.crew_pending.is_none()
            || !self.crew_notice.contains("advance")
            || !self.crew_notice.contains("3 days")
        {
            self.fail_crew("Crew smoke: skill confirm did not warn that the calendar advances.");
            return;
        }
        self.confirm_crew();
        let skilled = self.session.as_ref().is_some_and(|session| {
            let world = session.world();
            portlight_sim::skills::skill_level(&world.captain.skills, "blacksmith") == 1
                && Some(world.day) == day_trained.map(|day| day + 3)
        });
        if !skilled {
            self.fail_crew(
                "Crew smoke: skill training did not reach Apprentice or advance 3 days.",
            );
            return;
        }
        self.close_crew();
        if self.crew_open {
            self.fail_crew("Crew smoke: Close left the desk open.");
            return;
        }
        self.hire_sailor();
        self.buy_provisions();
        let shortcuts = self.session.as_ref().is_some_and(|session| {
            let world = session.world();
            world.captain.ship.as_ref().map(|ship| ship.crew) == Some(crew_before + 1)
                && world.captain.provisions >= provisions_before + 1 + 5 - 8
        });
        if !shortcuts {
            self.fail_crew("Crew smoke: chart Hire or Stores +5 did not apply.");
        }
    }

    fn fail_crew(&mut self, line: &str) {
        godot_print!("{line}");
        self.push_log(line.to_string());
        self.smoke_ok = false;
        self.crew_checked = true;
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
            button.set_pressed_no_signal(self.market_open);
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
        if let Some(button) = self.sea_hunt_button.as_mut() {
            button.set_visible(!docked && self.session.is_some());
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
        if let Some(button) = self.save_button.as_mut() {
            button.set_disabled(self.session.is_none());
        }
        if let Some(button) = self.journal_button.as_mut() {
            // Chronicle is not pier-only. The button stays up at sea.
            button.set_disabled(self.session.is_none());
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
        self.sync_newgame();
        self.sync_contracts();
        self.sync_shipyard();
        self.sync_journal();
        self.sync_harbour();
        self.sync_crew();
        self.sync_hunt();
        self.sync_day_report();
        self.sync_contract_strip();
    }

    /// The docked row stays one line, and its minimum width fits the width
    /// the panel actually gave it. A taller row pushes the lanes down. A
    /// wider minimum stretches the column and clips Sail.
    fn assert_port_row_fits(&mut self) {
        let Some(row) = self.port_row.clone() else {
            return;
        };
        if !row.is_visible() {
            return;
        }
        let min = row.get_combined_minimum_size();
        let size = row.get_size();
        if size.y > 32.0 || min.y > 32.0 || min.x > size.x + 0.5 {
            self.smoke_ok = false;
            let line = format!(
                "Smoke: port row is {w}x{h} min {mw}x{mh}; it must stay one line inside the panel.",
                w = size.x,
                h = size.y,
                mw = min.x,
                mh = min.y
            );
            godot_print!("{line}");
            self.push_log(line);
        }
    }

    fn assert_contract_strip_fits(&mut self) {
        let Some(nodes) = self.contract_strip_nodes.clone() else {
            return;
        };
        if !contract_strip::overlay_visible(&nodes) {
            let filter = nodes.root.get_mouse_filter();
            if filter != MouseFilter::IGNORE {
                self.smoke_ok = false;
                let line = "Smoke: hidden contract strip must IGNORE mouse.".to_string();
                godot_print!("{line}");
                self.push_log(line);
            }
            return;
        }
        let pos = nodes.root.get_position();
        let size = nodes.root.get_size();
        let min = nodes.root.get_combined_minimum_size();
        let host_w = chart_host_width();
        let y_ok = (pos.y - contract_strip::STRIP_Y).abs() <= 1.0;
        let h_ok = size.y <= 24.0 && min.y <= 24.0;
        let x_ok = pos.x + size.x <= host_w + 0.5;
        if !y_ok || !h_ok || !x_ok {
            self.smoke_ok = false;
            let line = format!(
                "Smoke: contract strip at ({x},{y}) size {w}x{h} min {mw}x{mh}; want y~{sy} h<=24 x+w<={host}.",
                x = pos.x,
                y = pos.y,
                w = size.x,
                h = size.y,
                mw = min.x,
                mh = min.y,
                sy = contract_strip::STRIP_Y,
                host = host_w
            );
            godot_print!("{line}");
            self.push_log(line);
        }
        self.assert_port_row_fits();
    }

    fn wire_contracts_chrome(&mut self, nodes: &mut ContractsNodes) {
        let game_id = self.instance_id();
        nodes.confirm_row.add_child(&encounter_button(
            "Confirm",
            game_id,
            Action::ConfirmAbandon,
        ));
        nodes
            .confirm_row
            .add_child(&encounter_button("Cancel", game_id, Action::CancelAbandon));
        nodes.footer.add_child(&encounter_button(
            "Refresh board",
            game_id,
            Action::RefreshContracts,
        ));
        nodes
            .footer
            .add_child(&encounter_button("Close", game_id, Action::CloseContracts));
    }

    fn open_contracts(&mut self) {
        if self.docked_id().is_none() {
            return;
        }
        self.close_day_report();
        {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.available_contracts();
        }
        self.contracts_open = true;
        self.play_sfx("sfx_ui_contracts_open");
        self.contracts_notice.clear();
        self.contracts_confirm = None;
        self.refresh();
    }

    fn close_contracts(&mut self) {
        self.contracts_open = false;
        self.contracts_confirm = None;
        self.refresh();
    }

    /// Explicit refresh. Accept, complete, and abandon do not call this.
    fn refresh_contracts(&mut self) {
        if !self.contracts_open || self.docked_id().is_none() {
            return;
        }
        if let Some(session) = self.session.as_mut() {
            session.available_contracts();
        }
        self.contracts_confirm = None;
        self.contracts_notice.clear();
        self.refresh();
    }

    fn accept_contract_offer(&mut self, id: &str) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.accept_contract(id)
        };
        self.contracts_confirm = None;
        match result {
            Ok(active) => {
                self.play_sfx("sfx_ui_contract_accept");
                self.contracts_notice = format!(
                    "Accepted {}.",
                    contracts_screen::ascii_sentence(&active.title)
                );
            }
            Err(err) => self.contracts_notice = err.to_string(),
        }
        self.refresh();
    }

    fn complete_contract_offer(&mut self, id: &str) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.complete_contract(id)
        };
        self.contracts_confirm = None;
        match result {
            Ok(outcome) => {
                self.contracts_notice = contracts_screen::outcome_notice(&outcome);
            }
            Err(err) => self.contracts_notice = err.to_string(),
        }
        self.refresh();
    }

    fn arm_abandon(&mut self, id: &str) {
        let title = self
            .session
            .as_ref()
            .and_then(|session| {
                session
                    .board()
                    .active
                    .iter()
                    .find(|contract| contract.offer_id == id)
                    .map(|contract| contract.title.clone())
            })
            .unwrap_or_else(|| id.to_string());
        self.contracts_confirm = Some(id.to_string());
        self.contracts_notice = contracts_screen::abandon_prompt(&title);
        self.refresh();
    }

    fn confirm_abandon(&mut self) {
        let Some(id) = self.contracts_confirm.clone() else {
            return;
        };
        self.contracts_confirm = None;
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.abandon_contract(&id)
        };
        match result {
            Ok(outcome) => {
                self.contracts_notice = contracts_screen::outcome_notice(&outcome);
            }
            Err(err) => self.contracts_notice = err.to_string(),
        }
        self.refresh();
    }

    fn cancel_abandon(&mut self) {
        self.contracts_confirm = None;
        self.contracts_notice.clear();
        self.refresh();
    }

    fn sync_contracts(&mut self) {
        let Some(mut nodes) = self.contracts_nodes.clone() else {
            return;
        };
        let open = self.contracts_open && self.session.is_some();
        nodes.root.set_visible(open);
        nodes.root.set_mouse_filter(if open {
            godot::classes::control::MouseFilter::STOP
        } else {
            godot::classes::control::MouseFilter::IGNORE
        });
        if !open {
            return;
        }
        let template = self
            .session
            .as_ref()
            .and_then(|session| {
                session
                    .world()
                    .captain
                    .ship
                    .as_ref()
                    .map(|ship| ship.template_id.clone())
            })
            .unwrap_or_default();
        set_ship_plate(
            &mut nodes.plate,
            &mut nodes.plate_panel,
            &mut nodes.placeholder,
            &mut nodes.plate_caption,
            &template,
        );
        let port_title = self
            .docked_id()
            .and_then(|id| {
                self.session
                    .as_ref()
                    .and_then(|session| session.world().port(id).map(|port| port.name.clone()))
            })
            .unwrap_or_else(|| "Contract board".to_string());
        nodes
            .title
            .set_text(&contracts_screen::ascii_sentence(&port_title));
        nodes.card.set_text(contracts_screen::BOARD_CARD);
        nodes.notice.set_text(&self.contracts_notice);
        let at_cap = self
            .session
            .as_ref()
            .is_some_and(|session| session.board().active.len() >= contracts_screen::MAX_ACTIVE);
        nodes.cap.set_visible(at_cap);
        nodes.cap.set_text(if at_cap {
            contracts_screen::CAP_FULL
        } else {
            ""
        });
        nodes
            .confirm_row
            .set_visible(self.contracts_confirm.is_some());
        self.rebuild_contract_list(at_cap);
    }

    fn rebuild_contract_list(&mut self, at_cap: bool) {
        let Some(mut list) = self
            .contracts_nodes
            .as_ref()
            .map(|nodes| nodes.list.clone())
        else {
            return;
        };
        clear_children(&mut list);
        let listed = {
            let Some(session) = self.session.as_ref() else {
                return;
            };
            contract_listing(session)
        };
        let game_id = self.instance_id();
        list.add_child(&contracts_screen::section_label(
            contracts_screen::SECTION_BOARD,
        ));
        if listed.offers.is_empty() {
            let mut empty = contracts_screen::body_line(contracts_screen::EMPTY_OFFERS, 15, CREAM);
            empty.set_name("EmptyOffers");
            list.add_child(&empty);
        }
        for offer in listed.offers {
            list.add_child(&self.offer_block(&offer, game_id, at_cap));
        }

        let mut active_header = contracts_screen::section_label(contracts_screen::SECTION_ACTIVE);
        active_header.set_name("ActiveSection");
        list.add_child(&active_header);
        if listed.active.is_empty() {
            list.add_child(&contracts_screen::body_line(
                "No active contracts.",
                15,
                CREAM,
            ));
        }
        for (index, contract) in listed.active.iter().enumerate() {
            list.add_child(&self.active_block(contract, game_id, index == 0));
        }

        list.add_child(&contracts_screen::section_label(
            contracts_screen::SECTION_RECENT,
        ));
        if listed.recent.is_empty() {
            list.add_child(&contracts_screen::body_line(
                "No settled contracts.",
                15,
                MUTED,
            ));
        }
        for line in &listed.recent {
            list.add_child(&contracts_screen::body_line(line, 14, MUTED));
        }
    }

    fn offer_block(
        &self,
        offer: &ListedOffer,
        game_id: InstanceId,
        at_cap: bool,
    ) -> Gd<VBoxContainer> {
        let mut block = VBoxContainer::new_alloc();
        block.add_theme_constant_override("separation", 2);
        let mut title_row = HBoxContainer::new_alloc();
        title_row.add_theme_constant_override("separation", 8);
        let mut title = contracts_screen::body_line(&offer.title, 16, CREAM);
        shrink_label(&mut title);
        title_row.add_child(&title);
        let mut accept =
            encounter_button("Accept", game_id, Action::AcceptContract(offer.id.clone()));
        accept.set_disabled(at_cap);
        accept.set_h_size_flags(SizeFlags::SHRINK_END);
        title_row.add_child(&accept);
        block.add_child(&title_row);
        block.add_child(&contracts_screen::body_line(&offer.detail, 14, CREAM));
        block.add_child(&contracts_screen::body_line(
            &offer.meta,
            13,
            contracts_screen::meta_color(offer.due),
        ));
        if let Some(tag) = &offer.availability {
            block.add_child(&contracts_screen::body_line(tag, 13, MUTED));
        }
        block
    }

    fn active_block(
        &self,
        contract: &ListedActive,
        game_id: InstanceId,
        first: bool,
    ) -> Gd<VBoxContainer> {
        let mut block = VBoxContainer::new_alloc();
        block.add_theme_constant_override("separation", 2);
        let mut title = contracts_screen::body_line(&contract.title, 16, CREAM);
        if first {
            title.set_name("ActiveContractTitle");
        }
        block.add_child(&title);
        let mut progress = contracts_screen::body_line(&contract.detail, 14, CREAM);
        progress.set_name("ActiveProgress");
        block.add_child(&progress);
        block.add_child(&contracts_screen::body_line(
            &contract.meta,
            13,
            contracts_screen::meta_color(contract.due),
        ));
        let mut actions = HBoxContainer::new_alloc();
        actions.add_theme_constant_override("separation", 8);
        if contract.can_complete {
            actions.add_child(&encounter_button(
                "Complete",
                game_id,
                Action::CompleteContract(contract.id.clone()),
            ));
        }
        let mut abandon =
            encounter_button("Abandon", game_id, Action::ArmAbandon(contract.id.clone()));
        if first {
            abandon.set_name("AbandonButton");
        }
        actions.add_child(&abandon);
        block.add_child(&actions);
        block
    }

    fn pin_contracts_scroll(&mut self) {
        let Some(shot) = self.contracts_shot else {
            return;
        };
        let Some(nodes) = self.contracts_nodes.clone() else {
            return;
        };
        let mut scroll = nodes.scroll.clone();
        let target = match shot {
            ContractsShot::Active => "AbandonButton",
            ContractsShot::Board | ContractsShot::Empty => "ContractList",
        };
        if shot == ContractsShot::Board || shot == ContractsShot::Empty {
            scroll.set_v_scroll(0);
            return;
        }
        if let Some(control) = find_control_named(&nodes.list.clone().upcast(), target) {
            scroll.ensure_control_visible(&control);
        }
    }

    fn begin_contracts_shots(&mut self) {
        self.contracts_shot_dir = Some(newgame_shot_dir(self.shot_path.as_deref()));
        self.contracts_shot = Some(ContractsShot::Board);
        self.open_contracts();
        if !self.contracts_have_offers() {
            self.fail_contracts("Contracts smoke: the docked board had no offers.");
        }
    }

    fn advance_contracts_shot(&mut self) -> bool {
        let Some(phase) = self.contracts_shot else {
            return false;
        };
        let Some(dir) = self.contracts_shot_dir.clone() else {
            return false;
        };
        if !self.contracts_frame_ready(phase) {
            self.capture_failed = true;
        }
        let path = format!("{dir}/{}", phase.file_name());
        if !self.save_shot(&path, true) {
            self.capture_failed = true;
        }
        match phase {
            ContractsShot::Board => {
                if self.accept_first_offer().is_none() {
                    self.fail_contracts("Contracts smoke: could not accept an offer.");
                }
                self.contracts_notice.clear();
                self.contracts_confirm = None;
                self.refresh();
                self.contracts_shot = Some(ContractsShot::Active);
            }
            ContractsShot::Active => {
                self.prove_unfilled_complete_and_abandon();
                self.drain_contract_board();
                self.contracts_notice.clear();
                self.contracts_confirm = None;
                self.refresh();
                self.contracts_shot = Some(ContractsShot::Empty);
            }
            ContractsShot::Empty => {
                self.finish_contracts_smoke();
                self.contracts_shot = None;
                return false;
            }
        }
        self.capture_frames = 4;
        true
    }

    fn run_contracts_smoke(&mut self) {
        if self.docked_id().is_none() || !self.contracts_button_in_tree() {
            self.fail_contracts("Contracts smoke: expected the docked Contracts button.");
        }
        self.open_contracts();
        if !self.contracts_open || !self.contracts_have_offers() {
            self.fail_contracts("Contracts smoke: opening the board did not list offers.");
            return;
        }
        if self.accept_first_offer().is_none() {
            self.fail_contracts("Contracts smoke: could not accept an offer.");
            return;
        }
        self.prove_unfilled_complete_and_abandon();
        self.drain_contract_board();
        if !self.board_is_empty() {
            self.fail_contracts("Contracts smoke: the board still had offers.");
        }
        self.finish_contracts_smoke();
    }

    fn finish_contracts_smoke(&mut self) {
        let day = self.session.as_ref().map(|session| session.world().day);
        self.close_contracts();
        if self.contracts_open {
            self.fail_contracts("Contracts smoke: Close left the board up.");
        }
        if self.docked_id().is_none() {
            self.fail_contracts("Contracts smoke: the chart was not docked after Close.");
        }
        self.perform(Action::Sail("al_manar".into()));
        self.perform(Action::NextDay);
        if self.docked_id().is_some() {
            self.fail_contracts("Contracts smoke: still docked after a sea day.");
        }
        if self.contracts_button_in_tree() {
            self.fail_contracts("Contracts smoke: Contracts stayed visible at sea.");
        }
        if self.session.as_ref().map(|session| session.world().day) == day {
            self.fail_contracts("Contracts smoke: the sea day did not advance.");
        }
    }

    fn accept_first_offer(&mut self) -> Option<String> {
        let id = self
            .session
            .as_ref()
            .and_then(|session| session.board().offers.first().map(|offer| offer.id.clone()))?;
        let before = self
            .session
            .as_ref()
            .map(|session| session.board().active.len())
            .unwrap_or(0);
        self.accept_contract_offer(&id);
        let moved = self.session.as_ref().is_some_and(|session| {
            session
                .board()
                .active
                .iter()
                .any(|contract| contract.offer_id == id)
                && session.board().active.len() == before + 1
                && session.board().offers.iter().all(|offer| offer.id != id)
        });
        if !moved {
            self.fail_contracts("Contracts smoke: Accept did not move the offer onto Active.");
            return None;
        }
        Some(id)
    }

    fn prove_unfilled_complete_and_abandon(&mut self) {
        let Some(id) = self.session.as_ref().and_then(|session| {
            session
                .board()
                .active
                .first()
                .map(|contract| contract.offer_id.clone())
        }) else {
            self.fail_contracts("Contracts smoke: no active contract to complete.");
            return;
        };
        let (silver, wanted, trust, heat, regional) = self.reputation_snapshot();
        self.complete_contract_offer(&id);
        if self.contracts_notice != "Contract is not yet fulfilled" {
            self.fail_contracts(format!(
                "Contracts smoke: unfilled complete said {:?}.",
                self.contracts_notice
            ));
        }
        let still_active = self.session.as_ref().is_some_and(|session| {
            session
                .board()
                .active
                .iter()
                .any(|contract| contract.offer_id == id)
        });
        if !still_active {
            self.fail_contracts("Contracts smoke: unfilled complete removed the contract.");
        }
        self.arm_abandon(&id);
        if self.contracts_confirm.as_deref() != Some(id.as_str())
            || !self.contracts_notice.starts_with("Abandon ")
        {
            self.fail_contracts("Contracts smoke: Abandon did not ask for confirm.");
        }
        self.confirm_abandon();
        if !self.contracts_notice.starts_with("Contract abandoned: ")
            || ["Silver", "Trust", "Standing", "Heat"]
                .iter()
                .any(|term| self.contracts_notice.contains(term))
        {
            self.fail_contracts(format!(
                "Contracts smoke: abandon outcome was {:?}.",
                self.contracts_notice
            ));
        }
        let gone =
            self.session.as_ref().is_some_and(|session| {
                session.board().breaches.is_empty()
                    && session
                        .board()
                        .active
                        .iter()
                        .all(|contract| contract.offer_id != id)
                    && session.board().completed.iter().any(|outcome| {
                        outcome.contract_id == id && outcome.outcome_type == "abandoned"
                    })
            });
        if !gone {
            self.fail_contracts("Contracts smoke: abandon did not record an outcome.");
        }
        let (silver_after, wanted_after, trust_after, heat_after, regional_after) =
            self.reputation_snapshot();
        if silver != silver_after
            || wanted != wanted_after
            || trust != trust_after
            || heat != heat_after
            || regional != regional_after
        {
            self.fail_contracts(
                "Contracts smoke: abandon changed silver, trust, standing, heat, or wanted level.",
            );
        }
    }

    fn drain_contract_board(&mut self) {
        let mut saw_cap = false;
        for _ in 0..12 {
            let offer_id = self
                .session
                .as_ref()
                .and_then(|session| session.board().offers.first().map(|offer| offer.id.clone()));
            let Some(offer_id) = offer_id else {
                break;
            };
            let active_ids: Vec<String> = self
                .session
                .as_ref()
                .map(|session| {
                    session
                        .board()
                        .active
                        .iter()
                        .map(|contract| contract.offer_id.clone())
                        .collect()
                })
                .unwrap_or_default();
            if active_ids.len() >= contracts_screen::MAX_ACTIVE {
                self.accept_contract_offer(&offer_id);
                if self.contracts_notice != "Too many active contracts (max 3)" {
                    self.fail_contracts(format!(
                        "Contracts smoke: the 3-cap said {:?}.",
                        self.contracts_notice
                    ));
                }
                saw_cap = true;
                for id in active_ids {
                    self.arm_abandon(&id);
                    self.confirm_abandon();
                }
            }
            self.accept_contract_offer(&offer_id);
        }
        if !saw_cap {
            self.fail_contracts("Contracts smoke: never reached the 3-contract cap.");
        }
        let leftover: Vec<String> = self
            .session
            .as_ref()
            .map(|session| {
                session
                    .board()
                    .active
                    .iter()
                    .map(|contract| contract.offer_id.clone())
                    .collect()
            })
            .unwrap_or_default();
        for id in leftover {
            self.arm_abandon(&id);
            self.confirm_abandon();
        }
        if !self.board_is_empty() {
            self.fail_contracts("Contracts smoke: offers or active work remained.");
        }
    }

    fn contracts_frame_ready(&mut self, phase: ContractsShot) -> bool {
        let Some(nodes) = self.contracts_nodes.clone() else {
            self.fail_contracts("Contracts smoke: the screen was missing.");
            return false;
        };
        if !nodes.root.is_visible() || nodes.confirm_row.is_visible() {
            self.fail_contracts(format!(
                "Contracts smoke: {} had the wrong confirm state.",
                phase.file_name()
            ));
            return false;
        }
        match phase {
            ContractsShot::Board => {
                if !self.contracts_have_offers()
                    || !tree_has_button(&nodes.list.clone().upcast(), "Accept")
                {
                    self.fail_contracts("Contracts smoke: the board frame had no Accept button.");
                    return false;
                }
            }
            ContractsShot::Active => {
                let Some(title) =
                    find_label_named(&nodes.list.clone().upcast(), "ActiveContractTitle")
                else {
                    self.fail_contracts("Contracts smoke: the active frame had no obligation.");
                    return false;
                };
                let progress = find_label_named(&nodes.list.clone().upcast(), "ActiveProgress");
                let progress_ok = progress
                    .as_ref()
                    .is_some_and(|label| label.get_text().to_string().contains('/'));
                if !progress_ok || tree_has_button(&nodes.list.clone().upcast(), "Complete") {
                    self.fail_contracts(
                        "Contracts smoke: an unfilled contract showed Complete or hid progress.",
                    );
                    return false;
                }
                let abandon = find_button_text(&nodes.list.clone().upcast(), "Abandon");
                let title_ok = control_on_screen(&title.upcast());
                let abandon_ok = abandon
                    .as_ref()
                    .is_some_and(|button| control_on_screen(&button.clone().upcast()));
                if !title_ok || !abandon_ok {
                    self.fail_contracts(
                        "Contracts smoke: the active obligation was outside the window.",
                    );
                    return false;
                }
            }
            ContractsShot::Empty => {
                let empty = find_label_named(&nodes.list.clone().upcast(), "EmptyOffers");
                let text_ok = empty
                    .as_ref()
                    .is_some_and(|label| label.get_text() == contracts_screen::EMPTY_OFFERS);
                if !text_ok
                    || !self.board_is_empty()
                    || tree_has_button(&nodes.list.clone().upcast(), "Accept")
                {
                    self.fail_contracts("Contracts smoke: the empty frame still listed an offer.");
                    return false;
                }
            }
        }
        true
    }

    fn contracts_have_offers(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| !session.board().offers.is_empty())
    }

    fn board_is_empty(&self) -> bool {
        self.session.as_ref().is_some_and(|session| {
            session.board().offers.is_empty() && session.board().active.is_empty()
        })
    }

    fn contracts_button_in_tree(&self) -> bool {
        self.port_row.as_ref().is_some_and(|row| {
            row.is_visible_in_tree() && tree_has_button(&row.clone().upcast(), "Contracts")
        })
    }

    fn reputation_snapshot(&self) -> (i64, i64, i64, [i64; 5], [i64; 5]) {
        let Some(session) = self.session.as_ref() else {
            return (0, 0, 0, [0; 5], [0; 5]);
        };
        let captain = &session.world().captain;
        (
            captain.silver,
            captain.wanted_level,
            captain.standing.commercial_trust,
            captain.standing.heat,
            captain.standing.regional,
        )
    }

    fn fail_contracts(&mut self, line: impl Into<String>) {
        let line = line.into();
        godot_print!("{line}");
        self.push_log(line);
        self.smoke_ok = false;
        self.contracts_checked = true;
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
        let game_id = self.instance_id();
        // AV.1: one line, no wrap. `paid_notice_lines` already trims the text
        // to `market::PAID_NOTICE_CHARS`, which fits the box width; clipping
        // only keeps a stray long line from widening the panel.
        for line in &self.market_notice {
            let mut notice = body_label(line, 13, GOLD);
            notice.set_autowrap_mode(AutowrapMode::OFF);
            notice.set_clip_text(true);
            notice.set_h_size_flags(SizeFlags::EXPAND_FILL);
            box_node.add_child(&notice);
        }
        let mut header = HBoxContainer::new_alloc();
        let mut title = body_label(&format!("Market at {port_name}"), 15, GOLD);
        shrink_label(&mut title);
        header.add_child(&title);
        let mut qty = action_button(
            &market::qty_label(self.trade_qty),
            game_id,
            Action::CycleTradeQty,
        );
        qty.set_h_size_flags(SizeFlags::SHRINK_END);
        header.add_child(&qty);
        box_node.add_child(&header);
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
        // R12: sim flavour carries `[bold]` / `[dim]` tags; a Label prints them.
        self.log_lines
            .push(crate::logic::strip_markup(&line.into()));
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

    /// 1280×800 new-game frame. The ink ground is most of the shot, so the
    /// check is the plate fill and a button fill, same as the encounter.
    fn save_newgame_shot(&self, path: &str) -> bool {
        let Some(image) = self.base().get_viewport().and_then(|viewport| {
            viewport
                .get_texture()
                .and_then(|texture| texture.get_image())
        }) else {
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
        godot_print!(
            "screenshot {path} {width}x{height} samples={} error={err:?}",
            samples.len()
        );
        if err != Error::OK {
            godot_print!("screenshot save failed");
            return false;
        }
        if newgame_frame_rejected(width, height, &samples) {
            godot_print!("screenshot rejected: expected a full 1280x800 new game frame");
            return false;
        }
        true
    }

    /// 1280×720 shipyard frame. The ink ground is most of the shot, so the
    /// check is the plate fill and a button fill, same as the encounter.
    fn save_shipyard_shot(&self, path: &str) -> bool {
        let Some(image) = self.base().get_viewport().and_then(|viewport| {
            viewport
                .get_texture()
                .and_then(|texture| texture.get_image())
        }) else {
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
        godot_print!(
            "screenshot {path} {width}x{height} samples={} error={err:?}",
            samples.len()
        );
        if err != Error::OK {
            godot_print!("screenshot save failed");
            return false;
        }
        if shipyard_frame_rejected(width, height, &samples) {
            godot_print!("screenshot rejected: expected a full 1280x720 shipyard frame");
            return false;
        }
        true
    }

    fn open_agency(&mut self, state: EncounterState, log: String) {
        self.close_day_report();
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
        let (silver_before, result) = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            let before = session.world().captain.silver;
            (before, session.fight(action))
        };
        // R11: a finished personal fight (the Hunt duel) logs its result.
        let outcome = result.as_ref().ok().and_then(|step| {
            let silver_after = self.session.as_ref()?.world().captain.silver;
            fight_result_line(step, silver_after - silver_before)
        });
        self.ingest(result);
        if let Some(line) = outcome {
            self.push_log(line);
        }
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
            Ok(receipt) => {
                let lines = victory_receipt_lines(&receipt);
                for line in &lines {
                    self.push_log(line.clone());
                }
                if let Some(facts) = self.encounter.as_mut() {
                    facts.pending_victory = false;
                    facts.phase = "resolved".to_string();
                    facts.on_session = false;
                    facts.kind = "resolved".to_string();
                    if !lines.is_empty() {
                        facts.log = lines.join("\n");
                    }
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
        let bounty_marked = view.is_some() && self.bounty_badge_on();
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
            nodes.bounty.set_visible(false);
            return;
        };
        nodes.bounty.set_text("Bounty");
        nodes.bounty.set_visible(bounty_marked);
        nodes.title.set_text(view.title);
        nodes.card.set_text(&view.card);
        encounter_screen::set_delta_line(&mut nodes.delta, &view.delta);
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
        let preview = view.choice_preview.clone();
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
            let line = preview
                .as_ref()
                .and_then(|preview| preview.line_for(&action))
                .map(str::to_string);
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
            let mut button = encounter_button(&caption, game_id, command);
            match line {
                Some(line) => {
                    row.add_child(&encounter_screen::choice_cell(button, &[line.as_str()]))
                }
                None => {
                    // Beside a preview cell, a bare button (Capture) keeps its
                    // own height instead of stretching to the cell's.
                    if preview.is_some() {
                        button.set_v_size_flags(SizeFlags::SHRINK_BEGIN);
                    }
                    row.add_child(&button);
                }
            }
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

    fn bounty_badge_on(&self) -> bool {
        let Some(facts) = self.encounter.as_ref() else {
            return false;
        };
        let Some(session) = self.session.as_ref() else {
            return false;
        };
        session
            .world()
            .captain
            .active_bounties
            .iter()
            .any(|id| id == &facts.captain_id)
    }

    /// `Session::hunt` for one day, then the bounty desk. Forage must not open
    /// the encounter screen. `hunt_bounty` does, through [`Self::open_agency`].
    fn run_hunt_smoke(&mut self) {
        self.start_game();
        if self.session.is_none() {
            self.fail_hunt("Hunt smoke: no session.");
            return;
        }
        let day = self
            .session
            .as_ref()
            .map(|session| session.world().day)
            .unwrap_or(0);
        self.open_hunt();
        if !self.hunt_open || self.hunt_desk.board.is_empty() {
            self.fail_hunt("Hunt smoke: open did not post the board.");
            self.hunt_checked = true;
            return;
        }
        let docked_name = self
            .session
            .as_ref()
            .and_then(|session| docked_port_id(session).map(|id| port_name(session.world(), id)));
        if docked_name.is_none()
            || self.hunt_model_now().as_ref().is_none_or(|model| {
                Some(&model.title) != docked_name.as_ref()
                    || model.eyebrow != hunt_screen::EYEBROW_DOCKED
            })
        {
            self.fail_hunt("Hunt smoke: docked title was not the port name.");
        }
        if !self.hunt_forage_helper_shown() {
            self.fail_hunt("Hunt smoke: docked Forage helper missing.");
        }
        self.forage_day();
        let sailed = self.session.as_ref().is_some_and(|session| {
            session.world().day == day + 1
                && session.world().captain.day == day + 1
                && session.world().pending_duel.is_none()
        });
        if !sailed || self.encounter.is_some() || !self.hunt_desk.notice.contains("Provisions") {
            self.fail_hunt(
                "Hunt smoke: forage did not advance the day, or it opened a fight.".to_string(),
            );
        }
        self.post_board();
        if self.hunt_desk.board.is_empty() || self.hunt_desk.board.len() > 3 {
            self.fail_hunt(format!(
                "Hunt smoke: refresh listed {} offers.",
                self.hunt_desk.board.len()
            ));
        }
        let first = self
            .hunt_desk
            .board
            .first()
            .map(|row| row.captain_id.clone())
            .unwrap_or_default();
        if first.is_empty() {
            self.fail_hunt("Hunt smoke: no offer to accept.");
            self.hunt_checked = true;
            return;
        }
        self.accept_listed(&first);
        let active_ok = self
            .session
            .as_ref()
            .is_some_and(|session| session.world().captain.active_bounties == vec![first.clone()]);
        if !active_ok {
            self.fail_hunt("Hunt smoke: accept did not store one active bounty.");
        }
        self.accept_listed(&first);
        if self.hunt_desk.notice != "Already hunting this target" {
            self.fail_hunt(format!(
                "Hunt smoke: second accept said {}.",
                self.hunt_desk.notice
            ));
        }
        self.close_hunt();
        if self.hunt_open {
            self.fail_hunt("Hunt smoke: Close left the overlay up.");
        }
        self.fill_bounty_cap(&first);
        let departed = {
            let Some(session) = self.session.as_mut() else {
                self.fail_hunt("Hunt smoke: session dropped.");
                return;
            };
            session.depart("al_manar")
        };
        if let Err(err) = departed {
            self.fail_hunt(format!("Hunt smoke: could not sail: {err}"));
            return;
        }
        self.open_hunt();
        let sea_ok = self.hunt_model_now().as_ref().is_some_and(|model| {
            model.title == "At sea"
                && model.forage_status.contains("Morale")
                && model.forage_status.contains("20")
                && model.forage_enabled
        });
        if !sea_ok {
            self.fail_hunt("Hunt smoke: sea forage status did not name the morale gate.");
        }
        if !self.hunt_forage_helper_shown() {
            self.fail_hunt("Hunt smoke: sea Forage helper missing.");
        }
        let sea_day = self
            .session
            .as_ref()
            .map(|session| session.world().day)
            .unwrap_or(0);
        self.forage_day();
        let sea_forage = self.session.as_ref().is_some_and(|session| {
            session.world().day == sea_day + 1 && session.world().pending_duel.is_none()
        });
        if !sea_forage || self.encounter.is_some() {
            self.fail_hunt("Hunt smoke: forage at sea opened a fight or skipped the day.");
        }
        self.ask_hunt(&first);
        let crew_fact = self
            .session
            .as_ref()
            .and_then(|session| session.world().captain.ship.as_ref())
            .map(|ship| {
                format!(
                    "Boarding can cost crew - you have {}, need {} to sail.",
                    ship.crew,
                    portlight_sim::ship::template_crew_min(ship)
                )
            });
        if crew_fact.is_none_or(|fact| !self.hunt_desk.notice.ends_with(&fact)) {
            self.fail_hunt(format!(
                "Hunt smoke: confirm did not state the crew fact ({}).",
                self.hunt_desk.notice
            ));
        }
        self.confirm_hunt();
        let hunting = self.phase_is(ScreenPhase::Approach)
            && self
                .session
                .as_ref()
                .is_some_and(|session| session.world().pending_duel.is_some())
            && !self.hunt_open
            && self.bounty_badge_on()
            && self
                .encounter_nodes
                .as_ref()
                .is_some_and(|nodes| nodes.bounty.is_visible());
        if !hunting {
            self.fail_hunt("Hunt smoke: hunt target did not open the bounty encounter.");
        }
        self.leave_encounter();
        self.open_hunt();
        self.ask_claim(&first);
        self.confirm_hunt();
        if self.hunt_desk.notice != "Target not yet defeated. Find and defeat them at sea." {
            self.fail_hunt(format!("Hunt smoke: claim said {}.", self.hunt_desk.notice));
        }
        if self.encounter.is_some() {
            self.fail_hunt("Hunt smoke: claim opened a fight.");
        }
        self.hunt_checked = true;
        if self.smoke_ok {
            self.push_log("Hunt smoke: forage, board, accept, pursue, and claim gate.".to_string());
        }
    }

    fn fill_bounty_cap(&mut self, first: &str) {
        let catalog = [
            "scarlet_ana",
            "the_butcher",
            "raj_the_quiet",
            "typhoon_mei",
            "old_coral",
            "the_diver",
            "sergeant_kruze",
            "gnaw",
        ];
        for id in catalog {
            let full = self
                .session
                .as_ref()
                .is_some_and(|session| session.world().captain.active_bounties.len() >= 3);
            if full {
                break;
            }
            if id == first {
                continue;
            }
            self.accept_listed(id);
        }
        let count = self
            .session
            .as_ref()
            .map(|session| session.world().captain.active_bounties.len())
            .unwrap_or(0);
        if count != 3 {
            self.fail_hunt(format!(
                "Hunt smoke: expected 3 active bounties, have {count}."
            ));
            return;
        }
        let extra = catalog.into_iter().find(|id| {
            self.session.as_ref().is_none_or(|session| {
                !session
                    .world()
                    .captain
                    .active_bounties
                    .iter()
                    .any(|active| active == id)
            })
        });
        let Some(extra) = extra else {
            self.fail_hunt("Hunt smoke: no id left for the cap check.");
            return;
        };
        self.accept_listed(extra);
        if self.hunt_desk.notice != "Maximum 3 active bounties" {
            self.fail_hunt(format!("Hunt smoke: cap said {}.", self.hunt_desk.notice));
        }
    }

    fn fail_hunt(&mut self, line: impl Into<String>) {
        let line = line.into();
        godot_print!("{line}");
        self.push_log(line);
        self.smoke_ok = false;
        self.hunt_checked = true;
    }

    fn hunt_forage_helper_shown(&self) -> bool {
        self.hunt_nodes
            .as_ref()
            .is_some_and(|nodes| hunt_screen::forage_helper_shown(&nodes.body))
    }

    fn hunt_model_now(&self) -> Option<hunt_screen::HuntModel> {
        let session = self.session.as_ref()?;
        Some(hunt_screen::hunt_model(session, &self.hunt_desk))
    }

    fn open_hunt(&mut self) {
        if self.session.is_none() {
            return;
        }
        self.close_day_report();
        self.hunt_open = true;
        self.hunt_desk.confirm = None;
        self.hunt_desk.notice.clear();
        self.post_board();
    }

    fn close_hunt(&mut self) {
        self.hunt_open = false;
        self.hunt_desk.confirm = None;
        self.refresh();
    }

    fn show_day_report_after_advance(
        &mut self,
        turn_contracts: &[portlight_sim::model::ContractOutcome],
        injuries_before: &[portlight_sim::model::Injury],
        prices_before: &std::collections::HashMap<String, i64>,
        arrival_day: bool,
    ) {
        let Some(session) = self.session.as_ref() else {
            return;
        };
        let mut memory = std::mem::take(&mut self.day_report_memory);
        // Captain's week: every successful advance feeds the window (post-advance reads).
        memory.week.push(day_report::week_sample(session));
        let mut doc = day_report::build_document(
            session,
            turn_contracts,
            injuries_before,
            prices_before,
            &mut memory,
            arrival_day,
            &self.hunt_desk.known,
        );
        if doc.has_notable() {
            // Ride-along only: the footer is attached after the notable gate.
            doc.footer = day_report::build_footer(session, &memory.week);
        }
        self.day_report_memory = memory;
        self.day_report_memory.price_memory = day_report::snapshot_docked_prices(session);
        if doc.has_notable() {
            self.day_report_doc = Some(doc);
            self.day_report_open = true;
        } else {
            self.day_report_doc = None;
            self.day_report_open = false;
        }
    }

    /// New game / load: the report and its Godot-only memory (incl. Captain's week) reset.
    fn reset_day_report(&mut self) {
        self.day_report_open = false;
        self.day_report_doc = None;
        self.day_report_memory.reset();
    }

    fn open_day_report_doc(&mut self, doc: DayReportDocument) {
        self.day_report_doc = Some(doc);
        self.day_report_open = true;
        self.refresh();
    }

    fn close_day_report(&mut self) {
        if !self.day_report_open && self.day_report_doc.is_none() {
            return;
        }
        self.day_report_open = false;
        self.refresh();
    }

    fn sync_day_report(&mut self) {
        let open = self.day_report_open;
        let Some(mut nodes) = self.day_report_nodes.clone() else {
            return;
        };
        day_report::set_open(&mut nodes, open);
        if !open {
            return;
        }
        day_report::place_card(&mut nodes.root);
        if let Some(doc) = self.day_report_doc.clone() {
            day_report::apply_document(&mut nodes, &doc);
        }
    }

    fn run_day_report_smoke(&mut self) {
        self.start_game();
        if self.session.is_none() {
            self.fail_day_report("Day-report smoke: no session.");
            return;
        }
        let day = self
            .session
            .as_ref()
            .map(|session| session.world().day)
            .unwrap_or(1);
        let full = day_report::smoke_full_document(day);
        self.open_day_report_doc(full);
        if !self.day_report_open
            || self
                .day_report_nodes
                .as_ref()
                .is_none_or(|nodes| !day_report::overlay_visible(nodes))
        {
            self.fail_day_report("Day-report smoke: forced full card was not visible.");
        }
        let text = self
            .day_report_nodes
            .as_ref()
            .map(day_report::overlay_text)
            .unwrap_or_default();
        if !text.contains("Deadlines") || !text.contains("Claim ready") {
            self.fail_day_report("Day-report smoke: full card missing expected sections.");
        }
        if text.contains('\u{2014}') || text.contains("due soon") {
            self.fail_day_report("Day-report smoke: forbidden copy on full card.");
        }
        // R1 yield: full 4-section body hides the footer (budget overflow).
        if text.contains("Week: ") || text.contains("Next: ") {
            self.fail_day_report("Day-report smoke: full card should yield Captain's week footer.");
        }
        self.assert_day_report_fits("full");
        self.close_day_report();
        if self.day_report_open {
            self.fail_day_report("Day-report smoke: Close left the card up.");
        }
        let deadline = day_report::smoke_deadline_document(day);
        self.open_day_report_doc(deadline);
        let deadline_text = self
            .day_report_nodes
            .as_ref()
            .map(day_report::overlay_text)
            .unwrap_or_default();
        if !deadline_text.contains("Spice charter - 1 day left - 10/10")
            || deadline_text.contains("Complete")
        {
            self.fail_day_report("Day-report smoke: deadline card line or no-Complete copy.");
        }
        // R1: Next without Week never — warm-up deadline carries no footer.
        if deadline_text.contains("Week: ") || deadline_text.contains("Next: ") {
            self.fail_day_report(
                "Day-report smoke: deadline card should yield footer (Next without Week never).",
            );
        }
        self.assert_day_report_fits("deadline");
        self.close_day_report();
        // Opening Hunt must close the report.
        self.open_day_report_doc(day_report::smoke_full_document(day));
        self.open_hunt();
        if self.day_report_open {
            self.fail_day_report("Day-report smoke: Hunt left the report open.");
        }
        self.close_hunt();
        // Quiet day at sea: Prices stay empty (§13.2); no deadline/heal/claimable.
        let departed = {
            let Some(session) = self.session.as_mut() else {
                self.fail_day_report("Day-report smoke: session dropped before quiet day.");
                return;
            };
            session.depart("al_manar")
        };
        if let Err(err) = departed {
            self.fail_day_report(format!(
                "Day-report smoke: could not sail for quiet day: {err}"
            ));
            return;
        }
        self.next_day();
        if self.day_report_open {
            self.fail_day_report("Day-report smoke: quiet Next day still showed the card.");
        }
        if self.smoke_ok {
            self.push_log(
                "Day-report smoke: full, deadline, overlay-close, quiet hide.".to_string(),
            );
        }
    }

    /// Card stays 520x380 with Close inside (footer may shrink the scroll <= 48 px).
    fn assert_day_report_fits(&mut self, what: &str) {
        let error = self
            .day_report_nodes
            .as_ref()
            .and_then(day_report::card_fit_error);
        if let Some(error) = error {
            self.fail_day_report(format!(
                "Day-report smoke: {what} card does not fit: {error}"
            ));
        }
    }

    fn day_report_footer_now(&self) -> String {
        self.day_report_nodes
            .as_ref()
            .map(day_report::footer_text)
            .unwrap_or_default()
    }

    /// Captain's week on real Session verbs (seed 1, docked at Porto Novo).
    /// Warm-up hides Week; a notable warm day shows Week; an empty ladder omits
    /// Next; then a contract three days out, a hire, and a new bounty fill the
    /// footer for `day-report-week.png`. Returns false when a step failed.
    fn run_day_report_week_smoke(&mut self) -> bool {
        self.start_game();
        if self.session.is_none() {
            self.fail_day_report("Day-report week: no session.");
            return false;
        }
        // Day 2: one sample. Week never shows while warming up.
        self.next_day();
        if self.day_report_memory.week.len() != 1 {
            self.fail_day_report("Day-report week: first advance did not push one sample.");
            return false;
        }
        if self.day_report_open && self.day_report_footer_now().contains("Week:") {
            self.fail_day_report("Day-report week: Week shown with one sample.");
        }
        // Day 3: Bounty accepted is notable; window of two shows Week; no Next applies.
        let accepted = self
            .session
            .as_mut()
            .map(|session| session.accept_bounty("raj_the_quiet"));
        if !matches!(accepted, Some(Ok(()))) {
            self.fail_day_report("Day-report week: could not accept a bounty.");
            return false;
        }
        self.next_day();
        // Ride-along data must carry Week; presentation may yield when the body
        // overflows the footer budget (this warm day often has many Prices).
        let doc_week = self
            .day_report_doc
            .as_ref()
            .and_then(|doc| doc.footer.week.clone());
        if !self.day_report_open
            || doc_week
                .as_deref()
                .is_none_or(|w| !w.contains("Week: ") || !w.contains("bounty +1"))
        {
            self.fail_day_report(format!(
                "Day-report week: notable warm day missing Week data: {doc_week:?}"
            ));
            return false;
        }
        let footer = self.day_report_footer_now();
        let yielded = self
            .day_report_doc
            .as_ref()
            .is_some_and(|doc| !day_report::footer_should_show(&doc.footer, &doc.sections));
        if yielded {
            if !footer.is_empty() {
                self.fail_day_report(format!(
                    "Day-report week: warm day yielded but footer still visible: {footer:?}"
                ));
            }
        } else if !footer.contains("Week: ") || !footer.contains("bounty +1") {
            self.fail_day_report(format!(
                "Day-report week: warm day missing visible Week line: {footer:?}"
            ));
            return false;
        }
        if footer.contains("Next:") {
            self.fail_day_report(format!(
                "Day-report week: empty ladder still showed Next: {footer:?}"
            ));
        }
        self.assert_day_report_fits("warm");
        self.close_day_report();
        // Nearest-deadline contract, then advance until it is three days out.
        let offer = {
            let Some(session) = self.session.as_mut() else {
                return false;
            };
            let mut offers = session.available_contracts();
            offers.sort_by(|a, b| {
                a.deadline_day
                    .cmp(&b.deadline_day)
                    .then_with(|| a.title.cmp(&b.title))
            });
            offers.into_iter().next().map(|offer| offer.id)
        };
        let Some(offer) = offer else {
            self.fail_day_report("Day-report week: no contract offer.");
            return false;
        };
        self.accept_contract_offer(&offer);
        let days_left = |game: &Self| {
            game.session.as_ref().and_then(|session| {
                let day = session.world().day;
                session
                    .board()
                    .active
                    .iter()
                    .map(|contract| contract.deadline_day - day)
                    .min()
            })
        };
        let mut hired = false;
        let mut hunted = false;
        for _ in 0..40 {
            let Some(left) = days_left(self) else {
                self.fail_day_report("Day-report week: contract not active.");
                return false;
            };
            if left <= 3 {
                break;
            }
            // Inside the final five-sample window: hire one hand, then take a bounty.
            if left == 6 && !hired {
                hired = self
                    .session
                    .as_mut()
                    .is_some_and(|session| session.hire_crew(1, "sailor").is_ok());
            }
            if left == 4 && !hunted {
                hunted = self
                    .session
                    .as_mut()
                    .is_some_and(|session| session.accept_bounty("scarlet_ana").is_ok());
            }
            self.close_day_report();
            self.next_day();
        }
        if days_left(self) != Some(3) || !hired || !hunted {
            self.fail_day_report(format!(
                "Day-report week: setup left {:?} days, hired {hired}, bounty {hunted}.",
                days_left(self)
            ));
            return false;
        }
        let text = self
            .day_report_nodes
            .as_ref()
            .map(day_report::overlay_text)
            .unwrap_or_default();
        let footer = self.day_report_footer_now();
        if !self.day_report_open || !text.contains("Deadlines") || !text.contains("Bounty accepted")
        {
            self.fail_day_report(format!("Day-report week: notable body missing: {text:?}"));
            return false;
        }
        let want_next = "Next: Contract 3 days left - open Contracts.";
        if !footer.contains("Week: ")
            || !footer.contains("crew 3 to 4")
            || !footer.contains("bounty +1")
            || !footer.contains(want_next)
        {
            self.fail_day_report(format!("Day-report week: footer {footer:?}"));
            return false;
        }
        if !footer.is_ascii() || footer.contains("due soon") || footer.contains("+0") {
            self.fail_day_report(format!("Day-report week: forbidden footer copy {footer:?}"));
        }
        let body_lines = self
            .day_report_doc
            .as_ref()
            .map(|doc| doc.sections.iter().map(|s| s.lines.len()).sum::<usize>())
            .unwrap_or(0);
        if body_lines > day_report::LINE_CAP {
            self.fail_day_report(format!(
                "Day-report week: body over LINE_CAP ({body_lines})."
            ));
        }
        self.assert_day_report_fits("week");
        if self.smoke_ok {
            let line = format!("Day-report week smoke: {}", footer.replace('\n', " | "));
            godot_print!("{line}");
            self.push_log(line);
        }
        self.smoke_ok
    }

    /// Arrival card: accept an Al-Manar contract at Porto Novo, sail, Next day
    /// until docked, assert arrival title + Arrival section, Escape closes, and
    /// an already-docked Next day keeps the day title. Quiet arrival is pinned
    /// by unit tests. Returns false when a step failed.
    fn run_day_report_arrival_smoke(&mut self) -> bool {
        self.start_game();
        if self.session.is_none() {
            self.fail_day_report("Day-report arrival: no session.");
            return false;
        }
        // Accept a contract deliverable at Al-Manar.
        let offer = {
            let Some(session) = self.session.as_mut() else {
                return false;
            };
            let offers = session.available_contracts();
            offers
                .into_iter()
                .find(|offer| offer.destination_port_id == "al_manar")
                .map(|offer| offer.id)
        };
        let Some(offer) = offer else {
            self.fail_day_report("Day-report arrival: no Al-Manar contract offer.");
            return false;
        };
        self.accept_contract_offer(&offer);
        let has_active = self.session.as_ref().is_some_and(|session| {
            session
                .board()
                .active
                .iter()
                .any(|c| c.destination_port_id == "al_manar")
        });
        if !has_active {
            self.fail_day_report("Day-report arrival: accept did not activate Al-Manar contract.");
            return false;
        }
        // Undock via Game::sail so visit_price_memory is written for Porto Novo.
        self.sail("al_manar");
        if self.docked_id().is_some() {
            self.fail_day_report("Day-report arrival: still docked after sail.");
            return false;
        }
        if !self
            .day_report_memory
            .visit_price_memory
            .contains_key("porto_novo")
        {
            self.fail_day_report("Day-report arrival: visit memory missing after undock.");
            return false;
        }
        // Advance until sailed→InPort at Al-Manar.
        let mut arrived = false;
        for _ in 0..60 {
            self.close_day_report();
            // Dismiss any sea encounter so Next day can keep advancing.
            if self.encounter.is_some() {
                self.encounter = None;
            }
            self.next_day();
            if self.docked_id() == Some("al_manar") {
                arrived = true;
                break;
            }
        }
        if !arrived {
            self.fail_day_report("Day-report arrival: never reached Al-Manar.");
            return false;
        }
        let text = self
            .day_report_nodes
            .as_ref()
            .map(day_report::overlay_text)
            .unwrap_or_default();
        if !self.day_report_open {
            self.fail_day_report(format!(
                "Day-report arrival: card hidden on arrival with deliverable contract: {text:?}"
            ));
            return false;
        }
        let title = self
            .day_report_doc
            .as_ref()
            .map(|doc| doc.title.clone())
            .unwrap_or_default();
        if !title.starts_with("Arrived - ") || !title.contains("Al-Manar") {
            self.fail_day_report(format!("Day-report arrival: bad title {title:?}"));
            return false;
        }
        if !text.contains("Arrival") || !text.contains("here") {
            self.fail_day_report(format!(
                "Day-report arrival: missing Arrival section: {text:?}"
            ));
            return false;
        }
        if text.contains("due soon") || text.contains('\u{2014}') {
            self.fail_day_report(format!("Day-report arrival: forbidden copy: {text:?}"));
        }
        // Prices section must not appear on arrival day.
        if self
            .day_report_doc
            .as_ref()
            .is_some_and(|doc| doc.sections.iter().any(|s| s.id == "prices"))
        {
            self.fail_day_report("Day-report arrival: Prices section present on arrival day.");
        }
        self.assert_day_report_fits("arrival");
        // Escape closes under the existing ladder (no new rung).
        self.dismiss_cancel();
        if self.day_report_open {
            self.fail_day_report("Day-report arrival: Escape left the card up.");
            return false;
        }
        // Already-docked Next day must not use arrival title.
        self.next_day();
        if self.day_report_open {
            let docked_title = self
                .day_report_doc
                .as_ref()
                .map(|doc| doc.title.clone())
                .unwrap_or_default();
            if docked_title.starts_with("Arrived - ") {
                self.fail_day_report(format!(
                    "Day-report arrival: already-docked Next day used arrival title: {docked_title}"
                ));
                return false;
            }
        }
        self.close_day_report();
        if self.smoke_ok {
            let line = format!("Day-report arrival smoke: {title}");
            godot_print!("{line}");
            self.push_log(line);
        }
        self.smoke_ok
    }

    fn fail_day_report(&mut self, line: impl Into<String>) {
        let line = line.into();
        godot_print!("{line}");
        self.push_log(line);
        self.smoke_ok = false;
        self.day_report_checked = true;
    }

    fn begin_day_report_shots(&mut self) {
        self.start_game();
        let day = self
            .session
            .as_ref()
            .map(|session| session.world().day)
            .unwrap_or(1);
        self.open_day_report_doc(day_report::smoke_full_document(day));
        self.day_report_shot_dir = Some(day_report_shot_dir(self.shot_path.as_deref()));
        self.day_report_shot = Some(DayReportShot::Full);
        self.refresh();
    }

    fn advance_day_report_shot(&mut self) -> bool {
        let Some(phase) = self.day_report_shot else {
            return false;
        };
        let Some(dir) = self.day_report_shot_dir.clone() else {
            return false;
        };
        if !self.day_report_open
            || self
                .day_report_nodes
                .as_ref()
                .is_none_or(|nodes| !day_report::overlay_visible(nodes))
        {
            self.smoke_ok = false;
            godot_print!(
                "Day-report smoke: frame {} was not visible.",
                phase.file_name()
            );
        }
        // Laid out and drawn: the open card must still be 520x380 with Close inside.
        self.assert_day_report_fits(phase.file_name());
        let path = format!("{dir}/{}", phase.file_name());
        // Compact card over the chart: use the chart flat-frame check, not the
        // encounter plate/button ink gate (no side plate on this panel).
        if !self.save_shot(&path, false) {
            self.capture_failed = true;
        }
        match phase {
            DayReportShot::Full => {
                let day = self
                    .session
                    .as_ref()
                    .map(|session| session.world().day)
                    .unwrap_or(1);
                self.open_day_report_doc(day_report::smoke_deadline_document(day));
                self.day_report_shot = Some(DayReportShot::Deadline);
                self.capture_frames = 4;
                true
            }
            DayReportShot::Deadline => {
                self.close_day_report();
                // Quiet-day hide is a smoke assertion, not a committed frame (§13.5).
                let departed = self
                    .session
                    .as_mut()
                    .map(|session| session.depart("al_manar"));
                if departed.transpose().is_err() {
                    self.smoke_ok = false;
                    godot_print!("Day-report smoke: could not sail for quiet-day check.");
                }
                self.next_day();
                if self.day_report_open {
                    self.smoke_ok = false;
                    godot_print!("Day-report smoke: quiet day left the card visible.");
                }
                self.day_report_shot = None;
                self.day_report_checked = true;
                false
            }
            DayReportShot::Week | DayReportShot::Arrival => {
                self.day_report_shot = None;
                self.day_report_checked = true;
                false
            }
        }
    }

    fn sync_contract_strip(&mut self) {
        let Some(mut nodes) = self.contract_strip_nodes.clone() else {
            return;
        };
        let doc = self
            .session
            .as_ref()
            .and_then(contract_strip::build_document);
        match doc {
            Some(doc) => {
                contract_strip::place_strip(&mut nodes.root);
                contract_strip::apply_document(&mut nodes, &doc);
                contract_strip::set_visible(&mut nodes, true);
            }
            None => {
                contract_strip::set_visible(&mut nodes, false);
            }
        }
        self.contract_strip_nodes = Some(nodes);
    }

    fn run_contract_strip_smoke(&mut self) {
        self.start_game();
        if self.session.is_none() {
            self.fail_contract_strip("Contract-strip smoke: no session.");
            return;
        }
        self.refresh();
        if self
            .contract_strip_nodes
            .as_ref()
            .is_some_and(contract_strip::overlay_visible)
        {
            self.fail_contract_strip("Contract-strip smoke: fresh game showed the strip.");
        }
        self.assert_contract_strip_fits();

        // Accept real seed-1 offers (nearest two by deadline after accept order).
        {
            let Some(session) = self.session.as_mut() else {
                self.fail_contract_strip("Contract-strip smoke: session dropped.");
                return;
            };
            let _ = session.available_contracts();
        }
        if self.accept_strip_offer().is_none() {
            self.fail_contract_strip("Contract-strip smoke: could not accept first offer.");
            return;
        }
        self.refresh();
        if !self
            .contract_strip_nodes
            .as_ref()
            .is_some_and(contract_strip::overlay_visible)
        {
            self.fail_contract_strip("Contract-strip smoke: strip hidden after Accept.");
        }
        let text_one = self
            .contract_strip_nodes
            .as_ref()
            .map(contract_strip::overlay_text)
            .unwrap_or_default();
        if text_one.contains('\u{2014}') || text_one.contains("due soon") {
            self.fail_contract_strip("Contract-strip smoke: forbidden copy after Accept.");
        }
        if !text_one.contains(" - ") {
            self.fail_contract_strip("Contract-strip smoke: missing ASCII separator.");
        }
        // Destination hint on single segment.
        if !text_one.contains(" - to ") {
            self.fail_contract_strip(format!(
                "Contract-strip smoke: single segment missing destination: {text_one}"
            ));
        }
        self.assert_contract_strip_fits();

        // Second accept -> two segments, no destination, still GOLD path.
        if self.accept_strip_offer().is_none() {
            self.fail_contract_strip("Contract-strip smoke: could not accept second offer.");
            return;
        }
        self.refresh();
        let text_two = self
            .contract_strip_nodes
            .as_ref()
            .map(contract_strip::overlay_text)
            .unwrap_or_default();
        if text_two.contains(" - to ") {
            self.fail_contract_strip("Contract-strip smoke: destination shown with two segments.");
        }
        if text_two.contains("+") && text_two.contains("more") {
            self.fail_contract_strip("Contract-strip smoke: unexpected +N with only two actives.");
        }

        // Third accept -> +1 more
        if self.accept_strip_offer().is_none() {
            self.fail_contract_strip("Contract-strip smoke: could not accept third offer.");
            return;
        }
        self.refresh();
        let text_three = self
            .contract_strip_nodes
            .as_ref()
            .map(contract_strip::overlay_text)
            .unwrap_or_default();
        if !text_three.contains("+1 more") {
            self.fail_contract_strip(format!(
                "Contract-strip smoke: expected +1 more, got {text_three}"
            ));
        }

        // Click docked opens Contracts; day report closed.
        let day = self
            .session
            .as_ref()
            .map(|session| session.world().day)
            .unwrap_or(1);
        self.open_day_report_doc(day_report::smoke_full_document(day));
        if !self.day_report_open {
            self.fail_contract_strip(
                "Contract-strip smoke: could not open day report for click hygiene.",
            );
        }
        self.perform(Action::OpenContracts);
        if !self.contracts_open {
            self.fail_contract_strip(
                "Contract-strip smoke: docked strip click did not open Contracts.",
            );
        }
        if self.day_report_open {
            self.fail_contract_strip("Contract-strip smoke: Contracts left Day's report open.");
        }
        self.close_contracts();
        self.assert_contract_strip_fits();

        // Partial sell for progress (weapons buy+sell at dest auto-completes;
        // grain: buy at home, sail, sell partial so delivered stays below required).
        self.prepare_contract_strip_progress();
        self.refresh();
        let progress_text = self
            .contract_strip_nodes
            .as_ref()
            .map(contract_strip::overlay_text)
            .unwrap_or_default();
        if progress_text.contains("0/") {
            // Not fatal if voyage blocked; note and continue when progress present.
            // Prefer seeing a non-zero delivered count when sell succeeded.
        }
        let _ = progress_text;

        // Advance docked days until a shown contract is urgent (days_left <= 1).
        if !self.advance_until_strip_urgent(40) {
            self.fail_contract_strip(
                "Contract-strip smoke: could not reach urgent timing via Next day.",
            );
            return;
        }
        self.refresh();
        let urgent_text = self
            .contract_strip_nodes
            .as_ref()
            .map(contract_strip::overlay_text)
            .unwrap_or_default();
        if !(urgent_text.contains("1 day left")
            || urgent_text.contains("due today")
            || urgent_text.contains("overdue"))
        {
            self.fail_contract_strip(format!(
                "Contract-strip smoke: urgent copy missing: {urgent_text}"
            ));
        }
        if urgent_text.contains("due soon") {
            self.fail_contract_strip("Contract-strip smoke: urgent path said due soon.");
        }

        // At sea: strip stays visible; click does not open Contracts.
        self.close_contracts();
        let sailed = {
            let Some(session) = self.session.as_mut() else {
                self.fail_contract_strip("Contract-strip smoke: session dropped before sail.");
                return;
            };
            // Sail toward a different port if docked.
            let here = session.world().voyage.destination_id.clone();
            let dest = if here == "porto_novo" {
                "al_manar"
            } else {
                "porto_novo"
            };
            session.depart(dest)
        };
        if let Err(err) = sailed {
            self.fail_contract_strip(format!("Contract-strip smoke: depart failed: {err}"));
            return;
        }
        self.refresh();
        if self.docked_id().is_some() {
            self.fail_contract_strip("Contract-strip smoke: still docked after depart.");
        }
        if !self
            .contract_strip_nodes
            .as_ref()
            .is_some_and(contract_strip::overlay_visible)
        {
            self.fail_contract_strip("Contract-strip smoke: strip hidden at sea with actives.");
        }
        self.perform(Action::OpenContracts);
        if self.contracts_open {
            self.fail_contract_strip("Contract-strip smoke: at-sea click opened Contracts.");
        }

        // The strip cue is always the deadline timing. There is no Complete
        // cue: Session::sell settles a delivered contract on the sale.
        if self.smoke_ok {
            self.push_log(
                "Contract-strip smoke: hide, accept, cap, click, urgent, sea no-op ok. Cue is deadline timing."
                    .to_string(),
            );
        }
    }

    fn accept_strip_offer(&mut self) -> Option<String> {
        let id = {
            let session = self.session.as_mut()?;
            let _ = session.available_contracts();
            session.board().offers.first().map(|offer| offer.id.clone())
        }?;
        let before = self
            .session
            .as_ref()
            .map(|session| session.board().active.len())
            .unwrap_or(0);
        self.accept_contract_offer(&id);
        let moved = self.session.as_ref().is_some_and(|session| {
            session
                .board()
                .active
                .iter()
                .any(|contract| contract.offer_id == id)
                && session.board().active.len() == before + 1
        });
        if !moved {
            self.fail_contract_strip("Contract-strip smoke: Accept did not move the offer.");
            return None;
        }
        Some(id)
    }

    fn fail_contract_strip(&mut self, line: impl Into<String>) {
        let line = line.into();
        godot_print!("{line}");
        self.push_log(line);
        self.smoke_ok = false;
        self.contract_strip_checked = true;
    }

    fn fail_trade(&mut self, line: impl Into<String>) {
        let line = line.into();
        godot_print!("{line}");
        self.push_log(line);
        self.smoke_ok = false;
    }

    /// Seed 1 at Porto Novo: accept the grain run to Corsair's Rest, buy at
    /// `Qty 10`, sail, then sell at `Qty 10` until the sale settles the
    /// contract and the Market leads with the paid notice. False when the
    /// run could not get that far; a failed copy check still returns true.
    fn run_trade_settle(&mut self) -> bool {
        self.start_game();
        if self.trade_qty != 1 {
            self.fail_trade("Trade smoke: a new game did not start at Qty 1.");
        }
        self.perform(Action::CycleTradeQty);
        self.perform(Action::CycleTradeQty);
        if self.trade_qty != 10 {
            self.fail_trade(format!(
                "Trade smoke: two Qty presses gave {}.",
                self.trade_qty
            ));
            return false;
        }
        let offer = self.session.as_mut().and_then(|session| {
            session
                .available_contracts()
                .into_iter()
                .find(|offer| {
                    offer.good_id == "grain" && offer.destination_port_id == "corsairs_rest"
                })
                .map(|offer| (offer.id, offer.quantity))
        });
        let Some((offer_id, required)) = offer else {
            self.fail_trade("Trade smoke: no grain offer for Corsair's Rest.");
            return false;
        };
        self.accept_contract_offer(&offer_id);
        self.market_open = true;
        // Rough seas take a few units, so buy a margin over the order.
        let want = required + 5;
        for press in 0..6 {
            if self.held("grain") >= want {
                break;
            }
            let before = self.held("grain");
            self.perform(Action::Buy("grain".into()));
            let bought = self.held("grain") - before;
            if press == 0 && bought != 10 {
                self.fail_trade(format!("Trade smoke: Qty 10 buy took {bought} grain."));
                return false;
            }
            if bought <= 0 {
                self.fail_trade("Trade smoke: a clamped buy took nothing.");
                return false;
            }
        }
        if !self
            .log_lines
            .iter()
            .any(|line| line.starts_with("buy 10 Grain for "))
        {
            self.fail_trade("Trade smoke: the receipt did not log the real qty.");
        }
        self.sail("corsairs_rest");
        for _ in 0..40 {
            if self.docked_id() == Some("corsairs_rest") {
                break;
            }
            self.close_day_report();
            if self.encounter.is_some() {
                self.encounter = None;
            }
            self.next_day();
        }
        if self.docked_id() != Some("corsairs_rest") {
            self.fail_trade("Trade smoke: never docked at Corsair's Rest.");
            return false;
        }
        self.close_day_report();
        self.market_open = true;
        let active = |game: &Self| {
            game.session.as_ref().is_some_and(|session| {
                session
                    .board()
                    .active
                    .iter()
                    .any(|contract| contract.offer_id == offer_id)
            })
        };
        for press in 0..6 {
            if !active(self) {
                break;
            }
            let before = self.held("grain");
            self.perform(Action::Sell("grain".into()));
            let sold = before - self.held("grain");
            if sold != before.min(10) || (press == 0 && sold != 10) {
                self.fail_trade(format!(
                    "Trade smoke: Qty 10 sell moved {sold} of {before} grain."
                ));
                return false;
            }
        }
        if active(self) {
            self.fail_trade("Trade smoke: the Qty 10 sales did not settle the contract.");
            return false;
        }
        let paid = self.market_notice.first().cloned().unwrap_or_default();
        godot_print!("trade smoke paid notice: {paid}");
        if self.market_notice.len() != 1
            || !paid.starts_with("Contract paid: Silver +")
            || !paid.contains(" - 23 Grain to ")
            || paid.len() > market::PAID_NOTICE_CHARS
            || paid.contains("Delivered")
            || paid.contains("(+")
            || paid.contains("+0")
            || ["Trust", "Standing", "Heat"]
                .iter()
                .any(|term| paid.contains(term))
            || paid.contains("corsairs_rest")
            || !paid.is_ascii()
        {
            self.fail_trade(format!("Trade smoke: paid notice was {paid:?}."));
        }
        let shown = self
            .market_box
            .as_ref()
            .map(labels_under_box)
            .and_then(|labels| labels.first().map(|label| label.get_text().to_string()));
        if shown.as_deref() != Some(paid.as_str()) {
            self.fail_trade(format!(
                "Trade smoke: the Market box did not lead with the notice ({shown:?})."
            ));
        }
        true
    }

    /// `--trade-smoke` (T-Q, T-N). The settle above, then Work, Next day and a
    /// new game must each clear the paid notice; the Qty holds until new game.
    fn run_trade_smoke(&mut self) {
        if !self.run_trade_settle() {
            return;
        }
        let paid = self.market_notice.first().cloned().unwrap_or_default();
        self.perform(Action::Work);
        if !self.market_notice.is_empty() {
            self.fail_trade("Trade smoke: Work kept the paid notice.");
        }
        // Put the notice back so Next day has one to clear.
        self.market_notice = vec![paid.clone()];
        self.refresh();
        self.perform(Action::NextDay);
        if !self.market_notice.is_empty() {
            self.fail_trade("Trade smoke: Next day kept the paid notice.");
        }
        if self.trade_qty != 10 {
            self.fail_trade("Trade smoke: Qty did not hold across trades.");
        }
        self.start_game();
        if self.trade_qty != 1 || !self.market_notice.is_empty() {
            self.fail_trade("Trade smoke: a new game kept the Qty or notice.");
        }
    }

    /// T-N frame 2. One staged delivery, then the real outcome as the latest:
    /// one GOLD line with ` (+1 more)`, through `paid_notice_lines`.
    fn stage_market_paid_more(&mut self) {
        let real = self
            .session
            .as_ref()
            .and_then(|session| session.board().completed.last().cloned());
        let Some(real) = real else {
            self.fail_trade("Market paid frames: no completed contract to stage from.");
            return;
        };
        if market::paid_notice_lines(std::slice::from_ref(&real)) != self.market_notice {
            self.fail_trade(format!(
                "Market paid frames: last outcome does not match the notice ({:?}).",
                self.market_notice
            ));
        }
        self.market_notice = market::paid_notice_lines(&market::smoke_paid_more(&real));
        self.refresh();
        if self.market_notice.len() != 1
            || !self.market_notice[0].starts_with("Contract paid: Silver +")
            || !self.market_notice[0].contains(" (+1 more) - ")
            || self.market_notice[0].len() > market::PAID_NOTICE_CHARS
        {
            self.fail_trade(format!(
                "Market paid frames: staged notice was {:?}.",
                self.market_notice
            ));
        }
        self.check_market_notice_leads("more");
    }

    /// The Market box leads with every notice line, then the Market header.
    fn check_market_notice_leads(&mut self, kind: &str) {
        let shown: Vec<String> = self
            .market_box
            .as_ref()
            .map(labels_under_box)
            .unwrap_or_default()
            .iter()
            .map(|label| label.get_text().to_string())
            .collect();
        let count = self.market_notice.len();
        let leads = count > 0
            && shown.len() > count
            && shown[..count] == self.market_notice[..]
            && shown[count].starts_with("Market at ");
        if !leads {
            self.fail_trade(format!(
                "Market paid frames: {kind} notice did not lead the Market box ({:?}).",
                shown.iter().take(count + 1).collect::<Vec<_>>()
            ));
        }
    }

    fn advance_market_paid_shot(&mut self) -> bool {
        let Some(phase) = self.market_paid_shot else {
            return false;
        };
        let Some(dir) = self.market_paid_shot_dir.clone() else {
            return false;
        };
        if !self.market_open || self.market_notice.is_empty() {
            self.fail_trade(format!(
                "Market paid frames: {} had no open Market notice.",
                phase.file_name()
            ));
        }
        let path = format!("{dir}/{}", phase.file_name());
        if !self.save_shot(&path, false) {
            self.capture_failed = true;
        }
        match phase {
            MarketPaidShot::Single => {
                self.stage_market_paid_more();
                self.market_paid_shot = Some(MarketPaidShot::More);
                self.capture_frames = 4;
                true
            }
            MarketPaidShot::More => {
                self.market_paid_shot = None;
                false
            }
        }
    }

    /// Buy grain at Porto Novo, sail to Corsair's Rest, sell a partial lot for progress.
    fn prepare_contract_strip_progress(&mut self) {
        // Prefer a grain active if present; otherwise skip quietly.
        let grain_active = self.session.as_ref().is_some_and(|session| {
            session
                .board()
                .active
                .iter()
                .any(|contract| contract.good_id == "grain")
        });
        if !grain_active {
            return;
        }
        // Ensure docked at porto_novo with grain to buy.
        let home = self
            .session
            .as_ref()
            .map(|session| session.world().voyage.destination_id.clone())
            .unwrap_or_default();
        if home != "porto_novo" {
            return;
        }
        {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            let _ = session.buy("grain", 5);
        }
        let departed = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.depart("corsairs_rest")
        };
        if departed.is_err() {
            return;
        }
        for _ in 0..12 {
            self.next_day();
            if self.docked_id() == Some("corsairs_rest") {
                break;
            }
            if self
                .session
                .as_ref()
                .is_some_and(|session| session.world().pending_duel.is_some())
            {
                return;
            }
        }
        if self.docked_id() != Some("corsairs_rest") {
            return;
        }
        {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            let _ = session.sell("grain", 5);
        }
        self.refresh();
    }

    fn advance_until_strip_urgent(&mut self, max_days: usize) -> bool {
        for _ in 0..max_days {
            let urgent = self.session.as_ref().is_some_and(|session| {
                let day = session.world().day;
                let mut ordered: Vec<_> = session.board().active.iter().collect();
                ordered.sort_by(|a, b| {
                    a.deadline_day
                        .cmp(&b.deadline_day)
                        .then_with(|| a.title.cmp(&b.title))
                });
                ordered
                    .into_iter()
                    .take(2)
                    .any(|contract| contract.deadline_day - day <= 1)
            });
            if urgent {
                return true;
            }
            if self
                .session
                .as_ref()
                .is_none_or(|session| session.board().active.is_empty())
            {
                return false;
            }
            // Stay put: Next day while docked (or at sea) advances the calendar.
            self.next_day();
            if self
                .session
                .as_ref()
                .is_some_and(|session| session.world().pending_duel.is_some())
            {
                // Duel freezes the day; cannot reach urgent this way.
                return false;
            }
        }
        false
    }

    fn begin_contract_strip_shots(&mut self) {
        self.start_game();
        {
            let Some(session) = self.session.as_mut() else {
                self.fail_contract_strip("Contract-strip shots: no session.");
                return;
            };
            let _ = session.available_contracts();
        }
        // Accept two nearest-deadline offers for a calm two-segment strip.
        let sorted_ids = {
            let Some(session) = self.session.as_ref() else {
                return;
            };
            let mut offers = session.board().offers.clone();
            offers.sort_by(|a, b| {
                a.deadline_day
                    .cmp(&b.deadline_day)
                    .then_with(|| a.title.cmp(&b.title))
            });
            offers
                .into_iter()
                .take(2)
                .map(|offer| offer.id)
                .collect::<Vec<_>>()
        };
        for id in &sorted_ids {
            self.accept_contract_offer(id);
        }
        let _ = sorted_ids;
        self.refresh();
        if !self
            .contract_strip_nodes
            .as_ref()
            .is_some_and(contract_strip::overlay_visible)
        {
            self.fail_contract_strip("Contract-strip shots: active strip not visible.");
        }
        self.contract_strip_shot_dir = Some(contract_strip_shot_dir(self.shot_path.as_deref()));
        self.contract_strip_shot = Some(ContractStripShot::Active);
        self.refresh();
    }

    fn advance_contract_strip_shot(&mut self) -> bool {
        let Some(phase) = self.contract_strip_shot else {
            return false;
        };
        let Some(dir) = self.contract_strip_shot_dir.clone() else {
            return false;
        };
        if !self
            .contract_strip_nodes
            .as_ref()
            .is_some_and(contract_strip::overlay_visible)
        {
            self.smoke_ok = false;
            godot_print!(
                "Contract-strip smoke: frame {} was not visible.",
                phase.file_name()
            );
        }
        self.assert_contract_strip_fits();
        let path = format!("{dir}/{}", phase.file_name());
        if !self.save_shot(&path, false) {
            self.capture_failed = true;
        }
        match phase {
            ContractStripShot::Active => {
                if !self.advance_until_strip_urgent(40) {
                    self.smoke_ok = false;
                    godot_print!(
                        "Contract-strip smoke: could not reach urgent frame via Next day."
                    );
                }
                self.refresh();
                self.contract_strip_shot = Some(ContractStripShot::Urgent);
                self.capture_frames = 4;
                true
            }
            ContractStripShot::Urgent => {
                self.contract_strip_shot = None;
                self.contract_strip_checked = true;
                false
            }
        }
    }

    fn post_board(&mut self) {
        let board = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.bounty_board()
        };
        for target in &board {
            hunt_screen::remember(&mut self.hunt_desk.known, target.clone());
        }
        self.hunt_desk.board = board;
        self.hunt_desk.posted = true;
        self.refresh();
    }

    fn forage_day(&mut self) {
        self.hunt_desk.confirm = None;
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.hunt()
        };
        self.hunt_desk.notice = match result {
            Ok(result) => hunt_screen::forage_notice(&result),
            Err(err) => err.to_string(),
        };
        self.refresh();
    }

    fn accept_listed(&mut self, id: &str) {
        self.hunt_desk.confirm = None;
        if let Some(target) = self
            .hunt_desk
            .board
            .iter()
            .find(|row| row.captain_id == id)
            .cloned()
        {
            hunt_screen::remember(&mut self.hunt_desk.known, target);
        }
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.accept_bounty(id)
        };
        self.hunt_desk.notice = match result {
            Ok(()) => {
                let name = hunt_screen::display_name(id, &self.hunt_desk.known);
                format!("Accepted {name}.")
            }
            Err(err) => err.to_string(),
        };
        self.refresh();
    }

    fn ask_hunt(&mut self, id: &str) {
        let name = hunt_screen::display_name(id, &self.hunt_desk.known);
        // F1: the flagship's crew against the minimum `depart` refuses on.
        let crew = self
            .session
            .as_ref()
            .and_then(|session| session.world().captain.ship.as_ref())
            .map(|ship| (ship.crew, portlight_sim::ship::template_crew_min(ship)));
        self.hunt_desk.notice = hunt_screen::hunt_confirm_text(&name, crew);
        self.hunt_desk.confirm = Some(HuntConfirm::HuntTarget(id.to_string()));
        self.refresh();
    }

    fn ask_claim(&mut self, id: &str) {
        let name = hunt_screen::display_name(id, &self.hunt_desk.known);
        self.hunt_desk.notice = hunt_screen::claim_confirm_text(&name);
        self.hunt_desk.confirm = Some(HuntConfirm::Claim(id.to_string()));
        self.refresh();
    }

    fn confirm_hunt(&mut self) {
        let Some(confirm) = self.hunt_desk.confirm.take() else {
            return;
        };
        match confirm {
            HuntConfirm::HuntTarget(id) => self.pursue_bounty(&id),
            HuntConfirm::Claim(id) => self.claim_listed(&id),
        }
    }

    fn pursue_bounty(&mut self, id: &str) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.hunt_bounty(id)
        };
        match result {
            Ok(state) => {
                self.hunt_open = false;
                self.hunt_desk.confirm = None;
                self.open_agency(state, String::new());
            }
            Err(err) => {
                self.hunt_desk.notice = err.to_string();
            }
        }
        self.refresh();
    }

    fn claim_listed(&mut self, id: &str) {
        let result = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session.claim_bounty(id)
        };
        self.hunt_desk.notice = match result {
            Ok(silver) => hunt_screen::claim_notice(silver),
            Err(err) => err.to_string(),
        };
        self.refresh();
    }

    fn perform_hunt(&mut self, action: HuntAction) {
        match action {
            HuntAction::Forage => self.forage_day(),
            HuntAction::RefreshBoard => {
                self.hunt_desk.confirm = None;
                self.post_board();
            }
            HuntAction::Accept(id) => self.accept_listed(&id),
            HuntAction::AskHunt(id) => self.ask_hunt(&id),
            HuntAction::AskClaim(id) => self.ask_claim(&id),
            HuntAction::Confirm => self.confirm_hunt(),
            HuntAction::Cancel => {
                self.hunt_desk.confirm = None;
                self.hunt_desk.notice.clear();
                self.refresh();
            }
            HuntAction::Close => self.close_hunt(),
        }
    }

    fn sync_hunt(&mut self) {
        let Some(mut nodes) = self.hunt_nodes.clone() else {
            return;
        };
        hunt_screen::show_hunt(&mut nodes, self.hunt_open);
        if !self.hunt_open {
            return;
        }
        let Some(model) = self.hunt_model_now() else {
            return;
        };
        nodes.eyebrow.set_text(&model.eyebrow);
        nodes.title.set_text(&model.title);
        nodes.notice.set_text(&model.notice);
        let template_id = self
            .session
            .as_ref()
            .and_then(|session| session.world().captain.ship.as_ref())
            .map(|ship| ship.template_id.clone())
            .unwrap_or_default();
        set_ship_plate(
            &mut nodes.plate,
            &mut nodes.plate_panel,
            &mut nodes.placeholder,
            &mut nodes.plate_caption,
            &template_id,
        );
        let game = self.instance_id();
        let mut body = nodes.body.clone();
        hunt_screen::rebuild_body(&mut body, &model, &|text, action| {
            hunt_button(game, text, action)
        });
        let mut confirm_row = nodes.confirm_row.clone();
        hunt_screen::clear_row(&mut confirm_row);
        if self.hunt_desk.confirm.is_some() {
            confirm_row.set_visible(true);
            confirm_row.add_child(&hunt_button(game, "Confirm", HuntAction::Confirm));
            confirm_row.add_child(&hunt_button(game, "Cancel", HuntAction::Cancel));
        } else {
            confirm_row.set_visible(false);
        }
        let mut footer = nodes.footer.clone();
        hunt_screen::clear_box(&mut footer);
        footer.add_child(&hunt_button(game, "Close", HuntAction::Close));
    }

    fn scroll_hunt(&mut self, section: &str) {
        let Some(nodes) = self.hunt_nodes.clone() else {
            return;
        };
        let mut scroll = nodes.scroll.clone();
        hunt_screen::scroll_to(&mut scroll, &nodes.body, section);
    }

    fn begin_hunt_shots(&mut self) {
        self.start_game();
        self.open_hunt();
        self.forage_day();
        self.hunt_shot_dir = Some(hunt_shot_dir(self.shot_path.as_deref()));
        self.hunt_shot = Some(HuntShot::Forage);
        self.hunt_scrolled = false;
        self.refresh();
    }

    fn advance_hunt_shot(&mut self) -> bool {
        let Some(phase) = self.hunt_shot else {
            return false;
        };
        let Some(dir) = self.hunt_shot_dir.clone() else {
            return false;
        };
        if !self.hunt_scrolled {
            self.scroll_hunt(phase.section());
            self.hunt_scrolled = true;
            self.capture_frames = 2;
            return true;
        }
        self.hunt_scrolled = false;
        if !self.hunt_frame_ready(phase) {
            self.smoke_ok = false;
        }
        let path = format!("{dir}/{}", phase.file_name());
        if !self.save_shot(&path, true) {
            self.capture_failed = true;
        }
        match phase {
            HuntShot::Forage => {
                self.post_board();
                self.hunt_shot = Some(HuntShot::Board);
            }
            HuntShot::Board => {
                let id = self
                    .hunt_desk
                    .board
                    .first()
                    .map(|row| row.captain_id.clone());
                if let Some(id) = id {
                    self.accept_listed(&id);
                } else {
                    self.smoke_ok = false;
                    godot_print!("Hunt smoke: no offer to accept for the active frame");
                }
                self.hunt_shot = Some(HuntShot::Active);
            }
            HuntShot::Active => {
                self.hunt_shot = None;
                return false;
            }
        }
        self.capture_frames = 4;
        true
    }

    fn hunt_frame_ready(&mut self, phase: HuntShot) -> bool {
        let open = self.hunt_open;
        let notice = self.hunt_desk.notice.clone();
        let offers = self.hunt_desk.board.len();
        let active = self
            .session
            .as_ref()
            .map(|session| session.world().captain.active_bounties.len())
            .unwrap_or(0);
        let ok = match phase {
            HuntShot::Forage => open && notice.contains("Provisions") && self.encounter.is_none(),
            HuntShot::Board => open && (1..=3).contains(&offers),
            HuntShot::Active => open && active >= 1,
        };
        if !ok {
            let line = format!("Hunt smoke: frame {} was not ready.", phase.file_name());
            godot_print!("{line}");
            self.push_log(line);
        }
        ok
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
        self.expect_duel_logged("encounter");
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

    /// R11: the fight that reached Outcome logged its result as the last line.
    fn expect_duel_logged(&mut self, label: &str) {
        let last = self.log_lines.last().cloned().unwrap_or_default();
        if !last.starts_with("Won the duel") {
            self.smoke_ok = false;
            let line = format!("Encounter smoke: {label} duel result not logged ({last:?}).");
            godot_print!("{line}");
            self.push_log(line);
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
        self.expect_choice_previews("Encounter smoke");
        self.expect_escape_inert_mid_choice();
    }

    /// The drawn muted lines under the Outcome buttons, in button order.
    fn choice_preview_lines(&self) -> Vec<String> {
        let Some(nodes) = self.encounter_nodes.as_ref() else {
            return Vec::new();
        };
        labels_under_box(&nodes.actions)
            .into_iter()
            .filter(|label| {
                label.get_name() == encounter_screen::CHOICE_PREVIEW && label.is_visible_in_tree()
            })
            .map(|label| label.get_text().to_string())
            .collect()
    }

    /// Spare and Take all each carry the binding line from `facts.strength`.
    /// Capture carries none.
    fn expect_choice_previews(&mut self, scope: &str) {
        let strength = self
            .encounter
            .as_ref()
            .map(|facts| facts.strength)
            .unwrap_or_default();
        let expected = vec![
            crate::logic::spare_preview(strength),
            crate::logic::take_all_preview(strength),
        ];
        let drawn = self.choice_preview_lines();
        if drawn != expected {
            self.smoke_ok = false;
            self.push_log(format!(
                "{scope}: outcome previews were {drawn:?}, expected {expected:?}."
            ));
            return;
        }
        godot_print!("encounter choice preview {}", drawn.join(" | "));
    }

    /// `ui_cancel` mid-choice does not leave, spare, or take.
    fn expect_escape_inert_mid_choice(&mut self) {
        let silver = self
            .session
            .as_ref()
            .map(|session| session.world().captain.silver);
        self.dismiss_cancel();
        let still_pending = self
            .session
            .as_ref()
            .is_some_and(|session| session.pending_victory());
        let same_silver = self
            .session
            .as_ref()
            .map(|session| session.world().captain.silver)
            == silver;
        if !still_pending
            || !same_silver
            || self.encounter.is_none()
            || !self.phase_is(ScreenPhase::Outcome)
            || self.choice_preview_lines().len() != 2
        {
            self.smoke_ok = false;
            self.push_log("Encounter smoke: Escape acted on the outcome choice.".to_string());
        }
    }

    /// After the choice the previews are gone and #41's receipt carries the
    /// purse the preview named.
    fn expect_previews_cleared(&mut self, scope: &str, spared: bool) {
        if !self.choice_preview_lines().is_empty() {
            self.smoke_ok = false;
            self.push_log(format!(
                "{scope}: outcome previews stayed after the choice."
            ));
        }
        let strength = self
            .encounter
            .as_ref()
            .map(|facts| facts.strength)
            .unwrap_or_default();
        let purse = if spared {
            crate::logic::spare_purse(strength)
        } else {
            crate::logic::take_all_purse(strength)
        };
        let receipt = format!("+{purse} silver.");
        let logged = self
            .encounter
            .as_ref()
            .is_some_and(|facts| facts.log.lines().any(|line| line == receipt));
        if !logged {
            self.smoke_ok = false;
            self.push_log(format!("{scope}: receipt did not log {receipt}"));
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
        self.expect_previews_cleared("Encounter smoke", true);
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
        self.expect_duel_logged("bounty");
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
        self.expect_choice_previews("Bounty smoke");
        self.take_prize();
        self.expect_previews_cleared("Bounty smoke", false);
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

struct ListedOffer {
    id: String,
    title: String,
    detail: String,
    meta: String,
    due: bool,
    /// T-S. Docked market fact for the good; `None` at sea.
    availability: Option<String>,
}

struct ListedActive {
    id: String,
    title: String,
    detail: String,
    meta: String,
    due: bool,
    can_complete: bool,
}

struct ContractListing {
    offers: Vec<ListedOffer>,
    active: Vec<ListedActive>,
    recent: Vec<String>,
}

fn contract_listing(session: &Session) -> ContractListing {
    let day = session.world().day;
    let world = session.world();
    let docked_market = docked_port_id(session)
        .and_then(|id| world.port(id))
        .map(|port| port.market.as_slice());
    let offers = session
        .board()
        .offers
        .iter()
        .map(|offer| {
            let destination = port_name(world, &offer.destination_port_id);
            let good = good_name(&offer.good_id);
            let (days, due) = contracts_screen::days_left_text(day, offer.deadline_day);
            ListedOffer {
                id: offer.id.clone(),
                title: contracts_screen::ascii_sentence(&offer.title),
                detail: format!(
                    "{} x{} to {}   {}",
                    contracts_screen::ascii_sentence(&good),
                    offer.quantity,
                    contracts_screen::ascii_sentence(&destination),
                    contracts_screen::reward_text(offer.reward_silver, offer.bonus_reward)
                ),
                meta: format!(
                    "{days}   {}   {}",
                    contracts_screen::requirement_text(
                        &offer.required_trust_tier,
                        offer.required_standing
                    ),
                    contracts_screen::ascii_sentence(&offer.offer_reason)
                ),
                due,
                availability: contracts_screen::availability_tag(docked_market, &offer.good_id),
            }
        })
        .collect();
    let active = session
        .board()
        .active
        .iter()
        .filter(|contract| contract.status == "accepted")
        .map(|contract| {
            let destination = port_name(world, &contract.destination_port_id);
            let good = good_name(&contract.good_id);
            let progress = contracts_screen::progress_text(
                contract.delivered_quantity,
                contract.required_quantity,
            );
            let (days, due) = contracts_screen::days_left_text(day, contract.deadline_day);
            ListedActive {
                id: contract.offer_id.clone(),
                title: contracts_screen::ascii_sentence(&contract.title),
                detail: format!(
                    "{} {progress} to {}",
                    contracts_screen::ascii_sentence(&good),
                    contracts_screen::ascii_sentence(&destination)
                ),
                meta: format!(
                    "{days}   {}",
                    contracts_screen::reward_text(contract.reward_silver, contract.bonus_reward)
                ),
                due,
                can_complete: contracts_screen::can_complete(
                    &contract.status,
                    contract.delivered_quantity,
                    contract.required_quantity,
                ),
            }
        })
        .collect();
    let recent = session
        .board()
        .completed
        .iter()
        .rev()
        .take(4)
        .map(contracts_screen::recent_line)
        .collect();
    ContractListing {
        offers,
        active,
        recent,
    }
}

fn control_on_screen(control: &Gd<Control>) -> bool {
    let pos = control.get_global_position();
    let size = control.get_size();
    size.y >= 1.0 && pos.y >= 0.0 && pos.y + size.y <= WINDOW_H
}

fn find_control_named(node: &Gd<Node>, name: &str) -> Option<Gd<Control>> {
    if node.get_name() == name {
        if let Ok(control) = node.clone().try_cast::<Control>() {
            return Some(control);
        }
    }
    for child in node.get_children().iter_shared() {
        if let Some(found) = find_control_named(&child, name) {
            return Some(found);
        }
    }
    None
}

fn find_button_text(node: &Gd<Node>, text: &str) -> Option<Gd<Button>> {
    if let Ok(button) = node.clone().try_cast::<Button>() {
        if button.get_text() == text {
            return Some(button);
        }
    }
    for child in node.get_children().iter_shared() {
        if let Some(found) = find_button_text(&child, text) {
            return Some(found);
        }
    }
    None
}

fn find_label_named(node: &Gd<Node>, name: &str) -> Option<Gd<Label>> {
    if node.get_name() == name {
        if let Ok(label) = node.clone().try_cast::<Label>() {
            return Some(label);
        }
    }
    for child in node.get_children().iter_shared() {
        if let Some(found) = find_label_named(&child, name) {
            return Some(found);
        }
    }
    None
}

fn tree_has_button(node: &Gd<Node>, text: &str) -> bool {
    if let Ok(button) = node.clone().try_cast::<Button>() {
        if button.get_text() == text {
            return true;
        }
    }
    node.get_children()
        .iter_shared()
        .any(|child| tree_has_button(&child, text))
}

impl PortlightGame {
    fn encounter_text(&self) -> String {
        let Some(session) = self.session.as_ref() else {
            return String::new();
        };
        let Some(duel) = session.world().pending_duel.as_ref() else {
            return String::new();
        };
        crate::logic::duel_prompt(duel)
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
        receipt.action,
        receipt.quantity,
        good_name(&receipt.good_id),
        receipt.total_price
    )
}

fn sale_lines(sale: &Sale) -> Vec<String> {
    let mut lines = vec![receipt_line(&sale.receipt)];
    for contract in &sale.contracts {
        if !contract.summary.is_empty() {
            lines.push(contracts_screen::outcome_summary(contract));
        }
    }
    lines
}

fn duel_outcome_line(outcome: &DuelOutcome) -> String {
    let result = if outcome.player_won {
        "Won"
    } else if outcome.draw {
        "Drew"
    } else {
        "Lost"
    };
    crate::logic::duel_result_line(
        result,
        &outcome.opponent_name,
        outcome.silver_delta,
        outcome.standing_delta,
    )
}

/// R11. Result line for a personal-fight step that ended the duel. A win
/// logs before Spare / Take all add their own receipt lines.
fn fight_result_line(step: &EncounterStep, silver_delta: i64) -> Option<String> {
    let result = if step.player_won {
        "Won"
    } else if step.draw {
        "Drew"
    } else if step.phase == "resolved" {
        "Lost"
    } else {
        return None;
    };
    Some(crate::logic::duel_result_line(
        result,
        &step.enemy_captain_name,
        silver_delta,
        0,
    ))
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

fn scripted_launch() -> bool {
    user_arg("--encounter-screen")
        || user_arg("--encounter-galleon")
        || user_arg("--encounter")
        || user_arg("--duel")
        || user_arg("--resolve")
        || user_arg("--work")
        || user_arg("--crew-screen")
        || user_arg("--art")
        || user_arg("--contracts-screen")
        || user_arg("--harbour-screen")
        || flag_set("PORTLIGHT_SMOKE")
        || user_arg("--smoke")
}

fn harbour_frames_requested(shot_set: bool) -> bool {
    shot_set || docs_capture()
}

fn newgame_frames_requested(shot_set: bool) -> bool {
    shot_set || docs_capture()
}

fn crew_frames_requested(shot_set: bool) -> bool {
    shot_set || docs_capture()
}

fn hunt_frames_requested(shot_set: bool) -> bool {
    shot_set || docs_capture()
}

fn day_report_frames_requested(shot_set: bool) -> bool {
    shot_set || docs_capture()
}

fn day_report_shot_dir(shot: Option<&str>) -> String {
    newgame_shot_dir(shot)
}

fn contract_strip_frames_requested(shot_set: bool) -> bool {
    shot_set || docs_capture()
}

fn contract_strip_shot_dir(shot: Option<&str>) -> String {
    newgame_shot_dir(shot)
}

fn market_paid_frames_requested(shot_set: bool) -> bool {
    shot_set || docs_capture()
}

fn market_paid_shot_dir(shot: Option<&str>) -> String {
    newgame_shot_dir(shot)
}

fn hunt_shot_dir(shot: Option<&str>) -> String {
    newgame_shot_dir(shot)
}

/// `PORTLIGHT_SHOT` wins. A `.png` path contributes its directory. `--art-docs`
/// writes `docs/screenshots`. Headless still tries the save and fails it.
fn newgame_shot_dir(shot: Option<&str>) -> String {
    if let Some(path) = shot {
        let path = std::path::Path::new(path);
        if path.extension().and_then(|ext| ext.to_str()) == Some("png") {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    return parent.to_string_lossy().into_owned();
                }
            }
            return ".".to_string();
        }
        return path.to_string_lossy().into_owned();
    }
    if docs_capture() {
        resolve_repo_path("docs/screenshots")
    } else {
        "/tmp".to_string()
    }
}

fn fresh_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn leading_none(ids: impl Iterator<Item = String>) -> Vec<String> {
    let mut rows = vec![String::new()];
    rows.extend(ids);
    rows
}

fn cycled(ids: &[String], current: &str, delta: i32) -> String {
    let index = ids.iter().position(|id| id == current).unwrap_or(0);
    ids.get(cycle_index(ids.len(), index, delta))
        .cloned()
        .unwrap_or_default()
}

fn pool_name(pool: PointPool) -> &'static str {
    match pool {
        PointPool::Trade => "Trade",
        PointPool::Sailing => "Sailing",
        PointPool::Shadow => "Shadow",
        PointPool::Reputation => "Reputation",
    }
}

fn named_or_none(choices: &[portlight_sim::custom_captain::NamedChoice], id: &str) -> String {
    if id.is_empty() {
        return "none".to_string();
    }
    choices
        .iter()
        .find(|choice| choice.id == id)
        .map(|choice| ascii_label(&choice.name, &choice.id).to_string())
        .unwrap_or_else(|| id.to_string())
}

fn mentor_or_none(choices: &[portlight_sim::custom_captain::MentorChoice], id: &str) -> String {
    if id.is_empty() {
        return "none".to_string();
    }
    choices
        .iter()
        .find(|choice| choice.id == id)
        .map(|choice| ascii_label(&choice.name, &choice.id).to_string())
        .unwrap_or_else(|| id.to_string())
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

impl PortlightGame {
    /// Loads `res://assets/audio/sfx/{id}.ogg` once and round-robins the pool.
    /// Headless, smoke, and screenshot runs return before any load. A missing
    /// file is a silent no-op so smoke does not fail before the asset drop.
    fn play_sfx(&mut self, id: &str) {
        if headless_runtime() || self.smoke || self.shot_path.is_some() {
            return;
        }
        if self.sfx_players.is_empty() {
            return;
        }
        let path = format!("res://assets/audio/sfx/{id}.ogg");
        let stream = if let Some(cached) = self.sfx_cache.get(id) {
            cached.clone()
        } else if let Ok(stream) = try_load::<AudioStream>(&path) {
            self.sfx_cache.insert(id.to_string(), stream.clone());
            stream
        } else {
            return;
        };
        let i = self.sfx_rr % self.sfx_players.len();
        self.sfx_rr = self.sfx_rr.wrapping_add(1);
        let mut player = self.sfx_players[i].clone();
        player.set_stream(&stream);
        player.play();
    }
}

#[godot_api]
impl PortlightGame {
    /// What a playtest client can see and press. `perform` is synchronous, so
    /// the bridge does not wait on a frame.
    #[func]
    fn playtest_observation(&self) -> VarDictionary {
        self.playtest_observation_dict()
    }

    /// Empty string means the action was applied. A message means it was not.
    #[func]
    fn playtest_apply(&mut self, action: VarDictionary) -> GString {
        self.playtest_apply_dict(&action)
    }

    /// Fresh title. The next seat must not inherit a voyage or "Back to the chart".
    #[func]
    fn playtest_reset(&mut self) {
        self.playtest_reset_to_title();
    }

    #[func]
    fn playtest_ready(&self) -> bool {
        true
    }
}

impl PortlightGame {
    fn dismiss_cancel(&mut self) {
        if self.day_report_open {
            self.close_day_report();
            if let Some(mut viewport) = self.base().get_viewport() {
                viewport.set_input_as_handled();
            }
        }
    }

    fn playtest_reset_to_title(&mut self) {
        // Drop field handles before refresh frees the widgets they point at.
        self.name_edit = None;
        self.title_edit = None;
        self.story_edit = None;
        self.rename_edit = None;
        self.session = None;
        self.log_lines.clear();
        self.market_open = false;
        self.armed_sail = None;
        self.stances.clear();
        self.encounter = None;
        self.capture_crew = 0;
        self.hunt_open = false;
        self.hunt_desk = HuntDesk::default();
        self.day_report_open = false;
        self.day_report_doc = None;
        self.day_report_memory.reset();
        self.contracts_open = false;
        self.contracts_notice.clear();
        self.contracts_confirm = None;
        self.shipyard_open = false;
        self.shipyard_notice.clear();
        self.shipyard_confirm = None;
        self.shipyard_shown = None;
        self.rename_draft.clear();
        self.journal_open = false;
        self.journal_filled = false;
        self.journal_expanded.clear();
        self.harbour_open = false;
        self.harbour_notice.clear();
        self.harbour_pending = None;
        self.crew_open = false;
        self.crew_notice.clear();
        self.crew_pending = None;
        self.draft = CustomDraft::default();
        self.newgame_notice.clear();
        self.open_newgame(NewgamePage::Title);
    }

    fn playtest_apply_dict(&mut self, action: &VarDictionary) -> GString {
        match dict_string(action, "kind").as_str() {
            "choose" => {
                let id = dict_string(action, "id");
                if id.is_empty() {
                    return GString::from("choose needs an id");
                }
                self.playtest_choose(&id)
            }
            "line" => self.playtest_set_line(&dict_string(action, "line")),
            "key" => self.playtest_key(&dict_string(action, "key")),
            "call" => GString::from("call is not a Portlight action"),
            other => GString::from(format!("unknown action kind '{other}'").as_str()),
        }
    }

    fn playtest_choose(&mut self, id: &str) -> GString {
        let root = self.to_gd().upcast::<Node>();
        if !offered_choices(&root)
            .iter()
            .any(|(offered, _)| offered == id)
        {
            return GString::from(format!("'{id}' is not on screen").as_str());
        }
        match parse_playtest_id(id) {
            Some(PlaytestCommand::Action(action)) => self.perform(action),
            Some(PlaytestCommand::Hunt(action)) => self.perform_hunt(action),
            None => return GString::from(format!("'{id}' is not a known action").as_str()),
        }
        GString::new()
    }

    fn playtest_set_line(&mut self, line: &str) -> GString {
        // `read_draft_fields` copies the LineEdit over `draft.name` on the next
        // menu action, so both have to move together.
        let Some(edit) = self.name_edit.as_mut() else {
            return GString::from("no line field is on screen");
        };
        if !edit.is_visible_in_tree() {
            return GString::from("no line field is on screen");
        }
        edit.set_text(line);
        self.draft.name = line.to_string();
        GString::new()
    }

    fn playtest_key(&mut self, key: &str) -> GString {
        let normalized = key.trim().to_ascii_lowercase();
        if normalized == "escape" || normalized == "ui_cancel" {
            // A no-op is success. The next observation shows the overlay still open.
            self.dismiss_cancel();
            return GString::new();
        }
        GString::from("that key does nothing here")
    }

    fn playtest_observation_dict(&self) -> VarDictionary {
        let root = self.to_gd().upcast::<Node>();
        let choices = offered_choices(&root);
        let text = self.playtest_text(&choices);
        let mut options = VarArray::new();
        for (id, label) in &choices {
            options.push(&vdict! { "id" => id.as_str(), "label" => label.as_str() });
        }
        let mut actions = VarDictionary::new();
        actions.set("kind", "choice");
        actions.set("options", &options);
        let mut open = VarArray::new();
        for name in self.open_desks() {
            open.push(name);
        }
        let (captain, day, silver, docked, place) = self.playtest_facts();
        let hire = choices.iter().any(|(id, _)| id == "chart.hire");
        let mut state = VarDictionary::new();
        state.set("screen", self.playtest_screen());
        state.set("captain", captain.as_str());
        state.set("day", day);
        state.set("silver", silver);
        state.set("docked", docked.as_str());
        state.set("place", place.as_str());
        state.set("open", &open);
        state.set("hire_on_screen", hire);
        let mut result = VarDictionary::new();
        result.set("text", text.as_str());
        result.set("state", &state);
        result.set("actions", &actions);
        result.set("done", false);
        result
    }

    fn playtest_text(&self, choices: &[(String, String)]) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Screen: {}", self.playtest_screen()));
        lines.push(self.status_text());
        for notice in self.visible_notices() {
            lines.push(notice);
        }
        lines.push("Actions:".to_string());
        for (id, label) in choices {
            lines.push(format!("{id} — {label}"));
        }
        if !self.log_lines.is_empty() {
            lines.push("Log:".to_string());
            lines.extend(self.log_lines.iter().cloned());
        }
        lines.join("\n")
    }

    fn visible_notices(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if !self.newgame_notice.is_empty() {
            lines.push(self.newgame_notice.clone());
        }
        if self.contracts_open && !self.contracts_notice.is_empty() {
            lines.push(self.contracts_notice.clone());
        }
        if self.shipyard_open && !self.shipyard_notice.is_empty() {
            lines.push(self.shipyard_notice.clone());
        }
        if self.harbour_open && !self.harbour_notice.is_empty() {
            lines.push(self.harbour_notice.clone());
        }
        if self.crew_open && !self.crew_notice.is_empty() {
            lines.push(self.crew_notice.clone());
        }
        if self.hunt_open && !self.hunt_desk.notice.is_empty() {
            lines.push(self.hunt_desk.notice.clone());
        }
        lines
    }

    fn playtest_screen(&self) -> &'static str {
        if self.day_report_open {
            return "day-report";
        }
        if self.encounter.as_ref().and_then(present).is_some() {
            return "encounter";
        }
        if self.hunt_open {
            return "hunt";
        }
        if self.crew_open {
            return "crew";
        }
        if self.contracts_open {
            return "contracts";
        }
        if self.shipyard_open {
            return "shipyard";
        }
        if self.harbour_open {
            return "harbour";
        }
        if self.journal_open {
            return "journal";
        }
        match self.newgame_page {
            NewgamePage::Title => "title",
            NewgamePage::Captains => "captains",
            NewgamePage::Custom => "custom",
            NewgamePage::Load => "load",
            NewgamePage::Saved => "saved",
            NewgamePage::Hidden => "chart",
        }
    }

    fn open_desks(&self) -> Vec<&'static str> {
        let mut open = Vec::new();
        if self.day_report_open {
            open.push("day-report");
        }
        if self.encounter.as_ref().and_then(present).is_some() {
            open.push("encounter");
        }
        if self.hunt_open {
            open.push("hunt");
        }
        if self.crew_open {
            open.push("crew");
        }
        if self.contracts_open {
            open.push("contracts");
        }
        if self.shipyard_open {
            open.push("shipyard");
        }
        if self.harbour_open {
            open.push("harbour");
        }
        if self.journal_open {
            open.push("journal");
        }
        open
    }

    fn playtest_facts(&self) -> (String, i64, i64, String, String) {
        let Some(session) = self.session.as_ref() else {
            return (String::new(), 0, 0, String::new(), String::new());
        };
        let world = session.world();
        let docked = docked_port_id(session).unwrap_or("").to_string();
        let place = if docked.is_empty() {
            match world.voyage.status {
                VoyageStatus::AtSea => format!(
                    "{} -> {}",
                    port_name(world, &world.voyage.origin_id),
                    port_name(world, &world.voyage.destination_id)
                ),
                VoyageStatus::Arrived | VoyageStatus::InPort => {
                    port_name(world, &world.voyage.destination_id)
                }
            }
        } else {
            port_name(world, &docked)
        };
        (
            world.captain.name.clone(),
            world.day,
            world.captain.silver,
            docked,
            place,
        )
    }
}

fn dict_string(dict: &VarDictionary, key: &str) -> String {
    let Some(value) = dict.get(key) else {
        return String::new();
    };
    value
        .try_to::<GString>()
        .map(|text| text.to_string())
        .unwrap_or_default()
}

fn offered_choices(root: &Gd<Node>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    walk_playtest_buttons(root, &mut |button| {
        if !button.is_visible_in_tree() || button.is_disabled() || !button.has_meta("playtest_id") {
            return;
        }
        let Ok(id) = button.get_meta("playtest_id").try_to::<GString>() else {
            return;
        };
        let id = id.to_string();
        if id.is_empty() || !seen.insert(id.clone()) {
            return;
        }
        out.push((id, button.get_text().to_string()));
    });
    out
}

fn walk_playtest_buttons(node: &Gd<Node>, visit: &mut dyn FnMut(&Gd<Button>)) {
    if let Ok(button) = node.clone().try_cast::<Button>() {
        visit(&button);
    }
    for child in node.get_children().iter_shared() {
        walk_playtest_buttons(&child, visit);
    }
}

fn stamp_playtest_id(button: &mut Gd<Button>, id: &str) {
    let value = id.to_variant();
    button.set_meta("playtest_id", &value);
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

/// Chart port-row control. Same theme fill as [`action_button`], plus a
/// 1 px MUTED left and right border so each button reads as its own control.
///
/// The face is 10 px with no horizontal padding so Market, Contracts, Hire,
/// Stores, Work, Shipyard, Harbour, Crew, and Hunt stay on one line inside
/// the panel. An 11 px face fits seven labels; more grow the panel and slide
/// the chrome. The minimum height stays 31 px, so the button band stays
/// y 257-287.
fn port_row_button(text: &str, game: InstanceId, action: Action) -> Gd<Button> {
    let mut button = action_button(text, game, action);
    button.add_theme_font_size_override("font_size", 10);
    button.set_custom_minimum_size(Vector2::new(0.0, 31.0));
    // Default side padding plus an eighth label overruns the panel and
    // reflows everything below the row. Zero horizontal padding keeps the
    // 10 px face on one line. `align_to_largest_stylebox` uses the widest
    // state, including hover_pressed.
    for state in [
        "normal",
        "hover",
        "pressed",
        "focus",
        "disabled",
        "hover_disabled",
        "hover_pressed",
    ] {
        let Some(style) = button.get_theme_stylebox(state) else {
            continue;
        };
        let mut boxed = style.duplicate_resource();
        boxed.set_content_margin(godot::builtin::Side::LEFT, 0.0);
        boxed.set_content_margin(godot::builtin::Side::RIGHT, 0.0);
        // The border draws inside the box. With the content margin pinned
        // at 0 it adds no minimum width. Focus is the theme's outline
        // overlay, so it keeps its own border.
        if state != "focus" {
            if let Ok(mut flat) = boxed.clone().try_cast::<StyleBoxFlat>() {
                flat.set_border_width(godot::builtin::Side::LEFT, 1);
                flat.set_border_width(godot::builtin::Side::RIGHT, 1);
                flat.set_border_color(MUTED);
            }
        }
        button.add_theme_stylebox_override(state, &boxed);
    }
    button
}

fn hunt_button(game: InstanceId, text: &str, action: HuntAction) -> Gd<Button> {
    let mut button = Button::new_alloc();
    button.set_text(text);
    encounter_screen::style_encounter_button(&mut button);
    button.set_h_size_flags(SizeFlags::EXPAND_FILL);
    stamp_playtest_id(&mut button, &hunt_playtest_id(&action));
    button.signals().pressed().connect(move || {
        let Ok(mut gd) = Gd::<PortlightGame>::try_from_instance_id(game) else {
            return;
        };
        gd.bind_mut().perform_hunt(action.clone());
    });
    button
}

fn action_button(text: &str, game: InstanceId, action: Action) -> Gd<Button> {
    let mut button = Button::new_alloc();
    button.set_text(text);
    button.add_theme_color_override("font_color", CREAM);
    button.add_theme_color_override("font_hover_color", GOLD);
    stamp_playtest_id(&mut button, &action_playtest_id(&action));
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
fn crew_button(text: &str, game: InstanceId, action: Action) -> Gd<Button> {
    let mut button = Button::new_alloc();
    button.set_text(text);
    crew_screen::style_crew_button(&mut button);
    stamp_playtest_id(&mut button, &action_playtest_id(&action));
    let action_for_click = action;
    button.signals().pressed().connect(move || {
        let Ok(mut gd) = Gd::<PortlightGame>::try_from_instance_id(game) else {
            return;
        };
        gd.bind_mut().perform(action_for_click.clone());
    });
    button
}

fn crew_section(name: &str) -> Gd<VBoxContainer> {
    let mut section = VBoxContainer::new_alloc();
    section.set_name(name);
    section.set_h_size_flags(SizeFlags::EXPAND_FILL);
    section.add_theme_constant_override("separation", 6);
    section
}

fn crew_subhead(text: &str) -> Gd<Label> {
    body_label(text, 16, GOLD)
}

fn crew_copy(text: &str) -> Gd<Label> {
    let mut label = body_label(text, 15, CREAM);
    label.set_autowrap_mode(AutowrapMode::WORD_SMART);
    label.set_h_size_flags(SizeFlags::EXPAND_FILL);
    label
}

fn crew_muted(text: &str) -> Gd<Label> {
    let mut label = body_label(text, 14, MUTED);
    label.set_autowrap_mode(AutowrapMode::WORD_SMART);
    label.set_h_size_flags(SizeFlags::EXPAND_FILL);
    label
}

fn section_y(body: &Gd<VBoxContainer>, name: &str) -> Option<i32> {
    section_offset(&body.clone().upcast::<Node>(), name, 0.0).map(|y| y.round() as i32)
}

fn section_offset(node: &Gd<Node>, name: &str, origin_y: f32) -> Option<f32> {
    for child in node.get_children().iter_shared() {
        let Ok(control) = child.clone().try_cast::<Control>() else {
            continue;
        };
        if child.get_name() == name {
            return Some(origin_y + control.get_position().y);
        }
        if let Some(found) = section_offset(&child, name, origin_y + control.get_position().y) {
            return Some(found);
        }
    }
    None
}

fn encounter_button(text: &str, game: InstanceId, action: Action) -> Gd<Button> {
    let mut button = Button::new_alloc();
    button.set_text(text);
    encounter_screen::style_encounter_button(&mut button);
    stamp_playtest_id(&mut button, &action_playtest_id(&action));
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

fn clear_hbox(node: &mut Gd<HBoxContainer>) {
    let children = node.get_children();
    for mut child in children.iter_shared() {
        node.remove_child(&child);
        child.queue_free();
    }
}

fn infrastructure_args(intent: &HarbourIntent) -> Vec<String> {
    match intent {
        HarbourIntent::LeaseWarehouse(tier) => vec![tier.clone()],
        HarbourIntent::OpenBroker { region, tier } => vec![region.clone(), tier.clone()],
        HarbourIntent::BuyLicense(id) => vec![id.clone()],
        _ => Vec::new(),
    }
}

fn edit_text(edit: &Option<Gd<LineEdit>>) -> String {
    edit.as_ref()
        .map(|edit| edit.get_text().to_string())
        .unwrap_or_default()
}

fn named_child(body: &Gd<VBoxContainer>, name: &str) -> Option<Gd<Control>> {
    body.get_children().iter_shared().find_map(|child| {
        if child.get_name() == name {
            child.try_cast::<Control>().ok()
        } else {
            None
        }
    })
}

fn flag_set(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| value != "0" && !value.is_empty())
}

fn user_arg(flag: &str) -> bool {
    let args = Os::singleton().get_cmdline_user_args();
    (0..args.len()).any(|index| args.get(index).is_some_and(|value| value == flag))
}
