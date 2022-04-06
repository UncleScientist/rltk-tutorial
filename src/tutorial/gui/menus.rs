use rltk::prelude::*;

use crate::{Entity, ItemMenuResult};

pub fn menu_box<T: ToString>(draw_batch: &mut DrawBatch, x: i32, y: i32, width: i32, title: T) {
    draw_batch.draw_box(
        Rect::with_size(x, y - 2, 31, width),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    draw_batch.print_color(
        Point::new(18, y - 2),
        &title.to_string(),
        ColorPair::new(RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK)),
    );
}

pub fn menu_option<T: ToString>(
    draw_batch: &mut DrawBatch,
    x: i32,
    y: i32,
    hotkey: rltk::FontCharType,
    text: T,
) {
    draw_batch.set(
        Point::new(x, y),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        rltk::to_cp437('('),
    );
    draw_batch.set(
        Point::new(x + 1, y),
        ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
        hotkey,
    );
    draw_batch.set(
        Point::new(x + 2, y),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        rltk::to_cp437(')'),
    );

    draw_batch.print_color(
        Point::new(x + 5, y),
        &text.to_string(),
        ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
    );
}

pub fn item_result_menu<S: ToString>(
    draw_batch: &mut DrawBatch,
    title: S,
    items: &[(Entity, String)],
    key: Option<VirtualKeyCode>,
) -> (ItemMenuResult, Option<Entity>) {
    let white = RGB::named(WHITE);
    let black = RGB::named(BLACK);
    let yellow = RGB::named(YELLOW);
    let count = items.len();

    let mut y = (25 - (count / 2)) as i32;

    draw_batch.draw_box(
        Rect::with_size(15, y - 2, 31, (count + 3) as i32),
        ColorPair::new(white, black),
    );
    draw_batch.print_color(
        Point::new(18, y - 2),
        &title.to_string(),
        ColorPair::new(yellow, black),
    );
    draw_batch.print_color(
        Point::new(18, y + count as i32 + 1),
        "ESCAPE to cancel",
        ColorPair::new(yellow, black),
    );

    let mut item_list = Vec::new();
    for (j, item) in items.iter().enumerate() {
        menu_option(draw_batch, 17, y, 97 + j as rltk::FontCharType, &item.1);
        item_list.push(item.0);
        y += 1;
    }

    match key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(item_list[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}
