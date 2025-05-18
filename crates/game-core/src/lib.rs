//! Game engine entry point.

mod types;

use common::{GameResult};
use ecology::spawn_fish;
use fishing::{init as fishing_init, TensionMeter};
use mapgen::generate;
use ui::{init as ui_init, UIContext};

pub use types::Player;

/// Runs the game using stub modules.
pub fn run() {
    println!("Welcome to Lurhook! (engine stub)");
    if let Err(e) = init_subsystems() {
        eprintln!("Initialization error: {}", e);
    }
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
}
