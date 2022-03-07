use crate::{
    attr_bonus, gamelog::GameLog, AttributeBonus, Attributes, EquipmentChanged, Equipped,
    InBackpack, Item, Pools,
};
use specs::prelude::*;
use std::collections::HashMap;

pub struct EncumbranceSystem {}

type EncumbranceData<'a> = (
    WriteStorage<'a, EquipmentChanged>,
    Entities<'a>,
    ReadStorage<'a, Item>,
    ReadStorage<'a, InBackpack>,
    ReadStorage<'a, Equipped>,
    WriteStorage<'a, Pools>,
    WriteStorage<'a, Attributes>,
    ReadExpect<'a, Entity>,
    WriteExpect<'a, GameLog>,
    ReadStorage<'a, AttributeBonus>,
);

impl<'a> System<'a> for EncumbranceSystem {
    type SystemData = EncumbranceData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut equip_dirty,
            entities,
            items,
            backpacks,
            wielded,
            mut pools,
            mut attributes,
            player,
            mut gamelog,
            attrbonus,
        ) = data;

        if equip_dirty.is_empty() {
            return;
        }

        #[derive(Default)]
        struct ItemUpdate {
            weight: f32,
            initiative: f32,
            might: i32,
            fitness: i32,
            quickness: i32,
            intelligence: i32,
        }

        // Build the  map of who needs updating
        let mut to_update = HashMap::new(); // weight, inititaive
        for (entity, _dirty) in (&entities, &equip_dirty).join() {
            to_update.insert(entity, ItemUpdate::default());
        }

        // Remove all dirty statements
        equip_dirty.clear();

        for (item, equipped, entity) in (&items, &wielded, &entities).join() {
            if to_update.contains_key(&equipped.owner) {
                let totals = to_update.get_mut(&equipped.owner).unwrap();
                totals.weight += item.weight_lbs;
                totals.initiative += item.initiative_penalty;
                if let Some(attr) = attrbonus.get(entity) {
                    totals.might += attr.might.unwrap_or(0);
                    totals.fitness += attr.fitness.unwrap_or(0);
                    totals.quickness += attr.quickness.unwrap_or(0);
                    totals.intelligence += attr.intelligence.unwrap_or(0);
                }
            }
        }
        for (item, carried) in (&items, &backpacks).join() {
            if to_update.contains_key(&carried.owner) {
                let totals = to_update.get_mut(&carried.owner).unwrap();
                totals.weight += item.weight_lbs;
                totals.initiative += item.initiative_penalty;
            }
        }

        // Apply data to the pools
        for (entity, item) in to_update.iter() {
            if let Some(pool) = pools.get_mut(*entity) {
                pool.total_weight = item.weight;
                pool.total_initiative_penalty = item.initiative;

                if let Some(attr) = attributes.get_mut(*entity) {
                    attr.might.modifiers = item.might;
                    attr.fitness.modifiers = item.fitness;
                    attr.quickness.modifiers = item.quickness;
                    attr.intelligence.modifiers = item.intelligence;
                    attr.might.bonus = attr_bonus(attr.might.base + attr.might.modifiers);
                    attr.fitness.bonus = attr_bonus(attr.fitness.base + attr.fitness.modifiers);
                    attr.quickness.bonus =
                        attr_bonus(attr.quickness.base + attr.quickness.modifiers);
                    attr.intelligence.bonus =
                        attr_bonus(attr.intelligence.base + attr.intelligence.modifiers);
                    let carry_capacity_lbs = (attr.might.base + attr.might.modifiers) * 15;
                    if pool.total_weight as i32 > carry_capacity_lbs {
                        // Overburdened
                        pool.total_initiative_penalty += 4.0;
                        if *entity == *player {
                            gamelog.entries.push(
                                "You are overburdened, and suffering an initiative penalty"
                                    .to_string(),
                            );
                        }
                    }
                }
            }
        }
    }
}
