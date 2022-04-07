use crate::{
    effects::*,
    raws::{get_item_drop, spawn_named_entity, SpawnType, RAWS},
    AreaOfEffect, Equipped, InBackpack, LootTable, Map, Name, OnDeath, Player, Pools, Position,
    RunState,
};
use specs::prelude::*;

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();

    {
        let combat_stats = ecs.read_storage::<Pools>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();

        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hit_points.current < 1 {
                match players.get(entity) {
                    None => {
                        if let Some(victim_name) = names.get(entity) {
                            crate::gamelog::Logger::new()
                                .npc_name(&victim_name.name)
                                .color(rltk::WHITE)
                                .append("is dead")
                                .log();
                        }
                        dead.push(entity);
                    }
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;
                    }
                }
            }
        }
    }

    // Drop everything held by dead people
    let mut to_spawn = Vec::new();
    {
        let mut to_drop: Vec<(Entity, Position)> = Vec::new();
        let entities = ecs.entities();
        let mut equipped = ecs.write_storage::<Equipped>();
        let mut carried = ecs.write_storage::<InBackpack>();
        let mut positions = ecs.write_storage::<Position>();
        let loot_tables = ecs.read_storage::<LootTable>();

        for victim in dead.iter() {
            let pos = positions.get(*victim);

            for (entity, equipped) in (&entities, &equipped).join() {
                if equipped.owner == *victim {
                    // Drop their stuff
                    if let Some(pos) = pos {
                        to_drop.push((entity, pos.clone()));
                    }
                }
            }

            for (entity, backpack) in (&entities, &carried).join() {
                if backpack.owner == *victim {
                    // Drop their stuff
                    if let Some(pos) = pos {
                        to_drop.push((entity, pos.clone()));
                    }
                }
            }

            if let Some(table) = loot_tables.get(*victim) {
                if let Some(tag) = get_item_drop(&RAWS.lock().unwrap(), &table.table) {
                    if let Some(pos) = pos {
                        to_spawn.push((tag, pos.clone()));
                    }
                }
            }
        }

        for drop in to_drop.iter() {
            equipped.remove(drop.0);
            carried.remove(drop.0);
            positions
                .insert(drop.0, drop.1.clone())
                .expect("Unable to insert position");
        }
    }

    {
        for drop in to_spawn.iter() {
            spawn_named_entity(
                &RAWS.lock().unwrap(),
                ecs,
                &drop.0,
                SpawnType::AtPosition {
                    x: drop.1.x,
                    y: drop.1.y,
                },
            );
        }
    }

    for victim in dead.iter() {
        let death_effects = ecs.read_storage::<OnDeath>();
        if let Some(death_effect) = death_effects.get(*victim) {
            for effect in death_effect.abilities.iter() {
                if crate::tutorial::rng::roll_dice(1, 100) <= (effect.chance * 100.0) as i32 {
                    let map = ecs.fetch::<Map>();
                    if let Some(pos) = ecs.read_storage::<Position>().get(*victim) {
                        let spell_entity =
                            crate::raws::find_spell_entity(ecs, &effect.spell).unwrap();
                        let tile_idx = map.xy_idx(pos.x, pos.y);
                        let target = if let Some(aoe) =
                            ecs.read_storage::<AreaOfEffect>().get(spell_entity)
                        {
                            Targets::Tiles {
                                tiles: aoe_tiles(&map, rltk::Point::new(pos.x, pos.y), aoe.radius),
                            }
                        } else {
                            Targets::Tile {
                                tile_idx: tile_idx as i32,
                            }
                        };
                        add_effect(
                            None,
                            EffectType::SpellUse {
                                spell: spell_entity,
                            },
                            target,
                        );
                    }
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
