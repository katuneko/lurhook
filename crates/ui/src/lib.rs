//! UI context stubs.
use bracket_lib::prelude::{BTerm, CYAN, GRAY, GREEN, NAVY, RED, RGB, WHITE, YELLOW};

/// UI layout type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UILayout {
    /// Standard exploration layout.
    Standard,
    /// Layout used during the fishing mini game.
    Fishing,
    /// Layout displaying the inventory list.
    Inventory,
    /// Layout showing help and controls.
    Help,
}

/// Color palette for map and entity rendering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorPalette {
    pub land: RGB,
    pub shallow: RGB,
    pub deep: RGB,
    pub player: RGB,
    pub fish: RGB,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            land: RGB::named(GRAY),
            shallow: RGB::named(CYAN),
            deep: RGB::named(NAVY),
            player: RGB::named(YELLOW),
            fish: RGB::named(GREEN),
        }
    }
}

impl ColorPalette {
    /// Returns a high contrast palette suitable for colorblind players.
    pub fn colorblind() -> Self {
        Self {
            land: RGB::named(WHITE),
            shallow: RGB::named(YELLOW),
            deep: RGB::named(GRAY),
            player: RGB::named(WHITE),
            fish: RGB::named(RED),
        }
    }
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
        if self.layout == UILayout::Help {
            return Ok(());
        }
        let log_y = if self.layout == UILayout::Fishing {
            LOG_Y + 1
        } else {
            LOG_Y
        };
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
        hunger: i32,
        depth: i32,
        time: &str,
    ) -> GameResult<()> {
        if self.layout == UILayout::Help {
            return Ok(());
        }
        let base_y = if self.layout == UILayout::Fishing {
            LOG_Y + 1
        } else {
            LOG_Y
        };
        ctx.print(60, base_y, format!("HP: {}", hp));
        ctx.print(60, base_y + 1, format!("Line: {}", line));
        ctx.print(60, base_y + 2, format!("Depth: {}m", depth));
        let bar = hunger_bar_string(hunger, 100);
        use bracket_lib::prelude::*;
        let color = if hunger > 60 {
            GREEN
        } else if hunger > 30 {
            YELLOW
        } else {
            RED
        };
        ctx.print_color(
            60,
            base_y + 3,
            color,
            RGB::named(BLACK),
            format!("Food: {}", bar),
        );
        ctx.print(60, base_y + 4, format!("Time: {}", time));
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

    /// Draws the player's inventory when in `Inventory` layout.
    pub fn draw_inventory(&self, ctx: &mut BTerm, items: &[data::FishType]) -> GameResult<()> {
        if self.layout != UILayout::Inventory {
            return Ok(());
        }
        ctx.print_centered(10, "Inventory");
        for (i, line) in inventory_strings(items).iter().enumerate() {
            ctx.print_centered(11 + i as i32, line);
        }
        Ok(())
    }

    /// Draws help text when in `Help` layout.
    pub fn draw_help(&self, ctx: &mut BTerm) -> GameResult<()> {
        if self.layout != UILayout::Help {
            return Ok(());
        }
        for (i, line) in help_strings().iter().enumerate() {
            ctx.print_centered(5 + i as i32, line);
        }
        Ok(())
    }
}

fn tension_bar_string(tension: i32, max: i32) -> String {
    let width = 10;
    let filled = ((tension as f32 / max as f32) * width as f32).round() as usize;
    format!("[{}{}]", "#".repeat(filled), "-".repeat(width - filled))
}

fn hunger_bar_string(hunger: i32, max: i32) -> String {
    tension_bar_string(hunger, max)
}

pub fn init() {
    println!("Initialized crate: ui");
}

fn inventory_strings(items: &[data::FishType]) -> Vec<String> {
    if items.is_empty() {
        vec!["(empty)".to_string()]
    } else {
        items.iter().map(|f| f.name.clone()).collect()
    }
}

fn help_strings() -> Vec<String> {
    vec![
        "Controls:".to_string(),
        "Arrow keys / hjkl: Move".to_string(),
        "c: Cast line".to_string(),
        "r: Reel".to_string(),
        "i: Inventory".to_string(),
        "F1: Toggle this help".to_string(),
        "Esc/Q: Quit".to_string(),
    ]
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
    fn tension_bar_zero_and_full() {
        assert_eq!(super::tension_bar_string(0, 10), "[----------]");
        assert_eq!(super::tension_bar_string(10, 10), "[##########]");
    }

    #[test]
    fn hunger_bar_alias() {
        assert_eq!(super::hunger_bar_string(5, 10), "[#####-----]");
    }

    #[test]
    fn layout_switching() {
        let mut ui = UIContext::default();
        assert_eq!(ui.layout(), UILayout::Standard);
        ui.set_layout(UILayout::Fishing);
        assert_eq!(ui.layout(), UILayout::Fishing);
        ui.set_layout(UILayout::Inventory);
        assert_eq!(ui.layout(), UILayout::Inventory);
        ui.set_layout(UILayout::Help);
        assert_eq!(ui.layout(), UILayout::Help);
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

    #[test]
    fn inventory_string_generation() {
        let fish = data::FishType {
            id: "A".into(),
            name: "FishA".into(),
            rarity: 1.0,
            strength: 1,
            min_depth: 0,
            max_depth: 1,
        };
        assert_eq!(
            inventory_strings(&[fish.clone()]),
            vec!["FishA".to_string()]
        );
        assert_eq!(inventory_strings(&[]), vec!["(empty)".to_string()]);
    }

    #[test]
    fn colorblind_palette_differs() {
        let normal = ColorPalette::default();
        let cb = ColorPalette::colorblind();
        assert_ne!(normal.fish, cb.fish);
    }

    #[test]
    fn help_strings_contains_controls() {
        let lines = help_strings();
        assert_eq!(lines.first().unwrap(), "Controls:");
        assert!(lines.iter().any(|l| l.contains("F1")));
    }
}
