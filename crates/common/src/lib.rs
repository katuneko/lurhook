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
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
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

    #[test]
    fn game_error_display_parse() {
        let err = GameError::Parse("oops".into());
        assert_eq!(format!("{}", err), "parse error: oops");
    }

    #[test]
    fn io_error_conversion() {
        let io_err = std::io::Error::from(std::io::ErrorKind::Other);
        let err: GameError = io_err.into();
        assert!(matches!(err, GameError::Io(_)));
    }
}
