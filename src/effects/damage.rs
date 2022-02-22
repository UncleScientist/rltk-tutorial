use super::*;
use crate::components::Pools;
use crate::map::Map;
use targetting::entity_position;

pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if !pool.god_mode {
            if let EffectType::Damage { amount } = damage.effect_type {
                pool.hit_points.current -= amount;
                if let Some(tile_idx) = entity_position(ecs, target) {
                    add_effect(None, EffectType::Bloodstain, Targets::Tile { tile_idx });
                }
            }
        }
    }
}

pub fn bloodstain(ecs: &mut World, tile_idx: i32) {
    let mut map = ecs.fetch_mut::<Map>();
    map.bloodstains.insert(tile_idx as usize);
}
