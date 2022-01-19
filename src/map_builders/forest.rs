use super::{
    AreaStartingPosition, BuilderChain, BuilderMap, CellularAutomataBuilder, CullUnreachable,
    MetaMapBuilder, TileType, VoronoiSpawning, XStart, YStart,
};
use crate::map;
use rltk::{a_star_search, DistanceAlg, Point, RandomNumberGenerator};

pub fn forest_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into the Woods");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Center));

    // Set up an exit and spawn mobs
    chain.with(VoronoiSpawning::new());
    chain.with(YellowBrickRoad::new());

    chain
}

pub struct YellowBrickRoad {}

impl MetaMapBuilder for YellowBrickRoad {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl YellowBrickRoad {
    pub fn new() -> Box<YellowBrickRoad> {
        Box::new(YellowBrickRoad {})
    }

    fn find_exit(&self, build_data: &mut BuilderMap, seed_x: i32, seed_y: i32) -> (i32, i32) {
        let mut available_floors: Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if map::tile_walkable(*tiletype) {
                available_floors.push((
                    idx,
                    DistanceAlg::PythagorasSquared.distance2d(
                        Point::new(
                            idx as i32 % build_data.map.width,
                            idx as i32 / build_data.map.width,
                        ),
                        Point::new(seed_x, seed_y),
                    ),
                ));
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let end_x = available_floors[0].0 as i32 % build_data.map.width;
        let end_y = available_floors[0].0 as i32 / build_data.map.width;
        (end_x, end_y)
    }

    fn paint_road(&self, build_data: &mut BuilderMap, x: i32, y: i32) {
        if x < 1 || x > build_data.map.width - 2 || y < 1 || y > build_data.map.height - 2 {
            return;
        }
        let idx = build_data.map.xy_idx(x, y);
        if build_data.map.tiles[idx] != TileType::DownStairs {
            build_data.map.tiles[idx] = TileType::Road;
        }
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let start_idx = build_data.map.xy_idx(starting_pos.x, starting_pos.y);

        let (end_x, end_y) = self.find_exit(
            build_data,
            build_data.map.width - 2,
            build_data.map.height / 2,
        );
        let end_idx = build_data.map.xy_idx(end_x, end_y);
        build_data.map.tiles[end_idx] = TileType::DownStairs;

        let path = a_star_search(start_idx, end_idx, &build_data.map);
        for idx in path.steps.iter() {
            let x = *idx as i32 % build_data.map.width;
            let y = *idx as i32 / build_data.map.width;
            self.paint_road(build_data, x, y);
            self.paint_road(build_data, x - 1, y);
            self.paint_road(build_data, x + 1, y);
            self.paint_road(build_data, x, y - 1);
            self.paint_road(build_data, x, y + 1);
        }
        build_data.take_snapshot();
    }
}
