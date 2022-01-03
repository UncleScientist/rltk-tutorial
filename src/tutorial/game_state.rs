use crate::*;
use rltk::{GameState, Point, Rltk};

use crate::map_builders::*;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
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
    GameOver,
    MagicMapReveal {
        row: i32,
    },
    MapGeneration,
}

pub struct State {
    pub ecs: World,
    pub mapgen_next_state: Option<RunState>,
    pub mapgen_history: Vec<Map>,
    pub mapgen_index: usize,
    pub mapgen_timer: f32,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut newrunstate = *self.ecs.fetch::<RunState>();

        ctx.cls();
        cull_dead_particles(&mut self.ecs, ctx);

        match newrunstate {
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
            RunState::GameOver => {}
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    newrunstate = self.mapgen_next_state.unwrap();
                }
                draw_map(&self.mapgen_history[self.mapgen_index], ctx);

                self.mapgen_timer += ctx.frame_time_ms;
                if self.mapgen_timer > 300.0 {
                    self.mapgen_timer = 0.0;
                    self.mapgen_index += 1;
                    if self.mapgen_index >= self.mapgen_history.len() {
                        newrunstate = self.mapgen_next_state.unwrap();
                    }
                }
            }
            _ => {
                draw_map(&self.ecs.fetch::<Map>(), ctx);

                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let hidden = self.ecs.read_storage::<Hidden>();
                    let map = self.ecs.fetch::<Map>();

                    let mut data = (&positions, &renderables, !&hidden)
                        .join()
                        .collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
                    for (pos, render, _) in data.iter() {
                        if map.is_visible(pos.x, pos.y) {
                            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                        }
                    }
                    gui::draw_ui(&self.ecs, ctx);
                }
            }
        }

        match newrunstate {
            RunState::NextLevel => {
                self.goto_next_level();
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
            RunState::MainMenu { .. } => {}
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                match *self.ecs.fetch::<RunState>() {
                    RunState::MagicMapReveal { .. } => {
                        newrunstate = RunState::MagicMapReveal { row: 0 }
                    }
                    _ => newrunstate = RunState::MonsterTurn,
                }
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
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
                            newrunstate = RunState::PlayerTurn;
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
                        newrunstate = RunState::PlayerTurn;
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
                        newrunstate = RunState::PlayerTurn;
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
                        newrunstate = RunState::PlayerTurn;
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
                for x in 0..MAPWIDTH {
                    let idx = map.xy_idx(x as i32, row);
                    map.revealed_tiles[idx] = true;
                }
                if row == MAPHEIGHT - 1 {
                    newrunstate = RunState::MonsterTurn;
                } else {
                    newrunstate = RunState::MagicMapReveal { row: row + 1 }
                }
            }
            RunState::MapGeneration => {}
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);

        let mut triggers = TriggerSystem {};
        triggers.run_now(&self.ecs);

        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);

        let mut melee_combat_system = MeleeCombatSystem {};
        melee_combat_system.run_now(&self.ecs);

        let mut damage_system = DamageSystem {};
        damage_system.run_now(&self.ecs);

        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);

        let mut items = ItemUseSystem {};
        items.run_now(&self.ecs);

        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);

        let mut remove_items = ItemRemoveSystem {};
        remove_items.run_now(&self.ecs);

        let mut hunger = hunger_system::HungerSystem {};
        hunger.run_now(&self.ecs);

        let mut particles = ParticleSpawnSystem {};
        particles.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn entities_to_remove_on_game_over(&mut self) -> Vec<Entity> {
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        to_delete
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player_entity = self.ecs.fetch::<Entity>();

        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let equipped = self.ecs.read_storage::<Equipped>();

        let mut to_delete: Vec<Entity> = Vec::new();

        for entity in entities.join() {
            if player.get(entity).is_some() {
                continue;
            }

            if let Some(bp) = backpack.get(entity) {
                if bp.owner == *player_entity {
                    continue;
                }
            }

            if let Some(eq) = equipped.get(entity) {
                if eq.owner == *player_entity {
                    continue;
                }
            }

            to_delete.push(entity);
        }

        to_delete
    }

    pub fn game_over_cleanup(&mut self) {
        self.level_cleanup(true);
    }

    fn goto_next_level(&mut self) {
        self.level_cleanup(false);
    }

    fn level_cleanup(&mut self, everything: bool) {
        let to_delete = if everything {
            self.entities_to_remove_on_game_over()
        } else {
            self.entities_to_remove_on_level_change()
        };

        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("Unable to delete entity");
        }

        let current_depth = if everything {
            1
        } else {
            let worldmap_resource = self.ecs.fetch::<Map>();
            worldmap_resource.depth
        };
        self.generate_world_map(current_depth + 1);

        // Place the player and update resources
        let player_entity = if everything {
            let new_player = spawner::player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = new_player;
            *player_entity_writer
        } else {
            *self.ecs.fetch::<Entity>()
        };

        let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
        if everything {
            gamelog.entries.clear();
            gamelog
                .entries
                .push("Welcome to Rusty Roguelike... again!".to_string());
        } else {
            let mut player_health_store = self.ecs.write_storage::<CombatStats>();
            if let Some(player_health) = player_health_store.get_mut(player_entity) {
                player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2);
            }
            gamelog
                .entries
                .push("You descend to the next level, and take a moment to heal".to_string());
        }
    }

    pub fn generate_world_map(&mut self, new_depth: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();

        let mut rng = self.ecs.write_resource::<rltk::RandomNumberGenerator>();
        let mut builder = random_builder(new_depth, &mut rng);

        builder.build_map(&mut rng);
        std::mem::drop(rng);

        self.mapgen_history = builder.build_data.history.clone();

        let player_start = {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            *worldmap_resource = builder.build_data.map.clone();
            builder
                .build_data
                .starting_position
                .as_mut()
                .unwrap()
                .clone()
        };

        builder.spawn_entities(&mut self.ecs);

        // Place the player and update resources
        let (player_x, player_y) = (player_start.x, player_start.y);
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_x, player_y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        if let Some(player_pos_comp) = position_components.get_mut(*player_entity) {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        // Mark the player's visibility as dirty
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        if let Some(vs) = viewshed_components.get_mut(*player_entity) {
            vs.dirty = true;
        }
    }
}
