use super::*;
use rltk::{a_star_search, DistanceAlg, Point, RandomNumberGenerator};
use std::collections::HashSet;

enum BuildingTag {
    Pub,
    Temple,
    Blacksmith,
    Clothier,
    Alchemist,
    PlayerHouse,
    Hovel,
    Abandoned,
    Unassigned,
}

type Building = (i32, i32, i32, i32);

pub fn town_builder(new_depth: i32, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height);
    chain.start_with(TownBuilder::new());
    chain
}

pub struct TownBuilder {}

impl InitialMapBuilder for TownBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_rooms(rng, build_data);
    }
}

impl TownBuilder {
    pub fn new() -> Box<TownBuilder> {
        Box::new(TownBuilder {})
    }

    pub fn build_rooms(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.grass_layer(build_data);
        self.water_and_piers(rng, build_data);
        let (mut available_building_tiles, wall_gap_y) = self.town_walls(rng, build_data);
        let mut buildings = self.buildings(rng, build_data, &mut available_building_tiles);
        let doors = self.add_doors(rng, build_data, &mut buildings, wall_gap_y);
        self.add_paths(build_data, &doors);

        let exit_idx = build_data.map.xy_idx(build_data.width - 5, wall_gap_y);
        build_data.map.tiles[exit_idx] = TileType::DownStairs;

        let building_size = self.sort_buildings(&buildings);
        self.building_factory(rng, build_data, &buildings, &building_size);

        for t in build_data.map.visible_tiles.iter_mut() {
            *t = true;
        }
        build_data.take_snapshot();
    }

    fn grass_layer(&mut self, build_data: &mut BuilderMap) {
        for t in build_data.map.tiles.iter_mut() {
            *t = TileType::Grass;
        }
        build_data.take_snapshot();
    }

    fn water_and_piers(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut n = (rng.roll_dice(1, 65535) as f32) / 65535.0;
        let mut water_width: Vec<i32> = Vec::new();
        for y in 0..build_data.height {
            let n_water = (f32::sin(n) * 10.0) as i32 + 14 + rng.roll_dice(1, 6);
            water_width.push(n_water);
            n += 0.1;
            for x in 0..n_water {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::ShallowWater;
            }
        }
        build_data.take_snapshot();

        // Add piers
        for _ in 0..rng.roll_dice(1, 4) + 6 {
            let y = rng.roll_dice(1, build_data.height) - 1;
            for x in 2 + rng.roll_dice(1, 6)..water_width[y as usize] + 4 {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::WoodFloor;
            }
        }
        build_data.take_snapshot();
    }

    fn town_walls(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) -> (HashSet<usize>, i32) {
        let mut available_building_tiles = HashSet::new();
        let wall_gap_y = rng.roll_dice(1, build_data.height - 9) + 5;
        for y in 1..build_data.height - 2 {
            if !(y > wall_gap_y - 4 && y < wall_gap_y + 4) {
                let idx = build_data.map.xy_idx(30, y);
                build_data.map.tiles[idx] = TileType::Wall;
                build_data.map.tiles[idx - 1] = TileType::Floor;
                let idx_right = build_data.map.xy_idx(build_data.width - 2, y);
                build_data.map.tiles[idx_right] = TileType::Wall;
                for x in 31..build_data.width - 2 {
                    let gravel_idx = build_data.map.xy_idx(x, y);
                    build_data.map.tiles[gravel_idx] = TileType::Gravel;
                    if y > 2 && y < build_data.height - 1 {
                        available_building_tiles.insert(gravel_idx);
                    }
                }
            } else {
                for x in 30..build_data.width {
                    let road_idx = build_data.map.xy_idx(x, y);
                    build_data.map.tiles[road_idx] = TileType::Road;
                }
            }
        }
        build_data.take_snapshot();
        for x in 30..build_data.width - 1 {
            let idx_top = build_data.map.xy_idx(x, 1);
            build_data.map.tiles[idx_top] = TileType::Wall;
            let idx_bot = build_data.map.xy_idx(x, build_data.height - 2);
            build_data.map.tiles[idx_bot] = TileType::Wall;
        }

        (available_building_tiles, wall_gap_y)
    }

    fn buildings(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
        available_building_tiles: &mut HashSet<usize>,
    ) -> Vec<Building> {
        let mut buildings: Vec<Building> = Vec::new();
        let mut n_buildings = 0;
        while n_buildings < 12 {
            let bx = rng.roll_dice(1, build_data.map.width - 32) + 30;
            let by = rng.roll_dice(1, build_data.map.height) - 2;
            let bw = rng.roll_dice(1, 8) + 4;
            let bh = rng.roll_dice(1, 8) + 4;
            let mut possible = true;

            'done: for y in by..by + bh {
                for x in bx..bx + bw {
                    if x < 0 || x > build_data.width - 1 || y < 0 || y > build_data.height - 1 {
                        possible = false;
                        break 'done;
                    }

                    let idx = build_data.map.xy_idx(x, y);
                    if !available_building_tiles.contains(&idx) {
                        possible = false;
                        break 'done;
                    }
                }
            }

            if possible {
                n_buildings += 1;
                buildings.push((bx, by, bw, bh));
                for y in by..by + bh {
                    for x in bx..bx + bw {
                        let idx = build_data.map.xy_idx(x, y);
                        build_data.map.tiles[idx] = TileType::WoodFloor;
                        available_building_tiles.remove(&idx);
                        available_building_tiles.remove(&(idx + 1));
                        available_building_tiles.remove(&(idx + build_data.width as usize));
                        available_building_tiles.remove(&(idx - 1));
                        available_building_tiles.remove(&(idx - build_data.width as usize));
                    }
                }
                build_data.take_snapshot();
            }
        }

        // Outline buildings
        let mut mapclone = build_data.map.clone();
        for y in 2..build_data.height - 2 {
            for x in 32..build_data.width - 2 {
                let idx = build_data.map.xy_idx(x, y);
                #[allow(clippy::collapsible_if)]
                if build_data.map.tiles[idx] == TileType::WoodFloor {
                    if build_data.map.tiles[idx - 1] != TileType::WoodFloor
                        || build_data.map.tiles[idx + 1] != TileType::WoodFloor
                        || build_data.map.tiles[idx - build_data.width as usize]
                            != TileType::WoodFloor
                        || build_data.map.tiles[idx + build_data.width as usize]
                            != TileType::WoodFloor
                    {
                        mapclone.tiles[idx] = TileType::Wall;
                    }
                }
            }
        }

        build_data.map = mapclone;
        build_data.take_snapshot();

        buildings
    }

    fn add_doors(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &mut Vec<Building>,
        wall_gap_y: i32,
    ) -> Vec<usize> {
        let mut doors = Vec::new();

        for building in buildings.iter() {
            let door_x = building.0 + 1 + rng.roll_dice(1, building.2 - 3);
            let cy = building.1 + (building.3 / 2);
            let idx = if cy > wall_gap_y {
                // Door on the north wall
                build_data.map.xy_idx(door_x, building.1)
            } else {
                build_data.map.xy_idx(door_x, building.1 + building.3 - 1)
            };
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "Door".to_string()));
            doors.push(idx);
        }
        build_data.take_snapshot();
        doors
    }

    fn add_paths(&mut self, build_data: &mut BuilderMap, doors: &[usize]) {
        let mut roads = Vec::new();
        for y in 0..build_data.height {
            for x in 0..build_data.width {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::Road {
                    roads.push(idx);
                }
            }
        }

        build_data.map.populate_blocked();
        for door_idx in doors.iter() {
            let mut nearest_roads: Vec<(usize, f32)> = Vec::new();
            let door_pt = Point::new(
                *door_idx as i32 % build_data.map.width,
                *door_idx as i32 / build_data.map.width,
            );
            for r in roads.iter() {
                nearest_roads.push((
                    *r,
                    DistanceAlg::PythagorasSquared.distance2d(
                        door_pt,
                        Point::new(
                            *r as i32 % build_data.map.width,
                            *r as i32 / build_data.map.width,
                        ),
                    ),
                ));
            }
            nearest_roads.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            let destination = nearest_roads[0].0;
            let path = a_star_search(*door_idx, destination, &build_data.map);
            if path.success {
                for step in path.steps.iter() {
                    let idx = *step as usize;
                    build_data.map.tiles[idx] = TileType::Road;
                    roads.push(idx);
                }
            }
            build_data.take_snapshot();
        }
    }

    fn sort_buildings(&mut self, buildings: &[Building]) -> Vec<(usize, i32, BuildingTag)> {
        let mut building_size: Vec<(usize, i32, BuildingTag)> = Vec::new();
        for (i, building) in buildings.iter().enumerate() {
            building_size.push((i, building.2 * building.3, BuildingTag::Unassigned));
        }
        building_size.sort_by(|a, b| b.1.cmp(&a.1));

        building_size[0].2 = BuildingTag::Pub;
        building_size[1].2 = BuildingTag::Temple;
        building_size[2].2 = BuildingTag::Blacksmith;
        building_size[3].2 = BuildingTag::Clothier;
        building_size[4].2 = BuildingTag::Alchemist;
        building_size[5].2 = BuildingTag::PlayerHouse;

        for b in building_size.iter_mut().skip(6) {
            b.2 = BuildingTag::Hovel;
        }

        let last_index = building_size.len() - 1;
        building_size[last_index].2 = BuildingTag::Abandoned;

        building_size
    }

    fn building_factory(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &[Building],
        building_index: &[(usize, i32, BuildingTag)],
    ) {
        for (i, building) in buildings.iter().enumerate() {
            let build_type = &building_index[i].2;
            match build_type {
                BuildingTag::Pub => self.build_pub(&building, build_data, rng),
                BuildingTag::Temple => self.build_temple(&building, build_data, rng),
                _ => {}
            }
        }
    }

    fn random_building_spawn(
        &mut self,
        building: &Building,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
        to_place: &mut Vec<&str>,
        player_idx: usize,
    ) {
        for y in building.1..building.1 + building.3 {
            for x in building.0..building.0 + building.2 {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::WoodFloor
                    && idx != player_idx
                    && rng.roll_dice(1, 3) == 1
                    && !to_place.is_empty()
                {
                    let entity_tag = to_place[0];
                    to_place.remove(0);
                    build_data.spawn_list.push((idx, entity_tag.to_string()));
                }
            }
        }
    }

    fn build_pub(
        &mut self,
        building: &Building,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        // Place the player
        let (px, py) = (building.0 + (building.2 / 2), building.1 + (building.3 / 2));
        build_data.starting_position = Some(Position { x: px, y: py });

        let player_idx = build_data.map.xy_idx(px, py);

        // Place other items
        let mut to_place = vec![
            "Barkeep",
            "Shady Salesman",
            "Patron",
            "Patron",
            "Keg",
            "Table",
            "Chair",
            "Table",
            "Chair",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, player_idx);
    }

    fn build_temple(
        &mut self,
        building: &Building,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        let mut to_place = vec![
            "Priest",
            "Parishioner",
            "Parishioner",
            "Chair",
            "Chair",
            "Candle",
            "Candle",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }
}
