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
    /// Number of canned food items carried.
    pub canned_food: i32,
    /// Collected fish kinds.
    pub inventory: Vec<FishType>,
}
