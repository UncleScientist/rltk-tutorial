use super::{MetaMapBuilder, BuilderMap, Map, TileType};
use crate::map_builders::*;

mod constraints;
use constraints::*;

mod common;
use common::*;

mod solver;
use solver::*;

use crate::*;

pub struct WaveformCollapseBuilder {}

impl MetaMapBuilder for WaveformCollapseBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl WaveformCollapseBuilder {
    pub fn new() -> Box<WaveformCollapseBuilder> {
        Box::new(WaveformCollapseBuilder {})
    }

    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        const CHUNK_SIZE: i32 = 8;

        let patterns = build_patterns(&build_data.map, CHUNK_SIZE, true, true);
        let constraints = patterns_to_constraints(patterns, CHUNK_SIZE);
        self.render_tile_gallery(&constraints, CHUNK_SIZE, build_data);

        build_data.map = Map::new(build_data.map.depth);
        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &build_data.map);
            while !solver.iteration(&mut build_data.map, rng) {
                build_data.take_snapshot();
            }

            build_data.take_snapshot();
            if solver.possible {
                break;
            }
        }
        build_data.spawn_list.clear();
    }

    fn render_tile_gallery(&mut self, constraints: &[MapChunk], chunk_size: i32, build_data: &mut BuilderMap) {
        build_data.map = Map::new(0);
        let mut counter = 0;
        let mut x = 1;
        let mut y = 1;

        while counter < constraints.len() {
            render_pattern_to_map(&mut build_data.map, &constraints[counter], chunk_size, x, y);

            x += chunk_size + 1;
            if x + chunk_size > build_data.map.width {
                // Move to the next row
                x = 1;
                y += chunk_size + 1;
                if y + chunk_size > build_data.map.height {
                    build_data.take_snapshot();
                    build_data.map = Map::new(0);
                    x = 1;
                    y = 1;
                }
            }
            counter += 1;
        }

        build_data.take_snapshot();
    }
}
