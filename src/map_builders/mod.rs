use super::{spawner, Map, Position, Rect, TileType, MAPHEIGHT, MAPWIDTH};
use specs::prelude::*;

mod common;
use common::*;

mod simple_map;
use simple_map::SimpleMapBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    Box::new(SimpleMapBuilder::new(new_depth))
}
