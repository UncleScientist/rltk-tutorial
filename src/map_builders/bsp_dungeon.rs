use super::{BuilderMap, InitialMapBuilder, Rect, TileType};

pub struct BspDungeonBuilder {
    rects: Vec<Rect>,
}

impl InitialMapBuilder for BspDungeonBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl BspDungeonBuilder {
    pub fn new() -> Box<BspDungeonBuilder> {
        Box::new(BspDungeonBuilder { rects: Vec::new() })
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        let mut rooms: Vec<Rect> = Vec::new();

        self.rects.clear();
        self.rects.push(Rect::new(
            2,
            2,
            build_data.map.width - 5,
            build_data.map.height - 5,
        ));
        let first_room = self.rects[0];
        self.add_subrects(first_room);

        // Up to 240 times...
        let mut n_rooms = 0;
        while n_rooms < 240 {
            let rect = self.get_random_rect();
            let candidate = self.get_random_sub_rect(rect);

            if self.is_possible(candidate, build_data, &rooms) {
                rooms.push(candidate);
                self.add_subrects(rect);
            }

            n_rooms += 1;
        }

        build_data.rooms = Some(rooms)
    }

    fn add_subrects(&mut self, rect: Rect) {
        let width = i32::abs(rect.x1 - rect.x2);
        let height = i32::abs(rect.y1 - rect.y2);
        let half_width = i32::max(width / 2, 1);
        let half_height = i32::max(height / 2, 1);

        self.rects
            .push(Rect::new(rect.x1, rect.y1, half_width, half_height));
        self.rects.push(Rect::new(
            rect.x1,
            rect.y1 + half_height,
            half_width,
            half_height,
        ));
        self.rects.push(Rect::new(
            rect.x1 + half_width,
            rect.y1,
            half_width,
            half_height,
        ));
        self.rects.push(Rect::new(
            rect.x1 + half_width,
            rect.y1 + half_height,
            half_width,
            half_height,
        ));
    }

    fn get_random_rect(&mut self) -> Rect {
        if self.rects.len() == 1 {
            return self.rects[0];
        }
        let idx = (crate::tutorial::rng::roll_dice(1, self.rects.len() as i32) - 1) as usize;
        self.rects[idx]
    }

    fn get_random_sub_rect(&mut self, rect: Rect) -> Rect {
        let mut result = rect;
        let rect_width = i32::abs(rect.x1 - rect.x2);
        let rect_height = i32::abs(rect.y1 - rect.y2);

        let w = i32::max(
            3,
            crate::tutorial::rng::roll_dice(1, i32::min(rect_width, 10)) - 1,
        ) + 1;
        let h = i32::max(
            3,
            crate::tutorial::rng::roll_dice(1, i32::min(rect_height, 10)) - 1,
        ) + 1;

        result.x1 += crate::tutorial::rng::roll_dice(1, 6) - 1;
        result.y1 += crate::tutorial::rng::roll_dice(1, 6) - 1;
        result.x2 = result.x1 + w;
        result.y2 = result.y1 + h;

        result
    }

    fn is_possible(&self, rect: Rect, build_data: &BuilderMap, rooms: &[Rect]) -> bool {
        let expanded = Rect {
            x1: rect.x1 - 2,
            y1: rect.y1 - 2,
            x2: rect.x2 + 2,
            y2: rect.y2 + 2,
        };

        for r in rooms.iter() {
            if r.intersect(&rect) {
                return false;
            }
        }

        for y in expanded.y1..=expanded.y2 {
            for x in expanded.x1..=expanded.x2 {
                if x > build_data.map.width - 2 {
                    return false;
                }
                if y > build_data.map.height - 2 {
                    return false;
                }
                if x < 1 {
                    return false;
                }
                if y < 1 {
                    return false;
                }
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] != TileType::Wall {
                    return false;
                }
            }
        }

        true
    }
}
