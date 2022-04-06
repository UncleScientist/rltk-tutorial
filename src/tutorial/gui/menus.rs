use rltk::prelude::*;

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
