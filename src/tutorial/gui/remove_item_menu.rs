use super::{get_item_display_name, item_result_menu, ItemMenuResult};
use crate::{Equipped, State};
use rltk::prelude::*;
use specs::prelude::*;

pub fn remove_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let mut draw_batch = DrawBatch::new();
    let player_entity = gs.ecs.fetch::<Entity>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    let mut items = Vec::new();
    (&entities, &equipped)
        .join()
        .filter(|item| item.1.owner == *player_entity)
        .for_each(|item| items.push((item.0, get_item_display_name(&gs.ecs, item.0))));

    let result = item_result_menu(&mut draw_batch, "Remove which item?", &items, ctx.key);

    draw_batch
        .submit(6000)
        .expect("Unable to draw Remove Item menu");

    result
}
