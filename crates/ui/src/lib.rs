//! UI context stubs.
use bracket_lib::prelude::BTerm;
use common::GameResult;

const LOG_Y: i32 = 17;
const LOG_WINDOW: i32 = 8;

/// Basic UI context for logging and redraw requests.
#[derive(Default)]
pub struct UIContext {
    logs: Vec<String>,
    scroll: usize,
}

impl UIContext {
    /// Adds a message to the log queue.
    pub fn add_log(&mut self, msg: &str) -> GameResult<()> {
        self.logs.push(msg.to_string());
        println!("LOG: {}", msg);
        Ok(())
    }

    /// Scrolls log view one line up.
    pub fn scroll_up(&mut self) {
        if self.scroll + (LOG_WINDOW as usize) < self.logs.len() {
            self.scroll += 1;
        }
    }

    /// Scrolls log view one line down.
    pub fn scroll_down(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    /// Refreshes the screen (placeholder).
    pub fn refresh(&self) -> GameResult<()> {
        println!("Refreshed UI with {} log entries", self.logs.len());
        Ok(())
    }

    /// Draws log window to the screen.
    pub fn draw_logs(&self, ctx: &mut BTerm) -> GameResult<()> {
        let start = self
            .logs
            .len()
            .saturating_sub(LOG_WINDOW as usize + self.scroll);
        let end = std::cmp::min(start + LOG_WINDOW as usize, self.logs.len());
        for (i, line) in self.logs[start..end].iter().enumerate() {
            ctx.print(0, LOG_Y + i as i32, line);
        }
        Ok(())
    }

    /// Draws a status panel on the right side.
    pub fn draw_status(
        &self,
        ctx: &mut BTerm,
        hp: i32,
        line: i32,
        depth: i32,
        time: &str,
    ) -> GameResult<()> {
        ctx.print(60, LOG_Y, &format!("HP: {}", hp));
        ctx.print(60, LOG_Y + 1, &format!("Line: {}", line));
        ctx.print(60, LOG_Y + 2, &format!("Depth: {}m", depth));
        ctx.print(60, LOG_Y + 3, &format!("Time: {}", time));
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

    #[test]
    fn scrolling_bounds() {
        let mut ui = UIContext::default();
        for i in 0..10 {
            ui.add_log(&format!("{}", i)).unwrap();
        }
        ui.scroll_up();
        assert_eq!(ui.scroll, 1);
        for _ in 0..20 {
            ui.scroll_down();
        }
        assert_eq!(ui.scroll, 0);
    }
}
