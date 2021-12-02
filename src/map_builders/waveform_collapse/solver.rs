use super::*;

pub struct Solver {
    constraints: Vec<MapChunk>,
    chunk_size: i32,
    chunks: Vec<Option<usize>>,
    chunks_x: usize,
    chunks_y: usize,
    remaining: Vec<(usize, i32)>,
    pub possible: bool,
}

impl Solver {
    pub fn new(constraints: Vec<MapChunk>, chunk_size: i32, map: &Map) -> Solver {
        let chunks_x = (map.width / chunk_size) as usize;
        let chunks_y = (map.height / chunk_size) as usize;
        let mut remaining: Vec<(usize, i32)> = Vec::new();
        for i in 0..(chunks_x * chunks_y) {
            remaining.push((i, 0));
        }

        Solver {
            constraints,
            chunk_size,
            chunks: vec![None; chunks_x * chunks_y],
            chunks_x,
            chunks_y,
            remaining,
            possible: true,
        }
    }

    fn chunk_idx(&self, x: usize, y: usize) -> usize {
        ((y * self.chunks_x) + x) as usize
    }

    fn count_neighbors(&self, chunk_x: usize, chunk_y: usize) -> i32 {
        let mut neighbors = 0;

        if chunk_x > 0 {
            neighbors += self.chunks[self.chunk_idx(chunk_x - 1, chunk_y)].is_some() as i32;
        }
        if chunk_x < self.chunks_x - 1 {
            neighbors += self.chunks[self.chunk_idx(chunk_x + 1, chunk_y)].is_some() as i32;
        }
        if chunk_y > 0 {
            neighbors += self.chunks[self.chunk_idx(chunk_x, chunk_y - 1)].is_some() as i32;
        }
        if chunk_y < self.chunks_y - 1 {
            neighbors += self.chunks[self.chunk_idx(chunk_x, chunk_y + 1)].is_some() as i32;
        }

        neighbors
    }

    pub fn iteration(&mut self, map: &mut Map, rng: &mut rltk::RandomNumberGenerator) -> bool {
        // TODO
        false
    }
}
