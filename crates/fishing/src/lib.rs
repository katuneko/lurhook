//! Fishing minigame utilities.

use mapgen::TileKind;

/// Result of a [`TensionMeter::update`] call.
#[derive(Debug, PartialEq, Eq)]
pub enum MeterState {
    /// The mini game continues.
    Ongoing,
    /// The player reeled in the fish successfully.
    Success,
    /// The line tension exceeded the limit and snapped.
    Broken,
    /// The line went completely slack and the fish escaped.
    Lost,
}

/// Manages fishing line tension over time.
#[derive(Debug)]
pub struct TensionMeter {
    /// Current tension value.
    pub tension: i32,
    /// Maximum allowed tension before the line breaks.
    pub max_tension: i32,
    /// Remaining turns until the fish is caught.
    pub duration: i32,
    /// Strength applied by the hooked fish each turn.
    pub strength: i32,
}

impl TensionMeter {
    /// Creates a new [`TensionMeter`] with the given fish strength.
    pub fn new(strength: i32) -> Self {
        Self {
            tension: 0,
            max_tension: 100,
            duration: 5,
            strength,
        }
    }

    /// Updates internal tension.
    ///
    /// If `reel` is `true`, the player attempts to reduce tension by reeling
    /// in the line. Otherwise the fish pulls with its strength. The returned
    /// [`MeterState`] indicates whether the mini game has finished.
    pub fn update(&mut self, reel: bool) -> MeterState {
        let before = self.tension;
        if reel {
            self.tension = (self.tension - 10).max(0);
        } else {
            self.tension += self.strength;
        }
        self.duration -= 1;

        if self.tension >= self.max_tension {
            MeterState::Broken
        } else if before > 0 && self.tension == 0 {
            MeterState::Lost
        } else if self.duration <= 0 {
            MeterState::Success
        } else {
            MeterState::Ongoing
        }
    }

    /// Draws the tension meter to stdout.
    pub fn draw(&self) {
        println!("Tension meter: {}/{}", self.tension, self.max_tension);
    }
}

/// Calculates bite probability based on environment and gear.
///
/// `tile` determines the water depth; `bait_bonus` adds a flat bonus.
pub fn bite_probability(tile: TileKind, bait_bonus: f32) -> f32 {
    let depth_bonus = match tile {
        TileKind::ShallowWater => 0.1,
        TileKind::DeepWater => 0.3,
        TileKind::Land => 0.0,
    };
    (0.3 + depth_bonus + bait_bonus).clamp(0.0, 1.0)
}

impl Default for TensionMeter {
    fn default() -> Self {
        Self::new(5)
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
        assert_eq!(meter.update(false), MeterState::Ongoing);
        assert_eq!(meter.tension, meter.strength);
    }

    #[test]
    fn reel_reduces_tension() {
        let mut meter = TensionMeter::new(10);
        meter.update(false); // tension 10
        meter.update(true); // reel
        assert!(meter.tension < 10);
    }

    #[test]
    fn breaks_when_exceeding_max() {
        let mut meter = TensionMeter {
            max_tension: 5,
            ..TensionMeter::new(10)
        };
        assert_eq!(meter.update(false), MeterState::Broken);
    }

    #[test]
    fn succeeds_after_duration() {
        let mut meter = TensionMeter {
            duration: 1,
            ..TensionMeter::new(1)
        };
        assert_eq!(meter.update(false), MeterState::Success);
    }

    #[test]
    fn repeated_reel_zeroes_tension() {
        let mut meter = TensionMeter::new(5);
        meter.tension = 20;
        for _ in 0..3 {
            meter.update(true);
        }
        assert_eq!(meter.tension, 0);
    }

    #[test]
    fn lost_when_tension_drops_to_zero() {
        let mut meter = TensionMeter::new(5);
        meter.tension = 10;
        let state = meter.update(true);
        assert_eq!(state, MeterState::Lost);
    }

    #[test]
    fn default_values() {
        let meter = TensionMeter::default();
        assert_eq!(meter.strength, 5);
        assert_eq!(meter.max_tension, 100);
    }

    #[test]
    fn deep_water_increases_bite_chance() {
        let shallow = bite_probability(TileKind::ShallowWater, 0.0);
        let deep = bite_probability(TileKind::DeepWater, 0.0);
        assert!(deep > shallow);
    }

    #[test]
    fn bait_bonus_applied() {
        let base = bite_probability(TileKind::Land, 0.0);
        let bonus = bite_probability(TileKind::Land, 0.2);
        assert!(bonus > base);
        assert!(bonus <= 1.0);
    }
}
