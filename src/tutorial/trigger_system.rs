use crate::{
    effects::*, gamelog::GameLog, ApplyTeleport, EntityMoved, EntryTrigger, Hidden, InflictsDamage,
    Map, Name, ParticleBuilder, Position, SingleActivation, TeleportTo,
};
use specs::prelude::*;

pub struct TriggerSystem;

type TriggerData<'a> = (
    ReadExpect<'a, Map>,
    WriteStorage<'a, EntityMoved>,
    ReadStorage<'a, Position>,
    ReadStorage<'a, EntryTrigger>,
    WriteStorage<'a, Hidden>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, InflictsDamage>,
    WriteExpect<'a, ParticleBuilder>,
    Entities<'a>,
    WriteExpect<'a, GameLog>,
    ReadStorage<'a, SingleActivation>,
    ReadStorage<'a, TeleportTo>,
    WriteStorage<'a, ApplyTeleport>,
    ReadExpect<'a, Entity>,
);

impl<'a> System<'a> for TriggerSystem {
    type SystemData = TriggerData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut entity_moved,
            position,
            entry_trigger,
            mut hidden,
            names,
            inflicts_damage,
            mut particle_builder,
            entities,
            mut log,
            single_activation,
            teleporters,
            mut apply_teleport,
            player_entity,
        ) = data;

        let mut remove_entities: Vec<Entity> = Vec::new();
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);

            crate::spatial::for_each_tile_content(idx, |entity_id| {
                if entity != entity_id && entry_trigger.get(entity_id).is_some() {
                    if let Some(name) = names.get(entity_id) {
                        log.entries.push(format!("{} triggers!", &name.name));
                    }

                    hidden.remove(entity_id);

                    if let Some(damage) = inflicts_damage.get(entity_id) {
                        particle_builder.request(
                            pos.x,
                            pos.y,
                            rltk::RGB::named(rltk::ORANGE),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                        add_effect(
                            None,
                            EffectType::Damage {
                                amount: damage.damage,
                            },
                            Targets::Single { target: entity },
                        );
                    }

                    if let Some(teleport) = teleporters.get(entity_id) {
                        if !teleport.player_only || entity == *player_entity {
                            apply_teleport
                                .insert(
                                    entity,
                                    ApplyTeleport {
                                        dest_x: teleport.x,
                                        dest_y: teleport.y,
                                        dest_depth: teleport.depth,
                                    },
                                )
                                .expect("Unable to insert");
                        }
                    }

                    if single_activation.get(entity_id).is_some() {
                        remove_entities.push(entity_id);
                    }
                }
            });
        }

        for trap in remove_entities.iter() {
            entities.delete(*trap).expect("Unable to delete trap");
        }

        entity_moved.clear();
    }
}
