use super::{BuilderMap, MetaMapBuilder, TileType};

pub struct DoorPlacement {}

impl MetaMapBuilder for DoorPlacement {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.doors(build_data);
    }
}

impl DoorPlacement {
    pub fn new() -> Box<DoorPlacement> {
        Box::new(DoorPlacement {})
    }

    fn doors(&mut self, build_data: &mut BuilderMap) {
        if let Some(halls_original) = &build_data.corridors {
            let halls = halls_original.clone(); // To avoid nested borrowing?
            for hall in halls.iter() {
                if hall.len() > 2 && self.door_possible(build_data, hall[0]) {
                    build_data.spawn_list.push((hall[0], "Door".to_string()));
                }
            }
        } else {
            // There are no corridors - scan for possible places
            let tiles = build_data.map.tiles.clone();
            for (i, tile) in tiles.iter().enumerate() {
                if *tile == TileType::Floor
                    && self.door_possible(build_data, i)
                    && crate::tutorial::rng::roll_dice(1, 3) == 1
                {
                    build_data.spawn_list.push((i, "Door".to_string()));
                }
            }
        }
    }

    fn door_possible(&self, build_data: &mut BuilderMap, idx: usize) -> bool {
        use TileType::*;

        for spawn in build_data.spawn_list.iter() {
            if spawn.0 == idx {
                return false;
            }
        }

        let x = idx % build_data.map.width as usize;
        let y = idx / build_data.map.width as usize;
        let w = build_data.map.width as usize;
        let h = build_data.map.height as usize;

        // Check for east-west door possibility
        if build_data.map.tiles[idx] == Floor
            && (x > 1 && build_data.map.tiles[idx - 1] == Floor)
            && (x < w - 2 && build_data.map.tiles[idx + 1] == Floor)
            && (y > 1 && build_data.map.tiles[idx - w] == Wall)
            && (y < h - 2 && build_data.map.tiles[idx + w] == Wall)
        {
            return true;
        }

        // Check for north-south door possibility
        if build_data.map.tiles[idx] == Floor
            && (x > 1 && build_data.map.tiles[idx - 1] == Wall)
            && (x < w - 2 && build_data.map.tiles[idx + 1] == Wall)
            && (y > 1 && build_data.map.tiles[idx - w] == Floor)
            && (y < h - 2 && build_data.map.tiles[idx + w] == Floor)
        {
            return true;
        }

        false
    }
}
