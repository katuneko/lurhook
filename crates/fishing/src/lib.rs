//! Fishing minigame utilities.

use data::FightStyle;
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
    /// Behavior of the hooked fish.
    pub style: FightStyle,
    /// Effectiveness multiplier when reeling.
    pub reel_factor: f32,
}

impl TensionMeter {
    /// Creates a new [`TensionMeter`] with the given fish strength.
    pub fn new(strength: i32, style: FightStyle, reel_factor: f32) -> Self {
        Self {
            tension: 0,
            max_tension: 100,
            duration: 5,
            strength,
            style,
            reel_factor,
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
            let reduction = (10.0 * self.reel_factor).round() as i32;
            self.tension = (self.tension - reduction).max(0);
        } else {
            match self.style {
                FightStyle::Aggressive => {
                    self.tension += self.strength * 2;
                }
                FightStyle::Endurance => {
                    let bonus = if self.duration > 2 {
                        self.strength
                    } else {
                        self.strength / 2
                    };
                    self.tension += bonus;
                }
                FightStyle::Evasive => {
                    if self.tension <= 5 {
                        self.tension = 0;
                    } else {
                        self.tension += self.strength;
                    }
                }
            }
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
        Self::new(5, FightStyle::Aggressive, 1.0)
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
        assert_eq!(meter.tension, meter.strength * 2);
    }

    #[test]
    fn reel_reduces_tension() {
        let mut meter = TensionMeter::new(10, FightStyle::Aggressive, 1.0);
        meter.update(false); // tension 20
        meter.update(true); // reel -> 10
        assert!(meter.tension < 20);
    }

    #[test]
    fn breaks_when_exceeding_max() {
        let mut meter = TensionMeter {
            max_tension: 5,
            ..TensionMeter::new(10, FightStyle::Aggressive, 1.0)
        };
        assert_eq!(meter.update(false), MeterState::Broken);
    }

    #[test]
    fn succeeds_after_duration() {
        let mut meter = TensionMeter {
            duration: 1,
            ..TensionMeter::new(1, FightStyle::Aggressive, 1.0)
        };
        assert_eq!(meter.update(false), MeterState::Success);
    }

    #[test]
    fn repeated_reel_zeroes_tension() {
        let mut meter = TensionMeter::new(5, FightStyle::Aggressive, 1.0);
        meter.tension = 20;
        for _ in 0..3 {
            meter.update(true);
        }
        assert_eq!(meter.tension, 0);
    }

    #[test]
    fn lost_when_tension_drops_to_zero() {
        let mut meter = TensionMeter::new(5, FightStyle::Aggressive, 1.0);
        meter.tension = 10;
        let state = meter.update(true);
        assert_eq!(state, MeterState::Lost);
    }

    #[test]
    fn default_values() {
        let meter = TensionMeter::default();
        assert_eq!(meter.strength, 5);
        assert_eq!(meter.max_tension, 100);
        assert_eq!(meter.style, FightStyle::Aggressive);
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

    #[test]
    fn aggressive_style_spikes_tension() {
        let mut meter = TensionMeter::new(2, FightStyle::Aggressive, 1.0);
        meter.update(false);
        assert_eq!(meter.tension, 4);
    }

    #[test]
    fn endurance_style_slow_end() {
        let mut meter = TensionMeter::new(4, FightStyle::Endurance, 1.0);
        meter.update(false); // duration 5 -> add 4
        for _ in 0..3 {
            meter.update(false);
        }
        // near the end strength halves
        assert!(meter.tension < 4 * 4);
    }

    #[test]
    fn evasive_style_can_escape() {
        let mut meter = TensionMeter::new(3, FightStyle::Evasive, 1.0);
        meter.tension = 5;
        let state = meter.update(false);
        assert_eq!(state, MeterState::Lost);
    }

    #[test]
    fn reel_factor_increases_reduction() {
        let mut meter = TensionMeter::new(5, FightStyle::Aggressive, 2.0);
        meter.tension = 20;
        meter.update(true);
        assert!(meter.tension < 10); // reduction > default 10
    }
}
