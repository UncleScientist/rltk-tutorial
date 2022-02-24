use crate::{
    effects::*, gamelog::GameLog, AreaOfEffect, EntityMoved, EntryTrigger, Map, Name, Position,
};
use specs::prelude::*;

pub struct TriggerSystem;

type TriggerData<'a> = (
    ReadExpect<'a, Map>,
    WriteStorage<'a, EntityMoved>,
    ReadStorage<'a, Position>,
    ReadStorage<'a, EntryTrigger>,
    ReadStorage<'a, Name>,
    Entities<'a>,
    WriteExpect<'a, GameLog>,
    ReadStorage<'a, AreaOfEffect>,
);

impl<'a> System<'a> for TriggerSystem {
    type SystemData = TriggerData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut entity_moved,
            position,
            entry_trigger,
            names,
            entities,
            mut log,
            area_of_effect,
        ) = data;

        // Iterate the entities that moved and get their final position
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);

            crate::spatial::for_each_tile_content(idx, |entity_id| {
                if entity != entity_id && entry_trigger.get(entity_id).is_some() {
                    if let Some(name) = names.get(entity_id) {
                        log.entries.push(format!("{} triggers!", &name.name));
                    }

                    // Call the effects system
                    add_effect(
                        Some(entity),
                        EffectType::TriggerFire { trigger: entity_id },
                        if let Some(aoe) = area_of_effect.get(entity_id) {
                            Targets::Tiles {
                                tiles: aoe_tiles(&*map, rltk::Point::new(pos.x, pos.y), aoe.radius),
                            }
                        } else {
                            Targets::Tile {
                                tile_idx: idx as i32,
                            }
                        },
                    );
                }
            });
        }

        // Remove all entity movement markers
        entity_moved.clear();
    }
}
