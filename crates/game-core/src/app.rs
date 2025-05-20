use super::{Difficulty, GameMode, LurhookGame};
use bracket_lib::prelude::*;

pub enum AppState {
    Menu,
    Running(Box<LurhookGame>),
    Summary(i32),
}

pub struct LurhookApp {
    state: AppState,
}

impl LurhookApp {
    pub fn new() -> Self {
        Self {
            state: AppState::Menu,
        }
    }

    fn update_state(&mut self, ctx: &mut BTerm) -> bool {
        use VirtualKeyCode::*;
        let key = ctx.key;
        match &mut self.state {
            AppState::Menu => match key {
                Some(Key1) => {
                    self.state = AppState::Running(Box::new(
                        LurhookGame::new_with_difficulty(0, Difficulty::Easy).unwrap(),
                    ));
                    false
                }
                Some(Key2) => {
                    self.state = AppState::Running(Box::new(
                        LurhookGame::new_with_difficulty(0, Difficulty::Normal).unwrap(),
                    ));
                    false
                }
                Some(Key3) => {
                    self.state = AppState::Running(Box::new(
                        LurhookGame::new_with_difficulty(0, Difficulty::Hard).unwrap(),
                    ));
                    false
                }
                Some(Q) => true,
                _ => false,
            },
            AppState::Running(game) => {
                game.tick(ctx);
                if let GameMode::End { score } = game.mode() {
                    self.state = AppState::Summary(score);
                }
                false
            }
            AppState::Summary(_) => match key {
                Some(Return) => {
                    self.state = AppState::Menu;
                    false
                }
                Some(Q) => true,
                _ => false,
            },
        }
    }
}

impl Default for LurhookApp {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState for LurhookApp {
    fn tick(&mut self, ctx: &mut BTerm) {
        let quit = self.update_state(ctx);
        if quit {
            ctx.quit();
            return;
        }
        match &mut self.state {
            AppState::Menu => {
                ctx.cls();
                ctx.print_centered(10, "Lurhook");
                ctx.print_centered(12, "1: Easy  2: Normal  3: Hard");
                ctx.print_centered(14, "Press Q to Quit");
            }
            AppState::Running(_) => {
                // game.tick already rendered
            }
            AppState::Summary(score) => {
                ctx.cls();
                ctx.print_centered(10, "Run Complete!");
                ctx.print_centered(12, format!("Final score: {}", score));
                ctx.print_centered(14, "Press Enter for Menu, Q to Quit");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bracket_lib::prelude::{BTerm, VirtualKeyCode, RGB};

    fn dummy_ctx(key: VirtualKeyCode) -> BTerm {
        BTerm {
            width_pixels: 0,
            height_pixels: 0,
            original_height_pixels: 0,
            original_width_pixels: 0,
            fps: 0.0,
            frame_time_ms: 0.0,
            active_console: 0,
            key: Some(key),
            mouse_pos: (0, 0),
            left_click: false,
            shift: false,
            control: false,
            alt: false,
            web_button: None,
            quitting: false,
            post_scanlines: false,
            post_screenburn: false,
            screen_burn_color: RGB::from_f32(0.0, 0.0, 0.0),
            mouse_visible: true,
        }
    }

    #[test]
    fn enter_from_menu_starts_game() {
        let mut app = LurhookApp::new();
        let mut ctx = dummy_ctx(VirtualKeyCode::Key1);
        app.update_state(&mut ctx);
        match app.state {
            AppState::Running(_) => {}
            _ => panic!("did not start game"),
        }
    }

    #[test]
    fn summary_return_goes_to_menu() {
        let mut app = LurhookApp {
            state: AppState::Summary(10),
        };
        let mut ctx = dummy_ctx(VirtualKeyCode::Return);
        app.update_state(&mut ctx);
        assert!(matches!(app.state, AppState::Menu));
    }
}
