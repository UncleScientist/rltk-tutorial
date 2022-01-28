use super::{BlocksTile, Position};
use crate::{spatial::*, Map};
use specs::prelude::*;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, positions, blockers, entities) = data;

        clear();
        populate_blocked_from_map(&*map);

        for (entity, position) in (&entities, &positions).join() {
            let idx = map.xy_idx(position.x, position.y);
            index_entity(entity, idx, blockers.get(entity).is_some());
        }
    }
}
