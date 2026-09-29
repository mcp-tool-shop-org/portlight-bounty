//! Draws one [`portlight_chart::ChartModel`]. No rules live here.

use std::collections::HashMap;

use godot::classes::canvas_item::TextureFilter;
use godot::classes::{
    Camera2D, Font, INode2D, InputEvent, InputEventMouseButton, InputEventMouseMotion, Node2D,
    ResourceLoader, Texture2D, ThemeDb,
};
use godot::global::MouseButton;
use godot::prelude::*;
use portlight_chart::{
    asset, hover_at, ChartLane, ChartModel, ChartPort, Rgba, ShipMarker, SHIP_FOOTPRINT_CELLS,
};
use portlight_sim::LaneSuitability;

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct ChartCanvas {
    base: Base<Node2D>,
    model: Option<ChartModel>,
    textures: HashMap<String, Gd<Texture2D>>,
    /// In-chart names, badges, and lane captions. Separate so their filter
    /// can be linear while plates stay nearest.
    labels: Option<Gd<ChartLabels>>,
    camera: Option<Gd<Camera2D>>,
    hover: String,
    /// Presentation point while a day tween is in flight.
    shown_ship: Option<Vector2>,
    tween_from: Vector2,
    tween_to: Vector2,
    tween_left: f32,
}

#[godot_api]
impl INode2D for ChartCanvas {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            model: None,
            textures: HashMap::new(),
            labels: None,
            camera: None,
            hover: String::new(),
            shown_ship: None,
            tween_from: Vector2::ZERO,
            tween_to: Vector2::ZERO,
            tween_left: 0.0,
        }
    }

    fn ready(&mut self) {
        // Plates, water, and strokes. The viewport default is nearest too.
        self.base_mut().set_texture_filter(TextureFilter::NEAREST);
        let mut camera = Camera2D::new_alloc();
        camera.set_enabled(true);
        self.base_mut().add_child(&camera);
        camera.make_current();
        self.camera = Some(camera);
        let mut labels = ChartLabels::new_alloc();
        labels.set_name("ChartLabels");
        labels.set_texture_filter(TextureFilter::LINEAR);
        self.base_mut().add_child(&labels);
        self.labels = Some(labels);
    }

    fn process(&mut self, delta: f64) {
        if self.tween_left <= 0.0 {
            return;
        }
        self.tween_left = (self.tween_left - delta as f32).max(0.0);
        let duration = portlight_chart::DAY_TWEEN_SECS;
        let t = 1.0 - (self.tween_left / duration).clamp(0.0, 1.0);
        self.shown_ship = Some(self.tween_from.lerp(self.tween_to, t));
        self.base_mut().queue_redraw();
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if event.clone().try_cast::<InputEventMouseMotion>().is_ok() {
            self.update_hover();
            return;
        }
        let Ok(button) = event.try_cast::<InputEventMouseButton>() else {
            return;
        };
        if button.is_pressed() && button.get_button_index() == MouseButton::LEFT {
            self.click_port();
        }
    }

    fn draw(&mut self) {
        let Some(model) = self.model.clone() else {
            return;
        };
        self.ensure_textures(&model);
        let textures = self.textures.clone();
        let hover = self.hover.clone();
        let ship_at = self
            .shown_ship
            .unwrap_or(Vector2::new(model.ship.at.0, model.ship.at.1));
        let mut canvas = self.base_mut();
        let sea = Color::from_rgb(0.04, 0.07, 0.12);
        canvas.draw_rect(
            Rect2::new(
                Vector2::new(
                    model.focus.min_x as f32 - 4000.0,
                    model.focus.min_y as f32 - 4000.0,
                ),
                Vector2::new(8000.0, 8000.0),
            ),
            sea,
        );
        for tile in &model.tiles {
            draw_sprite(
                &mut canvas,
                &textures,
                tile.asset_id,
                Vector2::new(tile.origin.0, tile.origin.1),
                false,
            );
        }
        if let Some(leg) = &model.leg {
            draw_segment(&mut canvas, leg.from, leg.to, &leg.color, 2.0);
        }
        for lane in &model.lanes {
            let width = match lane.suitability {
                LaneSuitability::Blocked => 4.5,
                LaneSuitability::Warning => 3.0,
                LaneSuitability::Ok => 2.5,
            };
            draw_segment(&mut canvas, lane.from, lane.to, &lane.color, width);
        }
        let mut drawables: Vec<Sortable> = model
            .ports
            .iter()
            .map(|port| Sortable::Port(port.clone()))
            .collect();
        drawables.push(Sortable::Ship(model.ship.clone()));
        for extra in &model.gallery {
            drawables.push(Sortable::Gallery(extra.clone()));
        }
        drawables.sort_by(|a, b| {
            let y = |item: &Sortable| match item {
                Sortable::Port(port) => port.at.1,
                // Sort by the drawn contact, which moves during the day tween.
                Sortable::Ship(_) => ship_at.y,
                Sortable::Gallery(ship) => ship.at.1,
            };
            y(a).partial_cmp(&y(b)).unwrap_or(std::cmp::Ordering::Equal)
        });
        for item in &drawables {
            match item {
                Sortable::Port(port) => draw_port(&mut canvas, &textures, port, &hover),
                Sortable::Ship(ship) => {
                    draw_ship(&mut canvas, &textures, ship, ship_at);
                }
                Sortable::Gallery(ship) => {
                    draw_ship(
                        &mut canvas,
                        &textures,
                        ship,
                        Vector2::new(ship.at.0, ship.at.1),
                    );
                }
            }
        }
    }
}

#[godot_api]
impl ChartCanvas {
    #[signal]
    fn port_pressed(port_id: GString);
}

/// The generated `port_pressed` accessor is private to this module.
pub fn connect_port_pressed(
    canvas: &mut Gd<ChartCanvas>,
    mut on_press: impl FnMut(GString) + 'static,
) {
    canvas
        .signals()
        .port_pressed()
        .connect(move |port_id: GString| on_press(port_id));
}

enum Sortable {
    Port(ChartPort),
    Ship(ShipMarker),
    Gallery(ShipMarker),
}

impl ChartCanvas {
    pub fn show(&mut self, model: ChartModel, snap: bool) {
        if let Some(mut camera) = self.camera.clone() {
            // Integer camera pixels so a zoom of 1.0 lands plate texels on
            // framebuffer pixels.
            camera.set_position(Vector2::new(
                model.frame.center_x.round(),
                model.frame.center_y.round(),
            ));
            camera.set_zoom(Vector2::new(model.frame.zoom, model.frame.zoom));
        }
        debug_assert_eq!(model.ship.footprint, SHIP_FOOTPRINT_CELLS);
        let target = Vector2::new(model.ship.at.0, model.ship.at.1);
        if snap || self.shown_ship.is_none() {
            self.shown_ship = Some(target);
            self.tween_left = 0.0;
        } else if self
            .shown_ship
            .is_some_and(|shown| shown.distance_to(target) > 0.5)
        {
            self.tween_from = self.shown_ship.unwrap_or(target);
            self.tween_to = target;
            self.tween_left = portlight_chart::DAY_TWEEN_SECS;
        }
        self.model = Some(model);
        self.base_mut().set_process(true);
        self.sync_labels();
        self.base_mut().queue_redraw();
    }

    fn update_hover(&mut self) {
        let Some(model) = self.model.as_ref() else {
            return;
        };
        let at = self.base().get_global_mouse_position();
        let next = hover_at(model, at.x, at.y).unwrap_or_default();
        if next != self.hover {
            self.hover = next;
            self.sync_labels();
            self.base_mut().queue_redraw();
        }
    }

    /// Copy the chart text onto the linear label item. Font sizes stay in
    /// world pixels; the camera zoom scales that item and is not divided out.
    fn sync_labels(&mut self) {
        let Some(mut labels) = self.labels.clone() else {
            return;
        };
        let model = self.model.clone();
        let hover = self.hover.clone();
        let hover_at = self.base().get_local_mouse_position();
        labels.bind_mut().set_scene(model, hover, hover_at);
    }

    fn click_port(&mut self) {
        let Some(model) = self.model.as_ref() else {
            return;
        };
        let at = self.base().get_global_mouse_position();
        let Some(port) = model
            .ports
            .iter()
            .find(|port| {
                let dx = port.at.0 - at.x;
                let dy = port.at.1 - at.y;
                dx * dx + dy * dy <= 36.0 * 36.0
            })
            .map(|port| port.id.clone())
        else {
            return;
        };
        self.signals()
            .port_pressed()
            .emit(&GString::from(port.as_str()));
    }

    fn ensure_textures(&mut self, model: &ChartModel) {
        let mut ids = Vec::new();
        for tile in &model.tiles {
            ids.push(tile.asset_id);
        }
        for port in &model.ports {
            ids.push(port.marker_id);
        }
        ids.push(model.ship.asset_id);
        ids.push(model.ship.wake_id);
        for extra in &model.gallery {
            ids.push(extra.asset_id);
            ids.push(extra.wake_id);
        }
        for id in ids {
            if self.textures.contains_key(id) {
                continue;
            }
            let Some(asset) = asset(id) else {
                continue;
            };
            let path = asset.res_path();
            let Some(resource) = ResourceLoader::singleton().load(&GString::from(path.as_str()))
            else {
                godot_print!("missing texture {}", asset.res_path());
                continue;
            };
            if let Ok(texture) = resource.try_cast::<Texture2D>() {
                self.textures.insert(id.to_string(), texture);
            }
        }
    }
}

fn draw_port(
    canvas: &mut Node2D,
    textures: &HashMap<String, Gd<Texture2D>>,
    port: &ChartPort,
    hover: &str,
) {
    draw_ring(
        canvas,
        port.at,
        &port.region_color,
        hover.contains(&port.name) || port.is_here,
    );
    draw_sprite(
        canvas,
        textures,
        port.marker_id,
        Vector2::new(port.sprite_origin.0, port.sprite_origin.1),
        true,
    );
}

fn draw_ship(
    canvas: &mut Node2D,
    textures: &HashMap<String, Gd<Texture2D>>,
    ship: &ShipMarker,
    at: Vector2,
) {
    let delta = at - Vector2::new(ship.at.0, ship.at.1);
    if !ship.docked {
        draw_sprite(
            canvas,
            textures,
            ship.wake_id,
            Vector2::new(ship.wake_origin.0, ship.wake_origin.1) + delta,
            true,
        );
    }
    draw_sprite(
        canvas,
        textures,
        ship.asset_id,
        Vector2::new(ship.sprite_origin.0, ship.sprite_origin.1) + delta,
        true,
    );
}

fn draw_ring(canvas: &mut Node2D, at: (f32, f32), color: &Rgba, hot: bool) {
    let color = Color::from_rgba8(color.r, color.g, color.b, if hot { 255 } else { 180 });
    let steps = 24;
    let point = |step: i32| {
        let t = step as f32 / steps as f32 * std::f32::consts::TAU;
        Vector2::new(at.0 + t.cos() * 34.0, at.1 + t.sin() * 17.0)
    };
    for step in 0..steps {
        canvas
            .draw_line_ex(point(step), point(step + 1), color)
            .width(2.0)
            .antialiased(true)
            .done();
    }
}

fn draw_segment(canvas: &mut Node2D, from: (f32, f32), to: (f32, f32), color: &Rgba, width: f32) {
    canvas
        .draw_line_ex(
            Vector2::new(from.0, from.1),
            Vector2::new(to.0, to.1),
            Color::from_rgba8(color.r, color.g, color.b, color.a),
        )
        .width(width)
        .antialiased(true)
        .done();
}

fn draw_sprite(
    canvas: &mut Node2D,
    textures: &HashMap<String, Gd<Texture2D>>,
    id: &str,
    at: Vector2,
    snap: bool,
) {
    let Some(texture) = textures.get(id) else {
        return;
    };
    let at = if snap {
        Vector2::new(at.x.round(), at.y.round())
    } else {
        at
    };
    canvas.draw_texture(texture, at);
}

/// World-pixel sizes. [`ChartLabels`] is scaled by the camera; these are not
/// divided by zoom. Linear filtering on that item evens the 0.72 scale.
const PORT_NAME_PX: i32 = 18;
const BADGE_PX: i32 = 16;
const LANE_PX: i32 = 14;
const HOVER_PX: i32 = 15;

/// In-chart text only. Plates stay on [`ChartCanvas`] at nearest.
#[derive(GodotClass)]
#[class(base = Node2D)]
struct ChartLabels {
    base: Base<Node2D>,
    model: Option<ChartModel>,
    hover: String,
    hover_at: Vector2,
    font: Option<Gd<Font>>,
}

#[godot_api]
impl INode2D for ChartLabels {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            model: None,
            hover: String::new(),
            hover_at: Vector2::ZERO,
            font: None,
        }
    }

    fn ready(&mut self) {
        self.base_mut().set_texture_filter(TextureFilter::LINEAR);
        self.font = ThemeDb::singleton().get_fallback_font();
    }

    fn draw(&mut self) {
        if self.font.is_none() {
            self.font = ThemeDb::singleton().get_fallback_font();
        }
        let Some(model) = self.model.clone() else {
            return;
        };
        let Some(font) = self.font.clone() else {
            return;
        };
        let hover = self.hover.clone();
        let hover_at = self.hover_at;
        let mut canvas = self.base_mut();
        draw_chart_text(&mut canvas, &font, &model, &hover, hover_at);
    }
}

impl ChartLabels {
    fn set_scene(&mut self, model: Option<ChartModel>, hover: String, hover_at: Vector2) {
        self.model = model;
        self.hover = hover;
        self.hover_at = hover_at;
        self.base_mut().queue_redraw();
    }
}

fn draw_chart_text(
    canvas: &mut Node2D,
    font: &Gd<Font>,
    model: &ChartModel,
    hover: &str,
    hover_at: Vector2,
) {
    for port in &model.ports {
        let color = if port.is_here || hover.contains(&port.name) {
            Color::from_rgb(0.96, 0.84, 0.45)
        } else {
            Color::from_rgb(0.93, 0.9, 0.84)
        };
        canvas
            .draw_string_ex(
                font,
                Vector2::new(port.at.0 - 48.0, port.at.1 - 78.0),
                &GString::from(&port.name),
            )
            .font_size(PORT_NAME_PX)
            .modulate(color)
            .done();
        if let Some(badge) = port.badge {
            canvas
                .draw_string_ex(
                    font,
                    Vector2::new(port.at.0 + 28.0, port.at.1 - 36.0),
                    &GString::from(badge.to_string().as_str()),
                )
                .font_size(BADGE_PX)
                .modulate(Color::from_rgba8(
                    port.region_color.r,
                    port.region_color.g,
                    port.region_color.b,
                    255,
                ))
                .done();
        }
    }
    for lane in &model.lanes {
        let caption = lane_caption(lane);
        canvas
            .draw_string_ex(
                font,
                Vector2::new(
                    (lane.from.0 + lane.to.0) / 2.0 - 36.0,
                    (lane.from.1 + lane.to.1) / 2.0 - 18.0,
                ),
                &GString::from(caption.as_str()),
            )
            .font_size(LANE_PX)
            .modulate(Color::from_rgba8(
                lane.color.r,
                lane.color.g,
                lane.color.b,
                255,
            ))
            .done();
    }
    if !hover.is_empty() {
        canvas
            .draw_string_ex(
                font,
                hover_at + Vector2::new(14.0, -14.0),
                &GString::from(hover),
            )
            .font_size(HOVER_PX)
            .modulate(Color::from_rgb(0.96, 0.93, 0.86))
            .done();
    }
}

fn lane_caption(lane: &ChartLane) -> String {
    let tag = match lane.suitability {
        LaneSuitability::Ok => "",
        LaneSuitability::Warning => " WARNING",
        LaneSuitability::Blocked => " BLOCKED",
    };
    if lane.destination_on_chart {
        format!("{}d{tag}", lane.estimated_days)
    } else {
        format!("{} {}d{tag}", lane.destination_name, lane.estimated_days)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn chart_text_is_linear_and_the_plates_stay_nearest() {
        let src = include_str!("chart_canvas.rs");
        let (art, labels) = src.split_once("struct ChartLabels").expect("label item");
        assert!(
            art.contains("set_texture_filter(TextureFilter::NEAREST)"),
            "chart art and plates stay nearest"
        );
        assert!(
            labels.contains("set_texture_filter(TextureFilter::LINEAR)"),
            "in-chart text has its own linear filter"
        );
        let draw = labels
            .split_once("fn draw_chart_text")
            .expect("text draw")
            .1
            .split_once("fn lane_caption")
            .expect("lane caption")
            .0;
        assert!(draw.contains("font_size(PORT_NAME_PX)"));
        assert!(draw.contains("font_size(BADGE_PX)"));
        assert!(draw.contains("font_size(LANE_PX)"));
        assert!(draw.contains("font_size(HOVER_PX)"));
        assert!(
            !draw.contains("zoom"),
            "font size stays in world pixels and is not divided by zoom"
        );
    }
}
