use crate::*;
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
use std::fs::{read_to_string, File};

#[cfg(not(target_arch = "wasm32"))]
use specs::{
    error::NoError,
    saveload::{
        DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker,
        SimpleMarkerAllocator,
    },
};

#[cfg(not(target_arch = "wasm32"))]
macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty ), *) => {
        $(
            SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
                &( $ecs.read_storage::<$type>(),),
                &$data.0,
                &$data.1,
                &mut $ser,
            ).unwrap();
        )*
    };
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty ), *) => {
        $(
            DeserializeComponents::<NoError, _>::deserialize(
                &mut ( &mut $ecs.write_storage::<$type>(),),
                &$data.0,           // entities
                &mut $data.1,       // marker
                &mut $data.2,       // allocator
                &mut $de,
            ).unwrap();
        )*
    };
}

#[cfg(target_arch = "wasm32")]
pub fn save_game(_ecs: &World) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(ecs: &mut World) {
    // Create helper
    let mapcopy = ecs.get_mut::<crate::Map>().unwrap().clone();
    let dungeon_master = ecs.get_mut::<crate::MasterDungeonMap>().unwrap().clone();
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper { map: mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let savehelper2 = ecs
        .create_entity()
        .with(DMSerializationHelper {
            map: dungeon_master,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Actually serialize
    {
        let data = (
            ecs.entities(),
            ecs.read_storage::<SimpleMarker<SerializeMe>>(),
        );

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(
            ecs,
            serializer,
            data,
            Position,
            Renderable,
            Player,
            Attributes,
            Skills,
            Pools,
            LightSource,
            OtherLevelPosition,
            LootTable,
            Viewshed,
            Quips,
            Chasing,
            MoveMode,
            NaturalAttackDefense,
            Name,
            Faction,
            BlocksTile,
            BlocksVisibility,
            Door,
            SufferDamage,
            WantsToMelee,
            WantsToApproach,
            WantsToFlee,
            Item,
            Vendor,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            Initiative,
            MyTurn,
            WantsToPickupItem,
            WantsToUseItem,
            WantsToDropItem,
            Equippable,
            EquipmentChanged,
            Equipped,
            MeleeWeapon,
            Wearable,
            WantsToRemoveItem,
            ParticleLifetime,
            HungerClock,
            ProvidesFood,
            MagicMapper,
            TeleportTo,
            TownPortal,
            Hidden,
            EntryTrigger,
            EntityMoved,
            SingleActivation,
            SerializationHelper,
            DMSerializationHelper
        );
    }

    // Clean up
    ecs.delete_entity(savehelper).expect("Crash on cleanup");
    ecs.delete_entity(savehelper2).expect("Crash on cleanup");
}

#[cfg(target_arch = "wasm32")]
pub fn load_game(_ecs: &World) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_game(ecs: &mut World) {
    {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            ecs.delete_entity(*del).expect("Deletion failed");
        }
    }

    let datafile = read_to_string("./savegame.json").unwrap();
    let mut de = serde_json::Deserializer::from_str(&datafile);

    // Actually deserialize
    {
        let mut data = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
        );

        deserialize_individually!(
            ecs,
            de,
            data,
            Position,
            Renderable,
            Player,
            Attributes,
            Skills,
            Pools,
            LightSource,
            OtherLevelPosition,
            LootTable,
            Viewshed,
            Quips,
            Chasing,
            MoveMode,
            NaturalAttackDefense,
            Name,
            Faction,
            BlocksTile,
            BlocksVisibility,
            Door,
            SufferDamage,
            WantsToMelee,
            WantsToApproach,
            WantsToFlee,
            Item,
            Vendor,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            Initiative,
            MyTurn,
            WantsToPickupItem,
            WantsToUseItem,
            WantsToDropItem,
            Equippable,
            EquipmentChanged,
            Equipped,
            MeleeWeapon,
            Wearable,
            WantsToRemoveItem,
            ParticleLifetime,
            HungerClock,
            ProvidesFood,
            MagicMapper,
            TeleportTo,
            TownPortal,
            Hidden,
            EntryTrigger,
            EntityMoved,
            SingleActivation,
            SerializationHelper,
            DMSerializationHelper
        );
    }

    // Clean up
    let mut deleteme: Option<Entity> = None;
    let mut deleteme2: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let helper2 = ecs.read_storage::<DMSerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();
        for (e, h) in (&entities, &helper).join() {
            let mut worldmap = ecs.write_resource::<crate::Map>();
            *worldmap = h.map.clone();
            crate::spatial::set_size((worldmap.height * worldmap.width) as usize);
            deleteme = Some(e);
        }
        for (e, h) in (&entities, &helper2).join() {
            let mut dungeonmaster = ecs.write_resource::<crate::MasterDungeonMap>();
            *dungeonmaster = h.map.clone();
            deleteme2 = Some(e);
        }
        for (e, _, pos) in (&entities, &player, &position).join() {
            let mut ppos = ecs.write_resource::<rltk::Point>();
            *ppos = rltk::Point::new(pos.x, pos.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }
    ecs.delete_entity(deleteme.unwrap())
        .expect("Unable to delete helper");
    ecs.delete_entity(deleteme2.unwrap())
        .expect("Unable to delete helper");
}

pub fn does_save_exist() -> bool {
    Path::new("./savegame.json").exists()
}

pub fn delete_save() {
    if Path::new("./savegame.json").exists() {
        std::fs::remove_file("./savegame.json").expect("Unable to delete file");
    }
}
