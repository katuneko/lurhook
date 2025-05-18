//! Map generation utilities.
use bracket_lib::prelude::{FastNoise, NoiseType};
use common::{Point, GameResult};

/// Kind of a tile on the game map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileKind {
    /// Walkable land tile.
    Land,
    /// Shallow water tile where fish can spawn.
    ShallowWater,
    /// Deep water tile.
    DeepWater,
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

/// Generates a map using Perlin noise.
pub fn generate(seed: u64) -> GameResult<Map> {
    let width = 80;
    let height = 25;
    let mut map = Map::new(width, height);
    let mut noise = FastNoise::seeded(seed);
    noise.set_noise_type(NoiseType::Perlin);
    noise.set_frequency(0.08);

    for y in 0..height {
        for x in 0..width {
            let v = noise.get_noise(x as f32, y as f32);
            let kind = if v < -0.2 {
                TileKind::DeepWater
            } else if v < 0.0 {
                TileKind::ShallowWater
            } else {
                TileKind::Land
            };
            let idx = map.idx(Point::new(x as i32, y as i32));
            map.tiles[idx] = kind;
        }
    }

    println!("Initialized crate: mapgen");
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_map() {
        let map = generate(0).expect("map");
        assert_eq!(map.width, 80);
        assert_eq!(map.height, 25);
        assert_eq!(map.tiles.len(), 80 * 25);
    }

    #[test]
    fn snapshot_seed_0() {
        let map = generate(0).expect("map");
        let expected = include_str!("snapshot_seed0.txt");
        assert_eq!(format!("{:?}\n", map), expected);
    }

    #[test]
    fn index_calculation() {
        let map = Map::new(10, 10);
        let idx = map.idx(Point::new(3, 2));
        assert_eq!(idx, 2 * 10 + 3);
    }
}
