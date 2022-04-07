use crate::{ApplyMove, Map, MyTurn, Position, WantsToFlee};
use specs::prelude::*;

pub struct FleeAI {}

type FleeData<'a> = (
    WriteStorage<'a, MyTurn>,
    WriteStorage<'a, WantsToFlee>,
    ReadStorage<'a, Position>,
    ReadExpect<'a, Map>,
    Entities<'a>,
    WriteStorage<'a, ApplyMove>,
);

impl<'a> System<'a> for FleeAI {
    type SystemData = FleeData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut want_flee, positions, map, entities, mut apply_move) = data;

        let mut turn_done = Vec::new();

        for (entity, pos, flee, _myturn) in (&entities, &positions, &want_flee, &turns).join() {
            turn_done.push(entity);
            let my_idx = map.xy_idx(pos.x, pos.y);
            map.populate_blocked();
            let flee_map = rltk::DijkstraMap::new(
                map.width as usize,
                map.height as usize,
                &flee.indices,
                &*map,
                100.0,
            );
            let flee_target = rltk::DijkstraMap::find_highest_exit(&flee_map, my_idx, &*map);
            if let Some(flee_target) = flee_target {
                if !crate::spatial::is_blocked(flee_target as usize) {
                    apply_move
                        .insert(
                            entity,
                            ApplyMove {
                                dest_idx: flee_target,
                            },
                        )
                        .expect("Unable to insert marker");
                }
            }
        }

        want_flee.clear();

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
