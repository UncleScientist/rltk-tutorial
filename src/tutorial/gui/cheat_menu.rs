use rltk::prelude::*;

use super::{menu_box, menu_option};
use crate::{Editor, State};

#[derive(PartialEq, Copy, Clone)]
pub enum CheatMenuResult {
    NoResponse,
    Cancel,
    TeleportToExit,
    Heal,
    Reveal,
    Money,
    GodMode,
    SummonItem,
}

#[derive(PartialEq, Copy, Clone)]
pub enum SummonItemResult {
    Cancel,
    NoResponse,
    Done,
}

pub fn summon_item(gs: &mut State, ctx: &mut Rltk) -> (SummonItemResult, String) {
    use VirtualKeyCode::*;

    let mut draw_batch = DrawBatch::new();

    let white = RGB::named(WHITE);
    let black = RGB::named(BLACK);

    menu_box(&mut draw_batch, 15, 22, 4, "Summon Item");

    let mut editor = gs.ecs.fetch_mut::<Editor>();
    let txt = &editor.to_string();
    draw_batch.print_color(Point::new(18, 22), txt, ColorPair::new(white, black));
    draw_batch.print_color(Point::new(16, 22), ">", ColorPair::new(white, black));

    if editor.blink() {
        draw_batch.print_color(
            Point::new(txt.len() + 18, 22),
            " ",
            ColorPair::new(black, white),
        );
    }

    draw_batch.submit(6000).expect("Unable to draw editor menu");

    match ctx.key {
        None => (SummonItemResult::NoResponse, "".to_string()),
        Some(key) => match key {
            A | B | C | D | E | F | G | H | I | J | K | L | M | N | O | P | Q | R | S | T | U
            | V | W | X | Y | Z | Space | Key1 | Key2 | Key3 | Key4 | Key5 | Key6 | Key7 | Key8
            | Key9 | Key0 | Equals | Minus
                if txt.len() < 27 =>
            {
                editor.insert_keycode(key, ctx.shift);
                (SummonItemResult::NoResponse, "".to_string())
            }
            Back => {
                editor.backspace();
                (SummonItemResult::NoResponse, "".to_string())
            }
            Escape => (SummonItemResult::Cancel, "".to_string()),
            Return => (SummonItemResult::Done, editor.to_string()),
            _ => {
                console::log(format!("keycode {:?}", key));
                (SummonItemResult::NoResponse, "".to_string())
            }
        },
    }
}

pub fn show_cheat_mode(_gs: &mut State, ctx: &mut Rltk) -> CheatMenuResult {
    let black = RGB::named(BLACK);
    let yellow = RGB::named(YELLOW);
    let count = 6;

    let mut draw_batch = DrawBatch::new();
    let mut y = (25 - (count / 2)) as i32;
    menu_box(&mut draw_batch, 15, y, (count + 3) as i32, "Cheating!");

    draw_batch.print_color(
        Point::new(18, y + count as i32 + 1),
        "ESCAPE to cancel",
        ColorPair::new(yellow, black),
    );

    menu_option(
        &mut draw_batch,
        17,
        y,
        to_cp437('T'),
        "Teleport to next level",
    );

    y += 1;
    menu_option(&mut draw_batch, 17, y, to_cp437('H'), "Heal all wounds");

    y += 1;
    menu_option(&mut draw_batch, 17, y, to_cp437('R'), "Reveal the map");

    y += 1;
    menu_option(&mut draw_batch, 17, y, to_cp437('G'), "God Mode (No Death)");

    y += 1;
    menu_option(&mut draw_batch, 17, y, to_cp437('M'), "Make some Money");

    y += 1;
    menu_option(&mut draw_batch, 17, y, to_cp437('S'), "Summon item by name");

    draw_batch.submit(6000).expect("Unable to draw cheat menu");

    match ctx.key {
        None => CheatMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::G => CheatMenuResult::GodMode,
            VirtualKeyCode::H => CheatMenuResult::Heal,
            VirtualKeyCode::M => CheatMenuResult::Money,
            VirtualKeyCode::R => CheatMenuResult::Reveal,
            VirtualKeyCode::T => CheatMenuResult::TeleportToExit,
            VirtualKeyCode::S => CheatMenuResult::SummonItem,
            VirtualKeyCode::Escape => CheatMenuResult::Cancel,
            _ => CheatMenuResult::NoResponse,
        },
    }
}
