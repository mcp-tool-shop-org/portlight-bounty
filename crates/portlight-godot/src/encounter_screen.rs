//! Encounter overlay. Text card, the approved sloop plate, and a placeholder
//! where a portrait would be. Character art is held, so that panel is marked
//! [`PORTRAIT_PLACEHOLDER`] in the tree and on screen.

use godot::classes::canvas_item::TextureFilter;
use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::classes::text_server::AutowrapMode;
use godot::classes::{
    Control, HBoxContainer, Label, PanelContainer, ResourceLoader, StyleBoxFlat, Texture2D,
    TextureRect, VBoxContainer,
};
use godot::prelude::*;
use portlight_chart::Asset;

use crate::logic::{
    encounter_plate, ui_plate_panel, PORTRAIT_PLACEHOLDER, UI_PLATE_PAD, UI_PLATE_SCALE,
};

const INK: Color = Color::from_rgb(0.08, 0.11, 0.16);
const CREAM: Color = Color::from_rgb(0.94, 0.91, 0.84);
const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);
const PARCHMENT: Color = Color::from_rgb(0.72, 0.58, 0.36);
const PLACEHOLDER_FILL: Color = Color::from_rgb(0.32, 0.24, 0.18);
/// Untinted. A plate in a panel is not recoloured.
const PLATE_WHITE: Color = Color::from_rgb(1.0, 1.0, 1.0);

pub(crate) struct EncounterNodes {
    pub root: Gd<PanelContainer>,
    pub title: Gd<Label>,
    pub card: Gd<Label>,
    pub log: Gd<Label>,
    pub actions: Gd<VBoxContainer>,
    pub crew: Gd<Label>,
    pub plate: Gd<TextureRect>,
    pub plate_panel: Gd<PanelContainer>,
}

pub(crate) fn build_encounter_screen() -> EncounterNodes {
    let mut root = PanelContainer::new_alloc();
    root.set_name("EncounterScreen");
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

    let (side, plate, plate_panel) = side_column();
    row.add_child(&side);

    let mut column = VBoxContainer::new_alloc();
    column.set_h_size_flags(SizeFlags::EXPAND_FILL);
    column.set_v_size_flags(SizeFlags::EXPAND_FILL);
    column.add_theme_constant_override("separation", 10);
    row.add_child(&column);

    column.add_child(&text_label("Encounter", 14, MUTED));
    let title = text_label("", 28, GOLD);
    column.add_child(&title);
    let mut card = text_label("", 18, CREAM);
    card.set_autowrap_mode(AutowrapMode::WORD_SMART);
    column.add_child(&card);
    let mut log = text_label("", 16, CREAM);
    log.set_autowrap_mode(AutowrapMode::WORD_SMART);
    log.set_v_size_flags(SizeFlags::EXPAND_FILL);
    column.add_child(&log);

    let mut actions = VBoxContainer::new_alloc();
    actions.set_h_size_flags(SizeFlags::EXPAND_FILL);
    actions.add_theme_constant_override("separation", 8);
    column.add_child(&actions);

    let mut crew = text_label("Crew to the prize  0", 14, MUTED);
    crew.set_visible(false);
    column.add_child(&crew);

    EncounterNodes {
        root,
        title,
        card,
        log,
        actions,
        crew,
        plate,
        plate_panel,
    }
}

/// Full-rect overlay. Call after the node has a parent.
pub(crate) fn fill_parent(root: &mut Gd<PanelContainer>) {
    root.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
}

fn side_column() -> (Gd<VBoxContainer>, Gd<TextureRect>, Gd<PanelContainer>) {
    let mut column = VBoxContainer::new_alloc();
    column.set_name("ShipPlateColumn");
    column.set_h_size_flags(SizeFlags::SHRINK_BEGIN);
    column.set_v_size_flags(SizeFlags::SHRINK_BEGIN);
    column.add_theme_constant_override("separation", 8);

    let (mut panel, plate) = plate_panel();
    // Share the column width with the placeholder so the right edges meet.
    // The plate itself stays at exact 2×; only the parchment grows.
    panel.set_h_size_flags(SizeFlags::FILL);
    column.add_child(&panel);
    column.add_child(&text_label("Ship plate", 13, CREAM));
    column.add_child(&placeholder_panel());
    (column, plate, panel)
}

/// Tan panel, 20 px of padding, plate at exact 2× nearest. No tint.
fn plate_panel() -> (Gd<PanelContainer>, Gd<TextureRect>) {
    let mut panel = PanelContainer::new_alloc();
    panel.set_name("ShipPlatePanel");
    panel.set_h_size_flags(SizeFlags::FILL);
    panel.set_v_size_flags(SizeFlags::SHRINK_BEGIN);
    let mut style = StyleBoxFlat::new_gd();
    style.set_bg_color(PARCHMENT);
    style.set_content_margin_all(UI_PLATE_PAD as f32);
    panel.add_theme_stylebox_override("panel", &style);

    // PanelContainer stretches its direct child. This host is not a container,
    // so the plate keeps the exact 2× size when the panel is widened to match
    // the placeholder's right edge.
    let mut host = Control::new_alloc();
    host.set_name("ShipPlateHost");
    host.set_mouse_filter(MouseFilter::IGNORE);
    panel.add_child(&host);

    let mut rect = TextureRect::new_alloc();
    rect.set_name("ShipPlate");
    rect.set_texture_filter(TextureFilter::NEAREST);
    rect.set_modulate(PLATE_WHITE);
    rect.set_expand_mode(godot::classes::texture_rect::ExpandMode::IGNORE_SIZE);
    rect.set_stretch_mode(godot::classes::texture_rect::StretchMode::SCALE);
    rect.set_h_size_flags(SizeFlags::SHRINK_BEGIN);
    rect.set_v_size_flags(SizeFlags::SHRINK_BEGIN);
    host.add_child(&rect);
    set_ship_plate(&mut rect, &mut panel, "");
    (panel, rect)
}

/// Draw `class` at exact [`UI_PLATE_SCALE`], nearest, untinted.
/// The panel is the plate size times that scale, plus [`UI_PLATE_PAD`] each side.
pub(crate) fn set_ship_plate(
    rect: &mut Gd<TextureRect>,
    panel: &mut Gd<PanelContainer>,
    class: &str,
) {
    let plate = encounter_plate(class);
    let (panel_w, panel_h) = ui_plate_panel(plate.canvas_w, plate.canvas_h);
    panel.set_custom_minimum_size(Vector2::new(panel_w as f32, panel_h as f32));
    let draw = Vector2::new(
        (plate.canvas_w * UI_PLATE_SCALE) as f32,
        (plate.canvas_h * UI_PLATE_SCALE) as f32,
    );
    rect.set_custom_minimum_size(draw);
    rect.set_size(draw);
    rect.set_position(Vector2::ZERO);
    rect.set_texture_filter(TextureFilter::NEAREST);
    rect.set_modulate(PLATE_WHITE);
    load_plate(rect, plate);
}

fn load_plate(rect: &mut Gd<TextureRect>, plate: &Asset) {
    let path = plate.res_path();
    if let Some(resource) = ResourceLoader::singleton().load(&GString::from(path.as_str())) {
        if let Ok(texture) = resource.try_cast::<Texture2D>() {
            rect.set_texture(&texture);
        }
    }
}

/// Placeholder panel. Character portraits are held; this is not a portrait.
/// Shrink-wrapped to its two lines. Not a full-width block.
fn placeholder_panel() -> Gd<PanelContainer> {
    let mut panel = PanelContainer::new_alloc();
    panel.set_name(PORTRAIT_PLACEHOLDER);
    panel.set_h_size_flags(SizeFlags::FILL);
    panel.set_v_size_flags(SizeFlags::SHRINK_BEGIN);
    // Width follows the two lines (about 202). The held-portrait slot in the
    // sign-off mock is about 120 tall, so the panel does not collapse to a strip.
    panel.set_custom_minimum_size(Vector2::new(0.0, 120.0));
    let mut style = StyleBoxFlat::new_gd();
    style.set_bg_color(PLACEHOLDER_FILL);
    style.set_content_margin_all(12.0);
    panel.add_theme_stylebox_override("panel", &style);
    let mut column = VBoxContainer::new_alloc();
    column.set_h_size_flags(SizeFlags::SHRINK_BEGIN);
    column.add_child(&text_label(PORTRAIT_PLACEHOLDER, 18, GOLD));
    let mut note = text_label("Character portraits are held.", 13, CREAM);
    note.set_autowrap_mode(AutowrapMode::OFF);
    column.add_child(&note);
    panel.add_child(&column);
    panel
}

fn text_label(text: &str, size: i32, color: Color) -> Gd<Label> {
    let mut label = Label::new_alloc();
    label.set_text(text);
    label.add_theme_font_size_override("font_size", size);
    label.add_theme_color_override("font_color", color);
    label
}
