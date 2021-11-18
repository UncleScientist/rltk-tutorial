use rltk::{Point, RandomNumberGenerator};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

use rt::map_builders::*;
use rt::tutorial::*;

// pub mod lib;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<MeleePowerBonus>();
    gs.ecs.register::<DefenseBonus>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<ProvidesFood>();
    gs.ecs.register::<MagicMapper>();
    gs.ecs.register::<Hidden>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<SingleActivation>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let mut builder = random_builder(1);
    let (mut map, player_loc) = builder.build_map(1);

    gs.ecs.insert(Point::new(player_loc.x, player_loc.y));
    let player_entity = player(&mut gs.ecs, player_loc.x, player_loc.y);
    gs.ecs.insert(player_entity);

    gs.ecs.insert(RandomNumberGenerator::new());
    spawn_goodies(&mut gs.ecs, &map.rooms[0]);
    builder.spawn_entities(&mut map, &mut gs.ecs);

    gs.ecs.insert(map);
    gs.ecs.insert(RunState::MainMenu {
        menu_selection: gui::MainMenuSelection::NewGame,
    });
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    gs.ecs.insert(particle_system::ParticleBuilder::new());

    rltk::main_loop(context, gs)
}
