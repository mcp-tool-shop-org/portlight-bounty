//! Encounter overlay. Text card, the player's class plate, and a placeholder
//! where a portrait would be. Character art is held, so that panel is marked
//! [`PORTRAIT_PLACEHOLDER`] in the tree and on screen.

use godot::classes::canvas_item::TextureFilter;
use godot::classes::control::{LayoutPreset, MouseFilter, SizeFlags};
use godot::classes::text_server::AutowrapMode;
use godot::classes::{
    Button, Control, HBoxContainer, Label, PanelContainer, ResourceLoader, StyleBoxFlat, Texture2D,
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
    pub plate_caption: Gd<Label>,
    pub placeholder: Gd<PanelContainer>,
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

    let side = side_column();
    let plate = side.plate;
    let plate_panel = side.panel;
    let plate_caption = side.caption;
    let placeholder = side.placeholder;
    row.add_child(&side.column);

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
        plate_caption,
        placeholder,
    }
}

/// Full-rect overlay. Call after the node has a parent.
pub(crate) fn fill_parent(root: &mut Gd<PanelContainer>) {
    root.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);
}

struct SideColumn {
    column: Gd<VBoxContainer>,
    plate: Gd<TextureRect>,
    panel: Gd<PanelContainer>,
    caption: Gd<Label>,
    placeholder: Gd<PanelContainer>,
}

fn side_column() -> SideColumn {
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
    let mut caption = text_label("Ship plate", 13, CREAM);
    column.add_child(&caption);
    let mut placeholder = placeholder_panel();
    column.add_child(&placeholder);
    set_ship_plate(
        &mut plate.clone(),
        &mut panel,
        &mut placeholder,
        &mut caption,
        "",
    );
    SideColumn {
        column,
        plate,
        panel,
        caption,
        placeholder,
    }
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
    let host_for_resize = host.clone();
    let rect_for_resize = rect.clone();
    host.signals().resized().connect(move || {
        let mut rect = rect_for_resize.clone();
        center_plate(&host_for_resize, &mut rect);
    });
    (panel, rect)
}

/// Draw `template_id` at exact [`UI_PLATE_SCALE`], nearest, untinted.
/// The panel is the plate size times that scale, plus [`UI_PLATE_PAD`] each side.
pub(crate) fn set_ship_plate(
    rect: &mut Gd<TextureRect>,
    panel: &mut Gd<PanelContainer>,
    placeholder: &mut Gd<PanelContainer>,
    caption: &mut Gd<Label>,
    template_id: &str,
) {
    let drawn = encounter_plate(template_id);
    let plate = drawn.hull;
    let (panel_w, panel_h) = ui_plate_panel(plate.canvas_w, plate.canvas_h);
    panel.set_custom_minimum_size(Vector2::new(panel_w as f32, panel_h as f32));
    // The column width is this plate's panel, not a fixed cutter width. The
    // placeholder's own text can still make a narrower plate's column wider.
    let hold = placeholder.get_custom_minimum_size();
    let (hold_w, hold_h) = placeholder_minimum_size(panel_w, hold.y);
    placeholder.set_custom_minimum_size(Vector2::new(hold_w, hold_h));
    caption.set_text(plate_caption(drawn.class_name));
    let draw = Vector2::new(
        (plate.canvas_w * UI_PLATE_SCALE) as f32,
        (plate.canvas_h * UI_PLATE_SCALE) as f32,
    );
    rect.set_custom_minimum_size(draw);
    rect.set_size(draw);
    rect.set_texture_filter(TextureFilter::NEAREST);
    rect.set_modulate(PLATE_WHITE);
    load_plate(rect, plate);
    if let Some(parent) = rect.get_parent() {
        if let Ok(host) = parent.try_cast::<Control>() {
            center_plate(&host, rect);
        }
    }
}

/// Centre the 2× plate in the panel. The size stays the canvas times
/// [`UI_PLATE_SCALE`]; only the position moves. A 203-wide panel around a
/// 144-wide plate cannot split the spare pixel, so the two sides differ by 1.
fn center_plate(host: &Gd<Control>, rect: &mut Gd<TextureRect>) {
    let host_w = host.get_size().x;
    let host_h = host.get_size().y;
    let draw_w = rect.get_size().x;
    let draw_h = rect.get_size().y;
    if host_w <= 0.0 || host_h <= 0.0 || draw_w <= 0.0 || draw_h <= 0.0 {
        return;
    }
    let x = ((host_w - draw_w) / 2.0).round();
    let y = ((host_h - draw_h) / 2.0).round();
    rect.set_position(Vector2::new(x, y));
}

/// Galleon-class plates name the catalog class. `man_of_war` stays
/// `man_of_war` even though the canvas is the galleon. Other classes keep
/// the signed-off "Ship plate" caption.
fn plate_caption(class_name: &str) -> &str {
    match class_name {
        "man_of_war" | "galleon" => class_name,
        _ => "Ship plate",
    }
}

/// Placeholder minimum size. Width matches the plate panel so the column's
/// right edges meet. Height stays the held-portrait slot already on the node.
pub(crate) fn placeholder_minimum_size(panel_w: i32, held_height: f32) -> (f32, f32) {
    (panel_w as f32, held_height)
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

/// Tan fill, 2 px gold border, 8 px margin. Cream at rest. Dark ink on hover
/// and focus: cream on the hover fill is 2.76:1, and this ink is 5.08:1.
/// The chart panel does not use this style.
pub(crate) fn style_encounter_button(button: &mut Gd<Button>) {
    let normal = encounter_button_fill(Color::from_rgb(0.55, 0.42, 0.24));
    let hover = encounter_button_fill(Color::from_rgb(0.68, 0.52, 0.30));
    let pressed = encounter_button_fill(Color::from_rgb(0.40, 0.30, 0.16));
    button.add_theme_stylebox_override("normal", &normal);
    button.add_theme_stylebox_override("hover", &hover);
    button.add_theme_stylebox_override("pressed", &pressed);
    button.add_theme_stylebox_override("focus", &hover);
    button.add_theme_color_override("font_color", CREAM);
    button.add_theme_color_override("font_hover_color", INK);
    button.add_theme_color_override("font_pressed_color", GOLD);
    button.add_theme_color_override("font_focus_color", INK);
}

fn encounter_button_fill(fill: Color) -> Gd<StyleBoxFlat> {
    let mut style = StyleBoxFlat::new_gd();
    style.set_bg_color(fill);
    style.set_border_color(GOLD);
    style.set_border_width_all(2);
    style.set_content_margin_all(8.0);
    style.set_corner_radius_all(2);
    style
}

fn text_label(text: &str, size: i32, color: Color) -> Gd<Label> {
    let mut label = Label::new_alloc();
    label.set_text(text);
    label.add_theme_font_size_override("font_size", size);
    label.add_theme_color_override("font_color", color);
    label
}

#[cfg(test)]
mod tests {
    use super::{placeholder_minimum_size, plate_caption};
    use crate::logic::{encounter_plate, ui_plate_panel};

    #[test]
    fn plate_caption_names_only_the_galleon_classes() {
        assert_eq!(plate_caption("galleon"), "galleon");
        assert_eq!(plate_caption("man_of_war"), "man_of_war");
        assert_eq!(plate_caption("sloop"), "Ship plate");
        assert_eq!(plate_caption("cutter"), "Ship plate");
        assert_eq!(plate_caption("brigantine"), "Ship plate");
        assert_eq!(plate_caption(""), "Ship plate");
    }

    #[test]
    fn placeholder_width_matches_the_plate_panel() {
        for template in [
            "coastal_sloop",
            "swift_cutter",
            "trade_brigantine",
            "merchant_galleon",
            "royal_man_of_war",
        ] {
            let drawn = encounter_plate(template);
            let (panel_w, _) = ui_plate_panel(drawn.hull.canvas_w, drawn.hull.canvas_h);
            let (width, height) = placeholder_minimum_size(panel_w, 120.0);
            assert_eq!(width, panel_w as f32, "{template}");
            assert_eq!(height, 120.0, "{template}");
        }
    }
}
