use crate::{
    effects::*, skill_bonus, Attributes, EquipmentSlot, Equipped, HungerClock, HungerState, Map,
    Name, NaturalAttackDefense, Pools, Position, Skill, Skills, WantsToShoot, Weapon,
    WeaponAttribute, Wearable,
};
use rltk::{to_cp437, Point, RandomNumberGenerator, RGB};
use specs::prelude::*;

pub struct RangedCombatSystem {}

type RangedCombatData<'a> = (
    Entities<'a>,
    WriteStorage<'a, WantsToShoot>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, Attributes>,
    ReadStorage<'a, Skills>,
    ReadStorage<'a, HungerClock>,
    ReadStorage<'a, Pools>,
    WriteExpect<'a, RandomNumberGenerator>,
    ReadStorage<'a, Equipped>,
    ReadStorage<'a, Weapon>,
    ReadStorage<'a, Wearable>,
    ReadStorage<'a, NaturalAttackDefense>,
    ReadStorage<'a, Position>,
    ReadExpect<'a, Map>,
);

impl<'a> System<'a> for RangedCombatSystem {
    type SystemData = RangedCombatData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_shoot,
            names,
            attributes,
            skills,
            hunger_clock,
            pools,
            mut rng,
            equipped_items,
            weapon,
            wearables,
            natural,
            positions,
            map,
        ) = data;

        for (entity, wants_shoot, name, attacker_attributes, attacker_skills, attacker_pools) in (
            &entities,
            &wants_shoot,
            &names,
            &attributes,
            &skills,
            &pools,
        )
            .join()
        {
            // Are the attacker and defender alive? Only attack if they are
            let target_pools = pools.get(wants_shoot.target).unwrap();
            let target_attributes = attributes.get(wants_shoot.target).unwrap();
            let target_skills = skills.get(wants_shoot.target).unwrap();
            if attacker_pools.hit_points.current > 0 && target_pools.hit_points.current > 0 {
                let target_name = names.get(wants_shoot.target).unwrap();

                // Fire projectile effect
                let apos = positions.get(entity).unwrap();
                let dpos = positions.get(wants_shoot.target).unwrap();

                add_effect(
                    None,
                    EffectType::ParticleProjectile {
                        glyph: to_cp437('*'),
                        fg: RGB::named(rltk::CYAN),
                        bg: RGB::named(rltk::BLACK),
                        lifespan: 300.0,
                        speed: 50.0,
                        path: rltk::line2d(
                            rltk::LineAlg::Bresenham,
                            Point::new(apos.x, apos.y),
                            Point::new(dpos.x, dpos.y),
                        ),
                    },
                    Targets::Tile {
                        tile_idx: map.xy_idx(apos.x, apos.y) as i32,
                    },
                );

                // Define the basic unarmed attack - overriddent by wielding check below if a
                // weapon is equipped
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
                    if !nat.attacks.is_empty() {
                        let attack_index = if nat.attacks.len() == 1 {
                            0
                        } else {
                            rng.roll_dice(1, nat.attacks.len() as i32) as usize - 1
                        };
                        weapon_info.hit_bonus = nat.attacks[attack_index].hit_bonus;
                        weapon_info.damage_n_dice = nat.attacks[attack_index].damage_n_dice;
                        weapon_info.damage_die_type = nat.attacks[attack_index].damage_die_type;
                        weapon_info.damage_bonus = nat.attacks[attack_index].damage_bonus;
                    }
                }

                let mut weapon_entity = None;
                for (weaponentity, wielded, melee) in (&entities, &equipped_items, &weapon).join() {
                    if wielded.owner == entity && wielded.slot == EquipmentSlot::Melee {
                        weapon_info = melee.clone();
                        weapon_entity = Some(weaponentity);
                    }
                }

                let natural_roll = rng.roll_dice(1, 20);
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
                    if wielded.owner == wants_shoot.target {
                        armor_item_bonus_f += armor.armor_class;
                    }
                }

                let base_armor_class = match natural.get(wants_shoot.target) {
                    None => 10,
                    Some(nat) => nat.armor_class.unwrap_or(10),
                };
                let armor_quickness_bonus = target_attributes.quickness.bonus;
                let armor_skill_bonus = skill_bonus(Skill::Defense, &*target_skills);
                let armor_item_bonus = armor_item_bonus_f as i32;
                let armor_class =
                    base_armor_class + armor_quickness_bonus + armor_skill_bonus + armor_item_bonus;

                if natural_roll != 1 && (natural_roll == 20 || modified_hit_roll > armor_class) {
                    let base_damage =
                        rng.roll_dice(weapon_info.damage_n_dice, weapon_info.damage_die_type);
                    let attr_damage_bonus = attacker_attributes.might.bonus;
                    let skill_damage_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                    let weapon_damage_bonus = weapon_info.damage_bonus;

                    let damage = i32::max(
                        0,
                        base_damage + attr_damage_bonus + skill_damage_bonus + weapon_damage_bonus,
                    );

                    add_effect(
                        Some(entity),
                        EffectType::Damage { amount: damage },
                        Targets::Single {
                            target: wants_shoot.target,
                        },
                    );
                    crate::gamelog::Logger::new()
                        .color(rltk::YELLOW)
                        .append(&name.name)
                        .color(rltk::WHITE)
                        .append("hits")
                        .color(rltk::YELLOW)
                        .append(&target_name.name)
                        .append("for")
                        .color(rltk::RED)
                        .append(format!("{damage}"))
                        .color(rltk::WHITE)
                        .append("hp.")
                        .log();

                    // Proc effects
                    if let Some(chance) = &weapon_info.proc_chance {
                        let roll = rng.roll_dice(1, 100);
                        if roll <= (chance * 100.0) as i32 {
                            let effect_target = if let Some(weapon_target) = weapon_info.proc_target
                            {
                                if weapon_target == "Self" {
                                    Targets::Single { target: entity }
                                } else {
                                    Targets::Single {
                                        target: wants_shoot.target,
                                    }
                                }
                            } else {
                                Targets::Single {
                                    target: wants_shoot.target,
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
                    crate::gamelog::Logger::new()
                        .color(rltk::CYAN)
                        .append(&name.name)
                        .color(rltk::WHITE)
                        .append("considers attacking")
                        .color(rltk::CYAN)
                        .append(&target_name.name)
                        .color(rltk::WHITE)
                        .append("but misjudges the timing!")
                        .log();
                    add_effect(
                        None,
                        EffectType::Particle {
                            glyph: to_cp437('‼'),
                            fg: RGB::named(rltk::BLUE),
                            bg: RGB::named(rltk::BLACK),
                            lifespan: 200.0,
                        },
                        Targets::Single {
                            target: wants_shoot.target,
                        },
                    );
                } else {
                    crate::gamelog::Logger::new()
                        .color(rltk::CYAN)
                        .append(&name.name)
                        .color(rltk::WHITE)
                        .append("attacks")
                        .color(rltk::CYAN)
                        .append(&target_name.name)
                        .color(rltk::WHITE)
                        .append("but can't connect")
                        .log();
                    add_effect(
                        None,
                        EffectType::Particle {
                            glyph: to_cp437('‼'),
                            fg: RGB::named(rltk::CYAN),
                            bg: RGB::named(rltk::BLACK),
                            lifespan: 200.0,
                        },
                        Targets::Single {
                            target: wants_shoot.target,
                        },
                    );
                }
            }
        }

        wants_shoot.clear();
    }
}
