use crate::{
    CombatStats, DefenseBonus, Equipped, GameLog, HungerClock, HungerState, MeleePowerBonus, Name,
    ParticleBuilder, Position, SufferDamage, WantsToMelee,
};
use specs::prelude::*;

pub struct MeleeCombatSystem {}

type MeleeCombatData<'a> = (
    Entities<'a>,
    WriteExpect<'a, GameLog>,
    WriteStorage<'a, WantsToMelee>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, CombatStats>,
    WriteStorage<'a, SufferDamage>,
    ReadStorage<'a, MeleePowerBonus>,
    ReadStorage<'a, DefenseBonus>,
    ReadStorage<'a, Equipped>,
    WriteExpect<'a, ParticleBuilder>,
    ReadStorage<'a, Position>,
    WriteStorage<'a, HungerClock>,
);

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = MeleeCombatData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            mut wants_melee,
            names,
            combat_stats,
            mut inflict_damage,
            melee_power_bonuses,
            defense_bonuses,
            equipped,
            mut particle_builder,
            positions,
            hunger_clock,
        ) = data;

        for (entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            if stats.hp > 0 {
                let mut offensive_bonus = 0;
                for (_, power_bonus, equipped_by) in
                    (&entities, &melee_power_bonuses, &equipped).join()
                {
                    if equipped_by.owner == entity {
                        offensive_bonus += power_bonus.power;
                    }
                }

                if let Some(hc) = hunger_clock.get(entity) {
                    if hc.state == HungerState::WellFed {
                        offensive_bonus += 1;
                    }
                }

                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();

                    let mut defensive_bonus = 0;
                    for (_, defense_bonus, equipped_by) in
                        (&entities, &defense_bonuses, &equipped).join()
                    {
                        if equipped_by.owner == wants_melee.target {
                            defensive_bonus += defense_bonus.power;
                        }
                    }

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

                    let damage = i32::max(
                        0,
                        (stats.power + offensive_bonus) - (target_stats.defense + defensive_bonus),
                    );
                    if damage == 0 {
                        log.entries.push(format!(
                            "{} is unable to hurt {}",
                            &name.name, &target_name.name
                        ));
                    } else {
                        log.entries.push(format!(
                            "{} hits {}, for {} hp",
                            &name.name, &target_name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                }
            }
        }

        wants_melee.clear();
    }
}
