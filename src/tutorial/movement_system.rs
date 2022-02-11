use crate::{
    ApplyMove, ApplyTeleport, BlocksTile, EntityMoved, Map, OtherLevelPosition, Position, Viewshed,
};
use specs::prelude::*;

pub struct MovementSystem {}

type MovementData<'a> = (
    WriteExpect<'a, Map>,
    WriteStorage<'a, Position>,
    ReadStorage<'a, BlocksTile>,
    Entities<'a>,
    WriteStorage<'a, ApplyMove>,
    WriteStorage<'a, ApplyTeleport>,
    WriteStorage<'a, OtherLevelPosition>,
    WriteStorage<'a, EntityMoved>,
    WriteStorage<'a, Viewshed>,
    ReadExpect<'a, Entity>,
);

impl<'a> System<'a> for MovementSystem {
    type SystemData = MovementData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            mut position,
            blockers,
            entities,
            mut apply_move,
            mut apply_teleport,
            mut other_level,
            mut moved,
            mut viewsheds,
            player_entity,
        ) = data;

        // Apply teleports
        for (entity, teleport) in (&entities, &apply_teleport).join() {
            if teleport.dest_depth == map.depth {
                apply_move
                    .insert(
                        entity,
                        ApplyMove {
                            dest_idx: map.xy_idx(teleport.dest_x, teleport.dest_y),
                        },
                    )
                    .expect("Unable to insert");
            } else if entity == *player_entity {
                rltk::console::log(format!("Not implemented yet"));
            } else if let Some(pos) = position.get(entity) {
                let idx = map.xy_idx(pos.x, pos.y);
                let dest_idx = map.xy_idx(teleport.dest_x, teleport.dest_y);
                crate::spatial::move_entity(entity, idx, dest_idx);
                other_level
                    .insert(
                        entity,
                        OtherLevelPosition {
                            x: teleport.dest_x,
                            y: teleport.dest_y,
                            depth: teleport.dest_depth,
                        },
                    )
                    .expect("Unable to insert");
                position.remove(entity);
            }
        }
        apply_teleport.clear();

        // Apply broad movement
        for (entity, movement, mut pos) in (&entities, &apply_move, &mut position).join() {
            let start_idx = map.xy_idx(pos.x, pos.y);
            let dest_idx = movement.dest_idx as usize;
            crate::spatial::move_entity(entity, start_idx, dest_idx);
            pos.x = movement.dest_idx as i32 % map.width;
            pos.y = movement.dest_idx as i32 / map.width;
            if let Some(vs) = viewsheds.get_mut(entity) {
                vs.dirty = true;
            }
            moved
                .insert(entity, EntityMoved {})
                .expect("Unable to insert");
        }
        apply_move.clear();
    }
}
