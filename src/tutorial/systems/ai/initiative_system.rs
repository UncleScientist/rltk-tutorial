use crate::{
    Attributes, DamageOverTime, Duration, EquipmentChanged, Initiative, MyTurn, Pools, Position,
    RunState, StatusEffect,
};
use rltk::Point;
use specs::prelude::*;

pub struct InitiativeSystem {}

type InitiativeData<'a> = (
    WriteStorage<'a, Initiative>,
    ReadStorage<'a, Position>,
    WriteStorage<'a, MyTurn>,
    Entities<'a>,
    ReadStorage<'a, Attributes>,
    WriteExpect<'a, RunState>,
    ReadExpect<'a, Entity>,
    ReadExpect<'a, Point>,
    ReadStorage<'a, Pools>,
    WriteStorage<'a, Duration>,
    WriteStorage<'a, EquipmentChanged>,
    ReadStorage<'a, StatusEffect>,
    ReadStorage<'a, DamageOverTime>,
);

impl<'a> System<'a> for InitiativeSystem {
    type SystemData = InitiativeData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut initiatives,
            positions,
            mut turns,
            entities,
            attributes,
            mut runstate,
            player,
            player_pos,
            pools,
            mut durations,
            mut dirty,
            statuses,
            dots,
        ) = data;

        if *runstate != RunState::Ticking {
            return;
        }

        // Clear any remaining MyTurn we left by mistkae
        turns.clear();

        // Roll initiative
        for (entity, initiative, pos) in (&entities, &mut initiatives, &positions).join() {
            initiative.current -= 1;
            if initiative.current < 1 {
                let mut myturn = true;

                // Re-roll
                initiative.current = 6 + crate::tutorial::rng::roll_dice(1, 6);

                // Give a bonus for quickness
                if let Some(attr) = attributes.get(entity) {
                    initiative.current -= attr.quickness.bonus;
                }

                // Apply pool penalty
                if let Some(pools) = pools.get(entity) {
                    initiative.current += f32::floor(pools.total_initiative_penalty) as i32;
                }

                // TODO: More initiative granting boosts/penalties will go here later

                // If its the player, we want to go to an AwatingInput state
                if entity == *player {
                    *runstate = RunState::AwaitingInput;
                } else {
                    let distance = rltk::DistanceAlg::Pythagoras
                        .distance2d(*player_pos, Point::new(pos.x, pos.y));
                    if distance > 20.0 {
                        myturn = false;
                    }
                }
                // It's my turn!
                if myturn {
                    turns
                        .insert(entity, MyTurn {})
                        .expect("Unable to insert turn");
                }
            }
        }

        // Handle durations
        if *runstate == RunState::AwaitingInput {
            use crate::effects::*;
            for (effect_entity, duration, status) in (&entities, &mut durations, &statuses).join() {
                if entities.is_alive(status.target) {
                    duration.turns -= 1;
                    if let Some(dot) = dots.get(effect_entity) {
                        add_effect(
                            None,
                            EffectType::Damage { amount: dot.damage },
                            Targets::Single {
                                target: status.target,
                            },
                        );
                    }
                    if duration.turns < 1 {
                        dirty
                            .insert(status.target, EquipmentChanged {})
                            .expect("Unable to insert");
                        entities.delete(effect_entity).expect("Unable to delete");
                    }
                }
            }
        }
    }
}
