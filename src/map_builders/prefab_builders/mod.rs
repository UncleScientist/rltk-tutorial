use rltk::RandomNumberGenerator;

use crate::map_builders::*;
use crate::*;

mod prefab_levels;
mod prefab_sections;
mod prefab_rooms;

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
    RoomVaults,
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    #[allow(dead_code)]
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
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
            // mode: PrefabMode::Sectional { section: prefab_sections::UNDERGROUND_FORT, },
            mode: PrefabMode::RoomVaults,
            previous_builder,
            spawn_list: Vec::new(),
        }
    }

    fn build(&mut self) {
        match self.mode {
            /*PrefabMode::RexLevel { template } => self.load_rex_map(template),
            PrefabMode::Constant { level } => self.load_ascii_map(&level),*/
            PrefabMode::Sectional { section } => self.apply_sectional(&section),
            PrefabMode::RoomVaults => self.apply_room_vaults(),
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
                self.spawn_list.push((idx, "Goblin".to_string()));
            }
            'o' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Orc".to_string()));
            }
            '^' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Bear Trap".to_string()));
            }
            '%' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Rations".to_string()));
            }
            '!' => {
                rltk::console::log(format!("that was the health potion"));
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Health Potion".to_string()));
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

        self.apply_previous_iteration(|x, y, _| { x < chunk_x
            || x > (chunk_x + section.width as i32)
            || y < chunk_y
            || y > (chunk_y + section.height as i32)
        });

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

    fn apply_previous_iteration<F>(&mut self, mut filter: F)
        where F: FnMut(i32, i32, &(usize, String)) -> bool
    {
        // Build the map
        let prev_builder = self.previous_builder.as_mut().unwrap();
        prev_builder.build_map();
        self.starting_position = prev_builder.get_starting_position();
        self.map = prev_builder.get_map();
        for e in prev_builder.get_spawn_list().iter() {
            let idx = e.0;
            let x = idx as i32 % self.map.width;
            let y = idx as i32 / self.map.width;
            if filter(x, y, e) {
                self.spawn_list.push((idx, e.1.to_string()));
            }
        }
        self.take_snapshot();
    }

    fn apply_room_vaults(&mut self) {
        use prefab_rooms::*;
        let mut rng = RandomNumberGenerator::new();

        rltk::console::log("looking for room vault locations");
        
        self.apply_previous_iteration(|_, _, _| true);

        let master_vault_list = vec![TOTALLY_NOT_A_TRAP];

        let possible_vaults: Vec<&PrefabRoom> = master_vault_list.iter()
            .filter(|v| { self.depth >= v.first_depth && self.depth <= v.last_depth })
            .collect();

        if possible_vaults.is_empty() {
            return;
        }

        let vault_index = if possible_vaults.len() == 1 { 0 } else { 
            (rng.roll_dice(1, possible_vaults.len() as i32) - 1) as usize };
        let vault = possible_vaults[vault_index];

        let mut vault_positions: Vec<Position> = Vec::new();
        for idx in 0..self.map.tiles.len() {
            let x = (idx % self.map.width as usize) as i32;
            let y = (idx / self.map.width as usize) as i32;

            if x > 1 && (x + vault.width as i32) < self.map.width - 2 &&
                y > 1 && (y + vault.height as i32) < self.map.height - 2 {
                    let mut possible = true;
                    'out: for ty in 0..vault.height as i32 {
                        for tx in 0..vault.width as i32 {
                            let idx = self.map.xy_idx(tx + x, ty + y);
                            if self.map.tiles[idx] != TileType::Floor {
                                possible = false;
                                break 'out;
                            }
                        }
                    }

                    if possible {
                        vault_positions.push(Position{ x, y });
                    }
                }
        }

        if !vault_positions.is_empty() {
            let pos_idx = if vault_positions.len() == 1 { 0 } else { 
                (rng.roll_dice(1, vault_positions.len() as i32) - 1) as usize };
            let pos = &vault_positions[pos_idx];

            rltk::console::log(format!("room vault at {} {}", pos.x, pos.y));

            let chunk_x = pos.x;
            let chunk_y = pos.y;

            let string_vec = PrefabBuilder::read_ascii_to_vec(vault.template, vault.width);
            let mut i = 0;
            for ty in 0..vault.height {
                for tx in 0..vault.width {
                    let idx = self.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    rltk::console::log(format!("idx {}, char {}", idx, string_vec[i]));
                    self.char_to_map(string_vec[i], idx);
                    i += 1;
                }
            }
            self.take_snapshot();
        }
    }
}
