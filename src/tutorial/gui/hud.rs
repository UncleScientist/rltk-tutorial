use rltk::prelude::*;
use specs::prelude::*;

use crate::{
    draw_tooltips, gamelog, get_item_color, get_item_display_name, Attribute, Attributes,
    Consumable, Duration, Entity, Equipped, HungerClock, HungerState, InBackpack, KnownSpells, Map,
    Name, Pools, StatusEffect, Weapon,
};

fn draw_attribute(name: &str, attribute: &Attribute, y: i32, draw_batch: &mut DrawBatch) {
    let black: RGB = RGB::named(BLACK);
    let white: RGB = RGB::named(WHITE);
    let attr_grey: RGB = RGB::from_hex("#CCCCCC").expect("oops");

    draw_batch.print_color(Point::new(50, y), name, ColorPair::new(attr_grey, black));
    let color: RGB = match attribute.modifiers {
        x if x < 0 => RGB::from_f32(1., 0., 0.),
        0 => white,
        _ => RGB::from_f32(0., 1., 0.),
    };

    draw_batch.print_color(
        Point::new(67, y),
        &format!("{}", attribute.base + attribute.modifiers),
        ColorPair::new(color, black),
    );
    draw_batch.print_color(
        Point::new(73, y),
        &format!("{}", attribute.bonus),
        ColorPair::new(color, black),
    );
    if attribute.bonus > 0 {
        draw_batch.set(
            Point::new(72, y),
            ColorPair::new(color, black),
            to_cp437('+'),
        );
    }
}

fn box_framework(draw_batch: &mut DrawBatch) {
    let box_grey: RGB = RGB::from_hex("#999999").expect("oops");
    let black: RGB = RGB::named(BLACK);

    draw_batch.draw_hollow_box(
        Rect::with_size(0, 0, 79, 59),
        ColorPair::new(box_grey, black),
    );
    draw_batch.draw_hollow_box(
        Rect::with_size(0, 0, 49, 45),
        ColorPair::new(box_grey, black),
    );
    draw_batch.draw_hollow_box(
        Rect::with_size(0, 45, 79, 14),
        ColorPair::new(box_grey, black),
    );
    draw_batch.draw_hollow_box(
        Rect::with_size(49, 0, 30, 8),
        ColorPair::new(box_grey, black),
    );

    draw_batch.set(
        Point::new(0, 45),
        ColorPair::new(box_grey, black),
        to_cp437('├'),
    );
    draw_batch.set(
        Point::new(49, 8),
        ColorPair::new(box_grey, black),
        to_cp437('├'),
    );
    draw_batch.set(
        Point::new(49, 0),
        ColorPair::new(box_grey, black),
        to_cp437('┬'),
    );
    draw_batch.set(
        Point::new(49, 45),
        ColorPair::new(box_grey, black),
        to_cp437('┴'),
    );
    draw_batch.set(
        Point::new(49, 45),
        ColorPair::new(box_grey, black),
        to_cp437('┴'),
    );
    draw_batch.set(
        Point::new(79, 8),
        ColorPair::new(box_grey, black),
        to_cp437('┤'),
    );
    draw_batch.set(
        Point::new(79, 45),
        ColorPair::new(box_grey, black),
        to_cp437('┤'),
    );
}

fn map_label(ecs: &World, draw_batch: &mut DrawBatch) {
    let box_grey: RGB = RGB::from_hex("#999999").expect("oops");
    let black: RGB = RGB::named(BLACK);
    let white: RGB = RGB::named(WHITE);

    let map = ecs.fetch::<Map>();
    let name_length = map.name.len() + 2;
    let x_pos = (22 - (name_length / 2)) as i32;
    draw_batch.set(
        Point::new(x_pos, 0),
        ColorPair::new(box_grey, black),
        to_cp437('┤'),
    );
    draw_batch.set(
        Point::new(x_pos + name_length as i32 - 1, 0),
        ColorPair::new(box_grey, black),
        to_cp437('├'),
    );
    draw_batch.print_color(
        Point::new(x_pos + 1, 0),
        &map.name,
        ColorPair::new(white, black),
    );
}

fn draw_stats(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let black: RGB = RGB::named(BLACK);
    let white: RGB = RGB::named(WHITE);
    let red: RGB = RGB::named(RED);
    let blue: RGB = RGB::named(BLUE);
    let gold = RGB::named(GOLD);

    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();
    let health = format!(
        "Health: {}/{}",
        player_pools.hit_points.current, player_pools.hit_points.max
    );
    let mana = format!(
        "Mana: {}/{}",
        player_pools.mana.current, player_pools.mana.max
    );
    let xp = format!("Level: {}", player_pools.level);
    draw_batch.print_color(Point::new(50, 1), &health, ColorPair::new(white, black));
    draw_batch.print_color(Point::new(50, 2), &mana, ColorPair::new(white, black));
    draw_batch.print_color(Point::new(50, 3), &xp, ColorPair::new(white, black));
    draw_batch.bar_horizontal(
        Point::new(64, 1),
        14,
        player_pools.hit_points.current,
        player_pools.hit_points.max,
        ColorPair::new(red, black),
    );
    draw_batch.bar_horizontal(
        Point::new(64, 2),
        14,
        player_pools.mana.current,
        player_pools.mana.max,
        ColorPair::new(blue, black),
    );
    let xp_level_start = (player_pools.level - 1) * 1000;
    draw_batch.bar_horizontal(
        Point::new(64, 3),
        14,
        player_pools.xp - xp_level_start,
        1000,
        ColorPair::new(gold, black),
    );
}

fn draw_attributes(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let attributes = ecs.read_storage::<Attributes>();
    let attr = attributes.get(*player_entity).unwrap();
    draw_attribute("Might:", &attr.might, 4, draw_batch);
    draw_attribute("Quickness:", &attr.quickness, 5, draw_batch);
    draw_attribute("Fitness:", &attr.fitness, 6, draw_batch);
    draw_attribute("Intelligence:", &attr.intelligence, 7, draw_batch);
}

fn initiative_weight(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let black: RGB = RGB::named(BLACK);
    let white: RGB = RGB::named(WHITE);
    let gold: RGB = RGB::named(GOLD);

    let attributes = ecs.read_storage::<Attributes>();
    let attr = attributes.get(*player_entity).unwrap();
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();

    draw_batch.print_color(
        Point::new(50, 9),
        &format!(
            "{:.0} lbs ({} lbs max)",
            player_pools.total_weight,
            (attr.might.base + attr.might.modifiers) * 15
        ),
        ColorPair::new(white, black),
    );
    draw_batch.print_color(
        Point::new(50, 10),
        &format!(
            "Initiative Penalty: {:.0}",
            player_pools.total_initiative_penalty,
        ),
        ColorPair::new(white, black),
    );

    draw_batch.print_color(
        Point::new(50, 11),
        &format!("Gold: {:.1}", player_pools.gold),
        ColorPair::new(gold, black),
    );
}

fn equipped(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) -> i32 {
    let black: RGB = RGB::named(BLACK);
    let yellow: RGB = RGB::named(YELLOW);

    let mut y = 13;
    let entities = ecs.entities();
    let equipped = ecs.read_storage::<Equipped>();
    let weapon = ecs.read_storage::<Weapon>();
    for (entity, equipped_by) in (&entities, &equipped).join() {
        if equipped_by.owner == *player_entity {
            let name = get_item_display_name(ecs, entity);
            draw_batch.print_color(
                Point::new(50, y),
                &get_item_display_name(ecs, entity),
                ColorPair::new(get_item_color(ecs, entity), black),
            );
            y += 1;

            if let Some(weapon) = weapon.get(entity) {
                let mut weapon_info = match weapon.damage_bonus.cmp(&0) {
                    std::cmp::Ordering::Less => format!(
                        "┤ {} ({}d{}{})",
                        &name, weapon.damage_n_dice, weapon.damage_die_type, weapon.damage_bonus
                    ),
                    std::cmp::Ordering::Equal => format!(
                        "┤ {} ({}d{})",
                        &name, weapon.damage_n_dice, weapon.damage_die_type
                    ),
                    std::cmp::Ordering::Greater => format!(
                        "┤ {} ({}d{}+{})",
                        &name, weapon.damage_n_dice, weapon.damage_die_type, weapon.damage_bonus
                    ),
                };
                if let Some(range) = weapon.range {
                    weapon_info += &format!(" (range: {range}, F to fire, V cycle targets)");
                }
                weapon_info += " ├";
                draw_batch.print_color(
                    Point::new(3, 45),
                    &weapon_info,
                    ColorPair::new(yellow, black),
                );
            }
        }
    }

    y
}

fn consumables(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity, mut y: i32) -> i32 {
    let black: RGB = RGB::named(BLACK);
    let yellow: RGB = RGB::named(YELLOW);

    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    let entities = ecs.entities();

    y += 1;
    for (index, (entity, carried_by, _)) in (&entities, &backpack, &consumables).join().enumerate()
    {
        if carried_by.owner == *player_entity && index < 10 {
            draw_batch.print_color(
                Point::new(50, y),
                &format!("↑{}", index + 1),
                ColorPair::new(yellow, black),
            );
            draw_batch.print_color(
                Point::new(53, y),
                &get_item_display_name(ecs, entity),
                ColorPair::new(get_item_color(ecs, entity), black),
            );
            y += 1;
        }
    }

    y
}

fn spells(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity, mut y: i32) -> i32 {
    let cyan: RGB = RGB::named(CYAN);
    let black: RGB = RGB::named(BLACK);

    y += 1;
    let known_spells_storage = ecs.read_storage::<KnownSpells>();
    let known_spells = &known_spells_storage.get(*player_entity).unwrap().spells;
    for (index, spell) in known_spells.iter().enumerate() {
        draw_batch.print_color(
            Point::new(50, y),
            &format!("^{}", index + 1),
            ColorPair::new(cyan, black),
        );
        draw_batch.print_color(
            Point::new(53, y),
            &format!("{} ({})", spell.display_name, spell.mana_cost),
            ColorPair::new(cyan, black),
        );
        y += 1;
    }

    y
}

fn status(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let black: RGB = RGB::named(BLACK);
    let red: RGB = RGB::named(RED);
    let green: RGB = RGB::from_f32(0., 1., 0.);
    let orange = RGB::named(ORANGE);

    let mut y = 44;
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();
    match hc.state {
        HungerState::WellFed => {
            draw_batch.print_color(Point::new(50, 44), "Well Fed", ColorPair::new(green, black));
            y -= 1
        }

        HungerState::Normal => {}
        HungerState::Hungry => {
            draw_batch.print_color(Point::new(50, 44), "Hungry", ColorPair::new(orange, black));
            y -= 1
        }
        HungerState::Starving => {
            draw_batch.print_color(Point::new(50, 44), "Starving", ColorPair::new(red, black));
            y -= 1
        }
    }
    let statuses = ecs.read_storage::<StatusEffect>();
    let durations = ecs.read_storage::<Duration>();
    let names = ecs.read_storage::<Name>();
    for (status, duration, name) in (&statuses, &durations, &names).join() {
        if status.target == *player_entity {
            draw_batch.print_color(
                Point::new(50, y),
                &format!("{} ({})", name.name, duration.turns),
                ColorPair::new(red, black),
            );
            y -= 1;
        }
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    let mut draw_batch = DrawBatch::new();
    let player_entity = ecs.fetch::<Entity>();

    box_framework(&mut draw_batch);
    map_label(ecs, &mut draw_batch);
    draw_stats(ecs, &mut draw_batch, &player_entity);
    draw_attributes(ecs, &mut draw_batch, &player_entity);
    initiative_weight(ecs, &mut draw_batch, &player_entity);
    let mut y = equipped(ecs, &mut draw_batch, &player_entity);
    y += consumables(ecs, &mut draw_batch, &player_entity, y);
    spells(ecs, &mut draw_batch, &player_entity, y);
    status(ecs, &mut draw_batch, &player_entity);

    // Draw the log
    gamelog::print_log(
        &mut rltk::BACKEND_INTERNAL.lock().consoles[1].console,
        Point::new(1, 23),
    );

    draw_tooltips(ecs, ctx);

    draw_batch
        .submit(5000)
        .expect("Unable to draw batch from HUD");
}
