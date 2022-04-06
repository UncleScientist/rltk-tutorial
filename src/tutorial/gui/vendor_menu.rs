use super::*;
use rltk::prelude::*;

use crate::{Entity, InBackpack, Item, State, Vendor, VendorMode};

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
    let mut draw_batch = DrawBatch::new();

    let player_entity = gs.ecs.fetch::<Entity>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack)
        .join()
        .filter(|item| item.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    menu_box(
        &mut draw_batch,
        15,
        y,
        (count + 3) as i32,
        "Sell Which Item? (space to switch to buy mode)",
    );

    draw_batch.print_color(
        Point::new(18, y + count as i32 + 1),
        "ESC to cancel",
        ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
    );

    let mut equippable = Vec::new();
    for (j, (entity, _pack, item)) in (&entities, &backpack, &items)
        .join()
        .filter(|item| item.1.owner == *player_entity)
        .enumerate()
    {
        draw_batch.set(
            Point::new(17, y),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
            rltk::to_cp437('('),
        );
        draw_batch.set(
            Point::new(18, y),
            ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
            97 + j as rltk::FontCharType,
        );
        draw_batch.set(
            Point::new(19, y),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
            rltk::to_cp437(')'),
        );

        draw_batch.print_color(
            Point::new(21, y),
            &get_item_display_name(&gs.ecs, entity),
            ColorPair::new(get_item_color(&gs.ecs, entity), RGB::named(rltk::BLACK)),
        );
        draw_batch.print(
            Point::new(50, y),
            &format!("{:.1} gp", item.base_value * 0.8),
        );
        equippable.push(entity);
        y += 1;
    }

    draw_batch
        .submit(6000)
        .expect("Unable to draw Vendor Sell menu");

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
    let mut draw_batch = DrawBatch::new();

    let vendors = gs.ecs.read_storage::<Vendor>();

    let inventory = get_vendor_items(
        &vendors.get(vendor).unwrap().categories,
        &RAWS.lock().unwrap(),
    );
    let count = inventory.len();

    let mut y = (25 - (count / 2)) as i32;
    menu_box(
        &mut draw_batch,
        15,
        y,
        (count + 3) as i32,
        "Buy Which Item? (space to switch to sell mode)",
    );
    draw_batch.print_color(
        Point::new(18, y + count as i32 + 1),
        "ESCAPE to cancel",
        ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
    );

    for (j, sale) in inventory.iter().enumerate() {
        draw_batch.set(
            Point::new(17, y),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
            rltk::to_cp437('('),
        );
        draw_batch.set(
            Point::new(18, y),
            ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
            97 + j as rltk::FontCharType,
        );
        draw_batch.set(
            Point::new(19, y),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
            rltk::to_cp437(')'),
        );

        draw_batch.print(Point::new(21, y), &sale.0);
        draw_batch.print(Point::new(50, y), &format!("{:.1} gp", sale.1 * 0.8));
        y += 1;
    }

    draw_batch
        .submit(6000)
        .expect("Unable to draw Vendor Buy menu");

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
