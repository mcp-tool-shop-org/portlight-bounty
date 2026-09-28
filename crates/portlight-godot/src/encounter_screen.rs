//! Encounter overlay. Text card, the approved sloop plate, and a placeholder
//! where a portrait would be. Character art is held, so that panel is marked
//! [`PORTRAIT_PLACEHOLDER`] in the tree and on screen.

use godot::classes::canvas_item::TextureFilter;
use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::classes::text_server::AutowrapMode;
use godot::classes::{
    HBoxContainer, Label, PanelContainer, ResourceLoader, StyleBoxFlat, Texture2D, TextureRect,
    VBoxContainer,
};
use godot::prelude::*;
use portlight_chart::{ship_asset, Facing};

use crate::logic::PORTRAIT_PLACEHOLDER;

const INK: Color = Color::from_rgb(0.08, 0.11, 0.16);
const CREAM: Color = Color::from_rgb(0.94, 0.91, 0.84);
const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);
const PARCHMENT: Color = Color::from_rgb(0.72, 0.58, 0.36);
const PLACEHOLDER_FILL: Color = Color::from_rgb(0.32, 0.24, 0.18);
/// Half the 1280 window. A full-ink overlay is one colour and the capture
/// rejects it. This plate is a second colour so each phase frame stays mixed.
const PLATE_MIN_W: f32 = 560.0;

pub(crate) struct EncounterNodes {
    pub root: Gd<PanelContainer>,
    pub title: Gd<Label>,
    pub card: Gd<Label>,
    pub log: Gd<Label>,
    pub actions: Gd<VBoxContainer>,
    pub crew: Gd<Label>,
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
    row.add_theme_constant_override("separation", 28);
    root.add_child(&row);

    row.add_child(&side_column());

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
    }
}

/// Full-rect overlay. Call after the node has a parent.
pub(crate) fn fill_parent(root: &mut Gd<PanelContainer>) {
    root.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
}

fn side_column() -> Gd<PanelContainer> {
    let mut panel = PanelContainer::new_alloc();
    panel.set_name("ShipPlateColumn");
    panel.set_custom_minimum_size(Vector2::new(PLATE_MIN_W, 0.0));
    panel.set_h_size_flags(SizeFlags::EXPAND_FILL);
    panel.set_v_size_flags(SizeFlags::EXPAND_FILL);
    let mut style = StyleBoxFlat::new_gd();
    style.set_bg_color(PARCHMENT);
    style.set_content_margin_all(20.0);
    panel.add_theme_stylebox_override("panel", &style);

    let mut column = VBoxContainer::new_alloc();
    column.add_theme_constant_override("separation", 12);
    column.add_child(&ship_plate());
    column.add_child(&text_label("Ship plate", 13, INK));
    column.add_child(&placeholder_panel());
    panel.add_child(&column);
    panel
}

fn ship_plate() -> Gd<TextureRect> {
    let mut rect = TextureRect::new_alloc();
    rect.set_name("ShipPlate");
    // Native pixels. A fitted rect scales the plate even when the filter is nearest.
    rect.set_texture_filter(TextureFilter::NEAREST);
    rect.set_expand_mode(godot::classes::texture_rect::ExpandMode::KEEP_SIZE);
    rect.set_stretch_mode(godot::classes::texture_rect::StretchMode::KEEP);
    rect.set_h_size_flags(SizeFlags::SHRINK_BEGIN);
    rect.set_v_size_flags(SizeFlags::SHRINK_BEGIN);
    let plate = ship_asset(Facing::F1);
    let mut native = Vector2::new(plate.canvas_w as f32, plate.canvas_h as f32);
    let path = plate.res_path();
    if let Some(resource) = ResourceLoader::singleton().load(&GString::from(path.as_str())) {
        if let Ok(texture) = resource.try_cast::<Texture2D>() {
            let size = texture.get_size();
            if size.x > 0.0 && size.y > 0.0 {
                native = size;
            }
            rect.set_texture(&texture);
        }
    }
    rect.set_custom_minimum_size(native);
    rect
}

/// Placeholder panel. Character portraits are held; this is not a portrait.
fn placeholder_panel() -> Gd<PanelContainer> {
    let mut panel = PanelContainer::new_alloc();
    panel.set_name(PORTRAIT_PLACEHOLDER);
    panel.set_custom_minimum_size(Vector2::new(180.0, 120.0));
    let mut style = StyleBoxFlat::new_gd();
    style.set_bg_color(PLACEHOLDER_FILL);
    style.set_content_margin_all(12.0);
    panel.add_theme_stylebox_override("panel", &style);
    let mut column = VBoxContainer::new_alloc();
    column.add_child(&text_label(PORTRAIT_PLACEHOLDER, 18, GOLD));
    let mut note = text_label("Character portraits are held.", 13, CREAM);
    note.set_autowrap_mode(AutowrapMode::WORD_SMART);
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
