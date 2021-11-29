use super::{Map, Position, TileType};
use crate::map_builders::*;
use std::collections::HashMap;

use crate::*;

#[derive(PartialEq, Copy, Clone)]
pub enum DrunkSpawnMode {
    StartingPoint,
    Random,
}

pub struct DrunkardSettings {
    pub spawn_mode: DrunkSpawnMode,
    pub drunken_lifetime: i32,
    pub floor_percent: f32,
    pub brush_size: i32,
    pub symmetry: Symmetry,
}

pub struct DrunkardsWalkBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    settings: DrunkardSettings,
}

impl MapBuilder for DrunkardsWalkBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for area in self.noise_areas.iter() {
            spawner::spawn_region(ecs, area.1, self.depth);
        }
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl DrunkardsWalkBuilder {
    fn new(new_depth: i32, settings: DrunkardSettings) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            settings,
        }
    }

    pub fn fearful_symmetry(new_depth: i32) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder::new(
            new_depth,
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 1,
                symmetry: Symmetry::Both,
            },
        )
    }

    pub fn fat_passages(new_depth: i32) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder::new(
            new_depth,
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.3,
                brush_size: 2,
                symmetry: Symmetry::None,
            },
        )
    }

    pub fn open_area(new_depth: i32) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder::new(
            new_depth,
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::StartingPoint,
                drunken_lifetime: 400,
                floor_percent: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        )
    }

    pub fn open_halls(new_depth: i32) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder::new(
            new_depth,
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 400,
                floor_percent: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        )
    }

    pub fn winding_passages(new_depth: i32) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder::new(
            new_depth,
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::StartingPoint,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        )
    }

    fn build(&mut self) {
        let mut rng = rltk::RandomNumberGenerator::new();

        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };

        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);

        self.map.tiles[start_idx] = TileType::Floor;
        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (self.settings.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = self
            .map
            .tiles
            .iter()
            .filter(|a| **a == TileType::Floor)
            .count();
        let mut digger_count = 0;

        while floor_tile_count < desired_floor_tiles {
            let mut did_something = false;
            let mut drunk_life = self.settings.drunken_lifetime;

            let (mut drunk_x, mut drunk_y) = match self.settings.spawn_mode {
                DrunkSpawnMode::StartingPoint => {
                    (self.starting_position.x, self.starting_position.y)
                }
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        (self.starting_position.x, self.starting_position.y)
                    } else {
                        (
                            rng.roll_dice(1, self.map.width - 3) + 1,
                            rng.roll_dice(1, self.map.height - 3) + 1,
                        )
                    }
                }
            };

            while drunk_life > 0 {
                let drunk_idx = self.map.xy_idx(drunk_x, drunk_y);
                if self.map.tiles[drunk_idx] == TileType::Wall {
                    did_something = true;
                }
                paint(
                    &mut self.map,
                    self.settings.symmetry,
                    self.settings.brush_size,
                    drunk_x,
                    drunk_y,
                );
                self.map.tiles[drunk_idx] = TileType::DownStairs;

                match rng.roll_dice(1, 4) {
                    1 => {
                        if drunk_x > 2 {
                            drunk_x -= 1;
                        }
                    }
                    2 => {
                        if drunk_x < self.map.width - 2 {
                            drunk_x += 1
                        }
                    }
                    3 => {
                        if drunk_y > 2 {
                            drunk_y -= 1;
                        }
                    }
                    _ => {
                        if drunk_y < self.map.height - 2 {
                            drunk_y += 1
                        }
                    }
                }

                drunk_life -= 1;
            }

            if did_something {
                self.take_snapshot();
            }

            digger_count += 1;
            for t in self.map.tiles.iter_mut() {
                if *t == TileType::DownStairs {
                    *t = TileType::Floor;
                }
            }
            floor_tile_count = self
                .map
                .tiles
                .iter()
                .filter(|a| **a == TileType::Floor)
                .count();
        }

        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }
}
