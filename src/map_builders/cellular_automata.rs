use super::{Map, Position, Rect, TileType};
use crate::map_builders::*;
use crate::*;

pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
}

impl MapBuilder for CellularAutomataBuilder {
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
        /*
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room, self.depth);
        }
        */
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

impl CellularAutomataBuilder {
    pub fn new(new_depth: i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = rltk::RandomNumberGenerator::new();

        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = self.map.xy_idx(x, y);
                self.map.tiles[idx] = if roll > 55 {
                    TileType::Floor
                } else {
                    TileType::Wall
                }
            }
        }
        self.take_snapshot();

        for _ in 0..15 {
            let mut newtiles = self.map.tiles.clone();

            for y in 1..self.map.height - 1 {
                for x in 1..self.map.width - 1 {
                    let idx = self.map.xy_idx(x, y);
                    let mut neighbors = 0;

                    neighbors += (self.map.tiles[idx - 1] == TileType::Wall) as usize;

                    if self.map.tiles[idx + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - self.map.width as usize] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + self.map.width as usize] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - (self.map.width as usize - 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - (self.map.width as usize + 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + (self.map.width as usize - 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + (self.map.width as usize + 1)] == TileType::Wall {
                        neighbors += 1;
                    }

                    if neighbors > 4 || neighbors == 0 {
                        newtiles[idx] = TileType::Wall;
                    } else {
                        newtiles[idx] = TileType::Floor;
                    }
                }
            }

            self.map.tiles = newtiles.clone();
            self.take_snapshot();
        }

        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        let mut start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self
                .map
                .xy_idx(self.starting_position.x, self.starting_position.y);
        }

        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(
            self.map.width,
            self.map.height,
            &map_starts,
            &self.map,
            200.0,
        );
        let mut exit_tile = (0, 0.0f32);
        for (i, tile) in self.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                if distance_to_start == std::f32::MAX {
                    *tile = TileType::Wall;
                } else if distance_to_start > exit_tile.1 {
                    exit_tile.0 = i;
                    exit_tile.1 = distance_to_start;
                }
            }
        }
        self.take_snapshot();

        self.map.tiles[exit_tile.0] = TileType::DownStairs;
        self.take_snapshot();
    }
}
