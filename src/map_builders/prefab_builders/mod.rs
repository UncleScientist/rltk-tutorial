use rltk::RandomNumberGenerator;

use super::{BuilderMap, InitialMapBuilder, MetaMapBuilder, Position, TileType};

pub mod prefab_levels;
pub mod prefab_rooms;
pub mod prefab_sections;

#[derive(PartialEq, Clone)]
pub enum PrefabMode {
    RexLevel {
        template: &'static str,
    },
    Constant {
        level: prefab_levels::PrefabLevel,
    },
    Sectional {
        section: prefab_sections::PrefabSection,
    },
    RoomVaults,
}

pub struct PrefabBuilder {
    mode: PrefabMode,
}

impl MetaMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl InitialMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl PrefabBuilder {
    pub fn sectional(section: prefab_sections::PrefabSection) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::Sectional { section },
        })
    }

    pub fn constant(level: prefab_levels::PrefabLevel) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::Constant { level },
        })
    }

    pub fn rex_level(template: &'static str) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::RexLevel { template },
        })
    }

    pub fn vaults() -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::RoomVaults,
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(template, build_data),
            PrefabMode::Constant { level } => self.load_ascii_map(&level, build_data),
            PrefabMode::Sectional { section } => self.apply_sectional(&section, rng, build_data),
            PrefabMode::RoomVaults => self.apply_room_vaults(rng, build_data),
        }
        build_data.take_snapshot();
    }

    fn load_rex_map(&mut self, path: &str, build_data: &mut BuilderMap) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < build_data.map.width as usize && y < build_data.map.height as usize {
                        let idx = build_data.map.xy_idx(x as i32, y as i32);
                        self.char_to_map(cell.ch as u8 as char, idx, build_data);
                    }
                }
            }
        }
    }

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

    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel, build_data: &mut BuilderMap) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(level.template, level.width);

        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < build_data.map.width as usize && ty < build_data.map.height as usize {
                    let idx = build_data.map.xy_idx(tx as i32, ty as i32);
                    self.char_to_map(string_vec[i], idx, build_data);
                }
                i += 1;
            }
        }
    }

    fn char_to_map(&mut self, ch: char, idx: usize, build_data: &mut BuilderMap) {
        match ch {
            '≈' => build_data.map.tiles[idx] = TileType::DeepWater,
            'O' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Orc Leader".to_string()));
            }
            '☼' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Watch Fire".to_string()));
            }
            ' ' => build_data.map.tiles[idx] = TileType::Floor,
            '#' => build_data.map.tiles[idx] = TileType::Wall,
            '@' => {
                let x = idx as i32 % build_data.map.width;
                let y = idx as i32 / build_data.map.width;
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.starting_position = Some(Position {
                    x: x as i32,
                    y: y as i32,
                });
            }
            '>' => build_data.map.tiles[idx] = TileType::DownStairs,
            'e' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Dark Elf".to_string()));
            }
            'g' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Goblin".to_string()));
            }
            'o' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Orc".to_string()));
            }
            '^' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Bear Trap".to_string()));
            }
            '%' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Rations".to_string()));
            }
            '!' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data
                    .spawn_list
                    .push((idx, "Health Potion".to_string()));
            }
            _ => {
                rltk::console::log(format!("Unknown glyph {}", ch as u8 as char));
            }
        }
    }

    fn apply_sectional(
        &mut self,
        section: &prefab_sections::PrefabSection,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        use prefab_sections::*;

        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template, section.width);

        // Place the new section
        let chunk_x = match section.placement.0 {
            //HorizontalPlacement::Left => 0,
            HorizontalPlacement::Center => (build_data.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => (build_data.map.width - 1) - section.width as i32,
        };

        let chunk_y = match section.placement.1 {
            VerticalPlacement::Top => 0,
            VerticalPlacement::Center => (build_data.map.height / 2) - (section.height as i32 / 2),
            //VerticalPlacement::Bottom => (build_data.map.height - 1) - section.height as i32,
        };

        self.apply_previous_iteration(
            |x, y| {
                x < chunk_x
                    || x > (chunk_x + section.width as i32)
                    || y < chunk_y
                    || y > (chunk_y + section.height as i32)
            },
            rng,
            build_data,
        );

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx < build_data.map.width as usize && ty < build_data.map.height as usize {
                    let idx = build_data
                        .map
                        .xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.char_to_map(string_vec[i], idx, build_data);
                }
                i += 1;
            }
        }
        build_data.take_snapshot();
    }

    fn apply_previous_iteration<F>(
        &mut self,
        mut filter: F,
        _rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) where
        F: FnMut(i32, i32) -> bool,
    {
        let width = build_data.map.width;
        build_data.spawn_list.retain(|(idx, _)| {
            let x = *idx as i32 % width;
            let y = *idx as i32 / width;
            filter(x, y)
        });
        build_data.take_snapshot();
    }

    fn apply_room_vaults(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        use prefab_rooms::*;

        self.apply_previous_iteration(|_, _| true, rng, build_data);

        let vault_roll = rng.roll_dice(1, 6) + build_data.map.depth;
        if vault_roll < 4 {
            return;
        }

        let master_vault_list = vec![TOTALLY_NOT_A_TRAP, SILLY_SMILE, CHECKERBOARD];

        let possible_vaults: Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| {
                build_data.map.depth >= v.first_depth && build_data.map.depth <= v.last_depth
            })
            .collect();

        if possible_vaults.is_empty() {
            return;
        }

        let vault_index = if possible_vaults.len() == 1 {
            0
        } else {
            (rng.roll_dice(1, possible_vaults.len() as i32) - 1) as usize
        };
        let vault = possible_vaults[vault_index];

        let mut vault_positions: Vec<Position> = Vec::new();
        for idx in 0..build_data.map.tiles.len() {
            let x = (idx % build_data.map.width as usize) as i32;
            let y = (idx / build_data.map.width as usize) as i32;

            if x > 1
                && (x + vault.width as i32) < build_data.map.width - 2
                && y > 1
                && (y + vault.height as i32) < build_data.map.height - 2
            {
                let mut possible = true;
                'out: for ty in 0..vault.height as i32 {
                    for tx in 0..vault.width as i32 {
                        let idx = build_data.map.xy_idx(tx + x, ty + y);
                        if build_data.map.tiles[idx] != TileType::Floor {
                            possible = false;
                            break 'out;
                        }
                    }
                }

                if possible {
                    vault_positions.push(Position { x, y });
                }
            }
        }

        if !vault_positions.is_empty() {
            let pos_idx = if vault_positions.len() == 1 {
                0
            } else {
                (rng.roll_dice(1, vault_positions.len() as i32) - 1) as usize
            };
            let pos = &vault_positions[pos_idx];

            let chunk_x = pos.x;
            let chunk_y = pos.y;

            let string_vec = PrefabBuilder::read_ascii_to_vec(vault.template, vault.width);
            let mut i = 0;
            for ty in 0..vault.height {
                for tx in 0..vault.width {
                    let idx = build_data
                        .map
                        .xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.char_to_map(string_vec[i], idx, build_data);
                    i += 1;
                }
            }
            build_data.take_snapshot();
        }
    }
}
