use super::{spawner, Map, Position, Rect, TileType, MAPHEIGHT, MAPWIDTH};
use specs::prelude::*;

use crate::SHOW_MAPGEN_VISUALIZER;

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

mod prefab_builders;
use prefab_builders::PrefabBuilder;

mod maze;
use maze::MazeBuilder;

mod dla;
use dla::*;

mod voronoi;
use voronoi::*;

mod waveform_collapse;
use waveform_collapse::*;

mod room_based_spawner;
use room_based_spawner::*;

mod room_based_stairs;
use room_based_stairs::*;

mod room_based_starting_position;
use room_based_starting_position::*;

mod area_starting_position;
use area_starting_position::*;

mod cull_unreachable;
use cull_unreachable::*;

mod voronoi_spawning;
use voronoi_spawning::*;

mod distant_exit;
use distant_exit::*;

// --------------------------------------------------------------------------------
pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history: Vec<Map>,
}

impl BuilderMap {
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
// --------------------------------------------------------------------------------

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

impl BuilderChain {
    pub fn new(new_depth: i32) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth),
                starting_position: None,
                rooms: None,
                history: Vec::new(),
            }
        }
    }

    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder"),
        }
    }

    pub fn with(&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    pub fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting map"),
            Some(starter) => {
                // Build the starting map
                starter.build_map(rng, &mut self.build_data);
            }
        }

        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}
// --------------------------------------------------------------------------------
pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

// --------------------------------------------------------------------------------

pub trait MapBuilder {
    fn build_map(&mut self);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
    fn get_spawn_list(&self) -> &Vec<(usize, String)>;

    fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.get_spawn_list().iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);

    /*
    builder.start_with(BspDungeonBuilder::new());
    builder.with(RoomBasedSpawner::new());
    builder.with(RoomBasedStartingPosition::new());
    builder.with(RoomBasedStairs::new());
    */
    builder.start_with(MazeBuilder::new());
    builder.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    builder.with(CullUnreachable::new());
    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());

    builder

    /*
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 19);
    // let builder = 1;
    let mut result: Box<dyn MapBuilder> = match builder {
        1 => Box::new(PrefabBuilder::constant(
            new_depth,
            prefab_builders::prefab_levels::WFC_POPULATED,
        )),
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
        20 => Box::new(PrefabBuilder::rex_level(
            new_depth,
            "../../resources/SmallDungeon_80x50.xp",
        )),
        _ => Box::new(SimpleMapBuilder::new(new_depth)),
    };

    if rng.roll_dice(1, 3) == 1 {
        result = Box::new(WaveformCollapseBuilder::derived_map(new_depth, result));
    }

    if rng.roll_dice(1, 20) == 1 {
        result = Box::new(PrefabBuilder::sectional(
            new_depth,
            prefab_builders::prefab_sections::UNDERGROUND_FORT,
            result,
        ));
    }

    Box::new(PrefabBuilder::vaults(new_depth, result))
    */
}
