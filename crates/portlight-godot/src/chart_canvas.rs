//! Draws one [`portlight_chart::ChartModel`]. No rules live here.

use std::collections::HashMap;

use godot::classes::{Camera2D, Font, INode2D, Node2D, ResourceLoader, Texture2D, ThemeDb};
use godot::prelude::*;
use portlight_chart::{asset, ChartLane, ChartModel, Rgba, SHIP_FOOTPRINT_CELLS};
use portlight_sim::LaneSuitability;

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct ChartCanvas {
    base: Base<Node2D>,
    model: Option<ChartModel>,
    textures: HashMap<String, Gd<Texture2D>>,
    font: Option<Gd<Font>>,
    camera: Option<Gd<Camera2D>>,
}

#[godot_api]
impl INode2D for ChartCanvas {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            model: None,
            textures: HashMap::new(),
            font: None,
            camera: None,
        }
    }

    fn ready(&mut self) {
        let mut camera = Camera2D::new_alloc();
        camera.set_enabled(true);
        self.base_mut().add_child(&camera);
        camera.make_current();
        self.camera = Some(camera);
        self.font = ThemeDb::singleton().get_fallback_font();
    }

    fn draw(&mut self) {
        let Some(model) = self.model.clone() else {
            return;
        };
        self.ensure_textures(&model);
        if self.font.is_none() {
            self.font = ThemeDb::singleton().get_fallback_font();
        }
        let textures = self.textures.clone();
        let font = self.font.clone();
        let mut canvas = self.base_mut();
        let sea = Color::from_rgb(0.05, 0.1, 0.16);
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
            draw_swatch(&mut canvas, &textures, lane.asset_id, lane.from, lane.to);
        }
        if let Some(leg) = &model.leg {
            draw_swatch(
                &mut canvas,
                &textures,
                "chart.lane.underway",
                leg.from,
                leg.to,
            );
        }
        for port in &model.ports {
            draw_sprite(
                &mut canvas,
                &textures,
                port.tile_id,
                Vector2::new(port.sprite_origin.0 as f32, port.sprite_origin.1 as f32),
            );
            draw_sprite(
                &mut canvas,
                &textures,
                port.marker_id,
                Vector2::new(port.sprite_origin.0 as f32, port.sprite_origin.1 as f32),
            );
        }
        draw_sprite(
            &mut canvas,
            &textures,
            model.ship.asset_id,
            Vector2::new(model.ship.sprite_origin.0, model.ship.sprite_origin.1),
        );
        let Some(font) = font else {
            return;
        };
        for port in &model.ports {
            let color = if port.is_here {
                Color::from_rgb(0.96, 0.84, 0.45)
            } else {
                Color::from_rgb(0.93, 0.9, 0.84)
            };
            canvas
                .draw_string_ex(
                    &font,
                    Vector2::new(port.anchor.0 as f32 - 48.0, port.anchor.1 as f32 - 52.0),
                    &GString::from(&port.name),
                )
                .font_size(18)
                .modulate(color)
                .done();
        }
        for lane in &model.lanes {
            let caption = lane_caption(lane);
            canvas
                .draw_string_ex(
                    &font,
                    Vector2::new(
                        (lane.from.0 + lane.to.0) as f32 / 2.0 - 36.0,
                        (lane.from.1 + lane.to.1) as f32 / 2.0 - 28.0,
                    ),
                    &GString::from(caption.as_str()),
                )
                .font_size(14)
                .modulate(Color::from_rgba8(
                    lane.color.r,
                    lane.color.g,
                    lane.color.b,
                    255,
                ))
                .done();
        }
    }
}

impl ChartCanvas {
    pub fn show(&mut self, model: ChartModel) {
        if let Some(mut camera) = self.camera.clone() {
            camera.set_position(Vector2::new(model.frame.center_x, model.frame.center_y));
            camera.set_zoom(Vector2::new(model.frame.zoom, model.frame.zoom));
        }
        debug_assert_eq!(model.ship.footprint, SHIP_FOOTPRINT_CELLS);
        self.model = Some(model);
        self.base_mut().queue_redraw();
    }

    fn ensure_textures(&mut self, model: &ChartModel) {
        let mut ids = Vec::new();
        for port in &model.ports {
            ids.push(port.tile_id);
            ids.push(port.marker_id);
        }
        for lane in &model.lanes {
            ids.push(lane.asset_id);
        }
        if model.leg.is_some() {
            ids.push("chart.lane.underway");
        }
        ids.push(model.ship.asset_id);
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
                godot_print!("missing placeholder {}", asset.res_path());
                continue;
            };
            if let Ok(texture) = resource.try_cast::<Texture2D>() {
                self.textures.insert(id.to_string(), texture);
            }
        }
    }
}

fn draw_segment(canvas: &mut Node2D, from: (i32, i32), to: (i32, i32), color: &Rgba, width: f32) {
    canvas
        .draw_line_ex(
            Vector2::new(from.0 as f32, from.1 as f32),
            Vector2::new(to.0 as f32, to.1 as f32),
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
) {
    let Some(texture) = textures.get(id) else {
        return;
    };
    canvas.draw_texture(texture, at);
}

fn draw_swatch(
    canvas: &mut Node2D,
    textures: &HashMap<String, Gd<Texture2D>>,
    id: &str,
    from: (i32, i32),
    to: (i32, i32),
) {
    let Some(texture) = textures.get(id) else {
        return;
    };
    let width = 40.0;
    let height = 20.0;
    let center = Vector2::new((from.0 + to.0) as f32 / 2.0, (from.1 + to.1) as f32 / 2.0);
    canvas.draw_texture_rect(
        texture,
        Rect2::new(
            center - Vector2::new(width / 2.0, height / 2.0),
            Vector2::new(width, height),
        ),
        false,
    );
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
