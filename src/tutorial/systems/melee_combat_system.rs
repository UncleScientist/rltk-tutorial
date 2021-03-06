use crate::{
    effects::*, skill_bonus, Attributes, EquipmentSlot, Equipped, HungerClock, HungerState, Name,
    NaturalAttackDefense, Pools, Skill, Skills, WantsToMelee, Weapon, WeaponAttribute, Wearable,
};

use specs::prelude::*;

pub struct MeleeCombatSystem {}

type MeleeCombatData<'a> = (
    Entities<'a>,
    WriteStorage<'a, WantsToMelee>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, Attributes>,
    ReadStorage<'a, Skills>,
    WriteStorage<'a, HungerClock>,
    ReadStorage<'a, Pools>,
    ReadStorage<'a, Equipped>,
    ReadStorage<'a, Weapon>,
    ReadStorage<'a, Wearable>,
    ReadStorage<'a, NaturalAttackDefense>,
);

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = MeleeCombatData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_melee,
            names,
            attributes,
            skills,
            hunger_clock,
            pools,
            equipped_items,
            meleeweapons,
            wearables,
            natural,
        ) = data;

        for (entity, wants_melee, name, attacker_attributes, attacker_skills, attacker_pools) in (
            &entities,
            &wants_melee,
            &names,
            &attributes,
            &skills,
            &pools,
        )
            .join()
        {
            let target_pools = pools.get(wants_melee.target).unwrap();
            let target_attributes = attributes.get(wants_melee.target).unwrap();
            let target_skills = skills.get(wants_melee.target).unwrap();

            let mut weapon_info = Weapon {
                range: None,
                attribute: WeaponAttribute::Might,
                hit_bonus: 0,
                damage_n_dice: 1,
                damage_die_type: 4,
                damage_bonus: 0,
                proc_chance: None,
                proc_target: None,
            };

            if let Some(nat) = natural.get(entity) {
                if let Some(attack_index) = crate::tutorial::rng::random_slice_index(&nat.attacks) {
                    weapon_info.hit_bonus = nat.attacks[attack_index].hit_bonus;
                    weapon_info.damage_n_dice = nat.attacks[attack_index].damage_n_dice;
                    weapon_info.damage_die_type = nat.attacks[attack_index].damage_die_type;
                    weapon_info.damage_bonus = nat.attacks[attack_index].damage_bonus;
                }
            }

            // Find the wielded weapon
            let mut weapon_entity: Option<Entity> = None;
            for (wielder, wielded, melee) in (&entities, &equipped_items, &meleeweapons).join() {
                if wielded.owner == entity && wielded.slot == EquipmentSlot::Melee {
                    weapon_info = melee.clone();
                    weapon_entity = Some(wielder);
                }
            }

            if attacker_pools.hit_points.current > 0 && target_pools.hit_points.current > 0 {
                let target_name = names.get(wants_melee.target).unwrap();

                let natural_roll = crate::tutorial::rng::roll_dice(1, 20);
                let attribute_hit_bonus = if weapon_info.attribute == WeaponAttribute::Might {
                    attacker_attributes.might.bonus
                } else {
                    attacker_attributes.quickness.bonus
                };
                let skill_hit_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                let weapon_hit_bonus = weapon_info.hit_bonus;

                let mut status_hit_bonus = 0;
                if let Some(hc) = hunger_clock.get(entity) {
                    // Well-fed grants +1
                    if hc.state == HungerState::WellFed {
                        status_hit_bonus += 1;
                    }
                }

                let modified_hit_roll = natural_roll
                    + attribute_hit_bonus
                    + skill_hit_bonus
                    + weapon_hit_bonus
                    + status_hit_bonus;

                let mut armor_item_bonus_f = 0.0;
                for (wielded, armor) in (&equipped_items, &wearables).join() {
                    if wielded.owner == wants_melee.target {
                        armor_item_bonus_f += armor.armor_class;
                    }
                }

                let base_armor_class = match natural.get(wants_melee.target) {
                    None => 10,
                    Some(nat) => nat.armor_class.unwrap_or(10),
                };
                let armor_quickness_bonus = target_attributes.quickness.bonus;
                let armor_skill_bonus = skill_bonus(Skill::Defense, &*target_skills);
                let armor_item_bonus = armor_item_bonus_f as i32;
                let armor_class =
                    base_armor_class + armor_quickness_bonus + armor_skill_bonus + armor_item_bonus;

                if natural_roll != 1 && (natural_roll == 20 || modified_hit_roll > armor_class) {
                    let base_damage = crate::tutorial::rng::roll_dice(
                        weapon_info.damage_n_dice,
                        weapon_info.damage_die_type,
                    );
                    let attr_damage_bonus = attacker_attributes.might.bonus;
                    let skill_damage_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                    let weapon_damage_bonus = weapon_info.damage_bonus;

                    let damage = i32::max(
                        0,
                        base_damage
                            + attr_damage_bonus
                            + skill_hit_bonus
                            + skill_damage_bonus
                            + weapon_damage_bonus,
                    );
                    add_effect(
                        Some(entity),
                        EffectType::Damage { amount: damage },
                        Targets::Single {
                            target: wants_melee.target,
                        },
                    );
                    crate::gamelog::Logger::new()
                        .npc_name(&name.name)
                        .color(rltk::WHITE)
                        .append("hits")
                        .npc_name(&target_name.name)
                        .color(rltk::WHITE)
                        .append("for")
                        .damage(damage)
                        .color(rltk::WHITE)
                        .append("hp.")
                        .log();

                    // Proc effects
                    if let Some(chance) = &weapon_info.proc_chance {
                        if crate::tutorial::rng::roll_dice(1, 100) <= (chance * 100.0) as i32 {
                            let effect_target = if let Some(weapon_target) = weapon_info.proc_target
                            {
                                if weapon_target == "Self" {
                                    Targets::Single { target: entity }
                                } else {
                                    Targets::Single {
                                        target: wants_melee.target,
                                    }
                                }
                            } else {
                                Targets::Single {
                                    target: wants_melee.target,
                                }
                            };
                            add_effect(
                                Some(entity),
                                EffectType::ItemUse {
                                    item: weapon_entity.unwrap(),
                                },
                                effect_target,
                            );
                        }
                    }
                } else if natural_roll == 1 {
                    // Natural 1 miss
                    crate::gamelog::Logger::new()
                        .item_name(&name.name)
                        .color(rltk::WHITE)
                        .append("considers attacking")
                        .item_name(&target_name.name)
                        .color(rltk::WHITE)
                        .append("but misjudges the timing!")
                        .log();
                    add_effect(
                        Some(entity),
                        EffectType::Particle {
                            glyph: rltk::to_cp437('???'),
                            fg: rltk::RGB::named(rltk::BLUE),
                            bg: rltk::RGB::named(rltk::BLACK),
                            lifespan: 200.0,
                        },
                        Targets::Single {
                            target: wants_melee.target,
                        },
                    );
                } else {
                    // Miss
                    crate::gamelog::Logger::new()
                        .item_name(&name.name)
                        .color(rltk::WHITE)
                        .append("attacks")
                        .item_name(&target_name.name)
                        .color(rltk::WHITE)
                        .append("but can't connect")
                        .log();
                    add_effect(
                        Some(entity),
                        EffectType::Particle {
                            glyph: rltk::to_cp437('???'),
                            fg: rltk::RGB::named(rltk::CYAN),
                            bg: rltk::RGB::named(rltk::BLACK),
                            lifespan: 200.0,
                        },
                        Targets::Single {
                            target: wants_melee.target,
                        },
                    );
                }
            }
        }

        wants_melee.clear();
    }
}
