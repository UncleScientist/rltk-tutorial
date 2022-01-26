use crate::map::*;
use crate::{Carnivore, EntityMoved, Herbivore, Item, MyTurn, Position, Viewshed, WantsToMelee};
use rltk::{DijkstraMap, DistanceAlg, Point};
use specs::prelude::*;

pub struct AnimalAI {}

type AnimalAIData<'a> = (
    WriteExpect<'a, Map>,
    ReadExpect<'a, Entity>,
    Entities<'a>,
    WriteStorage<'a, Viewshed>,
    ReadStorage<'a, Herbivore>,
    ReadStorage<'a, Carnivore>,
    ReadStorage<'a, Item>,
    WriteStorage<'a, WantsToMelee>,
    WriteStorage<'a, EntityMoved>,
    WriteStorage<'a, Position>,
    ReadStorage<'a, MyTurn>,
);

impl<'a> System<'a> for AnimalAI {
    type SystemData = AnimalAIData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_entity,
            entities,
            mut viewshed,
            herbivore,
            carnivore,
            item,
            mut wants_to_melee,
            mut entity_moved,
            mut position,
            turns,
        ) = data;

        // Herbivores run away a lot
        for (entity, mut viewshed, _, mut pos, _turn) in
            (&entities, &mut viewshed, &herbivore, &mut position, &turns).join()
        {
            let mut run_away_from = Vec::new();
            for other_tile in viewshed.visible_tiles.iter() {
                let view_idx = map.xy_idx(other_tile.x, other_tile.y);
                for other_entity in map.tile_content[view_idx].iter() {
                    if item.get(*other_entity).is_none() {
                        run_away_from.push(view_idx);
                    }
                }
            }

            if !run_away_from.is_empty() {
                let my_idx = map.xy_idx(pos.x, pos.y);
                map.populate_blocked();
                let flee_map = DijkstraMap::new(
                    map.width as usize,
                    map.height as usize,
                    &run_away_from,
                    &*map,
                    100.0,
                );
                if let Some(flee_target) = DijkstraMap::find_highest_exit(&flee_map, my_idx, &*map)
                {
                    if !map.blocked[flee_target as usize] {
                        map.blocked[my_idx] = false;
                        map.blocked[flee_target as usize] = true;
                        viewshed.dirty = true;
                        pos.x = flee_target as i32 % map.width;
                        pos.y = flee_target as i32 / map.width;
                        entity_moved
                            .insert(entity, EntityMoved {})
                            .expect("Unable to insert marker");
                    }
                }
            }
        }

        // Carnivores just want to eat everything
        for (entity, mut viewshed, _, mut pos, _turn) in
            (&entities, &mut viewshed, &carnivore, &mut position, &turns).join()
        {
            let mut run_towards = Vec::new();
            let mut attacked = false;
            for other_tile in viewshed.visible_tiles.iter() {
                let view_idx = map.xy_idx(other_tile.x, other_tile.y);
                for other_entity in map.tile_content[view_idx].iter() {
                    if herbivore.get(*other_entity).is_some() || *other_entity == *player_entity {
                        let distance = DistanceAlg::Pythagoras
                            .distance2d(Point::new(pos.x, pos.y), *other_tile);
                        if distance < 1.5 {
                            wants_to_melee
                                .insert(
                                    entity,
                                    WantsToMelee {
                                        target: *other_entity,
                                    },
                                )
                                .expect("Unable to insert attack");
                            attacked = true;
                        } else {
                            run_towards.push(view_idx);
                        }
                    }
                }
            }

            if !run_towards.is_empty() && !attacked {
                let my_idx = map.xy_idx(pos.x, pos.y);
                map.populate_blocked();
                let chase_map = DijkstraMap::new(
                    map.width as usize,
                    map.height as usize,
                    &run_towards,
                    &*map,
                    100.0,
                );
                if let Some(chase_target) = DijkstraMap::find_lowest_exit(&chase_map, my_idx, &*map)
                {
                    if !map.blocked[chase_target as usize] {
                        map.blocked[my_idx] = false;
                        map.blocked[chase_target as usize] = true;
                        viewshed.dirty = true;
                        pos.x = chase_target as i32 % map.width;
                        pos.y = chase_target as i32 / map.width;
                        entity_moved
                            .insert(entity, EntityMoved {})
                            .expect("Unable to insert marker");
                    }
                }
            }
        }
    }
}
