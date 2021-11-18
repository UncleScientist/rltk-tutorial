use super::{Map, Position, Rect, TileType, MAPHEIGHT, MAPWIDTH};

mod common;
use common::*;

mod simple_map;
use simple_map::SimpleMapBuilder;

trait MapBuilder {
    fn build(new_depth: i32) -> (Map, Position);
}

pub fn build_random_map(new_depth: i32) -> (Map, Position) {
    SimpleMapBuilder::build(new_depth)
}
