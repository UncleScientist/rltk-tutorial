use super::{BuilderMap, MetaMapBuilder, Position};

pub struct RoomBasedStartingPosition {}

impl MetaMapBuilder for RoomBasedStartingPosition {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl RoomBasedStartingPosition {
    pub fn new() -> Box<RoomBasedStartingPosition> {
        Box::new(RoomBasedStartingPosition {})
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            let (x, y) = rooms[0].center();
            build_data.starting_position = Some(Position { x, y });
        } else {
            panic!("Room based starting position only works after rooms have been created");
        }
    }
}
