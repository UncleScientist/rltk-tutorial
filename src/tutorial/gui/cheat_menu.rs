use rltk::prelude::*;

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

    let white = RGB::named(WHITE);
    let black = RGB::named(BLACK);
    let yellow = RGB::named(YELLOW);

    ctx.draw_box(15, 20, 31, 4, white, black);
    ctx.print_color(18, 20, yellow, black, "Summon Item");

    let mut editor = gs.ecs.fetch_mut::<Editor>();
    let txt = &editor.to_string();
    ctx.print(18, 22, txt);
    ctx.print(16, 22, ">");

    if editor.blink() {
        ctx.print_color(txt.len() + 18, 22, black, white, " ");
    }

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
    let white = RGB::named(WHITE);
    let black = RGB::named(BLACK);
    let yellow = RGB::named(YELLOW);
    let count = 6;
    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, white, black);
    ctx.print_color(18, y - 2, yellow, black, "Cheating!");
    ctx.print_color(18, y + count as i32 + 1, yellow, black, "ESC to cancel");

    ctx.set(17, y, white, black, to_cp437('('));
    ctx.set(18, y, white, black, to_cp437('T'));
    ctx.set(19, y, white, black, to_cp437(')'));
    ctx.print(21, y, "Teleport to exit");

    y += 1;
    ctx.set(17, y, white, black, to_cp437('('));
    ctx.set(18, y, white, black, to_cp437('H'));
    ctx.set(19, y, white, black, to_cp437(')'));
    ctx.print(21, y, "Heal all wounds");

    y += 1;
    ctx.set(17, y, white, black, to_cp437('('));
    ctx.set(18, y, white, black, to_cp437('R'));
    ctx.set(19, y, white, black, to_cp437(')'));
    ctx.print(21, y, "Reveal the map");

    y += 1;
    ctx.set(17, y, white, black, to_cp437('('));
    ctx.set(18, y, white, black, to_cp437('G'));
    ctx.set(19, y, white, black, to_cp437(')'));
    ctx.print(21, y, "God Mode (no death)");

    y += 1;
    ctx.set(17, y, white, black, to_cp437('('));
    ctx.set(18, y, white, black, to_cp437('M'));
    ctx.set(19, y, white, black, to_cp437(')'));
    ctx.print(21, y, "Make some Money");

    y += 1;
    ctx.set(17, y, white, black, to_cp437('('));
    ctx.set(18, y, white, black, to_cp437('S'));
    ctx.set(19, y, white, black, to_cp437(')'));
    ctx.print(21, y, "Summon item by name");

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
