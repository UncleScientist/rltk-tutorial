use super::{spawner, Map, Position, Rect, TileType, MAPHEIGHT, MAPWIDTH};
use specs::prelude::*;

mod common;
use common::*;

mod simple_map;
use simple_map::SimpleMapBuilder;

pub trait MapBuilder {
    fn build_map(&mut self, new_depth: i32) -> (Map, Position);
    fn spawn_entities(&mut self, map: &mut Map, ecs: &mut World);
}

pub fn random_builder() -> Box<dyn MapBuilder> {
    Box::new(SimpleMapBuilder {})
}
