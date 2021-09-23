use rltk::{GameState, Rltk};
use specs::prelude::*;

pub mod components;
pub use components::*;

pub mod map;
pub use map::*;

pub mod player;
pub use player::*;

pub mod rect;
pub use rect::*;

pub mod visibility_system;
pub use visibility_system::*;

pub mod monster_ai_system;
pub use monster_ai_system::*;

// ------------------------------------------------------------
// World state section

pub struct State {
    pub ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        player_input(self, ctx);
        self.run_systems();

        ctx.cls();

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            if map.is_visible(pos.x, pos.y) {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);

        self.ecs.maintain();
    }
}
