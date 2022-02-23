use specs::prelude::*;

use crate::{
    AreaOfEffect, Confusion, Consumable, EquipmentChanged, GameLog, HungerClock, HungerState,
    IdentifiedItem, InflictsDamage, MagicMapper, Map, Name, ParticleBuilder, Pools, Position,
    ProvidesFood, ProvidesHealing, RunState, SufferDamage, TownPortal, WantsToUseItem,
};

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
