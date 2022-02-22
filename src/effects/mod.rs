use std::collections::VecDeque;
use std::sync::Mutex;

use specs::prelude::*;

use lazy_static::lazy_static;

mod damage;
pub use damage::*;

mod targetting;
pub use targetting::*;

lazy_static! {
    pub static ref EFFECT_QUEUE: Mutex<VecDeque<EffectSpawner>> = Mutex::new(VecDeque::new());
}

pub enum EffectType {
    Damage { amount: i32 },
    Bloodstain,
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
    /*
     * Cannot do it this way because the lock is being held inside the while
     *
    while let Some(effect) = EFFECT_QUEUE.lock().unwrap().pop_front() {
        target_applicator(ecs, &effect);
    }
    */
    loop {
        let effect = EFFECT_QUEUE.lock().unwrap().pop_front();
        if let Some(effect) = effect {
            target_applicator(ecs, &effect);
        } else {
            break;
        }
    }
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
        _ => false,
    }
}

fn affect_tile(ecs: &mut World, effect: &EffectSpawner, tile_idx: i32) {
    if tile_effect_hits_entities(&effect.effect_type) {
        crate::spatial::for_each_tile_content(tile_idx as usize, |entity_id| {
            affect_entity(ecs, effect, entity_id)
        });
    }

    match &effect.effect_type {
        EffectType::Bloodstain => damage::bloodstain(ecs, tile_idx),
        _ => {}
    }
}

fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage { .. } => damage::inflict_damage(ecs, effect, target),
        _ => {}
    }
}
