use crate::{
    camera, Attribute, Attributes, Equipped, GameLog, Hidden, HungerClock, HungerState, InBackpack,
    Map, Name, Owned, Player, Pools, Position, RexAssets, RunState, State, Viewshed,
};
use rltk::{
    to_cp437, Point, Rltk, VirtualKeyCode, BLACK, BLUE, CYAN, GREEN, GREY, MAGENTA, ORANGE, RED,
    RGB, WHITE, YELLOW,
};
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

pub fn draw_hollow_box(
    console: &mut Rltk,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGB,
    bg: RGB,
) {
    console.set(sx, sy, fg, bg, to_cp437('┌'));
    console.set(sx + width, sy, fg, bg, to_cp437('┐'));
    console.set(sx, sy + height, fg, bg, to_cp437('└'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('┘'));

    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('─'));
        console.set(x, sy + height, fg, bg, to_cp437('─'));
    }

    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('│'));
        console.set(sx + width, y, fg, bg, to_cp437('│'));
    }
}

fn draw_attribute(name: &str, attribute: &Attribute, y: i32, ctx: &mut Rltk) {
    let black: RGB = RGB::named(rltk::BLACK);
    let white: RGB = RGB::named(rltk::WHITE);
    let attr_grey: RGB = RGB::from_hex("#CCCCCC").expect("oops");

    ctx.print_color(50, y, attr_grey, black, name);
    let color: RGB = match attribute.modifiers {
        x if x < 0 => RGB::from_f32(1., 0., 0.),
        0 => white,
        _ => RGB::from_f32(0., 1., 0.),
    };

    ctx.print_color(
        67,
        y,
        color,
        black,
        &format!("{}", attribute.base + attribute.modifiers),
    );
    ctx.print_color(73, y, color, black, &format!("{}", attribute.bonus));
    if attribute.bonus > 0 {
        ctx.set(72, y, color, black, to_cp437('+'));
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    let box_grey: RGB = RGB::from_hex("#999999").expect("oops");
    let black: RGB = RGB::named(rltk::BLACK);
    let white: RGB = RGB::named(rltk::WHITE);
    let red: RGB = RGB::named(rltk::RED);
    let blue: RGB = RGB::named(rltk::BLUE);

    draw_hollow_box(ctx, 0, 0, 79, 59, box_grey, black);
    draw_hollow_box(ctx, 0, 0, 49, 45, box_grey, black);
    draw_hollow_box(ctx, 0, 45, 79, 14, box_grey, black);
    draw_hollow_box(ctx, 49, 0, 30, 8, box_grey, black);

    ctx.set(0, 45, box_grey, black, to_cp437('├'));
    ctx.set(49, 8, box_grey, black, to_cp437('├'));
    ctx.set(49, 0, box_grey, black, to_cp437('┬'));
    ctx.set(49, 45, box_grey, black, to_cp437('┴'));
    ctx.set(49, 45, box_grey, black, to_cp437('┴'));
    ctx.set(79, 8, box_grey, black, to_cp437('┤'));
    ctx.set(79, 45, box_grey, black, to_cp437('┤'));

    // Draw the town name
    let map = ecs.fetch::<Map>();
    let name_length = map.name.len() + 2;
    let x_pos = (22 - (name_length / 2)) as i32;
    ctx.set(x_pos, 0, box_grey, black, to_cp437('┤'));
    ctx.set(
        x_pos + name_length as i32,
        0,
        box_grey,
        black,
        to_cp437('├'),
    );
    ctx.print_color(x_pos + 1, 0, white, black, &map.name);
    std::mem::drop(map);

    let player_entity = ecs.fetch::<Entity>();
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();
    // let (hp, mana, _, _) ...
    let health = format!(
        "Health: {}/{}",
        player_pools.hit_points.current, player_pools.hit_points.max
    );
    let mana = format!(
        "Mana: {}/{}",
        player_pools.mana.current, player_pools.mana.max
    );
    ctx.print_color(50, 1, white, black, &health);
    ctx.print_color(50, 2, white, black, &mana);
    ctx.draw_bar_horizontal(
        64,
        1,
        14,
        player_pools.hit_points.current,
        player_pools.hit_points.max,
        red,
        black,
    );
    ctx.draw_bar_horizontal(
        64,
        2,
        14,
        player_pools.mana.current,
        player_pools.mana.max,
        blue,
        black,
    );

    // Attributes
    let attributes = ecs.read_storage::<Attributes>();
    let attr = attributes.get(*player_entity).unwrap();
    draw_attribute("Might:", &attr.might, 4, ctx);
    draw_attribute("Quickness:", &attr.quickness, 5, ctx);
    draw_attribute("Fitness:", &attr.fitness, 6, ctx);
    draw_attribute("Intelligence:", &attr.intelligence, 7, ctx);
}

pub fn _old_draw_ui(ecs: &World, ctx: &mut Rltk) {
    let white = RGB::named(WHITE);
    let black = RGB::named(BLACK);
    let red = RGB::named(RED);
    let green = RGB::named(GREEN);
    let orange = RGB::named(ORANGE);
    let yellow = RGB::named(YELLOW);

    ctx.draw_box(0, 43, 79, 6, white, black);

    let combat_stats = ecs.read_storage::<Pools>();
    let players = ecs.read_storage::<Player>();
    let hunger = ecs.read_storage::<HungerClock>();

    for (_player, stats, hc) in (&players, &combat_stats, &hunger).join() {
        let health = format!(
            " HP: {} / {} ",
            stats.hit_points.current, stats.hit_points.max
        );
        ctx.print_color(12, 43, yellow, black, &health);
        ctx.draw_bar_horizontal(
            28,
            43,
            51,
            stats.hit_points.current,
            stats.hit_points.max,
            red,
            black,
        );

        match hc.state {
            HungerState::WellFed => ctx.print_color(71, 42, green, black, "Well Fed"),
            HungerState::Normal => {}
            HungerState::Hungry => ctx.print_color(71, 42, orange, black, "Hungry"),
            HungerState::Starving => ctx.print_color(71, 42, red, black, "Starving"),
        }
    }

    let log = ecs.fetch::<GameLog>();
    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 {
            ctx.print(2, y, s);
        }
        y += 1;
    }

    let map = ecs.fetch::<Map>();
    let depth = format!("Depth: {}", map.depth);
    ctx.print_color(2, 43, RGB::named(YELLOW), RGB::named(BLACK), &depth);

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(MAGENTA));
    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let (min_x, _, min_y, _) = camera::get_screen_bounds(ecs, ctx);
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let hidden = ecs.read_storage::<Hidden>();

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;
    if mouse_map_pos.0 >= map.width - 1
        || mouse_map_pos.1 >= map.height - 1
        || mouse_map_pos.0 < 1
        || mouse_map_pos.1 < 1
    {
        return;
    }

    if !map.visible_tiles[map.xy_idx(mouse_map_pos.0, mouse_map_pos.1)] {
        return;
    }
    let mut tooltip: Vec<String> = Vec::new();
    for (name, position, _) in (&names, &positions, !&hidden).join() {
        if position.x == mouse_map_pos.0 && position.y == mouse_map_pos.1 {
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

        if mouse_pos.0 > map.width / 2 {
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
    show_menu::<InBackpack>(gs, ctx, "Inventory")
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    show_menu::<InBackpack>(gs, ctx, "Drop Which Item?")
}

pub fn remove_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    show_menu::<Equipped>(gs, ctx, "Remove Which Item?")
}

fn show_menu<T: Owned + Component>(
    gs: &mut State,
    ctx: &mut Rltk,
    heading: &str,
) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<T>();
    let entities = gs.ecs.entities();
    let white = RGB::named(WHITE);
    let yellow = RGB::named(YELLOW);
    let black = RGB::named(BLACK);

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owned_by() == *player_entity);
    let count = inventory.count();

    // TODO: remove hardcoded 25
    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, white, black);
    ctx.print_color(18, y - 2, yellow, black, heading);
    ctx.print_color(
        18,
        y + count as i32 + 1,
        yellow,
        black,
        "Press ESC to Cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    let inventory = (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owned_by() == *player_entity);
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

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut Rltk,
    range: i32,
) -> (ItemMenuResult, Option<Point>) {
    let (min_x, _, min_y, _) = camera::get_screen_bounds(&gs.ecs, ctx);
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(
        5,
        0,
        RGB::named(YELLOW),
        RGB::named(rltk::BLACK),
        "Select Target:",
    );

    let mut available_cells = Vec::new();
    if let Some(visible) = viewsheds.get(*player_entity) {
        for idx in visible.visible_tiles.iter() {
            let dist = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if dist <= range as f32 {
                let screen_x = idx.x - min_x;
                let screen_y = idx.y - min_y;
                ctx.set_bg(screen_x, screen_y, RGB::named(BLUE));
                available_cells.push(idx);
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
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(CYAN));
        if ctx.left_click {
            return (
                ItemMenuResult::Selected,
                Some(Point::new(mouse_pos.0, mouse_pos.1)),
            );
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    use MainMenuResult::*;
    use MainMenuSelection::*;

    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    let assets = gs.ecs.fetch::<RexAssets>();
    ctx.render_xp_sprite(&assets.menu, 0, 0);

    ctx.draw_box_double(
        24,
        18,
        31,
        10,
        RGB::named(rltk::WHEAT),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color_centered(
        20,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "Rusty Roguelike Tutorial",
    );
    ctx.print_color_centered(
        21,
        RGB::named(CYAN),
        RGB::named(BLACK),
        "by Herbert Wolverson",
    );
    ctx.print_color_centered(
        22,
        RGB::named(GREY),
        RGB::named(BLACK),
        "Use Up/Down Arrows and Enter",
    );

    let mut y = 24;
    if let RunState::MainMenu {
        menu_selection: selection,
    } = *runstate
    {
        if selection == NewGame {
            ctx.print_color_centered(y, RGB::named(MAGENTA), RGB::named(BLACK), "Begin New Game");
        } else {
            ctx.print_color_centered(y, RGB::named(WHITE), RGB::named(BLACK), "Begin New Game");
        }
        y += 1;

        if save_exists {
            if selection == LoadGame {
                ctx.print_color_centered(y, RGB::named(MAGENTA), RGB::named(BLACK), "Load Game");
            } else {
                ctx.print_color_centered(y, RGB::named(WHITE), RGB::named(BLACK), "Load Game");
            }
            y += 1;
        }

        if selection == Quit {
            ctx.print_color_centered(y, RGB::named(MAGENTA), RGB::named(BLACK), "Quit");
        } else {
            ctx.print_color_centered(y, RGB::named(WHITE), RGB::named(BLACK), "Quit");
        }

        if let Some(key) = ctx.key {
            use VirtualKeyCode::*;

            match key {
                Escape => {
                    return NoSelection { selected: Quit };
                }

                Up => {
                    return match selection {
                        NewGame => NoSelection { selected: Quit },
                        LoadGame => NoSelection { selected: NewGame },
                        Quit => NoSelection {
                            selected: if save_exists { LoadGame } else { NewGame },
                        },
                    };
                }

                Down => {
                    return match selection {
                        NewGame => NoSelection {
                            selected: if save_exists { LoadGame } else { Quit },
                        },
                        LoadGame => NoSelection { selected: Quit },
                        Quit => NoSelection { selected: NewGame },
                    };
                }

                Return => {
                    return Selected {
                        selected: selection,
                    };
                }

                _ => {
                    return NoSelection {
                        selected: selection,
                    }
                }
            }
        } else {
            return NoSelection {
                selected: selection,
            };
        }
    }
    NoSelection { selected: NewGame }
}

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    ctx.print_color_centered(
        15,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Your journey has ended!",
    );
    ctx.print_color_centered(
        17,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "One day, we'll tell you all about how you did.",
    );
    ctx.print_color_centered(
        18,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "That day, sadly, is not in this chapter...",
    );

    ctx.print_color_centered(
        20,
        RGB::named(rltk::MAGENTA),
        RGB::named(rltk::BLACK),
        "Press ESC to return to the menu.",
    );

    match ctx.key {
        Some(VirtualKeyCode::Escape) => GameOverResult::QuitToMenu,
        _ => GameOverResult::NoSelection,
    }
}
