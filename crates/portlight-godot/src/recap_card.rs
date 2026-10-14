//! The contract recap card: one card per settling sale (paid) or confirmed
//! abandon, drawn in the Day's report slot. The card is modal: a layer under
//! it stops every click until Close (or Escape) takes it down. Only the
//! Day's report ranks above it.

use godot::builtin::Side;
use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::classes::{
    Button, ColorRect, Control, Label, PanelContainer, StyleBoxFlat, VBoxContainer,
};
use godot::prelude::*;

use crate::encounter_screen::{set_recap_block, style_encounter_button};
use crate::recap::Recap;

pub(crate) const INK: Color = Color::from_rgb(0.08, 0.11, 0.16);
pub(crate) const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
pub(crate) const DUE: Color = Color::from_rgb(0.93, 0.55, 0.42);
pub(crate) const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);
const CREAM: Color = Color::from_rgb(0.94, 0.91, 0.84);

/// The Day's report slot (day_report.rs CARD_X/Y/W/H).
pub(crate) const CARD_X: f32 = 170.0;
pub(crate) const CARD_Y: f32 = 40.0;
pub(crate) const CARD_W: f32 = 520.0;
pub(crate) const CARD_H: f32 = 380.0;
/// The report's content inset.
const CARD_PAD: f32 = 18.0;
/// The card's content (headline and rows) must stay within this height.
pub(crate) const CONTENT_MAX_H: f32 = 320.0;
pub(crate) const HEADLINE_SIZE: i32 = 24;
/// The early-bonus row under its Silver row.
pub(crate) const BONUS_SIZE: i32 = 14;
pub(crate) const BONUS_PREFIX: &str = "Includes early bonus";
/// Alpha of the dim laid over the Contracts desk behind an abandon card.
pub(crate) const SCRIM_ALPHA: f32 = 0.55;

#[derive(Clone)]
pub(crate) struct RecapCardNodes {
    pub root: Gd<PanelContainer>,
    pub content: Gd<VBoxContainer>,
    pub close: Gd<Button>,
    pub scrim: Gd<ColorRect>,
}

pub(crate) fn build_recap_card() -> RecapCardNodes {
    let mut root = PanelContainer::new_alloc();
    root.set_name("RecapCard");
    root.set_mouse_filter(MouseFilter::STOP);
    root.set_visible(false);
    let mut style = StyleBoxFlat::new_gd();
    style.set_bg_color(INK);
    style.set_border_color(GOLD);
    style.set_border_width_all(2);
    style.set_content_margin_all(CARD_PAD);
    root.add_theme_stylebox_override("panel", &style);

    let mut column = VBoxContainer::new_alloc();
    column.set_name("RecapCardColumn");
    column.set_h_size_flags(SizeFlags::EXPAND_FILL);
    column.set_v_size_flags(SizeFlags::EXPAND_FILL);
    root.add_child(&column);

    let mut content = VBoxContainer::new_alloc();
    content.set_name("RecapCardContent");
    content.set_h_size_flags(SizeFlags::EXPAND_FILL);
    content.add_theme_constant_override("separation", 6);
    column.add_child(&content);

    // The spacer pushes Close to the card's foot, so its rect is the same
    // for every card.
    let mut spacer = Control::new_alloc();
    spacer.set_name("RecapCardSpacer");
    spacer.set_v_size_flags(SizeFlags::EXPAND_FILL);
    spacer.set_mouse_filter(MouseFilter::IGNORE);
    column.add_child(&spacer);

    let mut close = Button::new_alloc();
    close.set_name("RecapClose");
    close.set_text("Close");
    style_encounter_button(&mut close);
    // Focus stays trapped on Close; it keeps the plain fill (no hover look).
    close.add_theme_stylebox_override("focus", &godot::classes::StyleBoxEmpty::new_gd());
    close.add_theme_color_override("font_focus_color", CREAM);
    close.set_h_size_flags(SizeFlags::SHRINK_BEGIN);
    column.add_child(&close);

    let mut scrim = ColorRect::new_alloc();
    scrim.set_name("RecapCardScrim");
    scrim.set_color(Color {
        a: SCRIM_ALPHA,
        ..INK
    });
    scrim.set_visible(false);
    scrim.set_mouse_filter(MouseFilter::IGNORE);

    RecapCardNodes {
        root,
        content,
        close,
        scrim,
    }
}

pub(crate) fn place_card(root: &mut Gd<PanelContainer>) {
    root.set_anchors_and_offsets_preset(LayoutPreset::TOP_LEFT);
    root.set_position(Vector2::new(CARD_X, CARD_Y));
    root.set_size(Vector2::new(CARD_W, CARD_H));
}

pub(crate) fn set_open(nodes: &mut RecapCardNodes, open: bool) {
    nodes.root.set_visible(open);
    nodes.root.set_mouse_filter(if open {
        MouseFilter::STOP
    } else {
        MouseFilter::IGNORE
    });
}

const KEY_SIDES: [Side; 4] = [Side::LEFT, Side::TOP, Side::RIGHT, Side::BOTTOM];

/// The modal layer behind an open card: it stops every click that misses the
/// card and keeps Tab, Shift+Tab and the arrows on Close. `dim` (the abandon
/// card over the Contracts desk) draws INK at [`SCRIM_ALPHA`]; a paid card's
/// layer draws nothing.
pub(crate) fn set_scrim(nodes: &mut RecapCardNodes, on: bool, dim: bool) {
    nodes.scrim.set_visible(on);
    nodes.scrim.set_color(Color {
        a: if dim { SCRIM_ALPHA } else { 0.0 },
        ..INK
    });
    nodes.scrim.set_mouse_filter(if on {
        MouseFilter::STOP
    } else {
        MouseFilter::IGNORE
    });
    let target = if on {
        NodePath::from(".")
    } else {
        NodePath::default()
    };
    nodes.close.set_focus_next(&target);
    nodes.close.set_focus_previous(&target);
    for side in KEY_SIDES {
        nodes.close.set_focus_neighbor(side, &target);
    }
}

pub(crate) fn scrim_shown(nodes: &RecapCardNodes) -> bool {
    nodes.scrim.is_visible()
}

/// True when the layer behind the card draws the dim.
pub(crate) fn scrim_dimmed(nodes: &RecapCardNodes) -> bool {
    nodes.scrim.is_visible() && nodes.scrim.get_color().a > 0.0
}

/// True when every focus move from Close comes straight back to Close.
pub(crate) fn keyboard_trapped(nodes: &RecapCardNodes) -> bool {
    let close = nodes.close.clone().upcast::<Control>();
    let same = |found: Option<Gd<Control>>| found.is_some_and(|node| node == close);
    same(nodes.close.find_next_valid_focus())
        && same(nodes.close.find_prev_valid_focus())
        && KEY_SIDES
            .into_iter()
            .all(|side| same(nodes.close.find_valid_focus_neighbor(side)))
}

/// Draw the recap: the shared recap block (headline, then a label per row)
/// with the card's 22 px headline.
pub(crate) fn apply(nodes: &mut RecapCardNodes, recap: &Recap) {
    set_recap_block(&mut nodes.content, Some(recap));
    if let Some(mut headline) = nodes.content.try_get_node_as::<Label>("RecapHeadline") {
        headline.add_theme_font_size_override("font_size", HEADLINE_SIZE);
    }
    for child in nodes.content.get_children().iter_shared() {
        let Ok(mut label) = child.try_cast::<Label>() else {
            continue;
        };
        // Card copy is one line per row (ROW_CHARS); no wrap, so the card
        // never grows from a narrow first layout.
        label.set_autowrap_mode(godot::classes::text_server::AutowrapMode::OFF);
        if label.get_text().to_string().starts_with(BONUS_PREFIX) {
            label.add_theme_font_size_override("font_size", BONUS_SIZE);
            label.add_theme_color_override("font_color", MUTED);
        }
    }
}

/// Drawn texts: the headline, then every row label in order.
pub(crate) fn drawn_texts(nodes: &RecapCardNodes) -> (String, Vec<String>) {
    let mut headline = String::new();
    let mut rows = Vec::new();
    for child in nodes.content.get_children().iter_shared() {
        let Ok(label) = child.try_cast::<Label>() else {
            continue;
        };
        let text = label.get_text().to_string();
        if label.get_name() == "RecapHeadline" {
            headline = text;
        } else {
            rows.push(text);
        }
    }
    (headline, rows)
}

/// The one card up (or waiting under the Day's report). The card is modal,
/// so no sale or abandon can raise a second one while it is up. Godot memory
/// only: a new game or a load clears it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RecapQueue {
    card: Option<Recap>,
}

impl RecapQueue {
    /// Raise a card; `None` (an empty outcome list) raises nothing. A card
    /// already up stays: nothing can stack on it.
    pub(crate) fn push(&mut self, recap: Option<Recap>) {
        if self.card.is_none() {
            self.card = recap;
        }
    }

    pub(crate) fn front(&self) -> Option<&Recap> {
        self.card.as_ref()
    }

    /// Close the card.
    pub(crate) fn pop(&mut self) -> Option<Recap> {
        self.card.take()
    }

    pub(crate) fn reset(&mut self) {
        self.card = None;
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        usize::from(self.card.is_some())
    }
}

/// The card a sale opens: one per sale that settles contracts, none for a
/// partial sale.
pub(crate) fn sale_card(sale: &portlight_sim::session::Sale) -> Option<Recap> {
    crate::recap::contract_recap(&sale.contracts, &crate::recap::ContractExtras::default())
}

/// The card a confirmed abandon opens (`confirm_abandon`'s `Ok` arm), with
/// the units still held.
pub(crate) fn abandon_card(
    outcome: &portlight_sim::model::ContractOutcome,
    held: i64,
) -> Option<Recap> {
    crate::recap::contract_recap(
        std::slice::from_ref(outcome),
        &crate::recap::ContractExtras { held: vec![held] },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recap::RecapResult;
    use portlight_sim::model::VoyageStatus;
    use portlight_sim::Session;

    fn held(session: &Session, good: &str) -> i64 {
        crate::game::cargo_held(&session.world().captain.cargo, good)
    }

    /// Seed 1: accept the Grain run to Corsair's Rest, buy a margin, sail.
    fn grain_at_corsairs_rest() -> (Session, String) {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        let offer = session
            .available_contracts()
            .into_iter()
            .find(|o| o.good_id == "grain" && o.destination_port_id == "corsairs_rest")
            .unwrap();
        session.accept_contract(&offer.id).unwrap();
        while held(&session, "grain") < offer.quantity + 5 {
            session.buy("grain", 1).unwrap();
        }
        session.depart("corsairs_rest").unwrap();
        for _ in 0..60 {
            if session.world().voyage.status == VoyageStatus::InPort {
                break;
            }
            let _ = session.advance();
        }
        assert_eq!(session.world().voyage.destination_id, "corsairs_rest");
        (session, offer.id)
    }

    #[test]
    fn partial_sale_opens_no_card() {
        let (mut session, _) = grain_at_corsairs_rest();
        let sale = session.sell("grain", 10).unwrap();
        assert!(sale.contracts.is_empty());
        assert_eq!(sale_card(&sale), None);
    }

    #[test]
    fn one_card_per_settling_sale() {
        let (mut session, _) = grain_at_corsairs_rest();
        let mut queue = RecapQueue::default();
        for _ in 0..4 {
            let qty = held(&session, "grain").min(10);
            if qty == 0 {
                break;
            }
            let sale = session.sell("grain", qty).unwrap();
            queue.push(sale_card(&sale));
        }
        assert_eq!(queue.len(), 1);
        let card = queue.front().unwrap();
        assert_eq!(card.result, RecapResult::Paid);
        assert_eq!(card.headline, "Contract paid - settled by your sale");
        // The next sale settles nothing and opens nothing.
        if held(&session, "grain") > 0 {
            let sale = session.sell("grain", 1).unwrap();
            queue.push(sale_card(&sale));
        }
        assert_eq!(queue.len(), 1);
    }

    /// An expiry is a Day's report line: a day tick never reaches the queue,
    /// which only sales and abandons feed.
    #[test]
    fn day_tick_opens_no_card() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        let offer = session.available_contracts().remove(0);
        session.accept_contract(&offer.id).unwrap();
        let queue = RecapQueue::default();
        for _ in 0..80 {
            if session.board().active.is_empty() {
                break;
            }
            let _ = session.advance();
        }
        assert!(
            session.board().active.is_empty(),
            "the contract never expired"
        );
        assert!(session
            .board()
            .completed
            .iter()
            .any(|c| c.outcome_type.starts_with("expired")));
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn abandon_card_opens_only_on_ok() {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        let offer = session.available_contracts().remove(0);
        // An Err (and a cancel, which never calls the sim) has no outcome to
        // draw; the card is raised only inside confirm_abandon's Ok arm.
        assert!(session.abandon_contract("no-such-offer").is_err());
        let game = include_str!("game.rs");
        assert_eq!(game.matches("crate::recap_card::abandon_card(").count(), 1);
        let ok_arm = game
            .find("            Ok(outcome) => {\n                // Contract fail:")
            .unwrap();
        let call = game.find("crate::recap_card::abandon_card(").unwrap();
        let err_arm = game[ok_arm..].find("            Err(err) =>").unwrap() + ok_arm;
        assert!(ok_arm < call && call < err_arm);
        session.accept_contract(&offer.id).unwrap();
        let outcome = session.abandon_contract(&offer.id).unwrap();
        let card = abandon_card(&outcome, 3).unwrap();
        assert_eq!(card.result, RecapResult::Abandoned);
        assert_eq!(card.headline, "Contract abandoned");
    }

    #[test]
    fn abandon_card_has_no_silver_row() {
        let (mut session, offer_id) = grain_at_corsairs_rest();
        session.sell("grain", 10).unwrap();
        let held_now = held(&session, "grain");
        let outcome = session.abandon_contract(&offer_id).unwrap();
        let card = abandon_card(&outcome, held_now).unwrap();
        let texts: Vec<&str> = card.rows.iter().map(|r| r.text.as_str()).collect();
        assert!(texts.iter().all(|t| !t.starts_with("Silver")), "{texts:?}");
        assert!(
            texts.iter().any(|t| t.starts_with("Progress lost - 10/")),
            "{texts:?}"
        );
        assert!(
            texts.contains(&format!("{held_now} Grain still aboard").as_str()),
            "{texts:?}"
        );
    }

    fn card(headline: &str) -> Recap {
        let mut grain =
            crate::recap::contract_recap(&[outcome()], &crate::recap::ContractExtras::default())
                .unwrap();
        grain.headline = headline.into();
        grain
    }

    fn outcome() -> portlight_sim::model::ContractOutcome {
        let mut session = Session::new("Ada", "merchant", 1, None).unwrap();
        let offer = session.available_contracts().remove(0);
        session.accept_contract(&offer.id).unwrap();
        session.abandon_contract(&offer.id).unwrap()
    }

    /// One card per sale, no stacking: the card up stays until Close, and a
    /// sale that settles nothing raises nothing.
    #[test]
    fn recap_queue_keeps_sale_order() {
        let mut queue = RecapQueue::default();
        queue.push(Some(card("first")));
        queue.push(None);
        queue.push(Some(card("second")));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop().unwrap().headline, "first");
        assert_eq!(queue.front(), None);
        queue.push(Some(card("second")));
        assert_eq!(queue.front().unwrap().headline, "second");
    }

    /// A new game and a load run `reset` beside the Day's report reset.
    #[test]
    fn recap_queue_resets_on_new_game_and_load() {
        let mut queue = RecapQueue::default();
        queue.push(Some(card("first")));
        queue.reset();
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.front(), None);
        let game = include_str!("game.rs");
        let report_resets = game.matches("self.reset_day_report();").count();
        // Every Day's report reset clears the card on the next line.
        let paired = game
            .match_indices("self.reset_day_report();")
            .filter(|(at, call)| {
                game[at + call.len()..]
                    .trim_start()
                    .starts_with("self.recap_queue.reset();")
            })
            .count();
        assert!(report_resets >= 3);
        assert_eq!(paired, report_resets);
    }
}
