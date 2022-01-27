use rltk::{Point, RandomNumberGenerator};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

use rt::map::*;
use rt::tutorial::*;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple(80, 60)
        .unwrap()
        .with_title("Roguelike Tutorial")
        .build()?;

    let mut gs = State {
        ecs: World::new(),
        mapgen_next_state: Some(RunState::MainMenu {
            menu_selection: gui::MainMenuSelection::NewGame,
        }),
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
    };

    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Attributes>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<BlocksVisibility>();
    gs.ecs.register::<Bystander>();
    gs.ecs.register::<Carnivore>();
    gs.ecs.register::<Chasing>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<DMSerializationHelper>();
    gs.ecs.register::<Door>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<Faction>();
    gs.ecs.register::<Herbivore>();
    gs.ecs.register::<Hidden>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Initiative>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<LightSource>();
    gs.ecs.register::<LootTable>();
    gs.ecs.register::<MagicMapper>();
    gs.ecs.register::<MeleeWeapon>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<MoveMode>();
    gs.ecs.register::<MyTurn>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<NaturalAttackDefense>();
    gs.ecs.register::<OtherLevelPosition>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Pools>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<ProvidesFood>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<Quips>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SingleActivation>();
    gs.ecs.register::<Skills>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Vendor>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<WantsToApproach>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToFlee>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<Wearable>();

    game_state::load_raws();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    gs.ecs.insert(crate::MasterDungeonMap::new());
    gs.ecs.insert(Map::new(1, 64, 64, "New Map"));
    gs.ecs.insert(Point::new(0, 0));
    gs.ecs.insert(RandomNumberGenerator::new());

    let player_entity = player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player_entity);

    gs.ecs.insert(RunState::MapGeneration {});

    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    gs.ecs.insert(particle_system::ParticleBuilder::new());

    gs.generate_world_map(1, 0);

    // cheat: to help test with mapping
    let (x, y) = {
        let loc = gs.ecs.fetch::<Point>();
        (loc.x, loc.y)
    };
    spawner::magic_mapping_scroll(&mut gs.ecs, x, y);

    rltk::main_loop(context, gs)
}
