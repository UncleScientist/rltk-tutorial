use crate::{Monster, Position, Viewshed};
use rltk::console;
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Monster>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (viewshed, pos, monster) = data;

        for (_viewshed, _pos, _monster) in (&viewshed, &pos, &monster).join() {
            console::log("Monster considers their own existence");
        }
    }
}
