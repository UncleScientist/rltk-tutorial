use rltk::{to_cp437, FontCharType, RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};

use std::collections::{hash_map::Entry, HashMap};

use crate::{
    AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, InflictsDamage, Item, Monster,
    Name, Player, Position, ProvidesHealing, RandomTable, Ranged, Rect, Renderable, SerializeMe,
    Viewshed, MAPWIDTH,
};

/// Fills a room with stuff!
pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let spawn_table = room_table();
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mwusize = MAPWIDTH as usize; // TODO: clean up map i32 vs usize

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, 7) - 3;

        for _ in 0..num_spawns {
            let mut tries = 0;

            while tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = y * mwusize + x;
                if let Entry::Vacant(e) = spawn_points.entry(idx) {
                    e.insert(spawn_table.roll(&mut rng));
                    break;
                }
                tries += 1;
            }
        }
    }

    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % mwusize) as i32;
        let y = (*spawn.0 / mwusize) as i32;

        match spawn.1.as_ref() {
            "Goblin" => goblin(ecs, x, y),
            "Orc" => orc(ecs, x, y),
            "Health Potion" => health_potion(ecs, x, y),
            "Fireball Scroll" => fireball_scroll(ecs, x, y),
            "Confusion Scroll" => confusion_scroll(ecs, x, y),
            "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
            _ => {} // panic!("could not find {} in table", spawn.1),
        }
    }
}

pub fn spawn_goodies(ecs: &mut World, room: &Rect) {
    let mut item_spawn_points: Vec<usize> = Vec::new();
    let mwusize = MAPWIDTH as usize; // TODO: clean up map i32 vs usize

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        for _ in 0..5 {
            loop {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                // TODO: get map from ecs & call xy_idx()?
                let idx = (y * mwusize) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    break;
                }
            }
        }
    }

    for idx in item_spawn_points.iter() {
        let x = *idx % mwusize;
        let y = *idx / mwusize;
        random_item(ecs, x as i32, y as i32);
    }
}

/// Spawns the player and returns his/her entity object
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs.create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
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
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

fn orc(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, to_cp437('o'), "Orc");
}

fn goblin(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, to_cp437('g'), "Goblin");
}

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: FontCharType, name: S) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn random_item(ecs: &mut World, x: i32, y: i32) {
    let roll = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        rng.roll_dice(1, 4)
    };

    match roll {
        1 => health_potion(ecs, x, y),
        2 => fireball_scroll(ecs, x, y),
        3 => confusion_scroll(ecs, x, y),
        _ => magic_missile_scroll(ecs, x, y),
    };
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(ProvidesHealing { heal_amount: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Magic Missile Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Fireball Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Confusion Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn room_table() -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2)
        .add("Confusion Scroll", 2)
        .add("Magic Missile Scroll", 4)
}
