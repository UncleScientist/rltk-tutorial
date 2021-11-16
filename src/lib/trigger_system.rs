use super::{
    gamelog::GameLog, EntityMoved, EntryTrigger, Hidden, InflictsDamage, Map, Name,
    ParticleBuilder, Position, SufferDamage,
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
    WriteStorage<'a, SufferDamage>,
    WriteExpect<'a, ParticleBuilder>,
    Entities<'a>,
    WriteExpect<'a, GameLog>,
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
            mut inflict_damage,
            mut particle_builder,
            entities,
            mut log,
        ) = data;

        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);

            for entity_id in map.tile_content[idx].iter() {
                if entity != *entity_id && entry_trigger.get(*entity_id).is_some() {
                    if let Some(name) = names.get(*entity_id) {
                        log.entries.push(format!("{} triggers!", &name.name));
                    }
                    if let Some(damage) = inflicts_damage.get(*entity_id) {
                        particle_builder.request(
                            pos.x,
                            pos.y,
                            rltk::RGB::named(rltk::ORANGE),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                        SufferDamage::new_damage(&mut inflict_damage, entity, damage.damage);
                    }
                    hidden.remove(*entity_id);
                }
            }
        }

        entity_moved.clear();
    }
}
