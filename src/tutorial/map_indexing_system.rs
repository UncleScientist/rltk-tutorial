use super::{BlocksTile, Position};
use crate::Map;
use specs::prelude::*;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, positions, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();

        for (entity, position) in (&entities, &positions).join() {
            let idx = map.xy_idx(position.x, position.y);
            map.blocked[idx] = blockers.get(entity).is_some();
            map.tile_content[idx].push(entity);
        }
    }
}
