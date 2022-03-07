use specs::saveload::{MarkedBuilder, SimpleMarker};

use crate::map::Map;
use crate::*;

pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if !pool.god_mode {
            if let EffectType::Damage { amount } = damage.effect_type {
                pool.hit_points.current -= amount;
                if pool.hit_points.current < 1 {
                    add_effect(
                        damage.creator,
                        EffectType::EntityDeath,
                        Targets::Single { target },
                    );
                }
                add_effect(None, EffectType::Bloodstain, Targets::Single { target });
                add_effect(
                    None,
                    EffectType::Particle {
                        glyph: rltk::to_cp437('‼'),
                        fg: rltk::RGB::named(rltk::ORANGE),
                        bg: rltk::RGB::named(rltk::BLACK),
                        lifespan: 200.0,
                    },
                    Targets::Single { target },
                );
            }
        }
    }
}

pub fn heal_damage(ecs: &mut World, heal: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if let EffectType::Healing { amount } = heal.effect_type {
            pool.hit_points.current =
                i32::min(pool.hit_points.max, pool.hit_points.current + amount);
            add_effect(
                None,
                EffectType::Particle {
                    glyph: rltk::to_cp437('‼'),
                    fg: rltk::RGB::named(rltk::GREEN),
                    bg: rltk::RGB::named(rltk::BLACK),
                    lifespan: 200.0,
                },
                Targets::Single { target },
            );
        }
    }
}

pub fn bloodstain(ecs: &mut World, tile_idx: i32) {
    let mut map = ecs.fetch_mut::<Map>();
    map.bloodstains.insert(tile_idx as usize);
}

pub fn death(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    let mut xp_gain = 0;
    let mut gold_gain = 0.0f32;

    let mut pools = ecs.write_storage::<Pools>();
    let attributes = ecs.read_storage::<Attributes>();
    let map = ecs.fetch::<Map>();

    if let Some(pos) = entity_position(ecs, target) {
        crate::spatial::remove_entity(target, pos as usize);
    }

    let source = if effect.creator.is_none() {
        return;
    } else {
        effect.creator.unwrap()
    };

    if ecs.read_storage::<Player>().get(source).is_none() {
        return;
    }

    if let Some(stats) = pools.get(target) {
        xp_gain += stats.level * 100;
        gold_gain += stats.gold;
    }

    if xp_gain == 0 && gold_gain == 0.0 {
        return;
    }

    let mut log = ecs.fetch_mut::<GameLog>();
    let mut player_stats = pools.get_mut(source).unwrap();
    let player_attributes = attributes.get(source).unwrap();

    player_stats.xp += xp_gain;
    player_stats.gold += gold_gain;

    if player_stats.xp >= player_stats.level * 1000 {
        // We've gone up a level!
        player_stats.level += 1;
        log.entries.push(format!(
            "Congratulations, you are now level {}",
            player_stats.level
        ));

        player_stats.hit_points.max = player_hp_at_level(
            player_attributes.fitness.base + player_attributes.fitness.modifiers,
            player_stats.level,
        );
        player_stats.hit_points.current = player_stats.hit_points.max;

        player_stats.mana.max = mana_at_level(
            player_attributes.intelligence.base + player_attributes.intelligence.modifiers,
            player_stats.level,
        );
        player_stats.mana.current = player_stats.mana.max;

        let player_pos = ecs.fetch::<rltk::Point>();
        for i in 0..10 {
            if player_pos.y - i > 1 {
                add_effect(
                    None,
                    EffectType::Particle {
                        glyph: rltk::to_cp437('░'),
                        fg: rltk::RGB::named(rltk::GOLD),
                        bg: rltk::RGB::named(rltk::BLACK),
                        lifespan: 400.0,
                    },
                    Targets::Tile {
                        tile_idx: map.xy_idx(player_pos.x, player_pos.y - i) as i32,
                    },
                );
            }
        }
    }
}

pub fn add_confusion(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Confusion { turns } = &effect.effect_type {
        ecs.create_entity()
            .with(StatusEffect { target })
            .with(Confusion {})
            .with(Duration { turns: *turns })
            .with(Name {
                name: "Confusion".to_string(),
            })
            .build();
    }
}

pub fn attribute_effect(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::AttributeEffect {
        bonus,
        name,
        duration,
    } = &effect.effect_type
    {
        ecs.create_entity()
            .with(StatusEffect { target })
            .with(bonus.clone())
            .with(Duration { turns: *duration })
            .with(Name { name: name.clone() })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<EquipmentChanged>()
            .insert(target, EquipmentChanged {})
            .expect("Unable to insert");
    }
}
