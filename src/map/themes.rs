use super::{Map, RenderTile, TileType};
use rltk::{FontCharType, RGB};

pub fn tile_glyph(idx: usize, map: &Map) -> RenderTile {
    let RenderTile(glyph, mut fg, mut bg) = match map.depth {
        5 => {
            let x = idx as i32 % map.width;
            if x < map.width / 2 {
                get_limestone_cavern_glyph(idx, map)
            } else {
                get_tile_glyph_default(idx, map)
            }
        }
        3 | 4 => get_limestone_cavern_glyph(idx, map),
        2 => get_forest_glyph(idx, map),
        _ => get_tile_glyph_default(idx, map),
    };

    if map.bloodstains.contains(&idx) {
        bg = RGB::from_f32(0.75, 0., 0.);
    }
    if !map.visible_tiles[idx] {
        fg = fg.to_greyscale();
        bg = RGB::from_f32(0., 0., 0.);
    } else if !map.outdoors {
        fg = fg * map.light[idx];
        bg = bg * map.light[idx];
    }

    RenderTile(glyph, fg, bg)
}

fn get_tile_glyph_default(idx: usize, map: &Map) -> RenderTile {
    let bg = RGB::from_f32(0., 0., 0.);

    let (glyph, fg) = match map.tiles[idx] {
        TileType::Floor => (rltk::to_cp437('.'), RGB::from_f32(0., 0.5, 0.5)),
        TileType::WoodFloor => (rltk::to_cp437('.'), RGB::named(rltk::CHOCOLATE)),
        TileType::DownStairs => (rltk::to_cp437('>'), RGB::from_f32(0., 1., 1.)),
        TileType::UpStairs => (rltk::to_cp437('<'), RGB::from_f32(0., 1., 1.)),
        TileType::Wall => {
            let x = idx as i32 % map.width;
            let y = idx as i32 / map.width;

            (wall_glyph(&*map, x, y), RGB::from_f32(0., 1., 0.))
        }
        TileType::Bridge => (rltk::to_cp437('.'), RGB::named(rltk::CHOCOLATE)),
        TileType::Road => (rltk::to_cp437('≡'), RGB::named(rltk::GREY)),
        TileType::Grass => (rltk::to_cp437('"'), RGB::named(rltk::GREEN)),
        TileType::ShallowWater => (rltk::to_cp437('~'), RGB::named(rltk::CYAN)),
        TileType::DeepWater => (rltk::to_cp437('~'), RGB::named(rltk::NAVY_BLUE)),
        TileType::Gravel => (rltk::to_cp437(':'), RGB::named(rltk::GREY)),
        TileType::Stalagmite => (rltk::to_cp437('╨'), RGB::from_f32(0.5, 0.5, 0.5)),
        TileType::Stalactite => (rltk::to_cp437('╥'), RGB::from_f32(0.5, 0.5, 0.5)),
    };

    RenderTile(glyph, fg, bg)
}

fn get_forest_glyph(idx: usize, map: &Map) -> RenderTile {
    let bg = RGB::from_f32(0., 0., 0.);

    let (glyph, fg) = match map.tiles[idx] {
        TileType::Wall => (rltk::to_cp437('♣'), RGB::from_f32(0., 0.6, 0.)),
        TileType::Bridge => (rltk::to_cp437('.'), RGB::named(rltk::CHOCOLATE)),
        TileType::Road => (rltk::to_cp437('≡'), RGB::named(rltk::YELLOW)),
        TileType::Grass => (rltk::to_cp437('"'), RGB::named(rltk::GREEN)),
        TileType::ShallowWater => (rltk::to_cp437('~'), RGB::named(rltk::CYAN)),
        TileType::DeepWater => (rltk::to_cp437('~'), RGB::named(rltk::BLUE)),
        TileType::Gravel => (rltk::to_cp437(';'), RGB::named(rltk::GREY)),
        TileType::DownStairs => (rltk::to_cp437('>'), RGB::from_f32(0., 1., 1.)),
        TileType::UpStairs => (rltk::to_cp437('<'), RGB::from_f32(0., 1., 1.)),
        _ => (rltk::to_cp437('"'), RGB::from_f32(0., 0.6, 0.0)),
    };

    RenderTile(glyph, fg, bg)
}

fn get_limestone_cavern_glyph(idx: usize, map: &Map) -> RenderTile {
    let bg = RGB::from_f32(0., 0., 0.);

    let (glyph, fg) = match map.tiles[idx] {
        TileType::Wall => (rltk::to_cp437('▒'), RGB::from_f32(0.7, 0.7, 0.7)),
        TileType::Bridge => (rltk::to_cp437('.'), RGB::named(rltk::CHOCOLATE)),
        TileType::Road => (rltk::to_cp437('≡'), RGB::named(rltk::YELLOW)),
        TileType::Grass => (rltk::to_cp437('"'), RGB::named(rltk::GREEN)),
        TileType::ShallowWater => (rltk::to_cp437('░'), RGB::named(rltk::CYAN)),
        TileType::DeepWater => (rltk::to_cp437('▓'), RGB::from_f32(0.2, 0.2, 1.0)),
        TileType::Gravel => (rltk::to_cp437(';'), RGB::named(rltk::GREY)),
        TileType::DownStairs => (rltk::to_cp437('>'), RGB::from_f32(0., 1., 1.)),
        TileType::UpStairs => (rltk::to_cp437('<'), RGB::from_f32(0., 1., 1.)),
        TileType::Stalagmite => (rltk::to_cp437('╨'), RGB::from_f32(0.5, 0.5, 0.5)),
        TileType::Stalactite => (rltk::to_cp437('╥'), RGB::from_f32(0.5, 0.5, 0.5)),
        _ => (rltk::to_cp437('░'), RGB::from_f32(0.4, 0.4, 0.4)),
    };

    RenderTile(glyph, fg, bg)
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> FontCharType {
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
