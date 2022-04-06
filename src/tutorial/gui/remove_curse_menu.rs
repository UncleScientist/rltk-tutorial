use super::*;
use rltk::prelude::*;

use crate::{CursedItem, Entity, Equipped, InBackpack, Item, MasterDungeonMap, Name, State};

pub fn remove_curse_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let item_components = gs.ecs.read_storage::<Item>();
    let cursed = gs.ecs.read_storage::<CursedItem>();
    let names = gs.ecs.read_storage::<Name>();
    let dm = gs.ecs.fetch::<MasterDungeonMap>();

    let mut draw_batch = DrawBatch::new();

    let mut items = Vec::new();
    (&entities, &item_components, &cursed)
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
        .for_each(|item| items.push((item.0, get_item_display_name(&gs.ecs, item.0))));

    let result = item_result_menu(&mut draw_batch, "Inventory", &items, ctx.key);
    draw_batch
        .submit(6000)
        .expect("Unable to draw Remove Curse menu");

    result
}
