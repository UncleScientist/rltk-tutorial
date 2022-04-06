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

    let mut draw_batch = DrawBatch::new();
    let save_exists = crate::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();
    let assets = gs.ecs.fetch::<RexAssets>();

    ctx.render_xp_sprite(&assets.menu, 0, 0);

    draw_batch.draw_double_box(
        Rect::with_size(24, 18, 31, 10),
        ColorPair::new(RGB::named(rltk::WHEAT), RGB::named(rltk::BLACK)),
    );

    draw_batch.print_color_centered(
        20,
        "Rusty Roguelike Tutorial",
        ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
    );

    draw_batch.print_color_centered(
        21,
        "by Herbert Wolverson",
        ColorPair::new(RGB::named(CYAN), RGB::named(BLACK)),
    );

    draw_batch.print_color_centered(
        22,
        "Use Up/Down Arrows and Enter",
        ColorPair::new(RGB::named(GREY), RGB::named(BLACK)),
    );

    let mut y = 24;
    if let RunState::MainMenu {
        menu_selection: selection,
    } = *runstate
    {
        let highlight = ColorPair::new(RGB::named(MAGENTA), RGB::named(BLACK));
        let normal = ColorPair::new(RGB::named(WHITE), RGB::named(BLACK));

        draw_batch.print_color_centered(
            y,
            "Begin New Game",
            if selection == NewGame {
                highlight
            } else {
                normal
            },
        );
        y += 1;

        if save_exists {
            draw_batch.print_color_centered(
                y,
                "Load Game",
                if selection == LoadGame {
                    highlight
                } else {
                    normal
                },
            );
            y += 1;
        }

        draw_batch.print_color_centered(
            y,
            "Quit",
            if selection == Quit { highlight } else { normal },
        );

        draw_batch.submit(6000).expect("Unable to draw Main Menu");

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
