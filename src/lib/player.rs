use rltk::VirtualKeyCode;
use specs::prelude::*;
use std::cmp::{max, min};

use crate::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let players = ecs.read_storage::<Player>();
    let mut positions = ecs.write_storage::<Position>();
    let map = ecs.fetch::<Map>();

    for (_player, pos) in (&players, &mut positions).join() {
        let dest = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map.tiles[dest] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    if let Some(key) = ctx.key {
        use VirtualKeyCode::*;

        match key {
            Left | Numpad4 | H => try_move_player(-1, 0, &mut gs.ecs),
            Right | Numpad6 | L => try_move_player(1, 0, &mut gs.ecs),
            Up | Numpad8 | K => try_move_player(0, -1, &mut gs.ecs),
            Down | Numpad2 | J => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        };
    }
}