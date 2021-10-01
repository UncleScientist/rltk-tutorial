use crate::{Map, Monster, Name, Position, Viewshed};
use rltk::{console, Point};
use specs::prelude::*;

pub struct MonsterAI {}

type MonsterAIData<'a> = (
    ReadExpect<'a, Map>,
    ReadExpect<'a, Point>,
    WriteStorage<'a, Viewshed>,
    ReadStorage<'a, Monster>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, Position>,
);

impl<'a> System<'a> for MonsterAI {
    type SystemData = MonsterAIData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (map, player_pos, mut viewshed, monster, name, mut position) = data;

        for (mut viewshed, _monster, name, mut pos) in
            (&mut viewshed, &monster, &name, &mut position).join()
        {
            if viewshed.visible_tiles.contains(&*player_pos) {
                let distance =
                    rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance < 1.5 {
                    console::log(&format!("{} shouts insults", name.name));
                    return;
                }
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(player_pos.x, player_pos.y) as i32,
                    &*map,
                );

                if path.success && path.steps.len() > 1 {
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
