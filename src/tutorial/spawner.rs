use rltk::{to_cp437, Point, RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};

use std::collections::HashMap;

use crate::*;

/// Fills a room with stuff!
pub fn spawn_room(
    map: &Map,
    rng: &mut RandomNumberGenerator,
    room: &Rect,
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let mut possible_targets: Vec<usize> = Vec::new();
    {
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(rng, &possible_targets, map_depth, spawn_list);
}

pub fn spawn_region(
    rng: &mut RandomNumberGenerator,
    area: &[usize],
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    const MAX_SPAWNS: i32 = 3;

    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);

    {
        let num_spawns = i32::min(
            areas.len() as i32,
            rng.roll_dice(1, MAX_SPAWNS + 3) + (map_depth - 1) - 3,
        );
        if num_spawns == 0 {
            return;
        }

        for _ in 0..num_spawns {
            let array_index = if areas.len() == 1 {
                0usize
            } else {
                (rng.roll_dice(1, areas.len() as i32) - 1) as usize
            };
            let map_idx = areas[array_index];
            if let Some(spawn) = spawn_table.roll(rng) {
                spawn_points.insert(map_idx, spawn);
            }
            areas.remove(array_index);
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        spawn_list.push((*spawn.0, spawn.1.to_string()));
    }
}

pub fn spawn_entity(ecs: &mut World, spawn: &(&usize, &String)) {
    let map = ecs.fetch::<Map>();
    let width = map.width as usize;
    let x = (*spawn.0 % width as usize) as i32;
    let y = (*spawn.0 / width as usize) as i32;
    std::mem::drop(map);

    let item_result = spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        spawn.1,
        SpawnType::AtPosition { x, y },
    );
    if item_result.is_some() {
        return;
    }

    rltk::console::log(format!("Waring: we don't know how to spawn [{}]!", spawn.1));
}

pub fn spawn_town_portal(ecs: &mut World) {
    // Get current position & depth
    let map = ecs.fetch::<Map>();
    let player_depth = map.depth;
    let player_pos = ecs.fetch::<Point>();
    let player_x = player_pos.x;
    let player_y = player_pos.y;
    std::mem::drop(player_pos);
    std::mem::drop(map);

    // Find part of the town for the portal
    let dm = ecs.fetch::<MasterDungeonMap>();
    let town_map = dm.get_map(1).unwrap();
    let mut stairs_idx = 0;
    for (idx, tt) in town_map.tiles.iter().enumerate() {
        if *tt == TileType::DownStairs {
            stairs_idx = idx;
        }
    }

    let portal_x = (stairs_idx as i32 % town_map.width) - 2;
    let portal_y = stairs_idx as i32 / town_map.width;

    std::mem::drop(dm);

    // Spawn the portal itself
    ecs.create_entity()
        .with(OtherLevelPosition {
            x: portal_x,
            y: portal_y,
            depth: 1,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('♥'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(EntryTrigger {})
        .with(TeleportTo {
            x: player_x,
            y: player_y,
            depth: player_depth,
            player_only: true,
        })
        .with(Name {
            name: "Town Portal".to_string(),
        })
        .with(SingleActivation {})
        .build();
}

/// Spawns the player and returns his/her entity object
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    let mut skills = Skills {
        skills: HashMap::new(),
    };
    skills.skills.insert(Skill::Melee, 1);
    skills.skills.insert(Skill::Defense, 1);
    skills.skills.insert(Skill::Magic, 1);

    let player = ecs
        .create_entity()
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
        .with(Initiative { current: 0 })
        .with(Faction {
            name: "Player".to_string(),
        })
        .with(LightSource {
            color: RGB::from_f32(1., 1., 0.5),
            range: 8,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(HungerClock {
            state: HungerState::WellFed,
            duration: 20,
        })
        .with(Attributes {
            ..Default::default()
        })
        .with(skills)
        .with(Pools {
            hit_points: Pool {
                current: player_hp_at_level(11, 1),
                max: player_hp_at_level(11, 1),
            },
            mana: Pool {
                current: mana_at_level(11, 1),
                max: mana_at_level(11, 1),
            },
            xp: 0,
            level: 1,
            total_weight: 0.0,
            total_initiative_penalty: 0.0,
            gold: 0.0,
            god_mode: false,
        })
        .with(EquipmentChanged {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Starting Equipment
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Rusty Longsword",
        SpawnType::Equipped { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Dried Sausage",
        SpawnType::Carried { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Beer",
        SpawnType::Carried { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Stained Tunic",
        SpawnType::Equipped { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Torn Trousers",
        SpawnType::Equipped { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Old Boots",
        SpawnType::Equipped { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Town Portal Scroll",
        SpawnType::Carried { by: player },
    );

    player
}

fn room_table(map_depth: i32) -> RandomTable {
    get_spawn_table_for_depth(&RAWS.lock().unwrap(), map_depth)
}
