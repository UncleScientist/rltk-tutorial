use super::{MetaMapBuilder, BuilderMap, Position};
use rltk::RandomNumberGenerator;

pub struct RoomBasedStartingPosition {}

impl MetaMapBuilder for RoomBasedStartingPosition {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomBasedStartingPosition {
    pub fn new() -> Box<RoomBasedStartingPosition> {
        Box::new(RoomBasedStartingPosition{})
    }

    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            let (x, y) = rooms[0].center();
            build_data.starting_position = Some(Position { x, y });
        } else {
            panic!("Room based starting position only works after rooms have been created");
        }
    }
}
