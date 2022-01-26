use regex::Regex;
use serde::Deserialize;

mod faction_structs;
use faction_structs::*;

mod item_structs;
use item_structs::*;

mod mob_structs;
use mob_structs::*;

mod props_structs;
use props_structs::*;

mod spawn_table_structs;
use spawn_table_structs::*;

mod loot_structs;
use loot_structs::*;

mod rawmaster;
pub use rawmaster::*;

use std::sync::Mutex;

#[derive(Deserialize, Debug, Default)]
pub struct Raws {
    pub items: Vec<Item>,
    pub mobs: Vec<Mob>,
    pub props: Vec<Prop>,
    pub spawn_table: Vec<SpawnTableEntry>,
    pub loot_tables: Vec<LootTable>,
    pub faction_table: Vec<FactionInfo>,
}

use lazy_static::lazy_static;

rltk::embedded_resource!(RAW_FILE, "../../raws/spawns.json");

lazy_static! {
    pub static ref RAWS: Mutex<RawMaster> = Mutex::new(RawMaster::empty());
}

pub fn load_raws() {
    rltk::link_resource!(RAW_FILE, "../../raws/spawns.json");

    let raw_data = rltk::embedding::EMBED
        .lock()
        .get_resource("../../raws/spawns.json".to_string())
        .unwrap();
    let raw_string = std::str::from_utf8(raw_data).expect("Unable to convert a valid UTF-8 String");

    let decoder: Raws = serde_json::from_str(raw_string).expect("Unable to parse JSON");

    RAWS.lock().unwrap().load(decoder);
}

pub fn parse_dice_string(dice: &str) -> (i32, i32, i32) {
    lazy_static! {
        static ref DICE_RE: Regex = Regex::new(r"(\d+)d(\d+)([\+\-]\d+)?").unwrap();
    }

    let mut n_dice = 1;
    let mut die_type = 4;
    let mut die_bonus = 0;

    for cap in DICE_RE.captures_iter(dice) {
        if let Some(group) = cap.get(1) {
            n_dice = group.as_str().parse::<i32>().expect("not a digit");
        }
        if let Some(group) = cap.get(2) {
            die_type = group.as_str().parse::<i32>().expect("not a digit");
        }
        if let Some(group) = cap.get(3) {
            die_bonus = group.as_str().parse::<i32>().expect("not a digit");
        }
    }
    (n_dice, die_type, die_bonus)
}
