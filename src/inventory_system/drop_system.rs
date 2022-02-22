use specs::prelude::*;

use crate::{
    EquipmentChanged, GameLog, InBackpack, MagicItem, MasterDungeonMap, Name, ObfuscatedName,
    Position, WantsToDropItem,
};

pub struct ItemDropSystem;

type ItemDropData<'a> = (
    ReadExpect<'a, Entity>,
    WriteExpect<'a, GameLog>,
    Entities<'a>,
    WriteStorage<'a, WantsToDropItem>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, InBackpack>,
    WriteStorage<'a, EquipmentChanged>,
    ReadStorage<'a, MagicItem>,
    ReadStorage<'a, ObfuscatedName>,
    ReadExpect<'a, MasterDungeonMap>,
);

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = ItemDropData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack,
            mut dirty,
            magic_items,
            obfuscated_names,
            dm,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let dropper_pos = {
                let dropped_pos = positions.get(entity).unwrap();
                Position {
                    x: dropped_pos.x,
                    y: dropped_pos.y,
                }
            };

            positions
                .insert(to_drop.item, dropper_pos)
                .expect("Unable to insert position");
            backpack.remove(to_drop.item);
            dirty
                .insert(entity, EquipmentChanged {})
                .expect("Unable to insert");

            if entity == *player_entity {
                gamelog.entries.push(format!(
                    "You drop the {}.",
                    super::obfuscate_name(
                        to_drop.item,
                        &names,
                        &magic_items,
                        &obfuscated_names,
                        &dm
                    )
                ));
            }
        }

        wants_drop.clear();
    }
}
