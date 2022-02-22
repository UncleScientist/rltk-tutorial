use specs::prelude::*;
use std::collections::VecDeque;
use std::sync::Mutex;

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
    Area { target: Vec<Entity> },
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
