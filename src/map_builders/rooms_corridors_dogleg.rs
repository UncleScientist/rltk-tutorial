use super::{apply_horizontal_tunnel, apply_vertical_tunnel, BuilderMap, MetaMapBuilder};

pub struct DoglegCorridors {}

impl MetaMapBuilder for DoglegCorridors {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.corridors(build_data);
    }
}

impl DoglegCorridors {
    pub fn new() -> Box<DoglegCorridors> {
        Box::new(DoglegCorridors {})
    }

    fn corridors(&mut self, build_data: &mut BuilderMap) {
        let rooms = if let Some(rooms_builder) = &build_data.rooms {
            rooms_builder.clone()
        } else {
            panic!("Dogleg corridors require a builder with room structures")
        };

        let mut corridors = Vec::new();

        for (i, room) in rooms.iter().enumerate().skip(1) {
            let (new_x, new_y) = room.center();
            let (prev_x, prev_y) = rooms[i - 1].center();
            if crate::tutorial::rng::range(0, 2) == 1 {
                let mut c1 = apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, prev_y);
                let mut c2 = apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, new_x);
                c1.append(&mut c2);
                corridors.push(c1);
            } else {
                let mut c1 = apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, prev_x);
                let mut c2 = apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, new_y);
                c1.append(&mut c2);
                corridors.push(c1);
            }
            build_data.take_snapshot();
        }

        build_data.corridors = Some(corridors);
    }
}
