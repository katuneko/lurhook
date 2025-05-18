//! Fishing minigame stubs.

/// Manages fishing line tension.
#[derive(Debug, Default)]
pub struct TensionMeter {
    pub tension: u32,
}

impl TensionMeter {
    /// Updates internal tension. Placeholder implementation.
    pub fn update(&mut self) {
        self.tension = self.tension.saturating_add(1);
        println!("Tension updated: {}", self.tension);
    }

    /// Draws the tension meter to stdout.
    pub fn draw(&self) {
        println!("Tension meter: {}", self.tension);
    }
}

pub fn init() {
    println!("Initialized crate: fishing");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tension_increases() {
        let mut meter = TensionMeter::default();
        meter.update();
        assert_eq!(meter.tension, 1);
    }
}
