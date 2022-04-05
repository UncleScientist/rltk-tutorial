use super::*;

use crate::{Entity, Equipped, InBackpack, Item, MasterDungeonMap, Name, ObfuscatedName, State};

pub fn identify_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let items = gs.ecs.read_storage::<Item>();
    let names = gs.ecs.read_storage::<Name>();
    let dm = gs.ecs.fetch::<MasterDungeonMap>();
    let obfuscated = gs.ecs.read_storage::<ObfuscatedName>();
    let white = RGB::named(WHITE);
    let yellow = RGB::named(YELLOW);
    let black = RGB::named(BLACK);

    let build_obfuscated_iterator = || {
        (&entities, &items).join().filter(|(item_entity, _item)| {
            let mut keep = false;
            if let Some(bp) = backpack.get(*item_entity) {
                if bp.owner == *player_entity {
                    if let Some(name) = names.get(*item_entity) {
                        keep = obfuscated.get(*item_entity).is_some()
                            && !dm.identified_items.contains(&name.name);
                    }
                }
            }

            if let Some(equip) = equipped.get(*item_entity) {
                if equip.owner == *player_entity {
                    if let Some(name) = names.get(*item_entity) {
                        keep = obfuscated.get(*item_entity).is_some()
                            && !dm.identified_items.contains(&name.name);
                    }
                }
            }

            keep
        })
    };

    let count = build_obfuscated_iterator().count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 34, (count + 3) as i32, white, black);
    ctx.print_color(18, y - 2, yellow, black, "Identify which item?");
    ctx.print_color(
        18,
        y + count as i32 + 1,
        yellow,
        black,
        "Press ESC to Cancel",
    );

    let mut identifyable: Vec<Entity> = Vec::new();
    for (j, (entity, _item)) in build_obfuscated_iterator().enumerate() {
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
        identifyable.push(entity);
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
                        Some(identifyable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}
