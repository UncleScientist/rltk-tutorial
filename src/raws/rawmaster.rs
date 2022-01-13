use crate::components::*;
use crate::spawner::*;
use crate::RandomTable;
use specs::prelude::*;
use std::collections::{HashMap, HashSet};

use super::Raws;

#[derive(Default)]
pub struct RawMaster {
    raws: Raws,
    item_index: HashMap<String, usize>,
    mob_index: HashMap<String, usize>,
    prop_index: HashMap<String, usize>,
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            ..Default::default()
        }
    }

    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();

        let mut used_names = HashSet::new();

        for (i, item) in self.raws.items.iter().enumerate() {
            if used_names.contains(&item.name) {
                rltk::console::log(format!(
                    "WARNING - duplicate item name in raws [{}]",
                    item.name
                ));
            } else {
                self.item_index.insert(item.name.clone(), i);
                used_names.insert(item.name.clone());
            }
        }

        for (i, mob) in self.raws.mobs.iter().enumerate() {
            if used_names.contains(&mob.name) {
                rltk::console::log(format!(
                    "WARNING - duplicate mob name in raws [{}]",
                    mob.name
                ));
            } else {
                self.mob_index.insert(mob.name.clone(), i);
                used_names.insert(mob.name.clone());
            }
        }

        for (i, props) in self.raws.props.iter().enumerate() {
            if used_names.contains(&props.name) {
                rltk::console::log(format!(
                    "WARNING - duplicate prop name in raws [{}]",
                    props.name
                ));
            } else {
                self.prop_index.insert(props.name.clone(), i);
                used_names.insert(props.name.clone());
            }
        }

        for spawn in self.raws.spawn_table.iter() {
            if !used_names.contains(&spawn.name) {
                rltk::console::log(format!(
                    "WARNING - spawn tables references unspeicified entity {}",
                    spawn.name
                ));
            }
        }
    }
}

pub fn spawn_named_entity(
    raws: &RawMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        spawn_named_item(raws, new_entity, key, pos)
    } else if raws.mob_index.contains_key(key) {
        spawn_named_mob(raws, new_entity, key, pos)
    } else if raws.prop_index.contains_key(key) {
        spawn_named_prop(raws, new_entity, key, pos)
    } else {
        None
    }
}

fn spawn_named_item(
    raws: &RawMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];
        let mut eb = spawn_position(pos, new_entity);

        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        if let Some(weapon) = &item_template.weapon {
            eb = eb.with(Equippable {
                slot: EquipmentSlot::Melee,
            });
            eb = eb.with(MeleePowerBonus {
                power: weapon.power_bonus,
            });
        }

        if let Some(shield) = &item_template.shield {
            eb = eb.with(Equippable {
                slot: EquipmentSlot::Shield,
            });
            eb = eb.with(DefenseBonus {
                power: shield.defense_bonus,
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

fn spawn_named_mob(
    raws: &RawMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.mob_index.contains_key(key) {
        let mob_template = &raws.raws.mobs[raws.mob_index[key]];
        let mut eb = spawn_position(pos, new_entity);

        if let Some(renderable) = &mob_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        eb = eb.with(Name {
            name: mob_template.name.clone(),
        });
        eb = match mob_template.ai.as_ref() {
            "melee" => eb.with(Monster {}),
            "bystander" => eb.with(Bystander {}),
            _ => eb,
        };

        if mob_template.blocks_tile {
            eb = eb.with(BlocksTile {});
        }

        eb = eb.with(CombatStats {
            max_hp: mob_template.stats.max_hp,
            hp: mob_template.stats.hp,
            power: mob_template.stats.power,
            defense: mob_template.stats.defense,
        });
        eb = eb.with(Viewshed {
            visible_tiles: Vec::new(),
            range: mob_template.vision_range,
            dirty: true,
        });

        return Some(eb.build());
    }
    None
}

fn spawn_named_prop(
    raws: &RawMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.prop_index.contains_key(key) {
        let prop_template = &raws.raws.props[raws.prop_index[key]];
        let mut eb = spawn_position(pos, new_entity);

        if let Some(renderable) = &prop_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        eb = eb.with(Name {
            name: prop_template.name.clone(),
        });

        if Some(true) == prop_template.hidden {
            eb = eb.with(Hidden {});
        }

        if Some(true) == prop_template.blocks_tile {
            eb = eb.with(BlocksTile {});
        }

        if Some(true) == prop_template.blocks_visibility {
            eb = eb.with(BlocksVisibility {});
        }

        if let Some(door_open) = prop_template.door_open {
            eb = eb.with(Door { open: door_open })
        }

        if let Some(entry_trigger) = &prop_template.entry_trigger {
            eb = eb.with(EntryTrigger {});
            for effect in entry_trigger.effects.iter() {
                match effect.0.as_str() {
                    "damage" => {
                        eb = eb.with(InflictsDamage {
                            damage: str_to_i32(effect.1),
                        });
                    }
                    "single_activation" => {
                        eb = eb.with(SingleActivation {});
                    }
                    _ => {}
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

fn spawn_position(pos: SpawnType, new_entity: EntityBuilder) -> EntityBuilder {
    let mut eb = new_entity;

    // Spawn in the specified location
    match pos {
        SpawnType::AtPosition { x, y } => {
            eb = eb.with(Position { x, y });
        }
    }

    eb
}

fn get_renderable_component(
    renderable: &super::item_structs::Renderable,
) -> crate::components::Renderable {
    crate::components::Renderable {
        glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
        fg: rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg: rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order: renderable.order,
    }
}

pub fn get_spawn_table_for_depth(raws: &RawMaster, depth: i32) -> RandomTable {
    use super::SpawnTableEntry;

    let available_options: Vec<&SpawnTableEntry> = raws
        .raws
        .spawn_table
        .iter()
        .filter(|a| depth >= a.min_depth && depth <= a.max_depth)
        .collect();

    let mut rt = RandomTable::new();
    for e in available_options.iter() {
        let mut weight = e.weight;
        if e.add_map_depth_to_weight.is_some() {
            weight += depth;
        }
        rt = rt.add(e.name.clone(), weight);
    }

    rt
}
