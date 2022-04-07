use super::{BuilderMap, InitialMapBuilder, MetaMapBuilder, TileType};

pub struct CellularAutomataBuilder {}

impl MetaMapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.apply_iteration(build_data);
    }
}

impl InitialMapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl CellularAutomataBuilder {
    pub fn new() -> Box<CellularAutomataBuilder> {
        Box::new(CellularAutomataBuilder {})
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let roll = crate::tutorial::rng::roll_dice(1, 100);
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = if roll > 55 {
                    TileType::Floor
                } else {
                    TileType::Wall
                }
            }
        }
        build_data.take_snapshot();

        for _ in 0..15 {
            self.apply_iteration(build_data);
        }
    }

    fn apply_iteration(&mut self, build_data: &mut BuilderMap) {
        let mut newtiles = build_data.map.tiles.clone();

        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let idx = build_data.map.xy_idx(x, y);
                let mut neighbors = 0;

                neighbors += (build_data.map.tiles[idx - 1] == TileType::Wall) as usize;
                neighbors += (build_data.map.tiles[idx + 1] == TileType::Wall) as usize;
                neighbors += (build_data.map.tiles[idx - build_data.map.width as usize]
                    == TileType::Wall) as usize;
                neighbors += (build_data.map.tiles[idx + build_data.map.width as usize]
                    == TileType::Wall) as usize;
                neighbors += (build_data.map.tiles[idx - (build_data.map.width as usize - 1)]
                    == TileType::Wall) as usize;
                neighbors += (build_data.map.tiles[idx - (build_data.map.width as usize + 1)]
                    == TileType::Wall) as usize;
                neighbors += (build_data.map.tiles[idx + (build_data.map.width as usize - 1)]
                    == TileType::Wall) as usize;
                neighbors += (build_data.map.tiles[idx + (build_data.map.width as usize + 1)]
                    == TileType::Wall) as usize;

                if neighbors > 4 || neighbors == 0 {
                    newtiles[idx] = TileType::Wall;
                } else {
                    newtiles[idx] = TileType::Floor;
                }
            }
        }

        build_data.map.tiles = newtiles.clone();
        build_data.take_snapshot();
    }
}
