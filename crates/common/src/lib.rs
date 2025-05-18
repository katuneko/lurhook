//! Common types shared across Lurhook crates.

/// Simple 2D coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    /// Creates a new [`Point`].
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Unified error type for game logic.
#[derive(thiserror::Error, Debug)]
pub enum GameError {
    #[error("invalid operation")] 
    InvalidOperation,
}

pub type GameResult<T> = Result<T, GameError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_new_sets_coordinates() {
        let p = Point::new(2, 3);
        assert_eq!(p.x, 2);
        assert_eq!(p.y, 3);
    }
}
