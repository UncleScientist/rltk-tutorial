use specs::prelude::*;

pub const SHOW_MAPGEN_VISUALIZER: i32 = -1;

pub mod ai;
pub use ai::*;

pub mod map;
pub use map::*;

pub mod map_builders;
pub use map_builders::*;

pub mod raws;
pub use raws::*;

pub mod spatial;
pub use spatial::*;

pub mod tutorial;
pub use tutorial::*;

pub mod inventory_system;
pub use inventory_system::*;

pub mod effects;
pub use effects::*;

// --------------------------------------------------------------------------------

use rltk::VirtualKeyCode::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Editor {
    text: String,
    keymap: HashMap<rltk::VirtualKeyCode, char>,
    cursor: usize,
}

impl Editor {
    pub fn new() -> Self {
        let keymap = HashMap::from([
            (A, 'a'),
            (B, 'b'),
            (C, 'c'),
            (D, 'd'),
            (E, 'e'),
            (F, 'f'),
            (G, 'g'),
            (H, 'h'),
            (I, 'i'),
            (J, 'j'),
            (K, 'k'),
            (L, 'l'),
            (M, 'm'),
            (N, 'n'),
            (O, 'o'),
            (P, 'p'),
            (Q, 'q'),
            (R, 'r'),
            (S, 's'),
            (T, 't'),
            (U, 'u'),
            (V, 'v'),
            (W, 'w'),
            (X, 'x'),
            (Y, 'y'),
            (Z, 'z'),
            (Space, ' '),
            (Key0, '0'),
            (Key1, '1'),
            (Key2, '2'),
            (Key3, '3'),
            (Key4, '4'),
            (Key5, '5'),
            (Key6, '6'),
            (Key7, '7'),
            (Key8, '8'),
            (Key9, '9'),
            (Minus, '-'),
        ]);
        Self {
            text: "".to_string(),
            keymap,
            cursor: 0,
        }
    }

    pub fn blink(&mut self) -> bool {
        if self.cursor >= 60 {
            self.cursor = 0;
        } else {
            self.cursor += 1;
        }

        self.cursor < 30
    }

    pub fn insert_keycode(&mut self, code: rltk::VirtualKeyCode, is_shift: bool) {
        if is_shift {
            if code == rltk::VirtualKeyCode::Equals {
                self.text.push('+')
            } else {
                self.text.extend(
                    self.keymap
                        .get(&code)
                        .unwrap()
                        .to_uppercase()
                        .collect::<Vec<_>>(),
                );
            }
        } else {
            self.text.push(*self.keymap.get(&code).unwrap())
        }
        self.cursor = 0;
    }

    pub fn backspace(&mut self) {
        if !self.text.is_empty() {
            let last = self.text.len() - 1;
            self.text = self.text[0..last].to_string();
        }
        self.cursor = 0;
    }
}

impl ToString for Editor {
    fn to_string(&self) -> String {
        self.text.clone()
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
