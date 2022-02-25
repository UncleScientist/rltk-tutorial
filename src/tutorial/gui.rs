use crate::{
    camera, Attribute, Attributes, Consumable, CursedItem, Equipped, GameLog, Hidden, HungerClock,
    HungerState, InBackpack, Item, MagicItem, MagicItemClass, Map, MasterDungeonMap, Name,
    ObfuscatedName, Owned, Pools, Position, RexAssets, RunState, State, Vendor, VendorMode,
    Viewshed,
};
use rltk::{
    to_cp437, Point, Rltk, VirtualKeyCode, BLACK, BLUE, CYAN, GOLD, GREY, MAGENTA, ORANGE, RED,
    RGB, WHITE, YELLOW,
};
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum CheatMenuResult {
    NoResponse,
    Cancel,
    TeleportToExit,
    Heal,
    Reveal,
    Money,
    GodMode,
}

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
    let black: RGB = RGB::named(BLACK);
    let white: RGB = RGB::named(WHITE);
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
    let black: RGB = RGB::named(BLACK);
    let white: RGB = RGB::named(WHITE);
    let red: RGB = RGB::named(RED);
    let blue: RGB = RGB::named(BLUE);
    let yellow: RGB = RGB::named(YELLOW);
    let green: RGB = RGB::from_f32(0., 1., 0.);
    let orange = RGB::named(ORANGE);
    let gold = RGB::named(GOLD);

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
        x_pos + name_length as i32 - 1,
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
    let xp = format!("Level: {}", player_pools.level);
    ctx.print_color(50, 1, white, black, &health);
    ctx.print_color(50, 2, white, black, &mana);
    ctx.print_color(50, 3, white, black, &xp);
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
    let xp_level_start = (player_pools.level - 1) * 1000;
    ctx.draw_bar_horizontal(
        64,
        3,
        14,
        player_pools.xp - xp_level_start,
        1000,
        gold,
        black,
    );

    // Attributes
    let attributes = ecs.read_storage::<Attributes>();
    let attr = attributes.get(*player_entity).unwrap();
    draw_attribute("Might:", &attr.might, 4, ctx);
    draw_attribute("Quickness:", &attr.quickness, 5, ctx);
    draw_attribute("Fitness:", &attr.fitness, 6, ctx);
    draw_attribute("Intelligence:", &attr.intelligence, 7, ctx);

    // Initiative and weight
    ctx.print_color(
        50,
        9,
        white,
        black,
        &format!(
            "{:.0} lbs ({} lbs max)",
            player_pools.total_weight,
            (attr.might.base + attr.might.modifiers) * 15
        ),
    );
    ctx.print_color(
        50,
        10,
        white,
        black,
        &format!(
            "Initiative Penalty: {:.0}",
            player_pools.total_initiative_penalty,
        ),
    );

    // Money
    ctx.print_color(
        50,
        11,
        rltk::RGB::named(rltk::GOLD),
        black,
        &format!("Gold: {:.1}", player_pools.gold),
    );

    // Equipped
    let mut y = 13;
    let entities = ecs.entities();
    let equipped = ecs.read_storage::<Equipped>();
    for (entity, equipped_by) in (&entities, &equipped).join() {
        if equipped_by.owner == *player_entity {
            ctx.print_color(
                50,
                y,
                get_item_color(ecs, entity),
                black,
                &get_item_display_name(ecs, entity),
            );
            y += 1;
        }
    }

    // Consumables
    y += 1;
    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    let mut index = 1;
    for (entity, carried_by, _) in (&entities, &backpack, &consumables).join() {
        if carried_by.owner == *player_entity && index < 10 {
            ctx.print_color(50, y, yellow, black, &format!("↑{}", index));
            ctx.print_color(
                53,
                y,
                get_item_color(ecs, entity),
                black,
                &get_item_display_name(ecs, entity),
            );
            y += 1;
            index += 1;
        }
    }

    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();
    match hc.state {
        HungerState::WellFed => ctx.print_color(50, 44, green, black, "Well Fed"),
        HungerState::Normal => {}
        HungerState::Hungry => ctx.print_color(50, 44, orange, black, "Hungry"),
        HungerState::Starving => ctx.print_color(50, 44, red, black, "Starving"),
    }

    // Draw the log
    let log = ecs.fetch::<GameLog>();
    let mut y = 46;
    for s in log.entries.iter().rev() {
        if y < 59 {
            ctx.print(2, y, s);
        }
        y += 1;
    }

    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let (min_x, _, min_y, _) = camera::get_screen_bounds(ecs, ctx);
    let map = ecs.fetch::<Map>();
    let positions = ecs.read_storage::<Position>();
    let hidden = ecs.read_storage::<Hidden>();
    let attributes = ecs.read_storage::<Attributes>();
    let pools = ecs.read_storage::<Pools>();
    let entities = ecs.entities();

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

    let mut tip_boxes: Vec<Tooltip> = Vec::new();
    for (entity, position, _) in (&entities, &positions, !&hidden).join() {
        if position.x == mouse_map_pos.0 && position.y == mouse_map_pos.1 {
            let mut tip = Tooltip::new();
            tip.add(get_item_display_name(ecs, entity));

            if let Some(attr) = attributes.get(entity) {
                let mut s = "".to_string(); // String::new()
                if attr.might.bonus < 0 {
                    s += "Weak. "
                };
                if attr.might.bonus > 0 {
                    s += "Strong. "
                };
                if attr.quickness.bonus < 0 {
                    s += "Clumsy. "
                };
                if attr.quickness.bonus > 0 {
                    s += "Agile. "
                };
                if attr.fitness.bonus < 0 {
                    s += "Unhealthy. "
                };
                if attr.fitness.bonus > 0 {
                    s += "Healthy. "
                };
                if attr.intelligence.bonus < 0 {
                    s += "Unintelligent. "
                };
                if attr.intelligence.bonus > 0 {
                    s += "Smart. "
                };
                if s.is_empty() {
                    s = "Quite Average".to_string()
                }
                tip.add(s);
            }

            // Comment on pools
            if let Some(stat) = pools.get(entity) {
                tip.add(format!("Level: {}", stat.level));
            }
            tip_boxes.push(tip);
        }
    }

    if tip_boxes.is_empty() {
        return;
    }

    let box_grey = RGB::from_hex("#999999").expect("oops");
    let white = RGB::named(WHITE);

    let arrow_y = mouse_pos.1;
    let (arrow, arrow_x) = if mouse_pos.0 < 40 {
        (to_cp437('→'), mouse_pos.0 - 1)
    } else {
        (to_cp437('←'), mouse_pos.0 + 1)
    };
    ctx.set(arrow_x, arrow_y, white, box_grey, arrow);

    let total_height = tip_boxes.iter().map(|tt| tt.height()).sum::<i32>();
    let mut y = mouse_pos.1 - (total_height / 2);
    while y + (total_height / 2) > 50 {
        y -= 1;
    }

    for tt in tip_boxes.iter() {
        let x = if mouse_pos.0 < 40 {
            mouse_pos.0 - (1 + tt.width())
        } else {
            mouse_pos.0 + (1 + tt.width())
        };
        tt.render(ctx, x, y);
        y += tt.height();
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_cheat_mode(_gs: &mut State, ctx: &mut Rltk) -> CheatMenuResult {
    let white = RGB::named(WHITE);
    let black = RGB::named(BLACK);
    let yellow = RGB::named(YELLOW);
    let count = 5;
    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, white, black);
    ctx.print_color(18, y - 2, yellow, black, "Cheating!");
    ctx.print_color(18, y + count as i32 + 1, yellow, black, "ESC to cancel");

    ctx.set(17, y, white, black, rltk::to_cp437('('));
    ctx.set(18, y, white, black, rltk::to_cp437('T'));
    ctx.set(19, y, white, black, rltk::to_cp437(')'));
    ctx.print(21, y, "Teleport to exit");

    y += 1;
    ctx.set(17, y, white, black, rltk::to_cp437('('));
    ctx.set(18, y, white, black, rltk::to_cp437('H'));
    ctx.set(19, y, white, black, rltk::to_cp437(')'));
    ctx.print(21, y, "Heal all wounds");

    y += 1;
    ctx.set(17, y, white, black, rltk::to_cp437('('));
    ctx.set(18, y, white, black, rltk::to_cp437('R'));
    ctx.set(19, y, white, black, rltk::to_cp437(')'));
    ctx.print(21, y, "Reveal the map");

    y += 1;
    ctx.set(17, y, white, black, rltk::to_cp437('('));
    ctx.set(18, y, white, black, rltk::to_cp437('G'));
    ctx.set(19, y, white, black, rltk::to_cp437(')'));
    ctx.print(21, y, "God Mode (no death)");

    y += 1;
    ctx.set(17, y, white, black, rltk::to_cp437('('));
    ctx.set(18, y, white, black, rltk::to_cp437('M'));
    ctx.set(19, y, white, black, rltk::to_cp437(')'));
    ctx.print(21, y, "Make some Money");

    match ctx.key {
        None => CheatMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::G => CheatMenuResult::GodMode,
            VirtualKeyCode::H => CheatMenuResult::Heal,
            VirtualKeyCode::M => CheatMenuResult::Money,
            VirtualKeyCode::R => CheatMenuResult::Reveal,
            VirtualKeyCode::T => CheatMenuResult::TeleportToExit,
            VirtualKeyCode::Escape => CheatMenuResult::Cancel,
            _ => CheatMenuResult::NoResponse,
        },
    }
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
    let backpack = gs.ecs.read_storage::<T>();
    let entities = gs.ecs.entities();
    let white = RGB::named(WHITE);
    let yellow = RGB::named(YELLOW);
    let black = RGB::named(BLACK);

    let inventory = backpack
        .join()
        .filter(|item| item.owned_by() == *player_entity);
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
    let inventory = (&entities, &backpack)
        .join()
        .filter(|item| item.1.owned_by() == *player_entity);
    for (j, (entity, _)) in inventory.enumerate() {
        ctx.set(17, y, white, black, rltk::to_cp437('('));
        ctx.set(18, y, white, black, 97 + j as rltk::FontCharType);
        ctx.set(19, y, white, black, rltk::to_cp437(')'));

        ctx.print_color(
            21,
            y,
            get_item_color(&gs.ecs, entity),
            RGB::from_f32(0., 0., 0.),
            &get_item_display_name(&gs.ecs, entity),
        );
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

pub fn remove_curse_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();
    let items = gs.ecs.read_storage::<Item>();
    let cursed = gs.ecs.read_storage::<CursedItem>();
    let names = gs.ecs.read_storage::<Name>();
    let dm = gs.ecs.fetch::<MasterDungeonMap>();
    let white = RGB::named(WHITE);
    let yellow = RGB::named(YELLOW);
    let black = RGB::named(BLACK);

    let build_cursed_iterator = || {
        (&entities, &items, &cursed)
            .join()
            .filter(|(item_entity, _item, _cursed)| {
                let mut keep = false;
                if let Some(bp) = backpack.get(*item_entity) {
                    if bp.owner == *player_entity {
                        if let Some(name) = names.get(*item_entity) {
                            keep = dm.identified_items.contains(&name.name);
                        }
                    }
                }

                // It's equipped, so we know it's cursed
                if let Some(equip) = equipped.get(*item_entity) {
                    keep = equip.owner == *player_entity;
                }

                keep
            })
    };

    let count = build_cursed_iterator().count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 34, (count + 3) as i32, white, black);
    ctx.print_color(18, y - 2, yellow, black, "Remove Curse from Which Item?");
    ctx.print_color(
        18,
        y + count as i32 + 1,
        yellow,
        black,
        "Press ESC to Cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    for (j, (entity, _item, _cursed)) in build_cursed_iterator().enumerate() {
        ctx.set(17, y, white, black, rltk::to_cp437('('));
        ctx.set(18, y, white, black, 97 + j as rltk::FontCharType);
        ctx.set(19, y, white, black, rltk::to_cp437(')'));

        ctx.print_color(
            21,
            y,
            get_item_color(&gs.ecs, entity),
            black,
            &get_item_display_name(&gs.ecs, entity),
        );
        equippable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn get_item_display_name(ecs: &World, item: Entity) -> String {
    if let Some(name) = ecs.read_storage::<Name>().get(item) {
        if ecs.read_storage::<MagicItem>().get(item).is_some() {
            let dm = ecs.fetch::<crate::map::MasterDungeonMap>();
            if dm.identified_items.contains(&name.name) {
                name.name.clone()
            } else if let Some(obfuscated) = ecs.read_storage::<ObfuscatedName>().get(item) {
                obfuscated.name.clone()
            } else {
                "Unidentified magic item".to_string()
            }
        } else {
            name.name.clone()
        }
    } else {
        "Nameless item (bug)".to_string()
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
                Some(Point::new(mouse_map_pos.0, mouse_map_pos.1)),
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

pub fn show_vendor_menu(
    gs: &mut State,
    ctx: &mut Rltk,
    vendor: Entity,
    mode: VendorMode,
) -> (VendorResult, Option<Entity>, Option<String>, Option<f32>) {
    match mode {
        VendorMode::Buy => vendor_buy_menu(gs, ctx, vendor, mode),
        VendorMode::Sell => vendor_sell_menu(gs, ctx, vendor, mode),
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum VendorResult {
    NoResponse,
    Cancel,
    Sell,
    BuyMode,
    SellMode,
    Buy,
}

// Player sells, vendor buys
pub fn vendor_sell_menu(
    gs: &mut State,
    ctx: &mut Rltk,
    _vendor: Entity,
    _mode: VendorMode,
) -> (VendorResult, Option<Entity>, Option<String>, Option<f32>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack)
        .join()
        .filter(|item| item.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        51,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Sell Which Item? (space to switch to buy mode)",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESC to cancel",
    );

    let mut equippable = Vec::new();
    for (j, (entity, _pack, item)) in (&entities, &backpack, &items)
        .join()
        .filter(|item| item.1.owner == *player_entity)
        .enumerate()
    {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print_color(
            21,
            y,
            get_item_color(&gs.ecs, entity),
            RGB::named(rltk::BLACK),
            &get_item_display_name(&gs.ecs, entity),
        );
        ctx.print(50, y, &format!("{:.1} gp", item.base_value * 0.8));
        equippable.push(entity);
        y += 1;
    }
    match ctx.key {
        None => (VendorResult::NoResponse, None, None, None),
        Some(key) => match key {
            VirtualKeyCode::Space => (VendorResult::BuyMode, None, None, None),
            VirtualKeyCode::Escape => (VendorResult::Cancel, None, None, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        VendorResult::Sell,
                        Some(equippable[selection as usize]),
                        None,
                        None,
                    );
                }
                (VendorResult::NoResponse, None, None, None)
            }
        },
    }
}

pub fn vendor_buy_menu(
    gs: &mut State,
    ctx: &mut Rltk,
    vendor: Entity,
    _mode: VendorMode,
) -> (VendorResult, Option<Entity>, Option<String>, Option<f32>) {
    use crate::raws::*;

    let vendors = gs.ecs.read_storage::<Vendor>();

    let inventory = get_vendor_items(
        &vendors.get(vendor).unwrap().categories,
        &RAWS.lock().unwrap(),
    );
    let count = inventory.len();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        51,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Buy Which Item? (space to switch to sell mode)",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESC to cancel",
    );

    for (j, sale) in inventory.iter().enumerate() {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &sale.0);
        ctx.print(50, y, &format!("{:.1} gp", sale.1 * 0.8));
        y += 1;
    }
    match ctx.key {
        None => (VendorResult::NoResponse, None, None, None),
        Some(key) => match key {
            VirtualKeyCode::Space => (VendorResult::SellMode, None, None, None),
            VirtualKeyCode::Escape => (VendorResult::Cancel, None, None, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        VendorResult::Buy,
                        None,
                        Some(inventory[selection as usize].0.clone()),
                        Some(inventory[selection as usize].1),
                    );
                }
                (VendorResult::NoResponse, None, None, None)
            }
        },
    }
}

pub fn get_item_color(ecs: &World, item: Entity) -> RGB {
    let dm = ecs.fetch::<crate::map::MasterDungeonMap>();
    if let Some(name) = ecs.read_storage::<Name>().get(item) {
        if ecs.read_storage::<CursedItem>().get(item).is_some()
            && dm.identified_items.contains(&name.name)
        {
            return RGB::from_f32(1.0, 0.0, 0.0);
        }
    }

    if let Some(magic) = ecs.read_storage::<MagicItem>().get(item) {
        match magic.class {
            MagicItemClass::Common => return RGB::from_f32(0.5, 1.0, 0.5),
            MagicItemClass::Rare => return RGB::from_f32(0.0, 1.0, 1.0),
            MagicItemClass::Legendary => return RGB::from_f32(0.71, 0.15, 0.93),
        }
    }
    RGB::from_f32(1.0, 1.0, 1.0)
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

// --- Tooltips ---

struct Tooltip {
    lines: Vec<String>,
}

impl Tooltip {
    fn new() -> Tooltip {
        Tooltip { lines: Vec::new() }
    }

    fn add<S: ToString>(&mut self, line: S) {
        self.lines.push(line.to_string())
    }

    fn width(&self) -> i32 {
        self.lines.iter().map(|x| x.len()).max().unwrap() as i32 + 2
    }

    fn height(&self) -> i32 {
        self.lines.len() as i32 + 2
    }

    fn render(&self, ctx: &mut Rltk, x: i32, y: i32) {
        let box_grey = RGB::from_hex("#999999").expect("oops");
        let light_grey = RGB::from_hex("#DDDDDD").expect("oops");
        let white = RGB::named(WHITE);
        let black = RGB::named(BLACK);

        ctx.draw_box(x, y, self.width() - 1, self.height() - 1, white, box_grey);
        for (i, s) in self.lines.iter().enumerate() {
            let col = if i == 0 { white } else { light_grey };
            ctx.print_color(x + 1, y + i as i32 + 1, col, black, &s);
        }
    }
}
