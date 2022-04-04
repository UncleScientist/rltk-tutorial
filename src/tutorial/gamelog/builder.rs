use super::{append_entry, LogFragment};
use rltk::prelude::*;

pub struct Logger {
    current_color: RGB,
    fragments: Vec<LogFragment>,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            current_color: RGB::named(rltk::WHITE),
            fragments: Vec::new(),
        }
    }

    pub fn color(mut self, color: (u8, u8, u8)) -> Self {
        self.current_color = RGB::named(color);
        self
    }

    pub fn append<T: ToString>(mut self, text: T) -> Self {
        self.fragments.push(LogFragment {
            color: self.current_color,
            text: text.to_string(),
        });
        self
    }

    fn add_fragment(mut self, text: String, color: RGB) -> Self {
        self.fragments.push(LogFragment { color, text });
        self
    }

    pub fn npc_name<T: ToString>(self, text: T) -> Self {
        self.add_fragment(text.to_string(), RGB::named(rltk::YELLOW))
    }

    pub fn item_name<T: ToString>(self, text: T) -> Self {
        self.add_fragment(text.to_string(), RGB::named(rltk::CYAN))
    }

    pub fn damage(self, damage: i32) -> Self {
        self.add_fragment(format!("{damage}"), RGB::named(rltk::RED))
    }

    pub fn log(self) {
        append_entry(self.fragments)
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
