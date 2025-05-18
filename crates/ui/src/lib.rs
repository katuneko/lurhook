//! UI context stubs.
use bracket_lib::prelude::BTerm;

/// UI layout type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UILayout {
    /// Standard exploration layout.
    Standard,
    /// Layout used during the fishing mini game.
    Fishing,
}
use common::GameResult;

const LOG_Y: i32 = 17;
const LOG_WINDOW: i32 = 8;
const TENSION_Y: i32 = LOG_Y - 1;

/// Basic UI context for logging and redraw requests.
pub struct UIContext {
    logs: Vec<String>,
    scroll: usize,
    layout: UILayout,
}

impl Default for UIContext {
    fn default() -> Self {
        Self {
            logs: Vec::new(),
            scroll: 0,
            layout: UILayout::Standard,
        }
    }
}

impl UIContext {
    /// Sets the current layout.
    pub fn set_layout(&mut self, layout: UILayout) {
        self.layout = layout;
    }

    /// Returns the current layout.
    pub fn layout(&self) -> UILayout {
        self.layout
    }
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
        let log_y = if self.layout == UILayout::Fishing { LOG_Y + 1 } else { LOG_Y };
        let start = self
            .logs
            .len()
            .saturating_sub(LOG_WINDOW as usize + self.scroll);
        let end = std::cmp::min(start + LOG_WINDOW as usize, self.logs.len());
        for (i, line) in self.logs[start..end].iter().enumerate() {
            ctx.print(0, log_y + i as i32, line);
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
        let base_y = if self.layout == UILayout::Fishing { LOG_Y + 1 } else { LOG_Y };
        ctx.print(60, base_y, &format!("HP: {}", hp));
        ctx.print(60, base_y + 1, &format!("Line: {}", line));
        ctx.print(60, base_y + 2, &format!("Depth: {}m", depth));
        ctx.print(60, base_y + 3, &format!("Time: {}", time));
        Ok(())
    }

    /// Draws a simple tension bar using ASCII.
    pub fn draw_tension(&self, ctx: &mut BTerm, tension: i32, max: i32) -> GameResult<()> {
        if self.layout != UILayout::Fishing {
            return Ok(());
        }
        let bar = tension_bar_string(tension, max);
        ctx.print(0, TENSION_Y, bar);
        Ok(())
    }
}

fn tension_bar_string(tension: i32, max: i32) -> String {
    let width = 10;
    let filled = ((tension as f32 / max as f32) * width as f32).round() as usize;
    format!("[{}{}]", "#".repeat(filled), "-".repeat(width - filled))
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
    fn tension_bar_format() {
        let bar = super::tension_bar_string(5, 10);
        assert_eq!(bar, "[#####-----]");
    }

    #[test]
    fn layout_switching() {
        let mut ui = UIContext::default();
        assert_eq!(ui.layout(), UILayout::Standard);
        ui.set_layout(UILayout::Fishing);
        assert_eq!(ui.layout(), UILayout::Fishing);
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
