use super::{spawner, Map, Position, Rect, TileType, MAPHEIGHT, MAPWIDTH};
use specs::prelude::*;

mod common;
use common::*;

mod simple_map;
use simple_map::SimpleMapBuilder;

mod bsp_dungeon;
use bsp_dungeon::BspDungeonBuilder;

mod bsp_interior;
use bsp_interior::BspInteriorBuilder;

mod cellular_automata;
use cellular_automata::CellularAutomataBuilder;

mod drunkard;
use drunkard::DrunkardsWalkBuilder;

mod maze;
use maze::MazeBuilder;

mod dla;
use dla::*;

mod voronoi;
use voronoi::*;

mod waveform_collapse;
use waveform_collapse::*;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    // let mut rng = rltk::RandomNumberGenerator::new();
    // let builder = rng.roll_dice(1, 19);
    let builder = 1;
    match builder {
        1 => Box::new(WaveformCollapseBuilder::new(new_depth)),
        2 => Box::new(MazeBuilder::new(new_depth)),
        3 => Box::new(BspInteriorBuilder::new(new_depth)),
        4 => Box::new(CellularAutomataBuilder::new(new_depth)),
        5 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
        6 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
        7 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
        8 => Box::new(DLABuilder::walk_inwards(new_depth)),
        9 => Box::new(DLABuilder::walk_outwards(new_depth)),
        10 => Box::new(DLABuilder::central_attractor(new_depth)),
        11 => Box::new(DLABuilder::insectoid(new_depth)),
        12 => Box::new(DLABuilder::crazy(new_depth)),
        13 => Box::new(DLABuilder::rorschach(new_depth)),
        14 => Box::new(BspDungeonBuilder::new(new_depth)),
        15 => Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)),
        16 => Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)),
        17 => Box::new(VoronoiBuilder::manhattan(new_depth)),
        18 => Box::new(VoronoiBuilder::chebyshev(new_depth)),
        19 => Box::new(VoronoiBuilder::pythagoras(new_depth)),
        _ => Box::new(SimpleMapBuilder::new(new_depth)),
    }
}
