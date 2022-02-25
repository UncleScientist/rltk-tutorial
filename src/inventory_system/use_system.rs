use specs::prelude::*;

use crate::{
    add_effect, aoe_tiles, AreaOfEffect, EffectType, EquipmentChanged, IdentifiedItem, Map, Name,
    Targets, WantsToUseItem,
};

pub struct ItemUseSystem;

type ItemUseData<'a> = (
    ReadExpect<'a, Entity>,
    ReadExpect<'a, Map>,
    Entities<'a>,
    WriteStorage<'a, WantsToUseItem>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, AreaOfEffect>,
    WriteStorage<'a, EquipmentChanged>,
    WriteStorage<'a, IdentifiedItem>,
);

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ItemUseData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            map,
            entities,
            mut use_items,
            names,
            aoe,
            mut dirty,
            mut identified_item,
        ) = data;

        for (entity, useitem) in (&entities, &use_items).join() {
            dirty
                .insert(entity, EquipmentChanged {})
                .expect("Unable to insert marker");

            // Identify
            if entity == *player_entity {
                rltk::console::log(format!(
                    "Identfying item {}",
                    names.get(useitem.item).unwrap().name
                ));
                identified_item
                    .insert(
                        entity,
                        IdentifiedItem {
                            name: names.get(useitem.item).unwrap().name.clone(),
                        },
                    )
                    .expect("Unable to insert");
            }

            // Call the effects system
            add_effect(
                Some(entity),
                EffectType::ItemUse { item: useitem.item },
                match useitem.target {
                    None => Targets::Single {
                        target: *player_entity,
                    },
                    Some(target) => {
                        if let Some(aoe) = aoe.get(useitem.item) {
                            Targets::Tiles {
                                tiles: aoe_tiles(&*map, target, aoe.radius),
                            }
                        } else {
                            Targets::Tile {
                                tile_idx: map.xy_idx(target.x, target.y) as i32,
                            }
                        }
                    }
                },
            );
        }

        use_items.clear();
    }
}
