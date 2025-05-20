use common::Point;
use data::FishType;

/// Player entity with position, stats and inventory.
#[derive(Debug, Clone)]
pub struct Player {
    pub pos: Point,
    /// Remaining hit points.
    pub hp: i32,
    /// Current hunger level (0-100). 0 means starving.
    pub hunger: i32,
    /// Strength of the fishing line.
    pub line: i32,
    /// Bonus applied to bite probability from equipped bait/lure.
    pub bait_bonus: f32,
    /// Bonus added to maximum tension from equipped rod.
    pub tension_bonus: i32,
    /// Multiplier applied when reeling in line tension.
    pub reel_factor: f32,
    /// Number of canned food items carried.
    pub canned_food: i32,
    /// Collected fish kinds.
    pub inventory: Vec<FishType>,
}

/// Temporary hazard entity that damages the player on contact.
#[derive(Debug, Clone)]
pub struct Hazard {
    pub pos: Point,
    pub turns: u8,
}
