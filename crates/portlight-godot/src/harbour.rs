//! Places a [`portlight_chart::HarbourTile`] list.
//!
//! Water stays on its own layer and does not Y-sort. Quay paving uses the
//! existing Land ground layer (art-gate U.4: offset `0,0`, y-sort origin `0`),
//! which is the same ground band as water and is not a layer of its own above
//! the works. Land Y-sorts its own tiles by the footprint anchor and does not
//! sort against blocks or props. Every raised work is a child of one
//! `y_sort_enabled` node, positioned on the footprint-bottom anchor, so
//! Godot orders them by footprint depth (`col + row`). The seam uses that
//! depth sort. Equal Y keeps tree order, which is the builder's tie-break
//! (pilings before a pier or quay). Sprites do not get a z of their own.

use godot::classes::canvas_item::TextureFilter;
use godot::classes::{Node2D, ResourceLoader, Sprite2D, Texture2D};
use godot::prelude::*;
use portlight_chart::{HarbourLayer, HarbourTile};

/// `false` when a plate failed to load.
pub fn place_harbour(root: &mut Gd<Node2D>, tiles: &[HarbourTile]) -> bool {
    let mut water = Node2D::new_alloc();
    water.set_name("Water");
    water.set_z_index(0);
    water.set_y_sort_enabled(false);

    // Land is the quay-paving ground layer. z 0 keeps it with water, under
    // works at z 1, so paving cannot draw over quay blocks or props.
    // Y-sort origin 0 is the sprite anchor; the node position stays 0,0.
    let mut land = Node2D::new_alloc();
    land.set_name("Land");
    land.set_z_index(0);
    land.set_y_sort_enabled(true);

    let mut works = Node2D::new_alloc();
    works.set_name("Works");
    works.set_z_index(1);
    works.set_y_sort_enabled(true);

    let mut ok = true;
    for tile in tiles {
        let Some(sprite) = sprite_for(tile) else {
            ok = false;
            continue;
        };
        match tile.layer {
            HarbourLayer::Water => water.add_child(&sprite),
            HarbourLayer::Land => land.add_child(&sprite),
            HarbourLayer::Work => works.add_child(&sprite),
        }
    }
    root.add_child(&water);
    root.add_child(&land);
    root.add_child(&works);
    ok
}

fn sprite_for(tile: &HarbourTile) -> Option<Gd<Sprite2D>> {
    let resource = ResourceLoader::singleton().load(&GString::from(tile.path))?;
    let texture = resource.try_cast::<Texture2D>().ok()?;
    let mut sprite = Sprite2D::new_alloc();
    sprite.set_texture(&texture);
    sprite.set_centered(false);
    sprite.set_texture_filter(TextureFilter::NEAREST);
    sprite.set_position(Vector2::new(tile.screen_x as f32, tile.screen_y as f32));
    sprite.set_offset(Vector2::new(-tile.anchor_x as f32, -tile.anchor_y as f32));
    Some(sprite)
}
