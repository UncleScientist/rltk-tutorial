use crate::{ApplyMove, Chasing, Map, MyTurn, Position};
use specs::prelude::*;
use std::collections::HashMap;

pub struct ChaseAI {}

type ChaseData<'a> = (
    WriteStorage<'a, MyTurn>,
    WriteStorage<'a, Chasing>,
    ReadStorage<'a, Position>,
    ReadExpect<'a, Map>,
    Entities<'a>,
    WriteStorage<'a, ApplyMove>,
);

impl<'a> System<'a> for ChaseAI {
    type SystemData = ChaseData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut chasing, positions, map, entities, mut apply_move) = data;

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
        for (entity, pos, _chase, _myturn) in (&entities, &positions, &chasing, &turns).join() {
            turn_done.push(entity);
            let target_pos = targets[&entity];
            let path = rltk::a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(target_pos.0, target_pos.1) as i32,
                &*map,
            );
            if path.success && path.steps.len() > 1 && path.steps.len() < 15 {
                apply_move
                    .insert(
                        entity,
                        ApplyMove {
                            dest_idx: path.steps[1],
                        },
                    )
                    .expect("Unable to insert");
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
