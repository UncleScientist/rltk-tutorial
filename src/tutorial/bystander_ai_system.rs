use super::{Bystander, EntityMoved, Position, RunState, Viewshed};
use crate::map::*;
use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub struct BystanderAI {}

type BystanderAIData<'a> = (
    WriteExpect<'a, Map>,
    ReadExpect<'a, RunState>,
    Entities<'a>,
    WriteStorage<'a, Viewshed>,
    ReadStorage<'a, Bystander>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, EntityMoved>,
    WriteExpect<'a, RandomNumberGenerator>,
);

impl<'a> System<'a> for BystanderAI {
    type SystemData = BystanderAIData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            runstate,
            entities,
            mut viewshed,
            bystander,
            mut position,
            mut entity_moved,
            mut rng,
        ) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        }

        for (entity, mut viewshed, _, mut pos) in
            (&entities, &mut viewshed, &bystander, &mut position).join()
        {
            // Try to move randomly
            let mut x = pos.x;
            let mut y = pos.y;
            let move_roll = rng.roll_dice(1, 5);

            match move_roll {
                1 => x -= 1,
                2 => x += 1,
                3 => y -= 1,
                4 => y += 1,
                _ => {}
            }

            if x > 0 && x < map.width - 1 && y > 0 && y < map.height - 1 {
                let dest_idx = map.xy_idx(x, y);
                if !map.blocked[dest_idx] {
                    let idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = false;
                    pos.x = x;
                    pos.y = y;
                    entity_moved
                        .insert(entity, EntityMoved {})
                        .expect("unable to insert marker");
                    map.blocked[dest_idx] = true;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
