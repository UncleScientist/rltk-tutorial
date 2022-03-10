use rltk::RandomNumberGenerator;

use crate::components::*;
use crate::{mana_at_level, npc_hp, MasterTable, RandomTable};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::{HashMap, HashSet};

use super::{parse_dice_string, Raws, Reaction};

pub enum SpawnTableType {
    Item,
    Mob,
    Prop,
}

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
    spell_index: HashMap<String, usize>,
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

        for (i, spell) in self.raws.spells.iter().enumerate() {
            self.spell_index.insert(spell.name.clone(), i);
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

pub fn get_scroll_tags() -> Vec<String> {
    let raws = &super::RAWS.lock().unwrap();
    let mut result = Vec::new();

    for item in raws.raws.items.iter() {
        if let Some(magic) = &item.magic {
            if &magic.naming == "scroll" {
                result.push(item.name.clone());
            }
        }
    }

    result
}

pub fn get_potion_tags() -> Vec<String> {
    let raws = &super::RAWS.lock().unwrap();
    let mut result = Vec::new();

    for item in raws.raws.items.iter() {
        if let Some(magic) = &item.magic {
            if &magic.naming == "potion" {
                result.push(item.name.clone());
            }
        }
    }

    result
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
            rt.add(item.name.clone(), item.weight);
        }
        return Some(rt.roll(rng));
    }

    None
}

pub fn get_vendor_items(categories: &[String], raws: &RawMaster) -> Vec<(String, f32)> {
    let mut result = Vec::new();

    for item in raws.raws.items.iter() {
        if let Some(cat) = &item.vendor_category {
            if categories.contains(cat) && item.base_value.is_some() {
                result.push((item.name.clone(), item.base_value.unwrap()));
            }
        }
    }

    result
}

pub fn is_tag_magic(tag: &str) -> bool {
    let raws = &super::RAWS.lock().unwrap();
    if raws.item_index.contains_key(tag) {
        let item_template = &raws.raws.items[raws.item_index[tag]];
        item_template.magic.is_some()
    } else {
        false
    }
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

fn parse_particle_line(n: &str) -> SpawnParticleLine {
    let tokens: Vec<_> = n.split(';').collect();
    SpawnParticleLine {
        glyph: rltk::to_cp437(tokens[0].chars().next().unwrap()),
        color: rltk::RGB::from_hex(tokens[1]).expect("Bad RGB"),
        lifetime_ms: tokens[2].parse::<f32>().unwrap(),
    }
}

fn parse_particle(n: &str) -> SpawnParticleBurst {
    let tokens: Vec<_> = n.split(';').collect();
    SpawnParticleBurst {
        glyph: rltk::to_cp437(tokens[0].chars().next().unwrap()),
        color: rltk::RGB::from_hex(tokens[1]).expect("Bad RGB"),
        lifetime_ms: tokens[2].parse::<f32>().unwrap(),
    }
}

macro_rules! apply_effects {
    ( $effects:expr, $eb:expr ) => {
        for effect in $effects.iter() {
            let effect_name = effect.0.as_str();
            match effect_name {
                "teach_spell" => {
                    $eb = $eb.with(TeachesSpell {
                        spell: effect.1.to_string(),
                    })
                }
                "provides_healing" => {
                    $eb = $eb.with(ProvidesHealing {
                        heal_amount: effect.1.parse::<i32>().unwrap(),
                    });
                }
                "provides_mana" => {
                    $eb = $eb.with(ProvidesMana {
                        mana_amount: effect.1.parse::<i32>().unwrap(),
                    });
                }
                "ranged" => {
                    $eb = $eb.with(Ranged {
                        range: effect.1.parse::<i32>().unwrap(),
                    });
                }
                "damage" => {
                    $eb = $eb.with(InflictsDamage {
                        damage: effect.1.parse::<i32>().unwrap(),
                    });
                }
                "area_of_effect" => {
                    $eb = $eb.with(AreaOfEffect {
                        radius: effect.1.parse::<i32>().unwrap(),
                    });
                }
                "confusion" => {
                    $eb = $eb.with(Confusion {});
                    $eb = $eb.with(Duration {
                        turns: effect.1.parse::<i32>().unwrap(),
                    });
                }
                "town_portal" => $eb = $eb.with(TownPortal {}),
                "magic_mapping" => $eb = $eb.with(MagicMapper {}),
                "food" => $eb = $eb.with(ProvidesFood {}),
                "single_activation" => $eb = $eb.with(SingleActivation {}),
                "particle_line" => $eb = $eb.with(parse_particle_line(&effect.1)),
                "particle" => $eb = $eb.with(parse_particle(&effect.1)),
                "remove_curse" => $eb = $eb.with(ProvidesRemoveCurse {}),
                "identify" => $eb = $eb.with(ProvidesIdentification {}),
                "slow" => {
                    $eb = $eb.with(Slow {
                        initiative_penalty: effect.1.parse::<f32>().unwrap(),
                    })
                }
                "damage_over_time" => {
                    $eb = $eb.with(DamageOverTime {
                        damage: effect.1.parse::<i32>().unwrap(),
                    })
                }
                _ => rltk::console::log(format!(
                    "Warning: consumable effect {} not implemented",
                    effect_name
                )),
            }
        }
    };
}

pub fn find_spell_entity(ecs: &World, name: &str) -> Option<Entity> {
    let names = ecs.read_storage::<Name>();
    let spell_templates = ecs.read_storage::<SpellTemplate>();
    let entities = ecs.entities();

    for (entity, sname, _template) in (&entities, &names, &spell_templates).join() {
        if name == sname.name {
            return Some(entity);
        }
    }

    None
}

pub fn find_spell_entity_by_name(
    name: &str,
    names: &ReadStorage<Name>,
    spell_templates: &ReadStorage<SpellTemplate>,
    entities: &Entities,
) -> Option<Entity> {
    for (entity, sname, _template) in (entities, names, spell_templates).join() {
        if name == sname.name {
            return Some(entity);
        }
    }

    None
}

pub fn spawn_all_spells(ecs: &mut World) {
    let raws = &super::RAWS.lock().unwrap();
    for spell in raws.raws.spells.iter() {
        spawn_named_spell(raws, ecs, &spell.name);
    }
}

pub fn spawn_type_by_name(raws: &RawMaster, key: &str) -> SpawnTableType {
    if raws.item_index.contains_key(key) {
        SpawnTableType::Item
    } else if raws.mob_index.contains_key(key) {
        SpawnTableType::Mob
    } else {
        SpawnTableType::Prop
    }
}

// ---------------------------------------------------------------------------------

fn spawn_named_spell(raws: &RawMaster, ecs: &mut World, key: &str) -> Option<Entity> {
    if raws.spell_index.contains_key(key) {
        let spell_template = &raws.raws.spells[raws.spell_index[key]];

        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();
        eb = eb.with(SpellTemplate {
            mana_cost: spell_template.mana_cost,
        });
        eb = eb.with(Name {
            name: spell_template.name.clone(),
        });
        apply_effects!(spell_template.effects, eb);

        return Some(eb.build());
    }

    None
}

fn spawn_named_item(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];
        let dm = ecs.fetch::<crate::map::MasterDungeonMap>();
        let scroll_names = dm.scroll_mappings.clone();
        let potion_names = dm.potion_mappings.clone();
        let identified = dm.identified_items.clone();
        std::mem::drop(dm);

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
                proc_chance: weapon.proc_chance,
                proc_target: weapon.proc_target.clone(),
            };
            eb = eb.with(wpn);
            if let Some(proc_effects) = &weapon.proc_effects {
                apply_effects!(proc_effects, eb);
            }
        }

        if let Some(magic) = &item_template.magic {
            let class = match magic.class.as_str() {
                "rare" => MagicItemClass::Rare,
                "legendary" => MagicItemClass::Legendary,
                _ => MagicItemClass::Common,
            };
            eb = eb.with(MagicItem { class });

            if !identified.contains(&item_template.name) {
                match magic.naming.as_str() {
                    "scroll" => {
                        eb = eb.with(ObfuscatedName {
                            name: scroll_names[&item_template.name].clone(),
                        });
                    }
                    "potion" => {
                        eb = eb.with(ObfuscatedName {
                            name: potion_names[&item_template.name].clone(),
                        });
                    }
                    _ => {
                        eb = eb.with(ObfuscatedName {
                            name: magic.naming.clone(),
                        });
                    }
                }
            }

            if let Some(cursed) = magic.cursed {
                if cursed {
                    eb = eb.with(CursedItem {});
                }
            }
        }

        if let Some(wearable) = &item_template.wearable {
            let slot = string_to_slot(&wearable.slot);
            eb = eb.with(Equippable { slot });
            eb = eb.with(Wearable {
                slot,
                armor_class: wearable.armor_class,
            });
        }

        if let Some(ab) = &item_template.attributes {
            eb = eb.with(AttributeBonus {
                might: ab.might,
                fitness: ab.fitness,
                quickness: ab.quickness,
                intelligence: ab.intelligence,
            });
        }

        eb = eb.with(Name {
            name: item_template.name.clone(),
        });

        eb = eb.with(crate::components::Item {
            initiative_penalty: item_template.initiative_penalty.unwrap_or(0.0),
            weight_lbs: item_template.weight_lbs.unwrap_or(0.0),
            base_value: item_template.base_value.unwrap_or(0.0),
        });

        if let Some(consumable) = &item_template.consumable {
            let max_charges = consumable.charges.unwrap_or(0);
            eb = eb.with(Consumable {
                max_charges,
                charges: i32::max(1, max_charges),
            });
            apply_effects!(consumable.effects, eb);
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

        if let Some(ability_list) = &mob_template.abilities {
            let mut a = SpecialAbilities {
                abilities: Vec::new(),
            };
            for ability in ability_list.iter() {
                a.abilities.push(SpecialAbility {
                    chance: ability.chance,
                    spell: ability.spell.clone(),
                    range: ability.range,
                    min_range: ability.min_range,
                });
            }
            eb = eb.with(a);
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

        if let Some(vendor) = &mob_template.vendor {
            eb = eb.with(Vendor {
                categories: vendor.clone(),
            });
        }

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
            total_weight: 0.0,
            total_initiative_penalty: 0.0,
            gold: if let Some(gold) = &mob_template.gold {
                let mut rng = rltk::RandomNumberGenerator::new();
                let (n, d, b) = parse_dice_string(gold);
                (rng.roll_dice(n, d) + b) as f32
            } else {
                0.0
            },
            god_mode: false,
        };
        eb = eb.with(pools);
        eb = eb.with(EquipmentChanged {});

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
            "random_waypoint" => eb.with(MoveMode {
                mode: Movement::RandomWaypoint { path: None },
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

        if let Some(light) = &prop_template.light {
            eb = eb.with(LightSource {
                range: light.range,
                color: rltk::RGB::from_hex(&light.color).expect("Bad color"),
            });
            eb = eb.with(Viewshed {
                range: light.range,
                dirty: true,
                visible_tiles: Vec::new(),
            });
        }

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
            apply_effects!(entry_trigger.effects, eb);
        }

        return Some(eb.build());
    }
    None
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

pub fn get_spawn_table_for_depth(raws: &RawMaster, depth: i32) -> MasterTable {
    use super::SpawnTableEntry;

    let available_options: Vec<&SpawnTableEntry> = raws
        .raws
        .spawn_table
        .iter()
        .filter(|a| depth >= a.min_depth && depth <= a.max_depth)
        .collect();

    let mut rt = MasterTable::new();
    for e in available_options.iter() {
        let mut weight = e.weight;
        if e.add_map_depth_to_weight.is_some() {
            weight += depth;
        }
        rt.add(e.name.clone(), weight, raws);
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
