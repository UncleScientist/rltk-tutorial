use crate::components::*;
use crate::spawner::*;
use specs::prelude::*;
use std::collections::HashMap;

use super::Raws;

pub struct RawMaster {
    raws: Raws,
    pub item_index: HashMap<String, usize>,
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws: Raws { items: Vec::new() },
            item_index: HashMap::new(),
        }
    }

    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();
        for (i, item) in self.raws.items.iter().enumerate() {
            self.item_index.insert(item.name.clone(), i);
        }
    }
}

pub fn spawn_named_item(
    raws: &RawMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];
        let mut eb = new_entity;

        // Spawn in the specified location
        match pos {
            SpawnType::AtPosition { x, y } => {
                eb = eb.with(Position { x, y });
            }
        }

        // Renderable
        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(Renderable {
                glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
                fg: rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
                bg: rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
                render_order: renderable.order,
            });
        }

        eb = eb.with(Name {
            name: item_template.name.clone(),
        });
        eb = eb.with(Item {});

        if let Some(consumable) = &item_template.consumable {
            eb = eb.with(Consumable {});
            for effect in consumable.effects.iter() {
                let effect_name = effect.0.as_str();
                match effect_name {
                    "provides_healing" => {
                        eb = eb.with(ProvidesHealing {
                            heal_amount: str_to_i32(effect.1),
                        })
                    }
                    "ranged" => {
                        eb = eb.with(Ranged {
                            range: str_to_i32(effect.1),
                        })
                    }
                    "damage" => {
                        eb = eb.with(InflictsDamage {
                            damage: str_to_i32(effect.1),
                        })
                    }
                    "area_of_effect" => {
                        eb = eb.with(AreaOfEffect {
                            radius: str_to_i32(effect.1),
                        })
                    }
                    "confusion" => {
                        eb = eb.with(Confusion {
                            turns: str_to_i32(effect.1),
                        })
                    }
                    "magic_mapping" => eb = eb.with(MagicMapper {}),
                    "food" => eb = eb.with(ProvidesFood {}),
                    _ => {
                        rltk::console::log(format!(
                            "Warning: consumable effect {} not implemented",
                            effect_name
                        ));
                    }
                }
            }
        }

        return Some(eb.build());
    }
    None
}

fn str_to_i32(s: &str) -> i32 {
    s.parse::<i32>().unwrap()
}
