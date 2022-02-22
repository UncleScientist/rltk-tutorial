use specs::prelude::*;
use super::*;
use crate::components::Pools;

pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if !pool.god_mode {
            if let EffectType::Damage{amount} = damage.effect_type {
                pool.hit_points.current -= amount;
            }
        }
    }
}
