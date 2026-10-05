//! Shipyard and fleet overlay. The plate column and the button fill are the
//! encounter screen's. The flagship is drawn with
//! [`crate::encounter_screen::set_ship_plate`] at exact 2×.

use godot::classes::control::{MouseFilter, SizeFlags};
use godot::classes::text_server::AutowrapMode;
use godot::classes::{
    Button, HBoxContainer, Label, PanelContainer, ScrollContainer, StyleBoxFlat, VBoxContainer,
};
use godot::prelude::*;

use crate::encounter_screen::{self, SideColumn};

const INK: Color = Color::from_rgb(0.08, 0.11, 0.16);
const CREAM: Color = Color::from_rgb(0.94, 0.91, 0.84);
const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);

#[derive(Clone)]
pub(crate) struct ShipyardNodes {
    pub root: Gd<PanelContainer>,
    pub title: Gd<Label>,
    pub notice: Gd<Label>,
    pub scroll: Gd<ScrollContainer>,
    pub body: Gd<VBoxContainer>,
    pub confirm: Gd<HBoxContainer>,
    pub close: Gd<Button>,
    pub plate: Gd<godot::classes::TextureRect>,
    pub plate_panel: Gd<PanelContainer>,
    pub plate_caption: Gd<Label>,
    pub placeholder: Gd<PanelContainer>,
}

pub(crate) fn build_shipyard_screen() -> ShipyardNodes {
    let mut root = PanelContainer::new_alloc();
    root.set_name("ShipyardScreen");
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

    let side: SideColumn = encounter_screen::side_column();
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

    column.add_child(&text_label("Shipyard", 14, MUTED));
    let title = text_label("", 28, GOLD);
    column.add_child(&title);
    let mut notice = text_label("", 16, CREAM);
    notice.set_autowrap_mode(AutowrapMode::WORD_SMART);
    column.add_child(&notice);

    let mut confirm = HBoxContainer::new_alloc();
    confirm.set_name("ShipyardConfirm");
    confirm.set_visible(false);
    confirm.add_theme_constant_override("separation", 8);
    column.add_child(&confirm);

    let mut scroll = ScrollContainer::new_alloc();
    scroll.set_name("ShipyardScroll");
    scroll.set_h_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_v_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_horizontal_scroll_mode(godot::classes::scroll_container::ScrollMode::DISABLED);
    let mut body = VBoxContainer::new_alloc();
    body.set_name("ShipyardBody");
    body.set_h_size_flags(SizeFlags::EXPAND_FILL);
    body.add_theme_constant_override("separation", 8);
    scroll.add_child(&body);
    column.add_child(&scroll);

    let mut close = Button::new_alloc();
    close.set_name("ShipyardClose");
    close.set_text("Close");
    encounter_screen::style_encounter_button(&mut close);
    column.add_child(&close);

    ShipyardNodes {
        root,
        title,
        notice,
        scroll,
        body,
        confirm,
        close,
        plate,
        plate_panel,
        plate_caption,
        placeholder,
    }
}

fn text_label(text: &str, size: i32, color: Color) -> Gd<Label> {
    let mut label = Label::new_alloc();
    label.set_text(text);
    label.add_theme_font_size_override("font_size", size);
    label.add_theme_color_override("font_color", color);
    label
}
