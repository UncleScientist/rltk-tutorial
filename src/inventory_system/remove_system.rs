use specs::prelude::*;

use crate::{CursedItem, EquipmentChanged, Equipped, GameLog, InBackpack, Name, WantsToRemoveItem};

pub struct ItemRemoveSystem;

type ItemRemoveData<'a> = (
    WriteExpect<'a, GameLog>,
    Entities<'a>,
    WriteStorage<'a, WantsToRemoveItem>,
    WriteStorage<'a, Equipped>,
    WriteStorage<'a, InBackpack>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, CursedItem>,
    WriteStorage<'a, EquipmentChanged>,
);

impl<'a> System<'a> for ItemRemoveSystem {
    type SystemData = ItemRemoveData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut gamelog,
            entities,
            mut wants_remove,
            mut equipped,
            mut backpack,
            names,
            cursed,
            mut dirty,
        ) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            if cursed.get(to_remove.item).is_some() {
                gamelog.entries.push(format!(
                    "You cannot remove {}, it is cursed",
                    names.get(to_remove.item).unwrap().name
                ));
            } else {
                equipped.remove(to_remove.item);
                backpack
                    .insert(to_remove.item, InBackpack { owner: entity })
                    .expect("Unable to insert into backpack");
            }
            dirty
                .insert(entity, EquipmentChanged {})
                .expect("Unable to insert");
        }

        wants_remove.clear();
    }
}
