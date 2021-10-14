use rltk::{RandomNumberGenerator, RGB, FontCharType, to_cp437};
use specs::prelude::*;

use crate::{CombatStats, Player, Renderable, Name, Position, Viewshed, Monster, BlocksTile};

/// Spawns the player and returns his/her entity object
pub fn player(ecs : &mut World, player_x : i32, player_y: i32) -> Entity {
    ecs
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .build()
}

/// Spawns a random monster at a given location
pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        rng.roll_dice(1, 2)
    };

    match roll {
        1 => { orc(ecs, x, y) },
        _ => { goblin(ecs, x, y) },
    }
}

fn orc(ecs: &mut World, x: i32, y:i32) {
    monster(ecs, x, y, to_cp437('o'), "Orc");
}

fn goblin(ecs: &mut World, x: i32, y:i32) {
    monster(ecs, x, y, to_cp437('g'), "Goblin");
}

fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, glyph: FontCharType, name: S) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name { name: name.to_string() })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .build();
}
