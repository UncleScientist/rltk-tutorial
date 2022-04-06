use super::ItemMenuResult;
use rltk::prelude::*;
use specs::prelude::*;

use crate::{map::camera, State, Viewshed};

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut Rltk,
    range: i32,
) -> (ItemMenuResult, Option<Point>) {
    let (min_x, max_x, min_y, max_y) = camera::get_screen_bounds(&gs.ecs, ctx);
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    let mut draw_batch = DrawBatch::new();

    draw_batch.print_color(
        Point::new(5, 0),
        "Select Target:",
        ColorPair::new(RGB::named(YELLOW), RGB::named(rltk::BLACK)),
    );

    let mut available_cells = Vec::new();
    if let Some(visible) = viewsheds.get(*player_entity) {
        for idx in visible.visible_tiles.iter() {
            let dist = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if dist <= range as f32 {
                let screen_x = idx.x - min_x;
                let screen_y = idx.y - min_y;
                if screen_x > 1
                    && screen_x < (max_x - min_x) - 1
                    && screen_y > 1
                    && screen_y < (max_y - min_y) - 1
                {
                    draw_batch.set_bg(Point::new(screen_x, screen_y), RGB::named(BLUE));
                    available_cells.push(idx);
                }
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;
    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_map_pos.0 && idx.y == mouse_map_pos.1 {
            valid_target = true;
            break;
        }
    }

    if valid_target {
        draw_batch.set_bg(Point::new(mouse_pos.0, mouse_pos.1), RGB::named(CYAN));
        if ctx.left_click {
            return (
                ItemMenuResult::Selected,
                Some(Point::new(mouse_map_pos.0, mouse_map_pos.1)),
            );
        }
    } else {
        draw_batch.set_bg(Point::new(mouse_pos.0, mouse_pos.1), RGB::named(RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    draw_batch
        .submit(5000)
        .expect("Unable to draw mouse cursor");

    (ItemMenuResult::NoResponse, None)
}
