use super::*;
use super::{Map, TileType};
use std::collections::HashSet;

pub fn build_patterns(
    map: &Map,
    chunk_size: i32,
    include_flipping: bool,
    dedupe: bool,
) -> Vec<Vec<TileType>> {
    // Can we optimise Vec<Vec<>> here?
    let chunks_x = map.width / chunk_size;
    let chunks_y = map.height / chunk_size;
    let mut patterns = Vec::new();

    for cy in 0..chunks_y {
        for cx in 0..chunks_x {
            // Normal orientation
            let mut pattern: Vec<TileType> = Vec::new();
            let start_x = cx * chunk_size;
            let start_y = cy * chunk_size;
            let end_x = (cx + 1) * chunk_size;
            let end_y = (cy + 1) * chunk_size;

            for y in start_y..end_y {
                for x in start_x..end_x {
                    let idx = map.xy_idx(x, y);
                    pattern.push(map.tiles[idx]);
                }
            }

            patterns.push(pattern);

            if include_flipping {
                // Flip horizontal
                pattern = Vec::new();
                for y in start_y..end_y {
                    for x in start_x..end_x {
                        let idx = map.xy_idx(end_x - (x + 1), y);
                        pattern.push(map.tiles[idx]);
                    }
                }
                patterns.push(pattern);

                // Flip vertical
                pattern = Vec::new();
                for y in start_y..end_y {
                    for x in start_x..end_x {
                        let idx = map.xy_idx(x, end_y - (y + 1));
                        pattern.push(map.tiles[idx]);
                    }
                }
                patterns.push(pattern);

                // Flip both
                pattern = Vec::new();
                for y in start_y..end_y {
                    for x in start_x..end_x {
                        let idx = map.xy_idx(end_x - (x + 1), end_y - (y + 1));
                        pattern.push(map.tiles[idx]);
                    }
                }
                patterns.push(pattern);
            }
        }
    }

    // Dedupe
    if dedupe {
        let set: HashSet<Vec<TileType>> = patterns.drain(..).collect();
        patterns.extend(set.into_iter());
    }

    patterns
}

pub fn render_pattern_to_map(
    map: &mut Map,
    chunk: &MapChunk,
    chunk_size: i32,
    start_x: i32,
    start_y: i32,
) {
    let mut i = 0usize;
    for tile_y in 0..chunk_size {
        for tile_x in 0..chunk_size {
            let map_idx = map.xy_idx(start_x + tile_x, start_y + tile_y);
            map.tiles[map_idx] = chunk.pattern[i];
            map.visible_tiles[map_idx] = true;
            i += 1;
        }
    }

    for (x, northbound) in chunk.exits[0].iter().enumerate() {
        if *northbound {
            let map_idx = map.xy_idx(start_x + x as i32, start_y);
            map.tiles[map_idx] = TileType::DownStairs;
        }
    }

    for (x, southbound) in chunk.exits[1].iter().enumerate() {
        if *southbound {
            let map_idx = map.xy_idx(start_x + x as i32, start_y + chunk_size - 1);
            map.tiles[map_idx] = TileType::DownStairs;
        }
    }

    for (x, westbound) in chunk.exits[2].iter().enumerate() {
        if *westbound {
            let map_idx = map.xy_idx(start_x as i32, start_y + x as i32);
            map.tiles[map_idx] = TileType::DownStairs;
        }
    }

    for (x, eastbound) in chunk.exits[3].iter().enumerate() {
        if *eastbound {
            let map_idx = map.xy_idx(start_x + chunk_size - 1i32, start_y + x as i32);
            map.tiles[map_idx] = TileType::DownStairs;
        }
    }
}

pub fn patterns_to_constraints(patterns: Vec<Vec<TileType>>, chunk_size: i32) -> Vec<MapChunk> {
    // Move into the new constraints object
    let mut constraints: Vec<MapChunk> = Vec::new();
    for p in patterns {
        let mut new_chunk = MapChunk {
            pattern: p,
            exits: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            has_exits: true,
            compatible_with: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        };

        for exit in new_chunk.exits.iter_mut() {
            for _ in 0..chunk_size {
                exit.push(false);
            }
        }

        let mut n_exits = 0;
        for x in 0..chunk_size {
            // Check for north-bound exits
            let north_idx = tile_idx_in_chunk(chunk_size, x, 0);
            if new_chunk.pattern[north_idx] == TileType::Floor {
                new_chunk.exits[0][x as usize] = true;
                n_exits += 1;
            }

            // Check for south-bound exits
            let south_idx = tile_idx_in_chunk(chunk_size, x, 0);
            if new_chunk.pattern[south_idx] == TileType::Floor {
                new_chunk.exits[1][x as usize] = true;
                n_exits += 1;
            }

            // Check for west-bound exits
            let west_idx = tile_idx_in_chunk(chunk_size, x, 0);
            if new_chunk.pattern[west_idx] == TileType::Floor {
                new_chunk.exits[2][x as usize] = true;
                n_exits += 1;
            }

            // Check for east-bound exits
            let east_idx = tile_idx_in_chunk(chunk_size, x, 0);
            if new_chunk.pattern[east_idx] == TileType::Floor {
                new_chunk.exits[3][x as usize] = true;
                n_exits += 1;
            }
        }

        if n_exits == 0 {
            new_chunk.has_exits = false;
        }

        constraints.push(new_chunk);
    }

    let ch = constraints.clone();
    for c in constraints.iter_mut() {
        for (j, potential) in ch.iter().enumerate() {
            // If there are no exits at all, it's compatible
            if !c.has_exits || !potential.has_exits {
                for compat in c.compatible_with.iter_mut() {
                    compat.push(j);
                }
            } else {
                // Evaluate compatibility by direction
                for (direction, exit_list) in c.exits.iter_mut().enumerate() {
                    let opposite = match direction {
                        0 => 1,
                        1 => 0,
                        2 => 3,
                        _ => 2,
                    };

                    let mut has_any = false;
                    for (slot, can_enter) in exit_list.iter().enumerate() {
                        if *can_enter {
                            has_any = true;
                            if potential.exits[opposite][slot] {
                                c.compatible_with[direction].push(j);
                                break;
                            }
                        }
                    }

                    if !has_any {
                        for compat in c.compatible_with.iter_mut() {
                            compat.push(j);
                        }
                    }
                }
            }
        }
    }

    constraints
}
