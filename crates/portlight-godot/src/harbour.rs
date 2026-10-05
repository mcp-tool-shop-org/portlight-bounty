//! Places a [`portlight_chart::HarbourTile`] list.
//!
//! Water stays on its own layer and does not Y-sort. Quay paving uses the
//! existing Land ground layer (art-gate U.4: offset `0,0`, y-sort origin `0`).
//! The land anchor is the footprint bottom with no sea datum, which is where
//! the flag diamond meets the quay block's top face. A separate ground node
//! at a lower z would paint that diamond before the block and hide it, so
//! each paving plate is a child of its quay sprite: Godot draws the block,
//! then the child, and deck props (Y.1) are further children of that pier or
//! quay so they draw after the flag. The child's position is only the delta
//! from the block anchor to the land-offset screen point; the plate offset
//! stays the manifest anchor.
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

    // Structure sprites in list order. Paving and deck props parent under
    // the pier/quay on the cell so the flag draws after the block and deck
    // props draw after the flag (art-gate W / Y.1).
    let mut structures: Vec<((i32, i32), Gd<Sprite2D>)> = Vec::new();

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
                if matches!(tile.kind, Some(WorkKind::Quay) | Some(WorkKind::Pier)) {
                    structures.push(((tile.col, tile.row), sprite.clone()));
                }
                works.add_child(&sprite);
            }
            HarbourLayer::Land => {
                let Some((_, host)) = structures
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
                let host_pos = host.get_position();
                land.set_position(Vector2::new(
                    tile.screen_x as f32 - host_pos.x,
                    tile.screen_y as f32 - host_pos.y,
                ));
                sprite.set_position(Vector2::ZERO);
                land.add_child(&sprite);
                let mut host = host.clone();
                host.add_child(&land);
            }
            HarbourLayer::Prop => {
                let Some((_, host)) = structures
                    .iter()
                    .rev()
                    .find(|(cell, _)| *cell == (tile.col, tile.row))
                else {
                    ok = false;
                    continue;
                };
                // Land offset 0 relative to the SeaWorks host (same delta as
                // paving). Added after Land in tile order so props win.
                let host_pos = host.get_position();
                sprite.set_position(Vector2::new(
                    tile.screen_x as f32 - host_pos.x,
                    tile.screen_y as f32 - host_pos.y,
                ));
                sprite.set_y_sort_enabled(false);
                let mut host = host.clone();
                host.add_child(&sprite);
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

#[cfg(test)]
mod tests {
    use portlight_chart::{harbour_seam, HarbourLayer, WorkKind};

    /// Godot z and add order for one cell: the block, then its paving child,
    /// then a front prop. Paving is not a root layer under the blocks.
    #[test]
    fn block_is_below_paving_is_below_props() {
        let tiles = harbour_seam().expect("legal seam");
        let block_at = tiles
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Quay))
            .expect("block");
        let paving_at = tiles
            .iter()
            .position(|tile| tile.layer == HarbourLayer::Land)
            .expect("paving");
        let prop_at = tiles
            .iter()
            .position(|tile| tile.kind == Some(WorkKind::Prop))
            .expect("prop");
        assert!(
            block_at < paving_at && paving_at < prop_at,
            "the tile list is block, then paving, then props"
        );

        let body = include_str!("harbour.rs")
            .split_once("pub fn place_harbour")
            .expect("placer")
            .1
            .split_once("fn sprite_for")
            .expect("sprite_for")
            .0;

        let water_z = body.find("water.set_z_index(0)").expect("water z 0");
        let works_z = body.find("works.set_z_index(1)").expect("works z 1");
        let land_z = body.find("land.set_z_index(0)").expect("land z 0");
        assert!(
            water_z < works_z,
            "water stays under the works band that holds blocks and props"
        );

        let add_water = body.find("root.add_child(&water)").expect("add water");
        let add_works = body.find("root.add_child(&works)").expect("add works");
        assert!(
            add_water < add_works,
            "the root adds water, then the works node"
        );
        assert!(
            !body.contains("root.add_child(&land)"),
            "paving is not its own layer under the blocks"
        );

        // The loop follows the tile list. The block sprite joins Works, then
        // the paving node is a child of that sprite, so the block texture
        // draws first. Relative z 0 keeps the flag in the block's band; a
        // higher z would cover front props. Works y-sort draws a higher
        // footprint after that group.
        let works_add = body
            .find("works.add_child(&sprite)")
            .expect("block and props join works");
        let quay_add = body
            .find("host.add_child(&land)")
            .expect("paving is a child of the block");
        let y_sort = body
            .find("works.set_y_sort_enabled(true)")
            .expect("works y-sort");
        assert!(works_add < quay_add, "the block is added before its paving");
        assert!(
            land_z < quay_add,
            "paving z is set before it is parented on the block"
        );
        assert!(
            y_sort < works_add,
            "props on the works node y-sort above the block and its flag"
        );
        assert!(
            body.contains("sprite.set_y_sort_enabled(false)"),
            "a work sprite must not y-sort its paving child back behind the block"
        );
        assert!(body.contains("for tile in tiles"));
        let per_tile_z = ["set_z_index(", "tile"].concat();
        assert!(
            !body.contains(&per_tile_z),
            "sprites are not given a z per id"
        );
    }
}
