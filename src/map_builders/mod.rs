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

pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);
    let (random_starter, has_rooms) = random_initial_builder(rng);

    builder.start_with(random_starter);

    if has_rooms {
        builder.with(RoomBasedSpawner::new());
        builder.with(RoomBasedStartingPosition::new());
        builder.with(RoomBasedStairs::new());
    } else {
        builder.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
        builder.with(CullUnreachable::new());
        builder.with(VoronoiSpawning::new());
        builder.with(DistantExit::new());
    }

    builder.with(PrefabBuilder::vaults());

    if rng.roll_dice(1, 3) == 1 {
        builder.with(WaveformCollapseBuilder::new());
    }

    if rng.roll_dice(1, 20) == 1 {
        builder.with(PrefabBuilder::sectional(prefab_builders::prefab_sections::UNDERGROUND_FORT));
    }

    builder.with(PrefabBuilder::vaults());

    builder

}

fn random_initial_builder(rng: &mut rltk::RandomNumberGenerator)
                    -> (Box<dyn InitialMapBuilder>, bool) {
    let builder = rng.roll_dice(1, 19);

    match builder {
        1 =>  (PrefabBuilder::constant(prefab_builders::prefab_levels::WFC_POPULATED), false),
        2 =>  (MazeBuilder::new(), false),
        3 =>  (BspInteriorBuilder::new(), true),
        4 =>  (CellularAutomataBuilder::new(), false),
        5 =>  (DrunkardsWalkBuilder::open_area(), false),
        6 =>  (DrunkardsWalkBuilder::open_halls(), false),
        7 =>  (DrunkardsWalkBuilder::winding_passages(), false),
        8 =>  (DLABuilder::walk_inwards(), false),
        9 =>  (DLABuilder::walk_outwards(), false),
        10 => (DLABuilder::central_attractor(), false),
        11 => (DLABuilder::insectoid(), false),
        12 => (DLABuilder::crazy(), false),
        13 => (DLABuilder::rorschach(), false),
        14 => (BspDungeonBuilder::new(), true),
        15 => (DrunkardsWalkBuilder::fat_passages(), false),
        16 => (DrunkardsWalkBuilder::fearful_symmetry(), false),
        17 => (VoronoiBuilder::manhattan(), false),
        18 => (VoronoiBuilder::chebyshev(), false),
        19 => (VoronoiBuilder::pythagoras(), false),
        20 => (PrefabBuilder::rex_level("../../resources/SmallDungeon_80x50.xp"), false),
        _ =>  (SimpleMapBuilder::new(), true),
    }
}
