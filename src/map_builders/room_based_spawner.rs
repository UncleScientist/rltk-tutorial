use super::{spawner, BuilderMap, MetaMapBuilder};

pub struct RoomBasedSpawner {}

impl MetaMapBuilder for RoomBasedSpawner {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl RoomBasedSpawner {
    pub fn new() -> Box<RoomBasedSpawner> {
        Box::new(RoomBasedSpawner {})
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            for room in rooms.iter().skip(1) {
                spawner::spawn_room(
                    &build_data.map,
                    room,
                    build_data.map.depth,
                    &mut build_data.spawn_list,
                );
            }
        } else {
            panic!("Room based spawning only works after rooms have been created");
        }
    }
}
