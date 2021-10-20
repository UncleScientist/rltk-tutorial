use specs::prelude::*;

use crate::{
    CombatStats, GameLog, InBackpack, Name, Position, Potion, WantsToDropItem, WantsToPickupItem,
    WantsToUseItem,
};

pub struct ItemCollectionSystem {}

type ItemCollectionData<'a> = (
    ReadExpect<'a, Entity>,
    WriteExpect<'a, GameLog>,
    WriteStorage<'a, WantsToPickupItem>,
    WriteStorage<'a, Position>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, InBackpack>,
);

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = ItemCollectionData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

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
            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!(
                    "You pick up the {}.",
                    names.get(pickup.item).unwrap().name
                ));
            }
        }

        wants_pickup.clear();
    }
}

pub struct PotionUseSystem;

type PotionUseData<'a> = (
    ReadExpect<'a, Entity>,
    WriteExpect<'a, GameLog>,
    Entities<'a>,
    WriteStorage<'a, WantsToUseItem>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, Potion>,
    WriteStorage<'a, CombatStats>,
);

impl<'a> System<'a> for PotionUseSystem {
    type SystemData = PotionUseData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut use_items, names, potions, mut combat_stats) =
            data;

        for (entity, useitem, stats) in (&entities, &use_items, &mut combat_stats).join() {
            if let Some(potion) = potions.get(useitem.item) {
                stats.hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                if entity == *player_entity {
                    gamelog.entries.push(format!(
                        "You drink the {}, healing {} hp.",
                        names.get(useitem.item).unwrap().name,
                        potion.heal_amount
                    ));
                }
                entities.delete(useitem.item).expect("Delete failed");
            }
        }

        use_items.clear();
    }
}

pub struct ItemDropSystem;

type ItemDropData<'a> = (
    ReadExpect<'a, Entity>,
    WriteExpect<'a, GameLog>,
    Entities<'a>,
    WriteStorage<'a, WantsToDropItem>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, InBackpack>,
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

            if entity == *player_entity {
                gamelog.entries.push(format!(
                    "You drop the {}.",
                    names.get(to_drop.item).unwrap().name
                ));
            }
        }

        wants_drop.clear();
    }
}
