use std::collections::VecDeque;
use std::sync::Mutex;

mod damage;
use damage::inflict_damage;

use specs::prelude::*;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref EFFECT_QUEUE: Mutex<VecDeque<EffectSpawner>> = Mutex::new(VecDeque::new());
}

pub enum EffectType {
    Damage { amount: i32 },
}

#[derive(Clone)]
pub enum Targets {
    Single { target: Entity },
    TargetList { targets: Vec<Entity> },
    Tile { tile_idx: i32 },
    Tiles { tiles: Vec<i32> },
}

pub struct EffectSpawner {
    pub creator: Option<Entity>,
    pub effect_type: EffectType,
    pub targets: Targets,
}

pub fn add_effect(creator: Option<Entity>, effect_type: EffectType, targets: Targets) {
    EFFECT_QUEUE.lock().unwrap().push_back(EffectSpawner {
        creator,
        effect_type,
        targets,
    });
}

pub fn run_effects_queue(ecs: &mut World) {
    while let Some(effect) = EFFECT_QUEUE.lock().unwrap().pop_front() {
        target_applicator(ecs, &effect);
    }

    /* loop {
        let effect : Option<EffectSpawner> = EFFECT_QUEUE.lock().unwrap().pop_front();
        if let Some(effect) = effect {
            // target_applicator(ecs, &effect);
        } else {
            break;
        }
    } */
}

fn target_applicator(ecs: &mut World, effect: &EffectSpawner) {
    match &effect.targets {
        Targets::Tile { tile_idx } => affect_tile(ecs, effect, *tile_idx),
        Targets::Tiles { tiles } => tiles
            .iter()
            .for_each(|tile_idx| affect_tile(ecs, effect, *tile_idx)),
        Targets::Single { target } => affect_entity(ecs, effect, *target),
        Targets::TargetList { targets } => targets
            .iter()
            .for_each(|entity| affect_entity(ecs, effect, *entity)),
    }
}

fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    match effect {
        EffectType::Damage { .. } => true,
    }
}

fn affect_tile(ecs: &mut World, effect: &EffectSpawner, tile_idx: i32) {
    if tile_effect_hits_entities(&effect.effect_type) {
        crate::spatial::for_each_tile_content(tile_idx as usize, |entity_id| {
            affect_entity(ecs, effect, entity_id)
        });
    }

    // TODO: run the effect
}

fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    inflict_damage(ecs, effect, target);
}
