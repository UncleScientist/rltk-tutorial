use super::{get_item_display_name, item_result_menu, ItemMenuResult};
use crate::{InBackpack, State};
use rltk::prelude::*;
use specs::prelude::*;

pub fn drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    // show_menu::<InBackpack>(gs, ctx, "Drop Which Item?")

    let mut draw_batch = DrawBatch::new();
    let player_entity = gs.ecs.fetch::<Entity>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let mut items = Vec::new();
    (&entities, &backpack)
        .join()
        .filter(|item| item.1.owner == *player_entity)
        .for_each(|item| items.push((item.0, get_item_display_name(&gs.ecs, item.0))));

    let result = item_result_menu(&mut draw_batch, "Drop which item?", &items, ctx.key);

    draw_batch
        .submit(6000)
        .expect("Unable to draw Drop Item menu");

    result
}
