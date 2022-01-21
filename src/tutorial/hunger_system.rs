use super::{gamelog::GameLog, HungerClock, HungerState, RunState, SufferDamage};
use specs::prelude::*;

pub struct HungerSystem;

type HungerData<'a> = (
    Entities<'a>,
    WriteStorage<'a, HungerClock>,
    ReadExpect<'a, Entity>, // The player
    ReadExpect<'a, RunState>,
    WriteStorage<'a, SufferDamage>,
    WriteExpect<'a, GameLog>,
);

impl<'a> System<'a> for HungerSystem {
    type SystemData = HungerData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player_entity, runstate, mut inflict_damage, mut log) =
            data;

        for (entity, mut clock) in (&entities, &mut hunger_clock).join() {
            let proceed = match *runstate {
                RunState::PlayerTurn => entity == *player_entity,
                RunState::MonsterTurn => entity != *player_entity,
                _ => false,
            };

            if proceed {
                clock.duration -= 1;
                if clock.duration < 1 {
                    match clock.state {
                        HungerState::WellFed => {
                            clock.state = HungerState::Normal;
                            clock.duration = 200;
                            if entity == *player_entity {
                                log.entries.push("You are no longer well fed.".to_string());
                            }
                        }
                        HungerState::Normal => {
                            clock.state = HungerState::Hungry;
                            clock.duration = 200;
                            if entity == *player_entity {
                                log.entries.push("You are hungry.".to_string());
                            }
                        }
                        HungerState::Hungry => {
                            clock.state = HungerState::Starving;
                            clock.duration = 200;
                            if entity == *player_entity {
                                log.entries.push("You are starving!".to_string());
                            }
                        }
                        HungerState::Starving => {
                            if entity == *player_entity {
                                log.entries
                                    .push("Your hunger pangs are getting painful!".to_string());
                            }
                            SufferDamage::new_damage(&mut inflict_damage, entity, 1, false);
                        }
                    }
                }
            }
        }
    }
}
