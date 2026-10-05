//! Docked crew desk: roster, provisions, training, skills, and companions.
//!
//! The plate column and the button fill are the encounter screen's. Ships
//! are drawn with [`crate::encounter_screen::set_ship_plate`] at exact 2×.
//! There is no character art. `royal_man_of_war` stays the galleon placeholder
//! plate. Companion dismiss has no `Session` method, so this screen has no
//! dismiss control.
//!
//! Follow-on, not this screen: `train` / `recruit` / `skill` on the GameSession
//! runner. The buttons call [`portlight_sim::Session`] directly.
//! `train_crew` passes an empty injury list into `can_learn_style`, so this
//! desk has no injury control. The skill button says Learn and still calls
//! `spend_skill_point`. Smuggler is a companion role, not a hire row.

use godot::classes::control::{MouseFilter, SizeFlags};
use godot::classes::text_server::AutowrapMode;
use godot::classes::{
    Button, HBoxContainer, Label, PanelContainer, ScrollContainer, StyleBoxFlat, TextureRect,
    VBoxContainer,
};
use godot::prelude::*;

use crate::encounter_screen;

const INK: Color = Color::from_rgb(0.08, 0.11, 0.16);
const CREAM: Color = Color::from_rgb(0.94, 0.91, 0.84);
const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);

pub(crate) const SECTION_ROSTER: &str = "RosterSection";
pub(crate) const SECTION_PROVISIONS: &str = "ProvisionsSection";
pub(crate) const SECTION_TRAINING: &str = "TrainingSection";
pub(crate) const SECTION_COMPANIONS: &str = "CompanionsSection";

#[derive(Clone)]
pub(crate) struct CrewNodes {
    pub root: Gd<PanelContainer>,
    pub title: Gd<Label>,
    pub status: Gd<Label>,
    pub notice: Gd<Label>,
    pub scroll: Gd<ScrollContainer>,
    pub body: Gd<VBoxContainer>,
    pub column: Gd<VBoxContainer>,
    pub confirm: Gd<VBoxContainer>,
    pub confirm_label: Gd<Label>,
    pub plate: Gd<TextureRect>,
    pub plate_panel: Gd<PanelContainer>,
    pub plate_caption: Gd<Label>,
    pub placeholder: Gd<PanelContainer>,
}

pub(crate) fn build_crew_screen() -> CrewNodes {
    let mut root = PanelContainer::new_alloc();
    root.set_name("CrewScreen");
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
    column.set_name("CrewColumn");
    column.set_h_size_flags(SizeFlags::EXPAND_FILL);
    column.set_v_size_flags(SizeFlags::EXPAND_FILL);
    column.add_theme_constant_override("separation", 10);
    row.add_child(&column);
    let column_handle = column.clone();

    column.add_child(&text_label("Crew", 14, MUTED));
    let title = text_label("", 28, GOLD);
    column.add_child(&title);
    let mut status = text_label("", 16, CREAM);
    status.set_autowrap_mode(AutowrapMode::WORD_SMART);
    column.add_child(&status);
    let mut notice = text_label("", 15, CREAM);
    notice.set_autowrap_mode(AutowrapMode::WORD_SMART);
    column.add_child(&notice);

    let mut scroll = ScrollContainer::new_alloc();
    scroll.set_name("CrewScroll");
    scroll.set_h_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_v_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_horizontal_scroll_mode(godot::classes::scroll_container::ScrollMode::DISABLED);
    let mut body = VBoxContainer::new_alloc();
    body.set_name("CrewBody");
    body.set_h_size_flags(SizeFlags::EXPAND_FILL);
    body.add_theme_constant_override("separation", 14);
    scroll.add_child(&body);
    column.add_child(&scroll);

    let mut confirm = VBoxContainer::new_alloc();
    confirm.set_name("CrewConfirm");
    confirm.set_visible(false);
    confirm.add_theme_constant_override("separation", 8);
    let mut confirm_label = text_label("", 16, GOLD);
    confirm_label.set_autowrap_mode(AutowrapMode::WORD_SMART);
    confirm.add_child(&confirm_label);
    column.add_child(&confirm);

    CrewNodes {
        root,
        title,
        status,
        notice,
        scroll,
        body,
        column: column_handle,
        confirm,
        confirm_label,
        plate,
        plate_panel,
        plate_caption,
        placeholder,
    }
}

/// Full-rect overlay. Call after the node has a parent.
pub(crate) fn fill_parent(root: &mut Gd<PanelContainer>) {
    root.set_anchors_and_offsets_preset(godot::classes::control::LayoutPreset::FULL_RECT);
}

/// Encounter fill plus a disabled state that keeps the gold border.
pub(crate) fn style_crew_button(button: &mut Gd<Button>) {
    encounter_screen::style_encounter_button(button);
    let mut disabled = StyleBoxFlat::new_gd();
    disabled.set_bg_color(Color::from_rgb(0.28, 0.24, 0.18));
    disabled.set_border_color(GOLD);
    disabled.set_border_width_all(2);
    disabled.set_content_margin_all(8.0);
    disabled.set_corner_radius_all(2);
    button.add_theme_stylebox_override("disabled", &disabled);
    button.add_theme_color_override("font_disabled_color", MUTED);
}

fn text_label(text: &str, size: i32, color: Color) -> Gd<Label> {
    let mut label = Label::new_alloc();
    label.set_text(text);
    label.add_theme_font_size_override("font_size", size);
    label.add_theme_color_override("font_color", color);
    label
}
