//! Game engine entry point.

mod types;

use bracket_lib::prelude::*;

use common::GameResult;
use ecology::spawn_fish;
use fishing::{init as fishing_init, TensionMeter};
use mapgen::generate;
use ui::{init as ui_init, UIContext};

pub use types::Player;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 25;

/// Basic game state implementing [`GameState`].
pub struct LurhookGame {
    player: Player,
}

impl LurhookGame {
    /// Moves the player by the given delta, clamped to screen bounds.
    fn try_move(&mut self, delta: common::Point) {
        let mut x = self.player.pos.x + delta.x;
        let mut y = self.player.pos.y + delta.y;
        x = x.clamp(0, SCREEN_WIDTH - 1);
        y = y.clamp(0, SCREEN_HEIGHT - 1);
        self.player.pos.x = x;
        self.player.pos.y = y;
    }

    /// Handles input and updates the player position accordingly.
    fn handle_input(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            use VirtualKeyCode::*;
            let delta = match key {
                Left | H => common::Point::new(-1, 0),
                Right | L => common::Point::new(1, 0),
                Up | K => common::Point::new(0, -1),
                Down | J => common::Point::new(0, 1),
                Y => common::Point::new(-1, -1),
                U => common::Point::new(1, -1),
                B => common::Point::new(-1, 1),
                N => common::Point::new(1, 1),
                _ => common::Point::new(0, 0),
            };
            if delta.x != 0 || delta.y != 0 {
                self.try_move(delta);
            }
        }
    }
}

impl Default for LurhookGame {
    fn default() -> Self {
        Self {
            player: Player {
                pos: common::Point::new(40, 12),
            },
        }
    }
}

impl GameState for LurhookGame {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.handle_input(ctx);
        ctx.cls();
        ctx.print(self.player.pos.x, self.player.pos.y, "@");
    }
}

/// Runs the game loop using [`bracket-lib`].
pub fn run() -> BError {
    println!("Welcome to Lurhook! (engine stub)");
    init_subsystems()?;

    let context = BTermBuilder::new()
        .with_dimensions(80, 25)
        .with_title("Lurhook")
        .build()?;
    let gs = LurhookGame::default();
    main_loop(context, gs)
}

fn init_subsystems() -> GameResult<()> {
    let mut ui = UIContext::default();
    ui_init();
    ui.add_log("UI initialized")?;

    let mut map = generate(0)?;
    ui.add_log(&format!("Map {}x{} generated", map.width, map.height))?;

    let _fish = spawn_fish(&mut map)?;
    fishing_init();
    let mut meter = TensionMeter::default();
    meter.update();
    meter.draw();

    ui.refresh()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_ok() {
        assert!(init_subsystems().is_ok());
    }

    #[test]
    fn default_player_position() {
        let game = LurhookGame::default();
        assert_eq!(game.player.pos, common::Point::new(40, 12));
    }

    #[test]
    fn movement_clamped_to_bounds() {
        let mut game = LurhookGame::default();
        game.player.pos = common::Point::new(0, 0);
        game.try_move(common::Point::new(-1, -1));
        assert_eq!(game.player.pos, common::Point::new(0, 0));

        game.player.pos = common::Point::new(SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1);
        game.try_move(common::Point::new(1, 1));
        assert_eq!(
            game.player.pos,
            common::Point::new(SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1)
        );
    }

    #[test]
    fn diagonal_movement() {
        let mut game = LurhookGame::default();
        let start = game.player.pos;
        game.try_move(common::Point::new(1, 1));
        assert_eq!(
            game.player.pos,
            common::Point::new(start.x + 1, start.y + 1)
        );
    }
}
