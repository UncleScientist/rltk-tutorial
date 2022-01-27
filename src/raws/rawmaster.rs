use rltk::RandomNumberGenerator;

use crate::components::*;
use crate::{mana_at_level, npc_hp, RandomTable};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::{HashMap, HashSet};

use super::{parse_dice_string, Raws, Reaction};

pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
    Equipped { by: Entity },
    Carried { by: Entity },
}

#[derive(Default)]
pub struct RawMaster {
    raws: Raws,
    item_index: HashMap<String, usize>,
    mob_index: HashMap<String, usize>,
    prop_index: HashMap<String, usize>,
    loot_index: HashMap<String, usize>,
    faction_index: HashMap<String, HashMap<String, Reaction>>,
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

        for (i, loot) in self.raws.loot_tables.iter().enumerate() {
            self.loot_index.insert(loot.name.clone(), i);
        }

        for faction in self.raws.faction_table.iter() {
            let mut reactions: HashMap<String, Reaction> = HashMap::new();
            for other in faction.responses.iter() {
                reactions.insert(
                    other.0.clone(),
                    match other.1.as_str() {
                        "ignore" => Reaction::Ignore,
                        "flee" => Reaction::Flee,
                        _ => Reaction::Attack,
                    },
                );
            }
            self.faction_index.insert(faction.name.clone(), reactions);
        }
    }
}

pub fn faction_reaction(my_faction: &str, their_faction: &str, raws: &RawMaster) -> Reaction {
    if raws.faction_index.contains_key(my_faction) {
        let mf = &raws.faction_index[my_faction];
        if mf.contains_key(their_faction) {
            return mf[their_faction];
        } else if mf.contains_key("Default") {
            return mf["Default"];
        } else {
            return Reaction::Ignore;
        }
    }
    Reaction::Ignore
}

pub fn get_item_drop(
    raws: &RawMaster,
    rng: &mut RandomNumberGenerator,
    table: &str,
) -> Option<String> {
    if raws.loot_index.contains_key(table) {
        let mut rt = RandomTable::new();
        let available_options = &raws.raws.loot_tables[raws.loot_index[table]];
        for item in available_options.drops.iter() {
            rt = rt.add(item.name.clone(), item.weight);
        }
        return rt.roll(rng);
    }

    None
}

pub fn spawn_named_entity(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        spawn_named_item(raws, ecs, key, pos)
    } else if raws.mob_index.contains_key(key) {
        spawn_named_mob(raws, ecs, key, pos)
    } else if raws.prop_index.contains_key(key) {
        spawn_named_prop(raws, ecs, key, pos)
    } else {
        None
    }
}

fn spawn_named_item(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];

        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();
        eb = spawn_position(pos, eb, key, raws);

        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        if let Some(weapon) = &item_template.weapon {
            eb = eb.with(Equippable {
                slot: EquipmentSlot::Melee,
            });
            let (n_dice, die_type, bonus) = parse_dice_string(&weapon.base_damage);
            let wpn = MeleeWeapon {
                attribute: match weapon.attribute.as_str() {
                    "Quickness" => WeaponAttribute::Quickness,
                    _ => WeaponAttribute::Might,
                },
                damage_n_dice: n_dice,
                damage_die_type: die_type,
                damage_bonus: bonus,
                hit_bonus: weapon.hit_bonus,
            };

            eb = eb.with(wpn);
        }

        if let Some(wearable) = &item_template.wearable {
            let slot = string_to_slot(&wearable.slot);
            eb = eb.with(Equippable { slot });
            eb = eb.with(Wearable {
                slot,
                armor_class: wearable.armor_class,
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

fn spawn_named_mob(raws: &RawMaster, ecs: &mut World, key: &str, pos: SpawnType) -> Option<Entity> {
    if raws.mob_index.contains_key(key) {
        let mob_template = &raws.raws.mobs[raws.mob_index[key]];
        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();
        eb = spawn_position(pos, eb, key, raws);

        if let Some(renderable) = &mob_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        if let Some(quips) = &mob_template.quips {
            eb = eb.with(Quips {
                available: quips.clone(),
            })
        }

        if let Some(light) = &mob_template.light {
            eb = eb.with(LightSource {
                range: light.range,
                color: rltk::RGB::from_hex(&light.color).expect("Bad color"),
            });
        }

        if let Some(faction) = &mob_template.faction {
            eb = eb.with(Faction {
                name: faction.clone(),
            });
        } else {
            eb = eb.with(Faction {
                name: "Mindless".to_string(),
            });
        }

        // why is the mob defaulted to 2?
        eb = eb.with(Initiative { current: 2 });

        let mut attr = Attributes {
            ..Default::default()
        };

        let mut mob_fitness = 11;
        let mut mob_int = 11;
        if let Some(might) = mob_template.attributes.might {
            attr.might = Attribute::new_base(might);
        }

        if let Some(fitness) = mob_template.attributes.fitness {
            attr.fitness = Attribute::new_base(fitness);
            mob_fitness = fitness;
        }

        if let Some(quickness) = mob_template.attributes.quickness {
            attr.quickness = Attribute::new_base(quickness);
        }

        if let Some(intelligence) = mob_template.attributes.intelligence {
            attr.intelligence = Attribute::new_base(intelligence);
            mob_int = intelligence;
        }
        eb = eb.with(attr);

        let mob_level = mob_template.level.unwrap_or(1);
        let mob_hp = npc_hp(mob_fitness, mob_level);
        let mob_mana = mana_at_level(mob_int, mob_level);

        let pools = Pools {
            level: mob_level,
            xp: 0,
            hit_points: Pool {
                current: mob_hp,
                max: mob_hp,
            },
            mana: Pool {
                current: mob_mana,
                max: mob_mana,
            },
        };
        eb = eb.with(pools);

        if let Some(loot) = &mob_template.loot_table {
            eb = eb.with(LootTable {
                table: loot.clone(),
            });
        }

        let mut skills = Skills {
            skills: HashMap::new(),
        };
        skills.skills.insert(Skill::Melee, 1);
        skills.skills.insert(Skill::Defense, 1);
        skills.skills.insert(Skill::Magic, 1);
        if let Some(mobskills) = &mob_template.skills {
            for sk in mobskills.iter() {
                match sk.0.as_str() {
                    "Melee" => {
                        skills.skills.insert(Skill::Melee, *sk.1);
                    }
                    "Defense" => {
                        skills.skills.insert(Skill::Defense, *sk.1);
                    }
                    "Magic" => {
                        skills.skills.insert(Skill::Magic, *sk.1);
                    }
                    _ => {
                        rltk::console::log(format!("Unknown skill referneced: [{}]", sk.0));
                    }
                }
            }
        }
        eb = eb.with(skills);

        eb = eb.with(Name {
            name: mob_template.name.clone(),
        });

        eb = match mob_template.movement.as_ref() {
            "random" => eb.with(MoveMode {
                mode: Movement::Random,
            }),
            _ => eb.with(MoveMode {
                mode: Movement::Static,
            }),
        };

        /*
        eb = match mob_template.ai.as_ref() {
            "melee" => eb.with(Monster {}),
            "bystander" => eb.with(Bystander {}),
            "vendor" => eb.with(Vendor {}),
            "carnivore" => eb.with(Carnivore {}),
            "herbivore" => eb.with(Herbivore {}),
            _ => eb,
        };
        */

        if mob_template.blocks_tile {
            eb = eb.with(BlocksTile {});
        }

        eb = eb.with(Viewshed {
            visible_tiles: Vec::new(),
            range: mob_template.vision_range,
            dirty: true,
        });

        if let Some(na) = &mob_template.natural {
            let mut nature = NaturalAttackDefense {
                armor_class: na.armor_class,
                attacks: Vec::new(),
            };
            if let Some(attacks) = &na.attacks {
                for nattack in attacks.iter() {
                    let (damage_n_dice, damage_die_type, damage_bonus) =
                        parse_dice_string(&nattack.damage);
                    let attack = NaturalAttack {
                        name: nattack.name.clone(),
                        hit_bonus: nattack.hit_bonus,
                        damage_n_dice,
                        damage_die_type,
                        damage_bonus,
                    };
                    nature.attacks.push(attack);
                }
            }
            eb = eb.with(nature);
        }

        let new_mob = eb.build();

        // Are they wielding anything?
        if let Some(wielding) = &mob_template.equipped {
            for tag in wielding.iter() {
                spawn_named_entity(raws, ecs, tag, SpawnType::Equipped { by: new_mob });
            }
        }

        return Some(new_mob);
    }
    None
}

fn spawn_named_prop(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.prop_index.contains_key(key) {
        let prop_template = &raws.raws.props[raws.prop_index[key]];
        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();
        eb = spawn_position(pos, eb, key, raws);

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

fn spawn_position<'a>(
    pos: SpawnType,
    new_entity: EntityBuilder<'a>,
    tag: &str,
    raws: &RawMaster,
) -> EntityBuilder<'a> {
    let mut eb = new_entity;

    // Spawn in the specified location
    match pos {
        SpawnType::AtPosition { x, y } => eb = eb.with(Position { x, y }),
        SpawnType::Carried { by } => eb = eb.with(InBackpack { owner: by }),
        SpawnType::Equipped { by } => {
            let slot = find_slot_for_equippable_item(tag, raws);
            eb = eb.with(Equipped { owner: by, slot })
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

fn find_slot_for_equippable_item(tag: &str, raws: &RawMaster) -> EquipmentSlot {
    if !raws.item_index.contains_key(tag) {
        panic!("Trying to equip an unknown item: {}", tag);
    }

    let item_index = raws.item_index[tag];
    let item = &raws.raws.items[item_index];
    if item.weapon.is_some() {
        return EquipmentSlot::Melee;
    }

    if let Some(wearable) = &item.wearable {
        return string_to_slot(&wearable.slot);
    }

    panic!("Trying to equip {}, but it has no slot tag", tag);
}

fn string_to_slot(slot: &str) -> EquipmentSlot {
    match slot {
        "Shield" => EquipmentSlot::Shield,
        "Head" => EquipmentSlot::Head,
        "Torso" => EquipmentSlot::Torso,
        "Legs" => EquipmentSlot::Legs,
        "Feet" => EquipmentSlot::Feet,
        "Hands" => EquipmentSlot::Hands,
        "Melee" => EquipmentSlot::Melee,
        _ => {
            rltk::console::log(format!("Warning: unknown equipment slot type [{}]", slot));
            EquipmentSlot::Melee
        }
    }
}
