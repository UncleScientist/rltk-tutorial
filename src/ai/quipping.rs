use crate::{MyTurn, Name, Quips, Viewshed};
use rltk::{Point, RandomNumberGenerator};
use specs::prelude::*;

pub struct QuipSystem {}

type QuipData<'a> = (
    WriteStorage<'a, Quips>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, MyTurn>,
    ReadExpect<'a, Point>,
    ReadStorage<'a, Viewshed>,
    WriteExpect<'a, RandomNumberGenerator>,
);

impl<'a> System<'a> for QuipSystem {
    type SystemData = QuipData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (mut quips, names, turns, player_pos, viewsheds, mut rng) = data;

        for (quip, name, viewshed, _turn) in (&mut quips, &names, &viewsheds, &turns).join() {
            if !quip.available.is_empty()
                && viewshed.visible_tiles.contains(&player_pos)
                && rng.roll_dice(1, 6) == 1
            {
                if let Some(quip_index) = rng.random_slice_index(&quip.available) {
                    crate::gamelog::Logger::new()
                        .npc_name(&name.name)
                        .color(rltk::WHITE)
                        .append("says")
                        .item_name(&quip.available[quip_index])
                        .log();
                    quip.available.remove(quip_index);
                }
            }
        }
    }
}
