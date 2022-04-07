use super::{BuilderMap, MetaMapBuilder, Rect};
use rltk::{DistanceAlg, Point};

pub enum RoomSort {
    Leftmost,
    Rightmost,
    Topmost,
    Bottommost,
    Central,
}

pub struct RoomSorter {
    sort_by: RoomSort,
}

impl MetaMapBuilder for RoomSorter {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.sorter(build_data);
    }
}

impl RoomSorter {
    pub fn new(sort_by: RoomSort) -> Box<RoomSorter> {
        Box::new(RoomSorter { sort_by })
    }

    fn sorter(&mut self, build_data: &mut BuilderMap) {
        match self.sort_by {
            RoomSort::Leftmost => build_data
                .rooms
                .as_mut()
                .unwrap()
                .sort_by(|a, b| a.x1.cmp(&b.x1)),
            RoomSort::Rightmost => build_data
                .rooms
                .as_mut()
                .unwrap()
                .sort_by(|a, b| b.x2.cmp(&a.x2)),
            RoomSort::Topmost => build_data
                .rooms
                .as_mut()
                .unwrap()
                .sort_by(|a, b| a.y1.cmp(&b.y1)),
            RoomSort::Bottommost => build_data
                .rooms
                .as_mut()
                .unwrap()
                .sort_by(|a, b| b.y2.cmp(&a.y2)),
            RoomSort::Central => {
                let map_center = Point::new(build_data.map.width / 2, build_data.map.height / 2);
                let center_sort = |a: &Rect, b: &Rect| {
                    let a_center = a.center();
                    let a_center_pt = Point::new(a_center.0, a_center.1);
                    let b_center = b.center();
                    let b_center_pt = Point::new(b_center.0, b_center.1);

                    let distance_a = DistanceAlg::Pythagoras.distance2d(a_center_pt, map_center);
                    let distance_b = DistanceAlg::Pythagoras.distance2d(b_center_pt, map_center);

                    distance_a.partial_cmp(&distance_b).unwrap()
                };

                build_data.rooms.as_mut().unwrap().sort_by(center_sort);
            }
        }
    }
}
