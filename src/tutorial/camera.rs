use super::{Hidden, Position, Renderable, Target, TileSize};
use crate::map::{tile_glyph, RenderTile};
use crate::Map;
use rltk::{Point, Rltk, RGB};
use specs::prelude::*;

const SHOW_BOUNDARIES: bool = true;

pub fn get_screen_bounds(ecs: &World, _ctx: &mut Rltk) -> (i32, i32, i32, i32) {
    let player_pos = ecs.fetch::<Point>();
    // let (x_chars, y_chars) = ctx.get_char_size(); // window size, not char size
    let (x_chars, y_chars) = (48, 44);

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
                    let RenderTile(glyph, fg, bg) = tile_glyph(idx, &*map);
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

    // Render entities
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let hidden = ecs.read_storage::<Hidden>();
    let map = ecs.fetch::<Map>();
    let sizes = ecs.read_storage::<TileSize>();
    let entities = ecs.entities();
    let targets = ecs.read_storage::<Target>();

    let mut data = (&positions, &renderables, &entities, !&hidden)
        .join()
        .collect::<Vec<_>>();
    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
    for (pos, render, entity, _hidden) in data.iter() {
        if let Some(size) = sizes.get(*entity) {
            for cy in 0..size.y {
                for cx in 0..size.x {
                    let tile_x = cx + pos.x;
                    let tile_y = cy + pos.y;
                    let idx = map.xy_idx(tile_x, tile_y);
                    if map.visible_tiles[idx] {
                        let entity_screen_x = (cx + pos.x) - min_x;
                        let entity_screen_y = (cy + pos.y) - min_y;
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
        } else {
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

        if targets.get(*entity).is_some() {
            let entity_screen_x = pos.x - min_x;
            let entity_screen_y = pos.y - min_y;
            ctx.set(
                entity_screen_x - 1,
                entity_screen_y,
                rltk::RGB::named(rltk::RED),
                rltk::RGB::named(rltk::YELLOW),
                rltk::to_cp437('['),
            );
            ctx.set(
                entity_screen_x + 1,
                entity_screen_y,
                rltk::RGB::named(rltk::RED),
                rltk::RGB::named(rltk::YELLOW),
                rltk::to_cp437(']'),
            );
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
                    let RenderTile(glyph, fg, bg) = tile_glyph(idx, &*map);
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
