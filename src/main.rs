use rltk::Point;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

use rt::map::*;
use rt::tutorial::*;
use rt::Editor;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple(80, 60)
        .unwrap()
        .with_title("Roguelike Tutorial")
        .with_font("vga8x16.png", 8, 16)
        .with_sparse_console(80, 30, "vga8x16.png")
        .build()?;

    let mut gs = State {
        ecs: World::new(),
        mapgen_next_state: Some(RunState::MainMenu {
            menu_selection: gui::MainMenuSelection::NewGame,
        }),
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
        dispatcher: systems::build(),
    };

    gs.ecs.register::<AlwaysTargetsSelf>();
    gs.ecs.register::<ApplyMove>();
    gs.ecs.register::<ApplyTeleport>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<AttributeBonus>();
    gs.ecs.register::<Attributes>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<BlocksVisibility>();
    gs.ecs.register::<Chasing>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<CursedItem>();
    gs.ecs.register::<DMSerializationHelper>();
    gs.ecs.register::<DamageOverTime>();
    gs.ecs.register::<Door>();
    gs.ecs.register::<Duration>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<EquipmentChanged>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<Faction>();
    gs.ecs.register::<Hidden>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<IdentifiedItem>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Initiative>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<KnownSpells>();
    gs.ecs.register::<LightSource>();
    gs.ecs.register::<LootTable>();
    gs.ecs.register::<MagicItem>();
    gs.ecs.register::<MagicMapper>();
    gs.ecs.register::<MoveMode>();
    gs.ecs.register::<MyTurn>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<NaturalAttackDefense>();
    gs.ecs.register::<ObfuscatedName>();
    gs.ecs.register::<OnDeath>();
    gs.ecs.register::<OtherLevelPosition>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Pools>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<ProvidesFood>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<ProvidesIdentification>();
    gs.ecs.register::<ProvidesMana>();
    gs.ecs.register::<ProvidesRemoveCurse>();
    gs.ecs.register::<Quips>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SingleActivation>();
    gs.ecs.register::<Skills>();
    gs.ecs.register::<Slow>();
    gs.ecs.register::<SpawnParticleBurst>();
    gs.ecs.register::<SpawnParticleLine>();
    gs.ecs.register::<SpecialAbilities>();
    gs.ecs.register::<SpellTemplate>();
    gs.ecs.register::<StatusEffect>();
    gs.ecs.register::<Target>();
    gs.ecs.register::<TeachesSpell>();
    gs.ecs.register::<TeleportTo>();
    gs.ecs.register::<TileSize>();
    gs.ecs.register::<TownPortal>();
    gs.ecs.register::<Vendor>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<WantsToApproach>();
    gs.ecs.register::<WantsToCastSpell>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToFlee>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<WantsToShoot>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<Weapon>();
    gs.ecs.register::<Wearable>();

    game_state::load_raws();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    gs.ecs.insert(crate::MasterDungeonMap::new());
    gs.ecs.insert(Map::new(1, 64, 64, "New Map"));
    gs.ecs.insert(Point::new(0, 0));
    gs.ecs.insert(Editor::new());

    let player_entity = player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player_entity);

    gs.ecs.insert(RunState::MapGeneration {});

    gs.ecs.insert(crate::systems::ParticleBuilder::new());

    gs.generate_world_map(1, 0);

    rltk::main_loop(context, gs)
}
