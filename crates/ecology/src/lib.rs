//! Ecology system stubs.
use bracket_lib::prelude::RandomNumberGenerator;
use common::{GameError, GameResult, Point};
use mapgen::{Map, TileKind};

/// Fish species enumeration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FishKind {
    Trout,
}

/// Fish entity placeholder.
#[derive(Clone, Debug)]
pub struct Fish {
    pub kind: FishKind,
    pub position: Point,
}

/// Updates all fish positions with a simple random walk.
pub fn update_fish(map: &Map, fishes: &mut [Fish]) -> GameResult<()> {
    let mut rng = RandomNumberGenerator::new();
    for fish in fishes {
        let dx = rng.range(-1, 2);
        let dy = rng.range(-1, 2);
        let mut x = fish.position.x + dx;
        let mut y = fish.position.y + dy;
        x = x.clamp(0, map.width as i32 - 1);
        y = y.clamp(0, map.height as i32 - 1);
        let new_pt = Point::new(x, y);
        if matches!(map.tiles[map.idx(new_pt)], TileKind::ShallowWater | TileKind::DeepWater) {
            fish.position = new_pt;
        }
    }
    Ok(())
}

/// Spawns a dummy fish onto the map.
pub fn spawn_fish(map: &mut Map) -> GameResult<Fish> {
    // collect all water tile positions
    let mut water = Vec::new();
    for y in 0..map.height as i32 {
        for x in 0..map.width as i32 {
            let pt = Point::new(x, y);
            let tile = map.tiles[map.idx(pt)];
            if matches!(tile, TileKind::ShallowWater | TileKind::DeepWater) {
                water.push(pt);
            }
        }
    }

    if water.is_empty() {
        return Err(GameError::InvalidOperation);
    }

    let mut rng = RandomNumberGenerator::new();
    let idx = rng.range(0, water.len() as i32) as usize;
    let pos = water[idx];
    println!("Spawned fish at {:?}", pos);
    println!("Initialized crate: ecology");
    Ok(Fish {
        kind: FishKind::Trout,
        position: pos,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mapgen::generate;

    #[test]
    fn spawn_one_fish() {
        let mut map = generate(0).expect("map");
        let fish = spawn_fish(&mut map).expect("fish");
        let tile = map.tiles[map.idx(fish.position)];
        assert!(matches!(tile, TileKind::ShallowWater | TileKind::DeepWater));
    }

    #[test]
    fn fish_moves_within_water_bounds() {
        let mut map = generate(0).expect("map");
        let mut fish = spawn_fish(&mut map).expect("fish");
        for _ in 0..20 {
            update_fish(&map, std::slice::from_mut(&mut fish)).unwrap();
            assert!(fish.position.x >= 0 && fish.position.x < map.width as i32);
            assert!(fish.position.y >= 0 && fish.position.y < map.height as i32);
            let tile = map.tiles[map.idx(fish.position)];
            assert!(matches!(tile, TileKind::ShallowWater | TileKind::DeepWater));
        }
    }
}
