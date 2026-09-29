//! Draws [`portlight_chart::harbour_seam`]. Positions come from that list.
//! The list is [`portlight_chart::grid_to_screen`] plus the sea datum.
//! No label. Captures zoom 1, zoom 0.72, a crop of the interior vertex,
//! and the quay close-ups `quay-paving-z100.png` and `quay-paving-z072.png`.
//! Those close-ups are [`portlight_chart::quay_paving_crop`]: zoom 1 is
//! `(340, 160, 360, 340)`, and zoom 0.72 is that same world window.
//!
//! The layout is validated before any PNG is written. An illegal layout
//! exits non-zero and does not save a frame. A blank or mostly flat frame
//! fails the same check as `PORTLIGHT_SHOT`.

use godot::classes::{Camera2D, INode2D, Image, Node2D};
use godot::global::Error;
use godot::prelude::*;
use portlight_chart::{harbour_seam, quay_paving_crop, seam_camera_center, seam_interior_vertex};

use crate::harbour::place_harbour;
use crate::logic::{capture_frame_rejected, frame_samples, seam_exit_code, WINDOW_H, WINDOW_W};

const CROP_W: i32 = 192;
const CROP_H: i32 = 128;

#[derive(GodotClass)]
#[class(base = Node2D)]
struct HarbourSeam {
    base: Base<Node2D>,
    camera: Option<Gd<Camera2D>>,
    frames: i32,
    shot_dir: String,
    failed: bool,
    illegal: bool,
    done: bool,
}

#[godot_api]
impl INode2D for HarbourSeam {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            camera: None,
            frames: 0,
            shot_dir: std::env::var("PORTLIGHT_SEAM_DIR").unwrap_or_else(|_| "/tmp".to_string()),
            failed: false,
            illegal: false,
            done: false,
        }
    }

    fn ready(&mut self) {
        let tiles = match harbour_seam() {
            Ok(tiles) => tiles,
            Err(faults) => {
                for fault in &faults {
                    godot_print!("harbour seam illegal layout: {fault}");
                }
                self.illegal = true;
                self.failed = true;
                return;
            }
        };
        if !place_harbour(&mut self.base_mut(), &tiles) {
            godot_print!("harbour seam missing a plate");
            self.failed = true;
        }

        let (cx, cy) = seam_camera_center();
        let mut camera = Camera2D::new_alloc();
        camera.set_position(Vector2::new(cx as f32, cy as f32));
        camera.set_zoom(Vector2::new(1.0, 1.0));
        self.base_mut().add_child(&camera);
        camera.make_current();
        self.camera = Some(camera);
    }

    fn process(&mut self, _delta: f64) {
        if self.done {
            return;
        }
        if self.illegal {
            self.finish();
            return;
        }
        self.frames += 1;
        if self.frames == 4 {
            self.save_viewport("harbour-seam-z100.png");
            self.save_vertex_crop("harbour-seam-vertex-crop.png");
            self.save_quay_closeup("quay-paving-z100.png", 1.0);
            if let Some(mut camera) = self.camera.clone() {
                camera.set_zoom(Vector2::new(0.72, 0.72));
            }
            return;
        }
        if self.frames == 8 {
            self.save_viewport("harbour-seam-z072.png");
            self.save_quay_closeup("quay-paving-z072.png", 0.72);
            self.finish();
        }
    }
}

impl HarbourSeam {
    fn finish(&mut self) {
        if self.done {
            return;
        }
        self.done = true;
        let code = seam_exit_code(!self.illegal, !self.failed);
        godot_print!("harbour seam {}", if code == 0 { "ok" } else { "FAILED" });
        let mut tree = self.base().get_tree();
        tree.quit_ex().exit_code(code).done();
    }

    fn save_viewport(&mut self, name: &str) {
        let Some(image) = self.viewport_image() else {
            godot_print!("harbour seam viewport image was empty");
            self.failed = true;
            return;
        };
        self.write_checked(&image, name, WINDOW_W as i32, WINDOW_H as i32);
    }

    fn save_vertex_crop(&mut self, name: &str) {
        let Some(image) = self.viewport_image() else {
            self.failed = true;
            return;
        };
        let (wx, wy) = seam_interior_vertex();
        let Some(viewport) = self.base().get_viewport() else {
            self.failed = true;
            return;
        };
        let screen = viewport.get_canvas_transform() * Vector2::new(wx as f32, wy as f32);
        let x = (screen.x.round() as i32 - CROP_W / 2).clamp(0, image.get_width() - CROP_W);
        let y = (screen.y.round() as i32 - CROP_H / 2).clamp(0, image.get_height() - CROP_H);
        let Some(crop) = image.get_region(Rect2i::from_components(x, y, CROP_W, CROP_H)) else {
            godot_print!("harbour seam vertex crop was empty");
            self.failed = true;
            return;
        };
        self.write_checked(&crop, name, CROP_W, CROP_H);
    }

    /// Quay close-up. The box is the recorded zoom-1 crop, or that same
    /// world window at the capture zoom.
    fn save_quay_closeup(&mut self, name: &str, zoom: f32) {
        let Some(image) = self.viewport_image() else {
            self.failed = true;
            return;
        };
        let (x, y, w, h) = quay_paving_crop(zoom);
        if x < 0 || y < 0 || x + w > image.get_width() || y + h > image.get_height() {
            godot_print!("harbour seam quay close-up {name} is outside the frame");
            self.failed = true;
            return;
        }
        let Some(crop) = image.get_region(Rect2i::from_components(x, y, w, h)) else {
            godot_print!("harbour seam quay close-up {name} was empty");
            self.failed = true;
            return;
        };
        self.write_checked(&crop, name, w, h);
    }

    fn write_checked(&mut self, image: &Gd<Image>, name: &str, expect_w: i32, expect_h: i32) {
        let samples = frame_samples(image);
        if capture_frame_rejected(
            image.get_width(),
            image.get_height(),
            expect_w,
            expect_h,
            &samples,
        ) {
            godot_print!(
                "harbour seam rejected {name}: expected a {expect_w}x{expect_h} frame that is not blank or one flat colour"
            );
            self.failed = true;
            return;
        }
        self.write_png(image, name);
    }

    fn viewport_image(&self) -> Option<Gd<Image>> {
        self.base()
            .get_viewport()
            .and_then(|viewport| viewport.get_texture())
            .and_then(|texture| texture.get_image())
            .filter(|image| !image.is_empty())
    }

    fn write_png(&mut self, image: &Gd<Image>, name: &str) {
        let path = format!("{}/{name}", self.shot_dir);
        let err = image.save_png(&path);
        godot_print!(
            "harbour seam {path} {}x{} error={err:?}",
            image.get_width(),
            image.get_height()
        );
        if err != Error::OK {
            self.failed = true;
        }
    }
}
