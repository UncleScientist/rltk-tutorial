use crate::*;

pub fn item_trigger(creator: Option<Entity>, item: Entity, targets: &Targets, ecs: &mut World) {
    // Use the item via the generic system
    event_trigger(creator, item, targets, ecs);

    // If it was a consumable, then it gets deleted
    if ecs.read_storage::<Consumable>().get(item).is_some() {
        ecs.entities().delete(item).expect("Delete failed");
    }
}

fn event_trigger(creator: Option<Entity>, entity: Entity, targets: &Targets, ecs: &mut World) {
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    // Providing food
    if ecs.read_storage::<ProvidesFood>().get(entity).is_some() {
        add_effect(creator, EffectType::WellFed, targets.clone());
        let names = ecs.read_storage::<Name>();
        gamelog
            .entries
            .push(format!("You eat the {}", names.get(entity).unwrap().name));
    }

    // Magic mapper
    if ecs.read_storage::<MagicMapper>().get(entity).is_some() {
        let mut runstate = ecs.fetch_mut::<RunState>();
        gamelog
            .entries
            .push("The map is revealed to you!".to_string());
        *runstate = RunState::MagicMapReveal { row: 0 };
    }

    // Town Portal
    if ecs.read_storage::<TownPortal>().get(entity).is_some() {
        let map = ecs.fetch::<Map>();
        if map.depth == 1 {
            gamelog
                .entries
                .push("You are already in town, so the scroll does nothing".to_string());
        } else {
            gamelog
                .entries
                .push("You are teleported back to town!".to_string());
            let mut runstate = ecs.fetch_mut::<RunState>();
            *runstate = RunState::TownPortal;
        }
    }

    // Healing
    if let Some(heal) = ecs.read_storage::<ProvidesHealing>().get(entity) {
        add_effect(
            creator,
            EffectType::Healing {
                amount: heal.heal_amount,
            },
            targets.clone(),
        );
    }

    // Damage
    if let Some(damage) = ecs.read_storage::<InflictsDamage>().get(entity) {
        add_effect(
            creator,
            EffectType::Damage {
                amount: damage.damage,
            },
            targets.clone(),
        )
    }

    // Confusion
    if let Some(confusion) = ecs.read_storage::<Confusion>().get(entity) {
        add_effect(
            creator,
            EffectType::Confusion {
                turns: confusion.turns,
            },
            targets.clone(),
        )
    }

    // Teleport
    if let Some(teleport) = ecs.read_storage::<TeleportTo>().get(entity) {
        add_effect(
            creator,
            EffectType::TeleportTo {
                x: teleport.x,
                y: teleport.y,
                depth: teleport.depth,
                player_only: teleport.player_only,
            },
            targets.clone(),
        );
    }
}

pub fn trigger(creator: Option<Entity>, trigger: Entity, targets: &Targets, ecs: &mut World) {
    // The triggering item is no longer hidden
    ecs.write_storage::<Hidden>().remove(trigger);

    // Use the item via the generic system
    event_trigger(creator, trigger, targets, ecs);

    // If it was a single activation, then it gets deleted
    if ecs
        .read_storage::<SingleActivation>()
        .get(trigger)
        .is_some()
    {
        ecs.entities().delete(trigger).expect("Delete failed");
    }
}
