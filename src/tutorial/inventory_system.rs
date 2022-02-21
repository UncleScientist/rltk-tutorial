use specs::prelude::*;

use crate::{
    AreaOfEffect, Confusion, Consumable, EquipmentChanged, Equippable, Equipped, GameLog,
    HungerClock, HungerState, IdentifiedItem, InBackpack, InflictsDamage, Item, MagicMapper, Map,
    MasterDungeonMap, Name, ObfuscatedName, ParticleBuilder, Player, Pools, Position, ProvidesFood,
    ProvidesHealing, RunState, SufferDamage, TownPortal, WantsToDropItem, WantsToPickupItem,
    WantsToRemoveItem, WantsToUseItem,
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
                    names.get(pickup.item).unwrap().name
                ));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem;

type ItemUseData<'a> = (
    ReadExpect<'a, Entity>,
    WriteExpect<'a, GameLog>,
    Entities<'a>,
    WriteStorage<'a, WantsToUseItem>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, ProvidesHealing>,
    WriteStorage<'a, Pools>,
    ReadStorage<'a, Consumable>,
    ReadStorage<'a, InflictsDamage>,
    ReadStorage<'a, AreaOfEffect>,
    WriteStorage<'a, SufferDamage>,
    WriteStorage<'a, Confusion>,
    ReadExpect<'a, Map>,
    ReadStorage<'a, Equippable>,
    WriteStorage<'a, Equipped>,
    WriteStorage<'a, InBackpack>,
    WriteExpect<'a, ParticleBuilder>,
    ReadStorage<'a, Position>,
    ReadStorage<'a, ProvidesFood>,
    WriteStorage<'a, HungerClock>,
    ReadStorage<'a, MagicMapper>,
    ReadStorage<'a, TownPortal>,
    WriteExpect<'a, RunState>,
    WriteStorage<'a, EquipmentChanged>,
    WriteStorage<'a, IdentifiedItem>,
);

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ItemUseData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut use_items,
            names,
            healing,
            mut combat_stats,
            consumables,
            inflict_damage,
            aoe,
            mut suffer_damage,
            mut confused,
            map,
            equippable,
            mut equipped,
            mut backpack,
            mut particle_builder,
            positions,
            provides_food,
            mut hunger_clocks,
            magic_mapper,
            town_portal,
            mut runstate,
            mut dirty,
            mut identified_item,
        ) = data;

        for (entity, useitem) in (&entities, &use_items).join() {
            let mut consume_item = true;

            dirty
                .insert(entity, EquipmentChanged {})
                .expect("Unable to insert marker");
            let mut targets: Vec<Entity> = Vec::new();
            if let Some(target) = useitem.target {
                if let Some(area_effect) = aoe.get(useitem.item) {
                    let mut blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
                    blast_tiles.retain(|p| {
                        p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
                    });
                    for tile_xy in blast_tiles.iter() {
                        let idx = map.xy_idx(tile_xy.x, tile_xy.y);
                        crate::spatial::for_each_tile_content(idx, |mob| targets.push(mob));
                        particle_builder.request(
                            tile_xy.x,
                            tile_xy.y,
                            rltk::RGB::named(rltk::ORANGE),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('░'),
                            200.0,
                        );
                    }
                } else {
                    let idx = map.xy_idx(target.x, target.y);
                    crate::spatial::for_each_tile_content(idx, |mob| targets.push(mob));
                }
            } else {
                targets.push(*player_entity);
            }

            // Identify
            if entity == *player_entity {
                identified_item
                    .insert(
                        entity,
                        IdentifiedItem {
                            name: names.get(useitem.item).unwrap().name.clone(),
                        },
                    )
                    .expect("Unable to insert");
            }

            if let Some(can_equip) = equippable.get(useitem.item) {
                let target_slot = can_equip.slot;
                let target = targets[0];

                // Remove any items the target has in the item's slot
                let mut to_unequip: Vec<Entity> = Vec::new();
                for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                    if already_equipped.owner == target && already_equipped.slot == target_slot {
                        to_unequip.push(item_entity);
                        if target == *player_entity {
                            gamelog.entries.push(format!("You unequip {}.", name.name));
                        }
                    }
                }
                for item in to_unequip.iter() {
                    equipped.remove(*item);
                    backpack
                        .insert(*item, InBackpack { owner: target })
                        .expect("Unable to insert backpack entry");
                }

                // Wield the item
                equipped
                    .insert(
                        useitem.item,
                        Equipped {
                            owner: target,
                            slot: target_slot,
                        },
                    )
                    .expect("Unable to insert equipped component");
                backpack.remove(useitem.item);
                if target == *player_entity {
                    gamelog.entries.push(format!(
                        "You equip {}.",
                        names.get(useitem.item).unwrap().name
                    ));
                }
            }

            if let Some(healer) = healing.get(useitem.item) {
                for target in targets.iter() {
                    if let Some(stats) = combat_stats.get_mut(*target) {
                        stats.hit_points.current = i32::min(
                            stats.hit_points.max,
                            stats.hit_points.current + healer.heal_amount,
                        );
                        if entity == *player_entity {
                            gamelog.entries.push(format!(
                                "You drink the {}, healing {} hp.",
                                names.get(useitem.item).unwrap().name,
                                healer.heal_amount
                            ));
                        }
                        if let Some(pos) = positions.get(*target) {
                            particle_builder.request(
                                pos.x,
                                pos.y,
                                rltk::RGB::named(rltk::GREEN),
                                rltk::RGB::named(rltk::BLACK),
                                rltk::to_cp437('♥'),
                                200.0,
                            );
                        }
                    }
                }
            }

            if magic_mapper.get(useitem.item).is_some() {
                gamelog
                    .entries
                    .push("The map is revealed to you!".to_string());
                *runstate = RunState::MagicMapReveal { row: 0 };
            }

            if town_portal.get(useitem.item).is_some() {
                if map.depth == 1 {
                    gamelog
                        .entries
                        .push("You are already in town, so the scroll does nothing".to_string());
                    consume_item = false;
                } else {
                    gamelog
                        .entries
                        .push("You are teleported back to town!".to_string());
                    *runstate = RunState::TownPortal;
                }
            }

            if provides_food.get(useitem.item).is_some() {
                let target = targets[0];
                if let Some(hc) = hunger_clocks.get_mut(target) {
                    hc.state = HungerState::WellFed;
                    hc.duration = 20;
                    gamelog.entries.push(format!(
                        "You eat the {}.",
                        names.get(useitem.item).unwrap().name
                    ));
                }
            }

            if let Some(damage) = inflict_damage.get(useitem.item) {
                for mob in targets.iter() {
                    let item_name = names.get(useitem.item).unwrap();
                    let mob_name = names.get(*mob).unwrap();
                    if combat_stats.get(*mob).is_some() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage, true);
                        if entity == *player_entity {
                            gamelog.entries.push(format!(
                                "You use {} on {}, inflicting {} hp.",
                                item_name.name, mob_name.name, damage.damage
                            ));
                        }
                    } else {
                        gamelog.entries.push(format!(
                            "You use {} but it fizzles against {}",
                            item_name.name, mob_name.name
                        ));
                    }
                    if let Some(pos) = positions.get(*mob) {
                        particle_builder.request(
                            pos.x,
                            pos.y,
                            rltk::RGB::named(rltk::RED),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                    }
                }
            }

            let mut add_confusion = Vec::new();
            if let Some(confusion) = confused.get(useitem.item) {
                for mob in targets.iter() {
                    add_confusion.push((*mob, confusion.turns));
                    if entity == *player_entity {
                        let mob_name = names.get(*mob).unwrap();
                        let item_name = names.get(useitem.item).unwrap();
                        gamelog.entries.push(format!(
                            "You use {} on {}, confusing them",
                            item_name.name, mob_name.name
                        ));
                    }
                    if let Some(pos) = positions.get(*mob) {
                        particle_builder.request(
                            pos.x,
                            pos.y,
                            rltk::RGB::named(rltk::MAGENTA),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('?'),
                            200.0,
                        );
                    }
                }
            }
            for mob in add_confusion.iter() {
                confused
                    .insert(mob.0, Confusion { turns: mob.1 })
                    .expect("Unable to insert status");
            }

            if consume_item && consumables.get(useitem.item).is_some() {
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
    WriteStorage<'a, EquipmentChanged>,
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
                    names.get(to_drop.item).unwrap().name
                ));
            }
        }

        wants_drop.clear();
    }
}

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

pub struct ItemIdentificationSystem {}

type ItemIdentificationData<'a> = (
    ReadStorage<'a, Player>,
    WriteStorage<'a, IdentifiedItem>,
    WriteExpect<'a, MasterDungeonMap>,
    ReadStorage<'a, Item>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, ObfuscatedName>,
    Entities<'a>,
);

impl<'a> System<'a> for ItemIdentificationSystem {
    type SystemData = ItemIdentificationData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (player, mut identified, mut dm, items, names, mut obfuscated_names, entities) = data;

        for (_p, id) in (&player, &identified).join() {
            if !dm.identified_items.contains(&id.name) && crate::raws::is_tag_magic(&id.name) {
                dm.identified_items.insert(id.name.clone());

                for (entity, _item, name) in (&entities, &items, &names).join() {
                    if name.name == id.name {
                        obfuscated_names.remove(entity);
                    }
                }
            }
        }

        // Clean up
        identified.clear();
    }
}
