use specs::prelude::*;

use crate::{
    inventory_system::obfuscate_name, EquipmentChanged, GameLog, InBackpack, MagicItem,
    MasterDungeonMap, Name, ObfuscatedName, Position, WantsToPickupItem,
};

pub struct ItemCollectionSystem {}

type ItemCollectionData<'a> = (
    ReadExpect<'a, Entity>,
    WriteExpect<'a, GameLog>,
    WriteStorage<'a, WantsToPickupItem>,
    WriteStorage<'a, Position>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, InBackpack>,
    WriteStorage<'a, EquipmentChanged>,
    ReadStorage<'a, MagicItem>,
    ReadStorage<'a, ObfuscatedName>,
    ReadExpect<'a, MasterDungeonMap>,
);

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = ItemCollectionData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            mut wants_pickup,
            mut positions,
            names,
            mut backpack,
            mut dirty,
            magic_items,
            obfuscated_names,
            dm,
        ) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");
            dirty
                .insert(pickup.collected_by, EquipmentChanged {})
                .expect("Unable to insert");
            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!(
                    "You pick up the {}.",
                    obfuscate_name(pickup.item, &names, &magic_items, &obfuscated_names, &dm)
                ));
            }
        }

        wants_pickup.clear();
    }
}
