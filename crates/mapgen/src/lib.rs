//! Map generation utilities.
use common::{Point, GameResult};

/// Kind of a tile on the game map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileKind {
    /// Walkable land tile.
    Land,
    /// Water tile where fish can spawn.
    Water,
}

/// Simple map representation.
#[derive(Clone, Debug)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<TileKind>,
}

impl Map {
    /// Creates a new map filled with [`TileKind::Land`].
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, tiles: vec![TileKind::Land; (width * height) as usize] }
    }

    /// Returns tile index from coordinates.
    pub fn idx(&self, pt: Point) -> usize {
        (pt.y as usize) * self.width as usize + pt.x as usize
    }
}

/// Generates a placeholder [`Map`].
pub fn generate(_seed: u64) -> GameResult<Map> {
    let map = Map::new(10, 10);
    println!("Initialized crate: mapgen");
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_map() {
        let map = generate(0).expect("map");
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
        assert_eq!(map.tiles.len(), 100);
    }

    #[test]
    fn index_calculation() {
        let map = Map::new(10, 10);
        let idx = map.idx(Point::new(3, 2));
        assert_eq!(idx, 2 * 10 + 3);
    }
}
