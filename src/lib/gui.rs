use crate::{CombatStats, GameLog, InBackpack, Map, Name, Player, Position, State, MAPWIDTH, Viewshed};
use rltk::{Point, Rltk, VirtualKeyCode, BLACK, GREY, MAGENTA, RED, RGB, WHITE, YELLOW, BLUE, CYAN};
use specs::prelude::*;

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, RGB::named(WHITE), RGB::named(BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();

    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, RGB::named(YELLOW), RGB::named(BLACK), &health);
        ctx.draw_bar_horizontal(
            28,
            43,
            51,
            stats.hp,
            stats.max_hp,
            RGB::named(RED),
            RGB::named(BLACK),
        );
    }

    let log = ecs.fetch::<GameLog>();
    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 {
            ctx.print(2, y, s);
        }
        y += 1;
    }

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(MAGENTA));
    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height {
        return;
    }

    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.x, position.y);
        if position.x == mouse_pos.0 && position.y == mouse_pos.1 && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
        }
        width += 3;

        if mouse_pos.0 > MAPWIDTH / 2 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, RGB::named(WHITE), RGB::named(GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(WHITE),
                        RGB::named(GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(WHITE),
                RGB::named(GREY),
                &"->".to_string(),
            );
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(WHITE), RGB::named(GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + 1 + i,
                        y,
                        RGB::named(WHITE),
                        RGB::named(GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(WHITE),
                RGB::named(GREY),
                &"<-".to_string(),
            );
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let white = RGB::named(WHITE);
    let yellow = RGB::named(YELLOW);
    let black = RGB::named(BLACK);

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    // TODO: remove hardcoded 25
    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, white, black);
    ctx.print_color(18, y - 2, yellow, black, "Inventory");
    ctx.print_color(18, y + count as i32 + 1, yellow, black, "ESCAPE to Cancel");

    let mut equippable: Vec<Entity> = Vec::new();
    let inventory = (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity);
    for (j, (entity, _, name)) in inventory.enumerate() {
        ctx.set(17, y, white, black, rltk::to_cp437('('));
        ctx.set(18, y, white, black, 97 + j as rltk::FontCharType);
        ctx.set(19, y, white, black, rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(VirtualKeyCode::Escape) => (ItemMenuResult::Cancel, None),
        Some(key) => {
            let selection = rltk::letter_to_option(key);
            if selection > -1 && selection < count as i32 {
                return (
                    ItemMenuResult::Selected,
                    Some(equippable[selection as usize]),
                );
            }
            (ItemMenuResult::NoResponse, None)
        }
    }
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let white = RGB::named(WHITE);
    let yellow = RGB::named(YELLOW);
    let black = RGB::named(BLACK);

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    // TODO: remove hardcoded 25
    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, white, black);
    ctx.print_color(18, y - 2, yellow, black, "Drop Which Item?");
    ctx.print_color(18, y + count as i32 + 1, yellow, black, "ESCAPE to Cancel");

    let mut equippable: Vec<Entity> = Vec::new();
    let inventory = (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity);
    for (j, (entity, _, name)) in inventory.enumerate() {
        ctx.set(17, y, white, black, rltk::to_cp437('('));
        ctx.set(18, y, white, black, 97 + j as rltk::FontCharType);
        ctx.set(19, y, white, black, rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(VirtualKeyCode::Escape) => (ItemMenuResult::Cancel, None),
        Some(key) => {
            let selection = rltk::letter_to_option(key);
            if selection > -1 && selection < count as i32 {
                return (
                    ItemMenuResult::Selected,
                    Some(equippable[selection as usize]),
                );
            }
            (ItemMenuResult::NoResponse, None)
        }
    }
}

pub fn ranged_target(gs: &mut State, ctx: &mut Rltk, range: i32) -> (ItemMenuResult, Option<Point>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(5, 0, RGB::named(YELLOW), RGB::named(rltk::BLACK),
            "Select Target:");

    let mut available_cells = Vec::new();
    if let Some(visible) = viewsheds.get(*player_entity) {
        for idx in visible.visible_tiles.iter() {
            let dist = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if dist <= range as f32 {
                ctx.set_bg(idx.x, idx.y, RGB::named(BLUE));
                available_cells.push(idx);
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    let mouse_pos = ctx.mouse_pos();
    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_pos.0 && idx.y == mouse_pos.1 {
            valid_target = true;
            break;
        }
    }

    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(CYAN));
        if ctx.left_click {
            return (ItemMenuResult::Selected,
                    Some(Point::new(mouse_pos.0, mouse_pos.1)));
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}
