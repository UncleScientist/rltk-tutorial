use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

use crate::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let players = ecs.read_storage::<Player>();
    let mut positions = ecs.write_storage::<Position>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, viewshed) in
        (&entities, &players, &mut positions, &mut viewsheds).join()
    {
        let dest = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[dest].iter() {
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
                return;
            }
        }

        if !map.blocked[dest] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
            viewshed.dirty = true;

            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
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

        match key {
            Left | Numpad4 | H => try_move_player(-1, 0, &mut gs.ecs),
            Right | Numpad6 | L => try_move_player(1, 0, &mut gs.ecs),
            Up | Numpad8 | K => try_move_player(0, -1, &mut gs.ecs),
            Down | Numpad2 | J => try_move_player(0, 1, &mut gs.ecs),
            Numpad9 | U => try_move_player(1, -1, &mut gs.ecs),
            Numpad7 | Y => try_move_player(-1, -1, &mut gs.ecs),
            Numpad3 | N => try_move_player(1, 1, &mut gs.ecs),
            Numpad1 | B => try_move_player(-1, 1, &mut gs.ecs),
            Numpad5 | Space => return skip_turn(&mut gs.ecs),
            Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            }
            Escape => return RunState::SaveGame,
            D => return RunState::ShowDropItem,
            G => get_item(&mut gs.ecs),
            I => return RunState::ShowInventory,
            R => return RunState::ShowRemoveItem,
            _ => {
                return RunState::AwaitingInput;
            }
        }
        RunState::PlayerTurn
    } else {
        RunState::AwaitingInput
    }
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();

    let worldmap_resource = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        for entity_id in worldmap_resource.tile_content[idx].iter() {
            if monsters.get(*entity_id).is_some() {
                can_heal = false;
                break;
            }
        }
    }

    let hunger_clocks = ecs.read_storage::<HungerClock>();
    if let Some(hc) = hunger_clocks.get(*player_entity) {
        can_heal =
            can_heal && (hc.state != HungerState::Hungry && hc.state != HungerState::Starving);
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<CombatStats>();
        let player_hp = health_components.get_mut(*player_entity).unwrap();
        player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
    }

    RunState::PlayerTurn
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
