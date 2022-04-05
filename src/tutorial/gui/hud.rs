use super::*;

use crate::{
    gamelog, Attribute, Attributes, Consumable, Duration, Equipped, HungerClock, HungerState,
    InBackpack, KnownSpells, Map, Name, Pools, StatusEffect, Weapon,
};

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
    let cyan: RGB = RGB::named(CYAN);
    let yellow: RGB = RGB::named(YELLOW);
    let green: RGB = RGB::from_f32(0., 1., 0.);
    let orange = RGB::named(ORANGE);
    let gold = RGB::named(GOLD);

    ctx.draw_hollow_box(0, 0, 79, 59, box_grey, black);
    ctx.draw_hollow_box(0, 0, 49, 45, box_grey, black);
    ctx.draw_hollow_box(0, 45, 79, 14, box_grey, black);
    ctx.draw_hollow_box(49, 0, 30, 8, box_grey, black);

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
    let weapon = ecs.read_storage::<Weapon>();
    for (entity, equipped_by) in (&entities, &equipped).join() {
        if equipped_by.owner == *player_entity {
            let name = get_item_display_name(ecs, entity);
            ctx.print_color(
                50,
                y,
                get_item_color(ecs, entity),
                black,
                &get_item_display_name(ecs, entity),
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
                ctx.print_color(3, 45, yellow, black, &weapon_info);
            }
        }
    }

    // Consumables
    y += 1;
    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    for (index, (entity, carried_by, _)) in (&entities, &backpack, &consumables).join().enumerate()
    {
        if carried_by.owner == *player_entity && index < 10 {
            ctx.print_color(50, y, yellow, black, &format!("↑{}", index + 1));
            ctx.print_color(
                53,
                y,
                get_item_color(ecs, entity),
                black,
                &get_item_display_name(ecs, entity),
            );
            y += 1;
        }
    }

    // Spells
    y += 1;
    let known_spells_storage = ecs.read_storage::<KnownSpells>();
    let known_spells = &known_spells_storage.get(*player_entity).unwrap().spells;
    for (index, spell) in known_spells.iter().enumerate() {
        ctx.print_color(50, y, cyan, black, &format!("^{}", index + 1));
        ctx.print_color(
            53,
            y,
            cyan,
            black,
            &format!("{} ({})", spell.display_name, spell.mana_cost),
        );
        y += 1;
    }

    // Status
    let mut y = 44;
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();
    match hc.state {
        HungerState::WellFed => {
            ctx.print_color(50, 44, green, black, "Well Fed");
            y -= 1
        }

        HungerState::Normal => {}
        HungerState::Hungry => {
            ctx.print_color(50, 44, orange, black, "Hungry");
            y -= 1
        }
        HungerState::Starving => {
            ctx.print_color(50, 44, red, black, "Starving");
            y -= 1
        }
    }
    let statuses = ecs.read_storage::<StatusEffect>();
    let durations = ecs.read_storage::<Duration>();
    let names = ecs.read_storage::<Name>();
    for (status, duration, name) in (&statuses, &durations, &names).join() {
        if status.target == *player_entity {
            ctx.print_color(
                50,
                y,
                red,
                black,
                &format!("{} ({})", name.name, duration.turns),
            );
            y -= 1;
        }
    }

    // Draw the log
    gamelog::print_log(
        &mut rltk::BACKEND_INTERNAL.lock().consoles[1].console,
        Point::new(1, 23),
    );

    draw_tooltips(ecs, ctx);
}
