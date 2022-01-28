use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

use crate::components::Ranged;
use crate::*;

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<Pools>();
    let map = ecs.fetch::<Map>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut blocks_movement = ecs.write_storage::<BlocksTile>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let factions = ecs.read_storage::<Faction>();

    let mut result = RunState::AwaitingInput;
    let mut swap_entities: Vec<(Entity, i32, i32)> = Vec::new();

    for (entity, _player, pos, viewshed) in
        (&entities, &players, &mut positions, &mut viewsheds).join()
    {
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 1
            || pos.y + delta_y > map.height - 1
        {
            return result;
        }
        let dest = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[dest].iter() {
            let mut hostile = true;
            if combat_stats.get(*potential_target).is_some() {
                if let Some(faction) = factions.get(*potential_target) {
                    let reaction = crate::raws::faction_reaction(
                        &faction.name,
                        "Player",
                        &crate::raws::RAWS.lock().unwrap(),
                    );
                    if reaction != Reaction::Attack {
                        hostile = false;
                    }
                }
            }
            if !hostile {
                // Note that we want to move the bystander
                swap_entities.push((*potential_target, pos.x, pos.y));

                pos.x = min(map.width - 1, max(0, pos.x + delta_x));
                pos.y = min(map.height - 1, max(0, pos.y + delta_y));
                entity_moved
                    .insert(entity, EntityMoved {})
                    .expect("Unable to insert marker");

                viewshed.dirty = true;
                let mut ppos = ecs.write_resource::<Point>();
                ppos.x = pos.x;
                ppos.y = pos.y;
            } else {
                let target = combat_stats.get(*potential_target);

                if target.is_some() {
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *potential_target,
                            },
                        )
                        .expect("Add target failed");
                    return RunState::Ticking;
                }
            }

            if let Some(door) = doors.get_mut(*potential_target) {
                door.open = true;
                blocks_visibility.remove(*potential_target);
                blocks_movement.remove(*potential_target);
                let glyph = renderables.get_mut(*potential_target).unwrap();
                glyph.glyph = rltk::to_cp437('/');
                viewshed.dirty = true;
                result = RunState::Ticking;
            }
        }

        if !map.blocked[dest] {
            pos.x = min(map.width - 1, max(0, pos.x + delta_x));
            pos.y = min(map.height - 1, max(0, pos.y + delta_y));
            entity_moved
                .insert(entity, EntityMoved {})
                .expect("Unable to insert marker");

            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
            result = match map.tiles[dest] {
                TileType::DownStairs => RunState::NextLevel,
                TileType::UpStairs => RunState::PreviousLevel,
                _ => RunState::Ticking,
            };
        }
    }

    for m in swap_entities.iter() {
        if let Some(their_pos) = positions.get_mut(m.0) {
            their_pos.x = m.1;
            their_pos.y = m.2;
        }
    }

    result
}

fn try_previous_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::UpStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog
            .entries
            .push("There is no way up from here".to_string());
        false
    }
}

fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog
            .entries
            .push("There is no way down from here".to_string());
        false
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    if let Some(key) = ctx.key {
        use VirtualKeyCode::*;

        if ctx.shift {
            let key: Option<i32> = match key {
                Key1 => Some(1),
                Key2 => Some(2),
                Key3 => Some(3),
                Key4 => Some(4),
                Key5 => Some(5),
                Key6 => Some(6),
                Key7 => Some(7),
                Key8 => Some(8),
                Key9 => Some(9),
                _ => None,
            };
            if let Some(key) = key {
                return use_consumable_hotkey(gs, key - 1);
            }
        }

        match key {
            Left | Numpad4 | H => try_move_player(-1, 0, &mut gs.ecs),
            Right | Numpad6 | L => try_move_player(1, 0, &mut gs.ecs),
            Up | Numpad8 | K => try_move_player(0, -1, &mut gs.ecs),
            Down | Numpad2 | J => try_move_player(0, 1, &mut gs.ecs),
            Numpad9 | U => try_move_player(1, -1, &mut gs.ecs),
            Numpad7 | Y => try_move_player(-1, -1, &mut gs.ecs),
            Numpad3 | N => try_move_player(1, 1, &mut gs.ecs),
            Numpad1 | B => try_move_player(-1, 1, &mut gs.ecs),
            Numpad5 | Space => skip_turn(&mut gs.ecs),
            Period => {
                if try_next_level(&mut gs.ecs) {
                    RunState::NextLevel
                } else {
                    RunState::Ticking
                }
            }
            Comma => {
                if try_previous_level(&mut gs.ecs) {
                    RunState::PreviousLevel
                } else {
                    RunState::Ticking
                }
            }
            Escape => RunState::SaveGame,
            Backslash => RunState::ShowCheatMenu,
            D => RunState::ShowDropItem,
            G => {
                get_item(&mut gs.ecs);
                RunState::Ticking
            }
            I => RunState::ShowInventory,
            R => RunState::ShowRemoveItem,
            _ => RunState::AwaitingInput,
        }
    } else {
        RunState::AwaitingInput
    }
}

fn use_consumable_hotkey(gs: &mut State, key: i32) -> RunState {
    let consumables = gs.ecs.read_storage::<Consumable>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let player_entity = gs.ecs.fetch::<Entity>();
    let entities = gs.ecs.entities();

    let mut carried_consumables = Vec::new();
    for (entity, carried_by, _) in (&entities, &backpack, &consumables).join() {
        if carried_by.owner == *player_entity {
            carried_consumables.push(entity);
        }
    }

    if (key as usize) < carried_consumables.len() {
        if let Some(ranged) = gs
            .ecs
            .read_storage::<Ranged>()
            .get(carried_consumables[key as usize])
        {
            return RunState::ShowTargeting {
                range: ranged.range,
                item: carried_consumables[key as usize],
            };
        }

        let mut intent = gs.ecs.write_storage::<WantsToUseItem>();
        intent
            .insert(
                *player_entity,
                WantsToUseItem {
                    item: carried_consumables[key as usize],
                    target: None,
                },
            )
            .expect("Unable to insert intent");
        return RunState::Ticking;
    }

    RunState::Ticking
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let factions = ecs.read_storage::<Faction>();

    let worldmap_resource = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    'outer: for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        for entity_id in worldmap_resource.tile_content[idx].iter() {
            if let Some(faction) = factions.get(*entity_id) {
                let reaction = crate::raws::faction_reaction(
                    &faction.name,
                    "Player",
                    &crate::raws::RAWS.lock().unwrap(),
                );
                if reaction == Reaction::Attack {
                    can_heal = false;
                    break 'outer;
                }
            }
        }
    }

    let hunger_clocks = ecs.read_storage::<HungerClock>();
    if let Some(hc) = hunger_clocks.get(*player_entity) {
        can_heal =
            can_heal && (hc.state != HungerState::Hungry && hc.state != HungerState::Starving);
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<Pools>();
        let pools = health_components.get_mut(*player_entity).unwrap();
        pools.hit_points.current = i32::min(pools.hit_points.current + 1, pools.hit_points.max);
    }

    RunState::Ticking
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
            break;
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push("There is nothing here to pick up".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pickup");
        }
    }
}
