use super::{
    AreaStartingPosition, BuilderChain, BuilderMap, CullUnreachable, DLABuilder, DistantExit,
    DrunkardsWalkBuilder, MetaMapBuilder, PrefabBuilder, TileType, VoronoiSpawning, XStart, YStart,
};
use rltk::RandomNumberGenerator;

pub fn limestone_cavern_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Limestone Caverns");

    chain.start_with(DrunkardsWalkBuilder::winding_passages());
    chain.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Center));
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain.with(CaveDecorator::new());

    chain
}

pub fn limestone_deep_cavern_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Deep Limestone Caverns");

    chain.start_with(DLABuilder::central_attractor());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Top));
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain.with(CaveDecorator::new());
    chain.with(PrefabBuilder::sectional(
        super::prefab_builders::prefab_sections::ORC_CAMP,
    ));

    chain
}

pub struct CaveDecorator {}

impl MetaMapBuilder for CaveDecorator {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CaveDecorator {
    pub fn new() -> Box<CaveDecorator> {
        Box::new(CaveDecorator {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let old_map = build_data.map.clone();
        for (idx, tt) in build_data.map.tiles.iter_mut().enumerate() {
            // Gravel Spawning
            if *tt == TileType::Floor && rng.roll_dice(1, 6) == 1 {
                *tt = TileType::Gravel;
            } else if *tt == TileType::Floor && rng.roll_dice(1, 10) == 1 {
                *tt = TileType::ShallowWater;
            } else if *tt == TileType::Wall {
                // Spawn deep pools and stalactites
                let mut neighbors = 0;
                let x = idx as i32 % old_map.width;
                let y = idx as i32 / old_map.width;
                if x > 0 && old_map.tiles[idx - 1] == TileType::Wall {
                    neighbors += 1;
                }
                if x < old_map.width - 2 && old_map.tiles[idx + 1] == TileType::Wall {
                    neighbors += 1;
                }
                if y > 0 && old_map.tiles[idx - old_map.width as usize] == TileType::Wall {
                    neighbors += 1;
                }
                if y < old_map.height - 2
                    && old_map.tiles[idx + old_map.width as usize] == TileType::Wall
                {
                    neighbors += 1;
                }
                if neighbors == 2 {
                    *tt = TileType::DeepWater;
                } else if neighbors == 1 {
                    match rng.roll_dice(1, 4) {
                        1 => *tt = TileType::Stalactite,
                        2 => *tt = TileType::Stalagmite,
                        _ => {}
                    }
                }
            }
        }
        build_data.take_snapshot();
        build_data.map.outdoors = false;
    }
}
