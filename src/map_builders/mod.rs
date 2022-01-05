use rltk::RandomNumberGenerator;

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

mod rooms_draw;
use rooms_draw::*;

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

mod room_exploder;
use room_exploder::*;

mod room_sorter;
use room_sorter::*;

mod room_corner_rounding;
use room_corner_rounding::*;

mod rooms_corridors_dogleg;
use rooms_corridors_dogleg::*;

mod rooms_corridors_bsp;
use rooms_corridors_bsp::*;

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
            },
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

    pub fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
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
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}

fn random_start_position(rng: &mut RandomNumberGenerator) -> (XStart, YStart) {
    let x_start = match rng.roll_dice(1, 3) {
        1 => XStart::Left,
        2 => XStart::Center,
        _ => XStart::Right,
    };

    let y_start = match rng.roll_dice(1, 3) {
        1 => YStart::Bottom,
        2 => YStart::Center,
        _ => YStart::Top,
    };

    (x_start, y_start)
}

pub fn random_builder(new_depth: i32, rng: &mut RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);

    if std::env::var("QWER").is_err() {
        let type_roll = rng.roll_dice(1, 2);
        match type_roll {
            1 => random_room_builder(rng, &mut builder),
            _ => random_shape_builder(rng, &mut builder),
        }

        if rng.roll_dice(1, 20) == 1 {
            builder.with(PrefabBuilder::sectional(
                prefab_builders::prefab_sections::UNDERGROUND_FORT,
            ));
        }

        builder.with(PrefabBuilder::vaults());
    } else {
        builder.start_with(SimpleMapBuilder::new());
        builder.with(RoomDrawer::new());
        builder.with(RoomSorter::new(RoomSort::Leftmost));
        builder.with(BspCorridors::new());
        builder.with(RoomBasedSpawner::new());
        builder.with(RoomBasedStairs::new());
        builder.with(RoomBasedStartingPosition::new());
    }

    builder
}

// --------------------------------------------------------------------------------

fn random_room_builder(rng: &mut RandomNumberGenerator, builder: &mut BuilderChain) {
    let build_roll = rng.roll_dice(1, 3);
    match build_roll {
        1 => builder.start_with(SimpleMapBuilder::new()),
        2 => builder.start_with(BspDungeonBuilder::new()),
        _ => builder.start_with(BspInteriorBuilder::new()),
    }

    // BSP Interior still makes holes in walls
    if build_roll != 3 {
        match rng.roll_dice(1, 5) {
            1 => builder.with(RoomSorter::new(RoomSort::Leftmost)),
            2 => builder.with(RoomSorter::new(RoomSort::Rightmost)),
            3 => builder.with(RoomSorter::new(RoomSort::Topmost)),
            4 => builder.with(RoomSorter::new(RoomSort::Bottommost)),
            _ => builder.with(RoomSorter::new(RoomSort::Central)),
        }

        builder.with(RoomDrawer::new());

        match rng.roll_dice(1, 2) {
            1 => builder.with(DoglegCorridors::new()),
            _ => builder.with(BspCorridors::new()),
        }

        match rng.roll_dice(1, 6) {
            1 => builder.with(RoomExploder::new()),
            2 => builder.with(RoomCornerRounder::new()),
            _ => {}
        }
    }

    match rng.roll_dice(1, 2) {
        1 => builder.with(RoomBasedStartingPosition::new()),
        _ => {
            let (start_x, start_y) = random_start_position(rng);
            builder.with(AreaStartingPosition::new(start_x, start_y));
        }
    }

    match rng.roll_dice(1, 2) {
        1 => builder.with(RoomBasedStairs::new()),
        _ => builder.with(DistantExit::new()),
    }

    match rng.roll_dice(1, 2) {
        1 => builder.with(RoomBasedSpawner::new()),
        _ => builder.with(VoronoiSpawning::new()),
    }
}

fn random_shape_builder(rng: &mut RandomNumberGenerator, builder: &mut BuilderChain) {
    let builder_roll = rng.roll_dice(1, 19);

    match builder_roll {
        1 => builder.start_with(CellularAutomataBuilder::new()),
        2 => builder.start_with(DrunkardsWalkBuilder::open_area()),
        3 => builder.start_with(DrunkardsWalkBuilder::open_halls()),
        4 => builder.start_with(DrunkardsWalkBuilder::winding_passages()),
        5 => builder.start_with(DrunkardsWalkBuilder::fat_passages()),
        6 => builder.start_with(DrunkardsWalkBuilder::fearful_symmetry()),
        7 => builder.start_with(MazeBuilder::new()),
        8 => builder.start_with(DLABuilder::walk_inwards()),
        9 => builder.start_with(DLABuilder::walk_outwards()),
        10 => builder.start_with(DLABuilder::central_attractor()),
        11 => builder.start_with(DLABuilder::insectoid()),
        12 => builder.start_with(DLABuilder::crazy()),
        13 => builder.start_with(DLABuilder::rorschach()),
        14 => builder.start_with(DLABuilder::heavy_erosion()),
        15 => builder.start_with(VoronoiBuilder::manhattan()),
        16 => builder.start_with(VoronoiBuilder::chebyshev()),
        17 => builder.start_with(VoronoiBuilder::pythagoras()),

        18 => builder.start_with(PrefabBuilder::constant(
            prefab_builders::prefab_levels::WFC_POPULATED,
        )),

        _ => builder.start_with(PrefabBuilder::rex_level(
            "../../resources/SmallDungeon_80x50.xp",
        )),
    }

    if rng.roll_dice(1, 3) == 1 {
        builder.with(WaveformCollapseBuilder::new());
    }

    // Set the start to the center and cull
    builder.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    builder.with(CullUnreachable::new());

    // Now set the start to a random starting area
    let (start_x, start_y) = random_start_position(rng);
    builder.with(AreaStartingPosition::new(start_x, start_y));

    // Set up an exit and spawn the mobs
    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());
}
