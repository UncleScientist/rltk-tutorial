use crate::map_builders::*;
use crate::*;

mod prefab_levels;
mod prefab_sections;

#[derive(PartialEq, Clone)]
pub enum PrefabMode {
    /*
    RexLevel {
        template: &'static str,
    },
    Constant {
        level: prefab_levels::PrefabLevel,
    },
    */
    Sectional {
        section: prefab_sections::PrefabSection,
    },
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    #[allow(dead_code)]
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    spawns: Vec<(usize, String)>,
    previous_builder: Option<Box<dyn MapBuilder>>,
    spawn_list: Vec<(usize, String)>,
}

impl MapBuilder for PrefabBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.get_spawn_list().iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl PrefabBuilder {
    pub fn new(new_depth: i32, previous_builder: Option<Box<dyn MapBuilder>>) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            /*
            mode: PrefabMode::RexLevel {
                template: "../../resources/wfc-populated.xp",
            },
            */
            // mode: PrefabMode::Constant { level: prefab_levels::WFC_POPULATED },
            mode: PrefabMode::Sectional {
                section: prefab_sections::UNDERGROUND_FORT,
            },
            spawns: Vec::new(),
            previous_builder,
            spawn_list: Vec::new(),
        }
    }

    fn build(&mut self) {
        match self.mode {
            /*PrefabMode::RexLevel { template } => self.load_rex_map(template),
            PrefabMode::Constant { level } => self.load_ascii_map(&level),*/
            PrefabMode::Sectional { section } => self.apply_sectional(&section),
        }
        self.take_snapshot();

        if self.starting_position.x == 0 {
            // Find a starting point; start in the middle and walk left until we
            // find an open tile
            self.starting_position = Position {
                x: self.map.width / 2,
                y: self.map.height / 2,
            };
            let mut start_idx = self
                .map
                .xy_idx(self.starting_position.x, self.starting_position.y);
            while self.map.tiles[start_idx] != TileType::Floor {
                self.starting_position.x -= 1;
                start_idx = self
                    .map
                    .xy_idx(self.starting_position.x, self.starting_position.y);
            }
            self.take_snapshot();

            let exit_tile =
                remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
            self.take_snapshot();

            self.map.tiles[exit_tile] = TileType::DownStairs;
            self.take_snapshot();
        }
    }

    /*
    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        let idx = self.map.xy_idx(x as i32, y as i32);
                        self.char_to_map(cell.ch as u8 as char, idx);
                    }
                }
            }
        }
    }
    */
    fn read_ascii_to_vec(template: &str, width: usize) -> Vec<char> {
        let mut string_vec: Vec<char> = Vec::new();
        let mut hpos = 0;
        for c in template.chars() {
            if c as u8 == 160u8 {
                string_vec.push(' ');
                hpos += 1;
            } else if c == '\n' || c == '\r' {
                while hpos < width {
                    string_vec.push(' ');
                    hpos += 1;
                }
                hpos = 0;
            } else {
                string_vec.push(c);
                hpos += 1;
            }
        }
        string_vec
    }

    /*
    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(level.template, level.width);

        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let idx = self.map.xy_idx(tx as i32, ty as i32);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
    }
    */

    fn char_to_map(&mut self, ch: char, idx: usize) {
        match (ch as u8) as char {
            ' ' => self.map.tiles[idx] = TileType::Floor,
            '#' => self.map.tiles[idx] = TileType::Wall,
            '@' => {
                let x = idx as i32 % self.map.width;
                let y = idx as i32 / self.map.width;
                self.map.tiles[idx] = TileType::Floor;
                self.starting_position = Position {
                    x: x as i32,
                    y: y as i32,
                }
            }
            'g' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Goblin".to_string()));
            }
            'o' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Orc".to_string()));
            }
            '^' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Bear Trap".to_string()));
            }
            '%' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Rations".to_string()));
            }
            '!' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Health Potion".to_string()));
            }
            _ => {
                rltk::console::log(format!("Unknwon glyph {}", ch as u8 as char));
            }
        }
    }

    fn apply_sectional(&mut self, section: &prefab_sections::PrefabSection) {
        use prefab_sections::*;

        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template, section.width);

        // Place the new section
        let chunk_x = match section.placement.0 {
            //HorizontalPlacement::Left => 0,
            //HorizontalPlacement::Center => (self.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => (self.map.width - 1) - section.width as i32,
        };

        let chunk_y = match section.placement.1 {
            VerticalPlacement::Top => 0,
            //VerticalPlacement::Center => (self.map.height / 2) - (section.height as i32 / 2),
            //VerticalPlacement::Bottom => (self.map.height - 1) - section.height as i32,
        };

        // Build the map
        let prev_builder = self.previous_builder.as_mut().unwrap();
        prev_builder.build_map();
        self.starting_position = prev_builder.get_starting_position();
        self.map = prev_builder.get_map();
        for e in prev_builder.get_spawn_list().iter() {
            let idx = e.0;
            let x = idx as i32 % self.map.width;
            let y = idx as i32 / self.map.width;
            if x < chunk_x
                || x > (chunk_x + section.width as i32)
                || y < chunk_y
                || y > (chunk_y + section.height as i32)
            {
                rltk::console::log(format!(
                    "apply_sectional: spawning at {},{} - {}",
                    idx as i32 % self.map.width,
                    idx as i32 / self.map.width,
                    e.1
                ));
                self.spawn_list.push((idx, e.1.to_string()));
            }
        }
        self.take_snapshot();

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let idx = self.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
        self.take_snapshot();
    }
}
