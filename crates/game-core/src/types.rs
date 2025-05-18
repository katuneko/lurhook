use common::Point;
use ecology::FishKind;

/// Player entity with position and inventory.
#[derive(Debug)]
pub struct Player {
    pub pos: Point,
    /// Collected fish kinds.
    pub inventory: Vec<FishKind>,
}
