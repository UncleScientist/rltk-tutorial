use specs::prelude::*;

use crate::{EquipmentChanged, Equipped, GameLog, InBackpack, Name, WantsToRemoveItem};

pub struct ItemRemoveSystem;

type ItemRemoveData<'a> = (
    ReadExpect<'a, Entity>,
    WriteExpect<'a, GameLog>,
    Entities<'a>,
    WriteStorage<'a, WantsToRemoveItem>,
    WriteStorage<'a, Equipped>,
    WriteStorage<'a, InBackpack>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, EquipmentChanged>,
);

impl<'a> System<'a> for ItemRemoveSystem {
    type SystemData = ItemRemoveData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_remove,
            mut equipped,
            mut backpack,
            names,
            mut dirty,
        ) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            backpack
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("Unable to insert into backpack");
            dirty
                .insert(entity, EquipmentChanged {})
                .expect("Unable to insert");
            if entity == *player_entity {
                gamelog.entries.push(format!(
                    "You remove the {}.",
                    names.get(to_remove.item).unwrap().name
                ));
            }
        }

        wants_remove.clear();
    }
}
