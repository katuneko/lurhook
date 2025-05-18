//! UI context stubs.
use common::GameResult;

/// Basic UI context for logging and redraw requests.
#[derive(Default)]
pub struct UIContext {
    logs: Vec<String>,
}

impl UIContext {
    /// Adds a message to the log queue.
    pub fn add_log(&mut self, msg: &str) -> GameResult<()> {
        self.logs.push(msg.to_string());
        println!("LOG: {}", msg);
        Ok(())
    }

    /// Refreshes the screen (placeholder).
    pub fn refresh(&self) -> GameResult<()> {
        println!("Refreshed UI with {} log entries", self.logs.len());
        Ok(())
    }

    /// Draws a simple tension bar using ASCII.
    pub fn draw_tension(&self, tension: i32, max: i32) -> GameResult<()> {
        let width = 10;
        let filled = ((tension as f32 / max as f32) * width as f32).round() as usize;
        let bar = format!("[{}{}]", "#".repeat(filled), "-".repeat(width - filled));
        println!("Tension {}", bar);
        Ok(())
    }
}

pub fn init() {
    println!("Initialized crate: ui");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_addition() {
        let mut ui = UIContext::default();
        ui.add_log("test").unwrap();
        assert_eq!(ui.logs.len(), 1);
    }

    #[test]
    fn refresh_ok() {
        let mut ui = UIContext::default();
        ui.add_log("a").unwrap();
        ui.add_log("b").unwrap();
        assert!(ui.refresh().is_ok());
    }

    #[test]
    fn draw_tension_bar() {
        let ui = UIContext::default();
        assert!(ui.draw_tension(5, 10).is_ok());
    }
}
