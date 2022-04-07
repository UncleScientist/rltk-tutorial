use super::{BuilderMap, MetaMapBuilder, TileType};
use crate::draw_corridor;

pub struct BspCorridors {}

impl MetaMapBuilder for BspCorridors {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.corridors(build_data);
    }
}

impl BspCorridors {
    pub fn new() -> Box<BspCorridors> {
        Box::new(BspCorridors {})
    }

    fn corridors(&mut self, build_data: &mut BuilderMap) {
        let rooms = if let Some(rooms_builder) = &build_data.rooms {
            rooms_builder.clone()
        } else {
            panic!("BSP corridors requires a builder with room structures")
        };

        const MAX_SEARCH: usize = 32;
        let mut corridors = Vec::new();

        for i in 0..rooms.len() - 1 {
            let room = rooms[i];
            let next_room = rooms[i + 1];

            let mut start_x =
                room.x1 + (crate::tutorial::rng::roll_dice(1, i32::abs(room.x1 - room.x2)) - 1);
            let mut start_y =
                room.y1 + (crate::tutorial::rng::roll_dice(1, i32::abs(room.y1 - room.y2)) - 1);

            let mut count = 0;
            while count < MAX_SEARCH {
                let idx = build_data.map.xy_idx(start_x, start_y);
                if build_data.map.tiles[idx] == TileType::Floor {
                    break;
                }
                start_x =
                    room.x1 + (crate::tutorial::rng::roll_dice(1, i32::abs(room.x1 - room.x2)) - 1);
                start_y =
                    room.y1 + (crate::tutorial::rng::roll_dice(1, i32::abs(room.y1 - room.y2)) - 1);
                count += 1;
            }
            if count == MAX_SEARCH {
                let center = room.center();
                start_x = center.0;
                start_y = center.1;
            }

            let mut end_x = next_room.x1
                + (crate::tutorial::rng::roll_dice(1, i32::abs(next_room.x1 - next_room.x2)) - 1);
            let mut end_y = next_room.y1
                + (crate::tutorial::rng::roll_dice(1, i32::abs(next_room.y1 - next_room.y2)) - 1);

            count = 0;
            while count < MAX_SEARCH {
                let idx = build_data.map.xy_idx(end_x, end_y);
                if build_data.map.tiles[idx] == TileType::Floor {
                    break;
                }
                end_x = next_room.x1
                    + (crate::tutorial::rng::roll_dice(1, i32::abs(next_room.x1 - next_room.x2))
                        - 1);
                end_y = next_room.y1
                    + (crate::tutorial::rng::roll_dice(1, i32::abs(next_room.y1 - next_room.y2))
                        - 1);
                count += 1;
            }
            if count == MAX_SEARCH {
                let center = next_room.center();
                end_x = center.0;
                end_y = center.1;
            }

            let corridor = draw_corridor(&mut build_data.map, start_x, start_y, end_x, end_y);
            corridors.push(corridor);
            build_data.take_snapshot();
        }
        build_data.rooms = Some(rooms);
        build_data.corridors = Some(corridors);
    }
}
