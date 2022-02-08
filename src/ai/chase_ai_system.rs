use crate::{Chasing, EntityMoved, Map, MyTurn, Position, Viewshed};
use specs::prelude::*;
use std::collections::HashMap;

pub struct ChaseAI {}

type ChaseData<'a> = (
    WriteStorage<'a, MyTurn>,
    WriteStorage<'a, Chasing>,
    WriteStorage<'a, Position>,
    ReadExpect<'a, Map>,
    WriteStorage<'a, Viewshed>,
    WriteStorage<'a, EntityMoved>,
    Entities<'a>,
);

impl<'a> System<'a> for ChaseAI {
    type SystemData = ChaseData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut chasing, mut positions, map, mut viewsheds, mut entity_moved, entities) =
            data;

        let mut targets = HashMap::new();
        let mut end_chase = Vec::new();

        for (entity, _turn, chasing) in (&entities, &turns, &chasing).join() {
            if let Some(target_pos) = positions.get(chasing.target) {
                targets.insert(entity, (target_pos.x, target_pos.y));
            } else {
                end_chase.push(entity);
            }
        }

        for done in end_chase.iter() {
            chasing.remove(*done);
        }
        end_chase.clear();

        let mut turn_done = Vec::new();
        for (entity, mut pos, _chase, mut viewshed, _myturn) in
            (&entities, &mut positions, &chasing, &mut viewsheds, &turns).join()
        {
            turn_done.push(entity);
            let target_pos = targets[&entity];
            let path = rltk::a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(target_pos.0, target_pos.1) as i32,
                &*map,
            );
            if path.success && path.steps.len() > 1 && path.steps.len() < 15 {
                let idx = map.xy_idx(pos.x, pos.y);
                pos.x = path.steps[1] as i32 % map.width;
                pos.y = path.steps[1] as i32 / map.width;
                entity_moved
                    .insert(entity, EntityMoved {})
                    .expect("Unable to insert marker");
                let new_idx = map.xy_idx(pos.x, pos.y);
                crate::spatial::move_entity(entity, idx, new_idx);
                viewshed.dirty = true;
                turn_done.push(entity);
            } else {
                end_chase.push(entity);
            }
        }

        for done in end_chase.iter() {
            chasing.remove(*done);
        }
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
