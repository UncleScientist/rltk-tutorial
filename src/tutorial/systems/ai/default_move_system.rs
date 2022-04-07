use crate::{tile_walkable, ApplyMove, Map, MoveMode, Movement, MyTurn, Position};
use specs::prelude::*;

pub struct DefaultMoveAI {}

type DefaultMoveData<'a> = (
    WriteStorage<'a, MyTurn>,
    WriteStorage<'a, MoveMode>,
    ReadStorage<'a, Position>,
    ReadExpect<'a, Map>,
    Entities<'a>,
    WriteStorage<'a, ApplyMove>,
);

impl<'a> System<'a> for DefaultMoveAI {
    type SystemData = DefaultMoveData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut move_mode, positions, map, entities, mut apply_move) = data;

        let mut turn_done = Vec::new();

        for (entity, pos, mut mode, _myturn) in
            (&entities, &positions, &mut move_mode, &turns).join()
        {
            match &mut mode.mode {
                Movement::Static => {}
                Movement::Random => {
                    let mut x = pos.x;
                    let mut y = pos.y;
                    let move_roll = crate::tutorial::rng::roll_dice(1, 5);

                    match move_roll {
                        1 => x -= 1,
                        2 => x += 1,
                        3 => y -= 1,
                        4 => y += 1,
                        _ => {}
                    }

                    if x > 0 && x < map.width - 1 && y > 0 && y < map.height - 1 {
                        let dest_idx = map.xy_idx(x, y);
                        apply_move
                            .insert(entity, ApplyMove { dest_idx })
                            .expect("Unable to insert");
                        turn_done.push(entity);
                    }
                }
                Movement::RandomWaypoint { path } => {
                    if let Some(path) = path {
                        // We have a target - go there
                        if path.len() > 1 {
                            if !crate::spatial::is_blocked(path[1] as usize) {
                                apply_move
                                    .insert(entity, ApplyMove { dest_idx: path[1] })
                                    .expect("Unable to insert");
                                turn_done.push(entity);
                                path.remove(0); // remove the first step in the path
                            }
                            // Otherwise we wait a turn to see if the path clears up
                        } else {
                            mode.mode = Movement::RandomWaypoint { path: None };
                        }
                    } else {
                        let target_x = crate::tutorial::rng::roll_dice(1, map.width - 2);
                        let target_y = crate::tutorial::rng::roll_dice(1, map.height - 2);
                        let idx = map.xy_idx(target_x, target_y);
                        if tile_walkable(map.tiles[idx]) {
                            let path = rltk::a_star_search(
                                map.xy_idx(pos.x, pos.y) as i32,
                                map.xy_idx(target_x, target_y) as i32,
                                &*map,
                            );
                            if path.success && path.steps.len() > 1 {
                                mode.mode = Movement::RandomWaypoint {
                                    path: Some(path.steps),
                                }
                            }
                        }
                    }
                }
            }
        }

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
