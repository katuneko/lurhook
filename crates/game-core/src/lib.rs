//! Game engine entry point.

mod types;

use bracket_lib::prelude::*;

use common::GameResult;
use ecology::spawn_fish;
use fishing::{init as fishing_init, TensionMeter};
use mapgen::generate;
use ui::{init as ui_init, UIContext};

pub use types::Player;

/// Basic game state implementing [`GameState`].
pub struct LurhookGame {
    player: Player,
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
}
