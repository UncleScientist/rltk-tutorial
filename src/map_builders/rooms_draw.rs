use super::{BuilderMap, MetaMapBuilder, Rect, TileType};
use rltk::{DistanceAlg, Point};

pub struct RoomDrawer {}

impl MetaMapBuilder for RoomDrawer {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl RoomDrawer {
    pub fn new() -> Box<RoomDrawer> {
        Box::new(RoomDrawer {})
    }

    fn rectangle(&mut self, max_idx: usize, build_data: &mut BuilderMap, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = build_data.map.xy_idx(x, y);
                if idx > 0 && idx < max_idx {
                    build_data.map.tiles[idx] = TileType::Floor;
                }
            }
        }
    }

    fn circle(&mut self, max_idx: usize, build_data: &mut BuilderMap, room: &Rect) {
        let radius = i32::min(room.x2 - room.x1, room.y2 - room.y1) as f32 / 2.0;
        let center = room.center();
        let center_pt = Point::new(center.0, center.1);

        for y in room.y1..=room.y2 {
            for x in room.x1..=room.x2 {
                let idx = build_data.map.xy_idx(x, y);
                let distance = DistanceAlg::Pythagoras.distance2d(center_pt, Point::new(x, y));
                if idx > 0 && idx < max_idx && distance <= radius {
                    build_data.map.tiles[idx] = TileType::Floor;
                }
            }
        }
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        let rooms = if let Some(rooms_builder) = &build_data.rooms {
            rooms_builder.clone()
        } else {
            panic!("Room rounding requires a builder with room structures")
        };

        let max_idx = ((build_data.map.width * build_data.map.height) - 1) as usize;

        for room in rooms.iter() {
            if crate::tutorial::rng::roll_dice(1, 4) == 1 {
                self.circle(max_idx, build_data, room);
            } else {
                self.rectangle(max_idx, build_data, room);
            }
            build_data.take_snapshot();
        }
    }
}
