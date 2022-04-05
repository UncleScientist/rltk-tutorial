use super::*;
use rltk::prelude::*;

use crate::{map::camera, Attributes, Duration, Hidden, Map, Name, Pools, StatusEffect};

pub struct Tooltip {
    lines: Vec<String>,
}

impl Default for Tooltip {
    fn default() -> Self {
        Self::new()
    }
}

impl Tooltip {
    pub fn new() -> Tooltip {
        Tooltip { lines: Vec::new() }
    }

    pub fn add<S: ToString>(&mut self, line: S) {
        self.lines.push(line.to_string())
    }

    pub fn width(&self) -> i32 {
        self.lines.iter().map(|x| x.len()).max().unwrap() as i32 + 2
    }

    pub fn height(&self) -> i32 {
        self.lines.len() as i32 + 2
    }

    pub fn render(&self, ctx: &mut Rltk, x: i32, y: i32) {
        let box_grey = RGB::from_hex("#999999").expect("oops");
        let light_grey = RGB::from_hex("#DDDDDD").expect("oops");
        let white = RGB::named(WHITE);
        let black = RGB::named(BLACK);

        ctx.draw_box(x, y, self.width() - 1, self.height() - 1, white, box_grey);
        for (i, s) in self.lines.iter().enumerate() {
            let col = if i == 0 { white } else { light_grey };
            ctx.print_color(x + 1, y + i as i32 + 1, col, black, &s);
        }
    }
}

pub fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let (min_x, _, min_y, _) = camera::get_screen_bounds(ecs, ctx);
    let map = ecs.fetch::<Map>();
    let hidden = ecs.read_storage::<Hidden>();
    let attributes = ecs.read_storage::<Attributes>();
    let pools = ecs.read_storage::<Pools>();

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;
    if mouse_pos.0 < 1 || mouse_pos.0 > 49 || mouse_pos.1 < 1 || mouse_pos.1 > 40 {
        return;
    }

    if mouse_map_pos.0 >= map.width - 1
        || mouse_map_pos.1 >= map.height - 1
        || mouse_map_pos.0 < 1
        || mouse_map_pos.1 < 1
    {
        return;
    }

    // if !map.in_bounds(rltk::Point::new(mouse_map_pos.0, mouse_map_pos.1)) { return; }
    let mouse_idx = map.xy_idx(mouse_map_pos.0, mouse_map_pos.1);
    if !map.visible_tiles[mouse_idx] {
        return;
    }

    let mut tip_boxes: Vec<Tooltip> = Vec::new();
    crate::spatial::for_each_tile_content(mouse_idx, |entity| {
        if hidden.get(entity).is_some() {
            return;
        }

        let mut tip = Tooltip::new();
        tip.add(get_item_display_name(ecs, entity));

        if let Some(attr) = attributes.get(entity) {
            let mut s = "".to_string(); // String::new()
            if attr.might.bonus < 0 {
                s += "Weak. "
            };
            if attr.might.bonus > 0 {
                s += "Strong. "
            };
            if attr.quickness.bonus < 0 {
                s += "Clumsy. "
            };
            if attr.quickness.bonus > 0 {
                s += "Agile. "
            };
            if attr.fitness.bonus < 0 {
                s += "Unhealthy. "
            };
            if attr.fitness.bonus > 0 {
                s += "Healthy. "
            };
            if attr.intelligence.bonus < 0 {
                s += "Unintelligent. "
            };
            if attr.intelligence.bonus > 0 {
                s += "Smart. "
            };
            if s.is_empty() {
                s = "Quite Average".to_string()
            }
            tip.add(s);
        }

        // Comment on pools
        if let Some(stat) = pools.get(entity) {
            tip.add(format!("Level: {}", stat.level));
        }

        // Status effects
        let statuses = ecs.read_storage::<StatusEffect>();
        let durations = ecs.read_storage::<Duration>();
        let names = ecs.read_storage::<Name>();
        for (status, duration, name) in (&statuses, &durations, &names).join() {
            if status.target == entity {
                tip.add(format!("{} ({})", name.name, duration.turns));
            }
        }

        tip_boxes.push(tip);
    });

    if tip_boxes.is_empty() {
        return;
    }

    let box_grey = RGB::from_hex("#999999").expect("oops");
    let white = RGB::named(WHITE);

    let arrow_y = mouse_pos.1;
    let (arrow, arrow_x) = if mouse_pos.0 < 40 {
        (to_cp437('→'), mouse_pos.0 - 1)
    } else {
        (to_cp437('←'), mouse_pos.0 + 1)
    };
    ctx.set(arrow_x, arrow_y, white, box_grey, arrow);

    let total_height = tip_boxes.iter().map(|tt| tt.height()).sum::<i32>();
    let mut y = mouse_pos.1 - (total_height / 2);
    while y + (total_height / 2) > 50 {
        y -= 1;
    }

    for tt in tip_boxes.iter() {
        let x = if mouse_pos.0 < 40 {
            mouse_pos.0 - (1 + tt.width())
        } else {
            mouse_pos.0 + (1 + tt.width())
        };
        tt.render(ctx, x, y);
        y += tt.height();
    }
}
