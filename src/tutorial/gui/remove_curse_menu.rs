use super::*;
use rltk::prelude::*;

use crate::{CursedItem, Entity, Equipped, InBackpack, Item, MasterDungeonMap, Name, State};

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
