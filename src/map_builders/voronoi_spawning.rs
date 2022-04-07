use std::collections::HashMap;

use super::{BuilderMap, MetaMapBuilder, TileType};
use crate::map_builders::*;

pub struct VoronoiSpawning {}

impl MetaMapBuilder for VoronoiSpawning {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl VoronoiSpawning {
    pub fn new() -> Box<VoronoiSpawning> {
        Box::new(VoronoiSpawning {})
    }

    pub fn build(&mut self, build_data: &mut BuilderMap) {
        let mut noise_areas: HashMap<i32, Vec<usize>> = HashMap::new();
        let mut noise = rltk::FastNoise::seeded(crate::tutorial::rng::roll_dice(1, 65536) as u64);

        noise.set_noise_type(rltk::NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::Floor {
                    let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
                    let cell_value = cell_value_f as i32;

                    noise_areas.entry(cell_value).or_default().push(idx);
                }
            }
        }

        for area in noise_areas.iter() {
            spawner::spawn_region(area.1, build_data.map.depth, &mut build_data.spawn_list);
        }
    }
}
