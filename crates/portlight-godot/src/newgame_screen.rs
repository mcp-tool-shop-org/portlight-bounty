//! Title, captain roster, custom captain, load list, and the in-game save note.
//!
//! The plate column and the button fill are the encounter screen's. Ships
//! are drawn later with [`crate::encounter_screen::set_ship_plate`] at exact 2×.

use godot::classes::control::{MouseFilter, SizeFlags};
use godot::classes::text_server::AutowrapMode;
use godot::classes::{Control, HBoxContainer, Label, PanelContainer, StyleBoxFlat, VBoxContainer};
use godot::prelude::*;

use crate::encounter_screen;
use crate::logic::NewgamePage;

const INK: Color = Color::from_rgb(0.08, 0.11, 0.16);
const CREAM: Color = Color::from_rgb(0.94, 0.91, 0.84);
const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);

#[derive(Clone)]
pub(crate) struct NewgameNodes {
    pub root: Gd<Control>,
    pub menu: Gd<PanelContainer>,
    pub title: Gd<Label>,
    pub card: Gd<Label>,
    pub detail: Gd<Label>,
    pub actions: Gd<VBoxContainer>,
    pub plate: Gd<godot::classes::TextureRect>,
    pub plate_panel: Gd<PanelContainer>,
    pub plate_caption: Gd<Label>,
    pub placeholder: Gd<PanelContainer>,
    pub confirm: Gd<VBoxContainer>,
    pub confirm_title: Gd<Label>,
    pub confirm_detail: Gd<Label>,
    pub confirm_actions: Gd<VBoxContainer>,
    pub confirm_plate: Gd<godot::classes::TextureRect>,
    pub confirm_plate_panel: Gd<PanelContainer>,
    pub confirm_caption: Gd<Label>,
    pub confirm_placeholder: Gd<PanelContainer>,
}

pub(crate) fn build_newgame_screen() -> NewgameNodes {
    let mut root = Control::new_alloc();
    root.set_name("NewgameScreen");
    root.set_mouse_filter(MouseFilter::STOP);
    root.set_visible(false);

    let mut menu = ink_panel("NewgameMenu");
    root.add_child(&menu);
    let mut row = HBoxContainer::new_alloc();
    row.set_h_size_flags(SizeFlags::EXPAND_FILL);
    row.set_v_size_flags(SizeFlags::EXPAND_FILL);
    row.add_theme_constant_override("separation", 20);
    menu.add_child(&row);

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

    column.add_child(&text_label("Voyage", 14, MUTED));
    let title = text_label("Portlight", 28, GOLD);
    column.add_child(&title);
    let mut card = text_label("", 18, CREAM);
    card.set_autowrap_mode(AutowrapMode::WORD_SMART);
    column.add_child(&card);
    let mut detail = text_label("", 16, CREAM);
    detail.set_autowrap_mode(AutowrapMode::WORD_SMART);
    column.add_child(&detail);

    let mut scroll = godot::classes::ScrollContainer::new_alloc();
    scroll.set_h_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_v_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_horizontal_scroll_mode(godot::classes::scroll_container::ScrollMode::DISABLED);
    let mut actions = VBoxContainer::new_alloc();
    actions.set_h_size_flags(SizeFlags::EXPAND_FILL);
    actions.add_theme_constant_override("separation", 8);
    scroll.add_child(&actions);
    column.add_child(&scroll);

    let mut confirm = VBoxContainer::new_alloc();
    confirm.set_name("SaveConfirm");
    confirm.set_mouse_filter(MouseFilter::IGNORE);
    confirm.set_visible(false);
    root.add_child(&confirm);
    let mut spacer = Control::new_alloc();
    spacer.set_mouse_filter(MouseFilter::IGNORE);
    spacer.set_v_size_flags(SizeFlags::EXPAND_FILL);
    confirm.add_child(&spacer);

    let mut bar = ink_panel("SaveConfirmBar");
    bar.set_v_size_flags(SizeFlags::SHRINK_END);
    bar.set_custom_minimum_size(Vector2::new(0.0, 420.0));
    confirm.add_child(&bar);
    let mut bar_row = HBoxContainer::new_alloc();
    bar_row.set_h_size_flags(SizeFlags::EXPAND_FILL);
    bar_row.add_theme_constant_override("separation", 20);
    bar.add_child(&bar_row);
    let confirm_side = encounter_screen::side_column();
    let confirm_plate = confirm_side.plate.clone();
    let confirm_plate_panel = confirm_side.panel.clone();
    let confirm_caption = confirm_side.caption.clone();
    let confirm_placeholder = confirm_side.placeholder.clone();
    bar_row.add_child(&confirm_side.column);
    let mut confirm_column = VBoxContainer::new_alloc();
    confirm_column.set_h_size_flags(SizeFlags::EXPAND_FILL);
    confirm_column.add_theme_constant_override("separation", 10);
    bar_row.add_child(&confirm_column);
    let confirm_title = text_label("Game saved.", 28, GOLD);
    confirm_column.add_child(&confirm_title);
    let mut confirm_detail = text_label("", 18, CREAM);
    confirm_detail.set_autowrap_mode(AutowrapMode::WORD_SMART);
    confirm_column.add_child(&confirm_detail);
    let mut confirm_actions = VBoxContainer::new_alloc();
    confirm_actions.add_theme_constant_override("separation", 8);
    confirm_column.add_child(&confirm_actions);

    NewgameNodes {
        root,
        menu,
        title,
        card,
        detail,
        actions,
        plate,
        plate_panel,
        plate_caption,
        placeholder,
        confirm,
        confirm_title,
        confirm_detail,
        confirm_actions,
        confirm_plate,
        confirm_plate_panel,
        confirm_caption,
        confirm_placeholder,
    }
}

/// Full-rect overlay. Call after the node has a parent.
pub(crate) fn fill_parent(root: &mut Gd<Control>) {
    root.set_anchors_and_offsets_preset(godot::classes::control::LayoutPreset::FULL_RECT);
    for child in root.get_children().iter_shared() {
        if let Ok(mut control) = child.try_cast::<Control>() {
            control
                .set_anchors_and_offsets_preset(godot::classes::control::LayoutPreset::FULL_RECT);
        }
    }
}

pub(crate) fn show_page(nodes: &mut NewgameNodes, page: NewgamePage) {
    match page {
        NewgamePage::Hidden => {
            nodes.root.set_visible(false);
        }
        NewgamePage::Saved => {
            nodes.root.set_visible(true);
            nodes.root.set_mouse_filter(MouseFilter::IGNORE);
            nodes.menu.set_visible(false);
            nodes.confirm.set_visible(true);
        }
        _ => {
            nodes.root.set_visible(true);
            nodes.root.set_mouse_filter(MouseFilter::STOP);
            nodes.menu.set_visible(true);
            nodes.confirm.set_visible(false);
        }
    }
}

fn ink_panel(name: &str) -> Gd<PanelContainer> {
    let mut panel = PanelContainer::new_alloc();
    panel.set_name(name);
    panel.set_mouse_filter(MouseFilter::STOP);
    panel.set_h_size_flags(SizeFlags::EXPAND_FILL);
    let mut style = StyleBoxFlat::new_gd();
    style.set_bg_color(INK);
    style.set_content_margin_all(28.0);
    panel.add_theme_stylebox_override("panel", &style);
    panel
}

fn text_label(text: &str, size: i32, color: Color) -> Gd<Label> {
    let mut label = Label::new_alloc();
    label.set_text(text);
    label.add_theme_font_size_override("font_size", size);
    label.add_theme_color_override("font_color", color);
    label
}

/// Cream field, dark ink. The chart panel does not use this.
pub(crate) fn style_field(field: &mut godot::classes::LineEdit) {
    let mut style = StyleBoxFlat::new_gd();
    style.set_bg_color(CREAM);
    style.set_border_color(GOLD);
    style.set_border_width_all(2);
    style.set_content_margin_all(6.0);
    field.add_theme_stylebox_override("normal", &style);
    field.add_theme_stylebox_override("focus", &style);
    field.add_theme_color_override("font_color", INK);
    field.add_theme_color_override("font_placeholder_color", MUTED);
    field.set_custom_minimum_size(Vector2::new(320.0, 36.0));
}
