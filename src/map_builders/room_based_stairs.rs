use super::{BuilderMap, MetaMapBuilder, TileType};

pub struct RoomBasedStairs {}

impl MetaMapBuilder for RoomBasedStairs {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl RoomBasedStairs {
    pub fn new() -> Box<RoomBasedStairs> {
        Box::new(RoomBasedStairs {})
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            let (x, y) = rooms[rooms.len() - 1].center();
            let stairs_idx = build_data.map.xy_idx(x, y);
            build_data.map.tiles[stairs_idx] = TileType::DownStairs;
            build_data.take_snapshot();
        } else {
            panic!("room based stairs only works after rooms have been created");
        }
    }
}
