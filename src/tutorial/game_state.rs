use crate::*;
use rltk::{GameState, Point, Rltk};

#[derive(PartialEq, Copy, Clone)]
pub enum VendorMode {
    Buy,
    Sell,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    Ticking,
    ShowInventory,
    ShowDropItem,
    ShowRemoveItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
    PreviousLevel,
    TownPortal,
    GameOver,
    MagicMapReveal {
        row: i32,
    },
    MapGeneration,
    ShowCheatMenu,
    ShowVendor {
        vendor: Entity,
        mode: VendorMode,
    },
    TeleportingToOtherLevel {
        x: i32,
        y: i32,
        depth: i32,
    },
    ShowRemoveCurse,
    ShowIdentify,
    SummonItem,
}

pub struct State {
    pub ecs: World,
    pub mapgen_next_state: Option<RunState>,
    pub mapgen_history: Vec<Map>,
    pub mapgen_index: usize,
    pub mapgen_timer: f32,
    pub dispatcher: Box<dyn systems::UnifiedDispatcher + 'static>,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut newrunstate = *self.ecs.fetch::<RunState>();

        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(0);
        ctx.cls();

        crate::tutorial::systems::particle_system::update_particles(&mut self.ecs, ctx);

        match newrunstate {
            RunState::MainMenu { .. } => {}
            RunState::GameOver { .. } => {}

            _ => {
                map::camera::render_camera(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx);
            }
        }

        match newrunstate {
            RunState::MapGeneration => {
                if SHOW_MAPGEN_VISUALIZER == -1 {
                    newrunstate = self.mapgen_next_state.unwrap();
                } else if self.mapgen_index < self.mapgen_history.len() {
                    map::camera::render_debug_map(&self.mapgen_history[self.mapgen_index], ctx);
                    self.mapgen_timer += ctx.frame_time_ms;
                    if self.mapgen_timer > 300.0 {
                        self.mapgen_timer = 0.0;
                        self.mapgen_index += 1;
                        if self.mapgen_index >= self.mapgen_history.len() {
                            newrunstate = self.mapgen_next_state.unwrap();
                        }
                    }
                }
            }
            RunState::ShowCheatMenu => {
                let result = gui::show_cheat_mode(self, ctx);
                match result {
                    gui::CheatMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::CheatMenuResult::NoResponse => {}
                    gui::CheatMenuResult::Money => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let mut player_pools = pools.get_mut(*player).unwrap();
                        player_pools.gold += 100.0;
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::TeleportToExit => {
                        self.goto_level(1);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        newrunstate = RunState::MapGeneration;
                    }
                    gui::CheatMenuResult::Heal => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let mut player_pools = pools.get_mut(*player).unwrap();
                        player_pools.hit_points.current = player_pools.hit_points.max;
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::Reveal => {
                        let mut map = self.ecs.fetch_mut::<Map>();
                        for v in map.revealed_tiles.iter_mut() {
                            *v = true;
                        }
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::GodMode => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let mut player_pools = pools.get_mut(*player).unwrap();
                        player_pools.god_mode = true;
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::SummonItem => {
                        let mut editor = self.ecs.fetch_mut::<Editor>();
                        editor.reset();
                        newrunstate = RunState::SummonItem;
                    }
                }
            }
            RunState::PreviousLevel => {
                self.goto_level(-1);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::PreRun;
            }
            RunState::NextLevel => {
                self.goto_level(1);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::PreRun;
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu {
                    menu_selection: if saveload_system::does_save_exist() {
                        gui::MainMenuSelection::LoadGame
                    } else {
                        gui::MainMenuSelection::NewGame
                    },
                };
            }

            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection { selected } => {
                        newrunstate = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    gui::MainMenuResult::Selected { selected } => match selected {
                        gui::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                        gui::MainMenuSelection::LoadGame => {
                            saveload_system::load_game(&mut self.ecs);
                            newrunstate = RunState::AwaitingInput;
                            saveload_system::delete_save();
                        }
                        gui::MainMenuSelection::Quit => {
                            ::std::process::exit(0);
                        }
                    },
                }
            }
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
                if newrunstate != RunState::AwaitingInput {
                    crate::gamelog::record_event("Turn", 1);
                }
            }
            RunState::Ticking => {
                let mut should_change_target = false;
                while newrunstate == RunState::Ticking {
                    self.run_systems();
                    match *self.ecs.fetch::<RunState>() {
                        RunState::AwaitingInput => {
                            newrunstate = RunState::AwaitingInput;
                            should_change_target = true
                        }
                        RunState::MagicMapReveal { .. } => {
                            newrunstate = RunState::MagicMapReveal { row: 0 }
                        }
                        RunState::TownPortal => newrunstate = RunState::TownPortal,
                        RunState::TeleportingToOtherLevel { x, y, depth } => {
                            newrunstate = RunState::TeleportingToOtherLevel { x, y, depth }
                        }
                        RunState::ShowRemoveCurse => newrunstate = RunState::ShowRemoveCurse,
                        RunState::ShowIdentify => newrunstate = RunState::ShowIdentify,
                        _ => newrunstate = RunState::Ticking,
                    }
                }
                if should_change_target {
                    player::end_turn_targeting(&mut self.ecs);
                }
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();

                        if let Some(ranged_item) = is_ranged.get(item_entity) {
                            newrunstate = RunState::ShowTargeting {
                                range: ranged_item.range,
                                item: item_entity,
                            };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item: item_entity,
                                        target: None,
                                    },
                                )
                                .expect("Unable to insert intent");
                            newrunstate = RunState::Ticking;
                        }
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        if self.ecs.read_storage::<SpellTemplate>().get(item).is_some() {
                            let mut intent = self.ecs.write_storage::<WantsToCastSpell>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToCastSpell {
                                        spell: item,
                                        target: result.1,
                                    },
                                )
                                .expect("Unable to insert intent");
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item,
                                        target: result.1,
                                    },
                                )
                                .expect("Unable to insert intent");
                        }
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowRemoveItem => {
                let result = gui::remove_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToRemoveItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowVendor { vendor, mode } => {
                let result = gui::show_vendor_menu(self, ctx, vendor, mode);
                match result.0 {
                    gui::VendorResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::VendorResult::NoResponse => {}
                    gui::VendorResult::Sell => {
                        let price = self
                            .ecs
                            .read_storage::<Item>()
                            .get(result.1.unwrap())
                            .unwrap()
                            .base_value
                            * 0.8;
                        self.ecs
                            .write_storage::<Pools>()
                            .get_mut(*self.ecs.fetch::<Entity>())
                            .unwrap()
                            .gold += price;
                        self.ecs
                            .delete_entity(result.1.unwrap())
                            .expect("Unable to delete");
                    }
                    gui::VendorResult::Buy => {
                        let tag = result.2.unwrap();
                        let price = result.3.unwrap();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_entity = self.ecs.fetch::<Entity>();
                        let mut identified = self.ecs.write_storage::<IdentifiedItem>();

                        identified
                            .insert(*player_entity, IdentifiedItem { name: tag.clone() })
                            .expect("Unable to insert");
                        std::mem::drop(identified);

                        let player_pools = pools.get_mut(*player_entity).unwrap();
                        std::mem::drop(player_entity);

                        if player_pools.gold >= price {
                            player_pools.gold -= price;
                            std::mem::drop(pools);
                            let player_entity = *self.ecs.fetch::<Entity>();
                            crate::raws::spawn_named_entity(
                                &RAWS.lock().unwrap(),
                                &mut self.ecs,
                                &tag,
                                SpawnType::Carried { by: player_entity },
                            );
                        }
                    }
                    gui::VendorResult::BuyMode => {
                        newrunstate = RunState::ShowVendor {
                            vendor,
                            mode: VendorMode::Buy,
                        }
                    }
                    gui::VendorResult::SellMode => {
                        newrunstate = RunState::ShowVendor {
                            vendor,
                            mode: VendorMode::Sell,
                        }
                    }
                }
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        newrunstate = RunState::MainMenu {
                            menu_selection: gui::MainMenuSelection::NewGame,
                        };
                    }
                }
            }
            RunState::MagicMapReveal { row } => {
                let mut map = self.ecs.fetch_mut::<Map>();
                for x in 0..map.width {
                    let idx = map.xy_idx(x as i32, row);
                    map.revealed_tiles[idx] = true;
                }
                if row == map.height - 1 {
                    newrunstate = RunState::Ticking;
                } else {
                    newrunstate = RunState::MagicMapReveal { row: row + 1 }
                }
            }
            RunState::TownPortal => {
                // Spawn the portal
                spawner::spawn_town_portal(&mut self.ecs);

                // Transition
                let map_depth = self.ecs.fetch::<Map>().depth;
                let destination_offset = 0 - (map_depth - 1);
                self.goto_level(destination_offset);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::TeleportingToOtherLevel { x, y, depth } => {
                self.goto_level(depth - 1);
                let player_entity = self.ecs.fetch::<Entity>();
                if let Some(pos) = self.ecs.write_storage::<Position>().get_mut(*player_entity) {
                    pos.x = x;
                    pos.y = y;
                }

                let mut ppos = self.ecs.fetch_mut::<Point>();
                ppos.x = x;
                ppos.y = y;
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::ShowRemoveCurse => {
                let result = gui::remove_curse_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        self.ecs.write_storage::<CursedItem>().remove(item_entity);
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowIdentify => {
                let result = gui::identify_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        if let Some(name) = self.ecs.read_storage::<Name>().get(item_entity) {
                            let mut dm = self.ecs.fetch_mut::<MasterDungeonMap>();
                            dm.identified_items.insert(name.name.clone());
                        }
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::SummonItem => {
                let result = gui::summon_item(self, ctx);
                match result.0 {
                    gui::SummonItemResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::SummonItemResult::NoResponse => {}
                    gui::SummonItemResult::Done => {
                        rltk::console::log(format!("Trying to summon '{}'", result.1));
                        let player = *self.ecs.fetch::<Entity>();
                        spawn_named_entity(
                            &RAWS.lock().unwrap(),
                            &mut self.ecs,
                            &result.1,
                            SpawnType::Carried { by: player },
                        );
                        newrunstate = RunState::Ticking;
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        crate::tutorial::damage_system::delete_the_dead(&mut self.ecs);

        rltk::render_draw_buffer(ctx).expect("Unable to draw buffer");
        if SHOW_FPS {
            ctx.print(1, 59, &format!("FPS: {}", ctx.fps));
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        self.dispatcher.run_now(&mut self.ecs);
        self.ecs.maintain();
    }

    pub fn goto_level(&mut self, offset: i32) {
        freeze_level_entities(&mut self.ecs);

        // Build a new map and place the player
        let current_depth = self.ecs.fetch::<Map>().depth;
        self.generate_world_map(current_depth + offset, offset);

        // Notify the player
        crate::gamelog::Logger::new()
            .color(rltk::WHITE)
            .append("You change level")
            .log();
    }

    pub fn game_over_cleanup(&mut self) {
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }

        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("Unable to delete entity");
        }

        {
            // Place the player and update resources
            let new_player = spawner::player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = new_player;
        }

        // Replace the world maps
        self.ecs.insert(map::MasterDungeonMap::new());

        // Build a new map and place the player
        self.generate_world_map(1, 0);
        gamelog::clear_events();

        crate::gamelog::Logger::new()
            .color(rltk::WHITE)
            .append("Welcome to Rusty Roguelike... again!");
    }

    pub fn generate_world_map(&mut self, new_depth: i32, offset: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();

        if let Some(history) = map::level_transition(&mut self.ecs, new_depth, offset) {
            self.mapgen_history = history;
        } else {
            map::thaw_level_entities(&mut self.ecs);
        }

        gamelog::clear_log();
        gamelog::Logger::new()
            .append("Welcome to")
            .color(rltk::CYAN)
            .append("Rusty Roguelike")
            .log();
    }
}

pub fn load_raws() {
    raws::load_raws();
}
