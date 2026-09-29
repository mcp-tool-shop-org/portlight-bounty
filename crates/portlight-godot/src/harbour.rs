//! Places a [`portlight_chart::HarbourTile`] list.
//!
//! Water stays on its own layer and does not Y-sort. Quay paving uses the
//! existing Land ground layer (art-gate U.4: offset `0,0`, y-sort origin `0`).
//! The land anchor is the footprint bottom with no sea datum, which is where
//! the flag diamond meets the quay block's top face. A separate ground node
//! at a lower z would paint that diamond before the block and hide it, so
//! each paving plate is a child of its quay sprite: Godot draws the block,
//! then the child, and a front prop on the works Y-sort still draws after
//! that group. The child's position is only the delta from the block anchor
//! to the U.4 screen point; the plate offset stays the manifest anchor.
//! Every raised work is a child of one `y_sort_enabled` node, positioned on
//! the footprint-bottom anchor, so Godot orders cells by footprint depth
//! (`col + row`). Equal Y keeps tree order, which is the builder's tie-break
//! (pilings before a pier or quay). Sprites do not get a z of their own.

use godot::classes::canvas_item::TextureFilter;
use godot::classes::{Node2D, ResourceLoader, Sprite2D, Texture2D};
use godot::prelude::*;
use portlight_chart::{HarbourLayer, HarbourTile, WorkKind};

/// `false` when a plate failed to load.
pub fn place_harbour(root: &mut Gd<Node2D>, tiles: &[HarbourTile]) -> bool {
    let mut water = Node2D::new_alloc();
    water.set_name("Water");
    water.set_z_index(0);
    water.set_y_sort_enabled(false);

    let mut works = Node2D::new_alloc();
    works.set_name("Works");
    works.set_z_index(1);
    works.set_y_sort_enabled(true);

    // Quay sprites in list order. Paving follows its block and is parented
    // there, so the flag draws after the block and before later props.
    let mut quays: Vec<((i32, i32), Gd<Sprite2D>)> = Vec::new();

    let mut ok = true;
    for tile in tiles {
        let Some(mut sprite) = sprite_for(tile) else {
            ok = false;
            continue;
        };
        match tile.layer {
            HarbourLayer::Water => water.add_child(&sprite),
            HarbourLayer::Work => {
                // Children stay in tree order at this sprite's Y. Y-sort on
                // the sprite would lift the land diamond (higher on screen)
                // back behind the block.
                sprite.set_y_sort_enabled(false);
                if tile.kind == Some(WorkKind::Quay) {
                    quays.push(((tile.col, tile.row), sprite.clone()));
                }
                works.add_child(&sprite);
            }
            HarbourLayer::Land => {
                let Some((_, quay)) = quays
                    .iter()
                    .rev()
                    .find(|(cell, _)| *cell == (tile.col, tile.row))
                else {
                    ok = false;
                    continue;
                };
                let mut land = Node2D::new_alloc();
                land.set_name("Land");
                // Relative z 0 stays in the block's band. A higher z would
                // paint the flag over front props as well as the top face.
                land.set_z_index(0);
                land.set_y_sort_enabled(false);
                let quay_pos = quay.get_position();
                land.set_position(Vector2::new(
                    tile.screen_x as f32 - quay_pos.x,
                    tile.screen_y as f32 - quay_pos.y,
                ));
                sprite.set_position(Vector2::ZERO);
                land.add_child(&sprite);
                let mut quay = quay.clone();
                quay.add_child(&land);
            }
        }
    }
    root.add_child(&water);
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
