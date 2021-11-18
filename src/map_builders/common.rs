use super::{Map, Rect, TileType};
use std::cmp::{max, min};

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    let last = (map.width * map.height) as usize;

    for x in min(x1, x2)..=max(x1, x2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < last {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    let last = (map.width * map.height) as usize;

    for y in min(y1, y2)..=max(y1, y2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < last {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            let loc = map.xy_idx(x, y);
            map.tiles[loc] = TileType::Floor;
        }
    }
}
