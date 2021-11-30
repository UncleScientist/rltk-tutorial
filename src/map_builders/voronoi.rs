use std::collections::HashMap;

use super::{Map, Position, TileType};
use crate::map_builders::*;
use crate::*;

#[derive(PartialEq, Copy, Clone)]
pub enum DistanceAlgorithm {
    Pythagoras,
    Manhattan,
    Chebyshev,
}

pub struct VoronoiBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    n_seeds: usize,
    distance_algorithm: DistanceAlgorithm,
}

impl MapBuilder for VoronoiBuilder {
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

impl Default for VoronoiBuilder {
    fn default() -> VoronoiBuilder {
        VoronoiBuilder {
            map: Map::new(1),
            starting_position: Position { x: 0, y: 0 },
            depth: 1,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Pythagoras,
        }
    }
}

impl VoronoiBuilder {
    pub fn pythagoras(new_depth: i32) -> VoronoiBuilder {
        VoronoiBuilder {
            map: Map::new(new_depth),
            depth: new_depth,
            ..Default::default()
        }
    }

    pub fn manhattan(new_depth: i32) -> VoronoiBuilder {
        VoronoiBuilder {
            map: Map::new(new_depth),
            depth: new_depth,
            distance_algorithm: DistanceAlgorithm::Manhattan,
            ..Default::default()
        }
    }

    pub fn chebyshev(new_depth: i32) -> VoronoiBuilder {
        VoronoiBuilder {
            map: Map::new(new_depth),
            depth: new_depth,
            distance_algorithm: DistanceAlgorithm::Chebyshev,
            ..Default::default()
        }
    }

    fn build(&mut self) {
        let mut rng = rltk::RandomNumberGenerator::new();
        let mut voronoi_seeds: Vec<(usize, rltk::Point)> = Vec::new();

        while voronoi_seeds.len() < self.n_seeds {
            let vx = rng.roll_dice(1, self.map.width - 1);
            let vy = rng.roll_dice(1, self.map.height - 1);
            let vidx = self.map.xy_idx(vx, vy);
            let candidate = (vidx, rltk::Point::new(vx, vy));
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        let mut voronoi_distance = vec![(0, 0.0f32); self.n_seeds];
        let mut voronoi_membership: Vec<i32> =
            vec![0; self.map.width as usize * self.map.height as usize];
        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let x = i as i32 % self.map.width;
            let y = i as i32 / self.map.width;

            for (seed, pos) in voronoi_seeds.iter().enumerate() {
                let distance = match self.distance_algorithm {
                    DistanceAlgorithm::Pythagoras => rltk::DistanceAlg::PythagorasSquared
                        .distance2d(rltk::Point::new(x, y), pos.1),
                    DistanceAlgorithm::Manhattan => {
                        rltk::DistanceAlg::Manhattan.distance2d(rltk::Point::new(x, y), pos.1)
                    }
                    DistanceAlgorithm::Chebyshev => {
                        rltk::DistanceAlg::Chebyshev.distance2d(rltk::Point::new(x, y), pos.1)
                    }
                };
                voronoi_distance[seed] = (seed, distance);
            }

            voronoi_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            *vid = voronoi_distance[0].0 as i32;
        }

        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let mut neighbors = 0;
                let my_idx = self.map.xy_idx(x, y);
                let my_seed = voronoi_membership[my_idx];
                if voronoi_membership[self.map.xy_idx(x - 1, y)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[self.map.xy_idx(x + 1, y)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[self.map.xy_idx(x, y - 1)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[self.map.xy_idx(x, y + 1)] != my_seed {
                    neighbors += 1;
                }

                if neighbors < 2 {
                    self.map.tiles[my_idx] = TileType::Floor;
                }
            }
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

        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }
}
