use std::collections::HashSet;

use super::{BuilderMap, MetaMapBuilder, TileType};
use rltk::{line2d, DistanceAlg, LineAlg, Point, RandomNumberGenerator};

pub struct StraightLineCorridors {}

impl MetaMapBuilder for StraightLineCorridors {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.corridors(rng, build_data);
    }
}

impl StraightLineCorridors {
    pub fn new() -> Box<StraightLineCorridors> {
        Box::new(StraightLineCorridors {})
    }

    fn corridors(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms = if let Some(rooms_builder) = &build_data.rooms {
            rooms_builder.clone()
        } else {
            panic!("straight line corridors require a builder with room structures")
        };

        let mut connected = HashSet::new();
        for (i, room) in rooms.iter().enumerate() {
            let mut room_distance = Vec::new();
            let room_center = room.center();
            let room_center_pt = Point::new(room_center.0, room_center.1);
            for (j, other_room) in rooms.iter().enumerate() {
                if i != j && !connected.contains(&j) {
                    let other_center = other_room.center();
                    let other_center_pt = Point::new(other_center.0, other_center.1);
                    let distance =
                        DistanceAlg::Pythagoras.distance2d(room_center_pt, other_center_pt);
                    room_distance.push((j, distance));
                }
            }

            if !room_distance.is_empty() {
                room_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                let dest_center = rooms[room_distance[0].0].center();

                let line = line2d(
                    LineAlg::Bresenham,
                    room_center_pt,
                    Point::new(dest_center.0, dest_center.1),
                );
                for cell in line.iter() {
                    let idx = build_data.map.xy_idx(cell.x, cell.y);
                    build_data.map.tiles[idx] = TileType::Floor;
                }

                connected.insert(i);
                build_data.take_snapshot();
            }
        }
    }
}