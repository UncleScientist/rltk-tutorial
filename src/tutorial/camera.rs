use super::{Hidden, Position, Renderable};
use crate::{Map, TileType};
use rltk::{Point, Rltk, RGB};
use specs::prelude::*;

const SHOW_BOUNDARIES: bool = true;

pub fn get_screen_bounds(ecs: &World, ctx: &mut Rltk) -> (i32, i32, i32, i32) {
    let player_pos = ecs.fetch::<Point>();
    let (x_chars, y_chars) = ctx.get_char_size(); // window size, not char size

    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let min_x = player_pos.x - center_x;
    let max_x = min_x + x_chars as i32;
    let min_y = player_pos.y - center_y;
    let max_y = min_y + y_chars as i32;

    (min_x, max_x, min_y, max_y)
}

pub fn render_camera(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let (min_x, max_x, min_y, max_y) = get_screen_bounds(ecs, ctx);

    let map_width = map.width - 1;
    let map_height = map.height - 1;

    // (x, y) -> screen coords
    // (tx, ty) -> tile-in-map coords
    for (y, ty) in (min_y..max_y).enumerate() {
        for (x, tx) in (min_x..max_x).enumerate() {
            if tx > 0 && tx < map_width && ty > 0 && ty < map_height {
                let idx = map.xy_idx(tx, ty);
                if map.revealed_tiles[idx] {
                    let (glyph, fg, bg) = get_tile_glyph(idx, &*map);
                    ctx.set(x, y, fg, bg, glyph);
                }
            } else if SHOW_BOUNDARIES {
                ctx.set(
                    x,
                    y,
                    RGB::named(rltk::GRAY),
                    RGB::named(rltk::BLACK),
                    rltk::to_cp437('·'),
                );
            }
        }
    }

    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let hidden = ecs.read_storage::<Hidden>();

    let mut data = (&positions, &renderables, !&hidden)
        .join()
        .collect::<Vec<_>>();
    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
    for (pos, render, _hidden) in data.iter() {
        let idx = map.xy_idx(pos.x, pos.y);
        if map.visible_tiles[idx] {
            let entity_screen_x = pos.x - min_x;
            let entity_screen_y = pos.y - min_y;
            if entity_screen_x > 0
                && entity_screen_x < map_width
                && entity_screen_y > 0
                && entity_screen_y < map_height
            {
                ctx.set(
                    entity_screen_x,
                    entity_screen_y,
                    render.fg,
                    render.bg,
                    render.glyph,
                );
            }
        }
    }
}

pub fn render_debug_map(map: &Map, ctx: &mut Rltk) {
    let player_pos = Point::new(map.width / 2, map.height / 2);
    let (x_chars, y_chars) = ctx.get_char_size(); // get screen size

    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let min_x = player_pos.x - center_x;
    let max_x = min_x + x_chars as i32;
    let min_y = player_pos.y - center_y;
    let max_y = min_y + y_chars as i32;

    let map_width = map.width - 1;
    let map_height = map.height - 1;

    for (y, ty) in (min_y..max_y).enumerate() {
        for (x, tx) in (min_x..max_x).enumerate() {
            if tx > 0 && tx < map_width && ty > 0 && ty < map_height {
                let idx = map.xy_idx(tx, ty);
                if map.revealed_tiles[idx] {
                    let (glyph, fg, bg) = get_tile_glyph(idx, &*map);
                    ctx.set(x, y, fg, bg, glyph);
                }
            } else if SHOW_BOUNDARIES {
                ctx.set(
                    x,
                    y,
                    RGB::named(rltk::GRAY),
                    RGB::named(rltk::BLACK),
                    rltk::to_cp437('·'),
                );
            }
        }
    }
}

fn get_tile_glyph(idx: usize, map: &Map) -> (rltk::FontCharType, RGB, RGB) {
    let mut bg = RGB::from_f32(0., 0., 0.);

    let (glyph, mut fg) = match map.tiles[idx] {
        TileType::Floor => (rltk::to_cp437('.'), RGB::from_f32(0., 0.5, 0.5)),
        TileType::WoodFloor => (rltk::to_cp437('.'), RGB::named(rltk::CHOCOLATE)),
        TileType::DownStairs => (rltk::to_cp437('>'), RGB::from_f32(0., 1., 1.)),
        TileType::Wall => {
            let x = idx as i32 % map.width;
            let y = idx as i32 / map.width;

            (wall_glyph(&*map, x, y), RGB::from_f32(0., 1., 0.))
        }
        TileType::Bridge => (rltk::to_cp437('.'), RGB::named(rltk::CHOCOLATE)),
        TileType::Road => (rltk::to_cp437('!'), RGB::named(rltk::GREY)),
        TileType::Grass => (rltk::to_cp437('"'), RGB::named(rltk::GREEN)),
        TileType::ShallowWater => (rltk::to_cp437('≈'), RGB::named(rltk::CYAN)),
        TileType::DeepWater => (rltk::to_cp437('≈'), RGB::named(rltk::NAVY_BLUE)),
    };

    if map.bloodstains.contains(&idx) {
        bg = RGB::from_f32(0.75, 0., 0.);
    }
    if !map.visible_tiles[idx] {
        fg = fg.to_greyscale();
        bg = RGB::from_f32(0., 0., 0.);
    }

    (glyph, fg, bg)
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> rltk::FontCharType {
    if x < 1 || x > map.width - 2 || y < 1 || y > map.height - 2 {
        return 35;
    }

    let mut mask: u8 = 0;

    if is_revealed_and_wall(map, x, y - 1) {
        mask += 1
    };
    if is_revealed_and_wall(map, x, y + 1) {
        mask += 2
    };
    if is_revealed_and_wall(map, x - 1, y) {
        mask += 4
    };
    if is_revealed_and_wall(map, x + 1, y) {
        mask += 8
    };

    match mask {
        0 => 9,
        1 => 186,
        2 => 186,
        3 => 186,
        4 => 205,
        5 => 188,
        6 => 187,
        7 => 185,
        8 => 205,
        9 => 200,
        10 => 201,
        11 => 204,
        12 => 205,
        13 => 202,
        14 => 203,
        15 => 206,
        _ => 35,
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}
