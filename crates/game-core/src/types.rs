use common::Point;
use data::FishType;

/// Player entity with position, stats and inventory.
#[derive(Debug, Clone)]
pub struct Player {
    pub pos: Point,
    /// Remaining hit points.
    pub hp: i32,
    /// Strength of the fishing line.
    pub line: i32,
    /// Collected fish kinds.
    pub inventory: Vec<FishType>,
}
