use rltk::{Rltk, RGB};
use specs::prelude::*;

pub fn draw_ui(_ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(
        0,
        43,
        79,
        6,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
}
