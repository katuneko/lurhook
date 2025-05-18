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
}
