use rltk::prelude::*;

use crate::{RexAssets, RunState, State};

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    use MainMenuResult::*;
    use MainMenuSelection::*;

    let save_exists = crate::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    let assets = gs.ecs.fetch::<RexAssets>();
    ctx.render_xp_sprite(&assets.menu, 0, 0);

    ctx.draw_box_double(
        24,
        18,
        31,
        10,
        RGB::named(rltk::WHEAT),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color_centered(
        20,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "Rusty Roguelike Tutorial",
    );
    ctx.print_color_centered(
        21,
        RGB::named(CYAN),
        RGB::named(BLACK),
        "by Herbert Wolverson",
    );
    ctx.print_color_centered(
        22,
        RGB::named(GREY),
        RGB::named(BLACK),
        "Use Up/Down Arrows and Enter",
    );

    let mut y = 24;
    if let RunState::MainMenu {
        menu_selection: selection,
    } = *runstate
    {
        if selection == NewGame {
            ctx.print_color_centered(y, RGB::named(MAGENTA), RGB::named(BLACK), "Begin New Game");
        } else {
            ctx.print_color_centered(y, RGB::named(WHITE), RGB::named(BLACK), "Begin New Game");
        }
        y += 1;

        if save_exists {
            if selection == LoadGame {
                ctx.print_color_centered(y, RGB::named(MAGENTA), RGB::named(BLACK), "Load Game");
            } else {
                ctx.print_color_centered(y, RGB::named(WHITE), RGB::named(BLACK), "Load Game");
            }
            y += 1;
        }

        if selection == Quit {
            ctx.print_color_centered(y, RGB::named(MAGENTA), RGB::named(BLACK), "Quit");
        } else {
            ctx.print_color_centered(y, RGB::named(WHITE), RGB::named(BLACK), "Quit");
        }

        if let Some(key) = ctx.key {
            use VirtualKeyCode::*;

            match key {
                Escape => {
                    return NoSelection { selected: Quit };
                }

                Up => {
                    return match selection {
                        NewGame => NoSelection { selected: Quit },
                        LoadGame => NoSelection { selected: NewGame },
                        Quit => NoSelection {
                            selected: if save_exists { LoadGame } else { NewGame },
                        },
                    };
                }

                Down => {
                    return match selection {
                        NewGame => NoSelection {
                            selected: if save_exists { LoadGame } else { Quit },
                        },
                        LoadGame => NoSelection { selected: Quit },
                        Quit => NoSelection { selected: NewGame },
                    };
                }

                Return => {
                    return Selected {
                        selected: selection,
                    };
                }

                _ => {
                    return NoSelection {
                        selected: selection,
                    }
                }
            }
        } else {
            return NoSelection {
                selected: selection,
            };
        }
    }
    NoSelection { selected: NewGame }
}
