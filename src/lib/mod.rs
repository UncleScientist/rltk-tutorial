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

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        self.ecs.maintain();
    }
}
