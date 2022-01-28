use crate::{Attributes, Initiative, MyTurn, Position, RunState};
use rltk::{Point, RandomNumberGenerator};
use specs::prelude::*;

pub struct InitiativeSystem {}

type InitiativeData<'a> = (
    WriteStorage<'a, Initiative>,
    ReadStorage<'a, Position>,
    WriteStorage<'a, MyTurn>,
    Entities<'a>,
    WriteExpect<'a, RandomNumberGenerator>,
    ReadStorage<'a, Attributes>,
    WriteExpect<'a, RunState>,
    ReadExpect<'a, Entity>,
    ReadExpect<'a, Point>,
);

impl<'a> System<'a> for InitiativeSystem {
    type SystemData = InitiativeData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut initiatives,
            positions,
            mut turns,
            entities,
            mut rng,
            attributes,
            mut runstate,
            player,
            player_pos,
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
                initiative.current = 6 + rng.roll_dice(1, 6);

                // Give a bonus for quickness
                if let Some(attr) = attributes.get(entity) {
                    initiative.current -= attr.quickness.bonus;
                }

                // TODO: More initiatie granting boosts/penalties will go here later

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
    }
}
