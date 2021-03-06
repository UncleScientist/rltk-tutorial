use crate::{BlocksVisibility, Hidden, Map, Name, Player, Position, Viewshed};
use rltk::{field_of_view, Point};
use specs::prelude::*;

pub struct VisibilitySystem {}

type VisibilityData<'a> = (
    WriteExpect<'a, Map>,
    Entities<'a>,
    WriteStorage<'a, Viewshed>,
    WriteStorage<'a, Position>,
    ReadStorage<'a, Player>,
    WriteStorage<'a, Hidden>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, BlocksVisibility>,
);

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = VisibilityData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player, mut hidden, names, blocks_visibility) =
            data;

        map.view_blocked.clear();
        for (block_pos, _block) in (&pos, &blocks_visibility).join() {
            let idx = map.xy_idx(block_pos.x, block_pos.y);
            map.view_blocked.insert(idx);
        }

        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles =
                    field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed
                    .visible_tiles
                    .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                // If this is the player, reveal what they can see
                if player.get(ent).is_some() {
                    for t in map.visible_tiles.iter_mut() {
                        *t = false;
                    }
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;

                        // Chance to reveal hidden things
                        crate::spatial::for_each_tile_content(idx, |e| {
                            if hidden.get(e).is_some()
                                && crate::tutorial::rng::roll_dice(1, 24) == 1
                            {
                                if let Some(name) = names.get(e) {
                                    crate::gamelog::Logger::new()
                                        .append("You spotted:")
                                        .color(rltk::RED)
                                        .append(&name.name)
                                        .log();
                                }
                                hidden.remove(e);
                            }
                        });
                    }
                }
            }
        }
    }
}
