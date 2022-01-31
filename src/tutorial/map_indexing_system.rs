use super::{BlocksTile, Pools, Position};
use crate::{spatial::*, Map};
use specs::prelude::*;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        ReadStorage<'a, Pools>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, positions, blockers, pools, entities) = data;

        clear();
        populate_blocked_from_map(&*map);

        for (entity, position) in (&entities, &positions).join() {
            let mut alive = true;
            if let Some(pools) = pools.get(entity) {
                if pools.hit_points.current < 1 {
                    alive = false;
                }
            }
            if alive {
                let idx = map.xy_idx(position.x, position.y);
                index_entity(entity, idx, blockers.get(entity).is_some());
            }
        }
    }
}
