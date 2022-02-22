use crate::{
    effects::*, skill_bonus, Attributes, EquipmentSlot, Equipped, GameLog, HungerClock,
    HungerState, MeleeWeapon, Name, NaturalAttackDefense, ParticleBuilder, Pools, Position, Skill,
    Skills, WantsToMelee, WeaponAttribute, Wearable,
};

use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub struct MeleeCombatSystem {}

type MeleeCombatData<'a> = (
    Entities<'a>,
    WriteExpect<'a, GameLog>,
    WriteStorage<'a, WantsToMelee>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, Attributes>,
    ReadStorage<'a, Skills>,
    WriteExpect<'a, ParticleBuilder>,
    ReadStorage<'a, Position>,
    WriteStorage<'a, HungerClock>,
    ReadStorage<'a, Pools>,
    WriteExpect<'a, RandomNumberGenerator>,
    ReadStorage<'a, Equipped>,
    ReadStorage<'a, MeleeWeapon>,
    ReadStorage<'a, Wearable>,
    ReadStorage<'a, NaturalAttackDefense>,
);

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = MeleeCombatData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            mut wants_melee,
            names,
            attributes,
            skills,
            mut particle_builder,
            positions,
            hunger_clock,
            pools,
            mut rng,
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

            let mut weapon_info = MeleeWeapon {
                attribute: WeaponAttribute::Might,
                hit_bonus: 0,
                damage_n_dice: 1,
                damage_die_type: 4,
                damage_bonus: 0,
            };

            if let Some(nat) = natural.get(entity) {
                if let Some(attack_index) = rng.random_slice_index(&nat.attacks) {
                    weapon_info.hit_bonus = nat.attacks[attack_index].hit_bonus;
                    weapon_info.damage_n_dice = nat.attacks[attack_index].damage_n_dice;
                    weapon_info.damage_die_type = nat.attacks[attack_index].damage_die_type;
                    weapon_info.damage_bonus = nat.attacks[attack_index].damage_bonus;
                }
            }

            for (wielded, melee) in (&equipped_items, &meleeweapons).join() {
                if wielded.owner == entity && wielded.slot == EquipmentSlot::Melee {
                    weapon_info = melee.clone();
                }
            }

            if attacker_pools.hit_points.current > 0 && target_pools.hit_points.current > 0 {
                let target_name = names.get(wants_melee.target).unwrap();

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
                    let base_damage =
                        rng.roll_dice(weapon_info.damage_n_dice, weapon_info.damage_die_type);
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
                    log.entries.push(format!(
                        "{} hits {}, for {} hp",
                        &name.name, &target_name.name, damage
                    ));
                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(
                            pos.x,
                            pos.y,
                            rltk::RGB::named(rltk::ORANGE),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                    }
                } else if natural_roll == 1 {
                    // Natural 1 miss
                    log.entries.push(format!(
                        "{} considers attacking {}, but misjudges the timing",
                        &name.name, &target_name.name
                    ));
                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(
                            pos.x,
                            pos.y,
                            rltk::RGB::named(rltk::BLUE),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                    }
                } else {
                    // Miss
                    log.entries.push(format!(
                        "{} attacks {}, but can't connect",
                        &name.name, &target_name.name
                    ));
                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(
                            pos.x,
                            pos.y,
                            rltk::RGB::named(rltk::CYAN),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                    }
                }
            }
        }

        wants_melee.clear();
    }
}
