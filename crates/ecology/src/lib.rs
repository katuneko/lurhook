//! Ecology system stubs.
use common::{GameResult, Point};
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

/// Spawns a dummy fish onto the map.
pub fn spawn_fish(map: &mut Map) -> GameResult<Fish> {
    // pick first water tile or origin
    let pos = Point::new(0, 0);
    if matches!(map.tiles[map.idx(pos)], TileKind::Water) {
        println!("Spawned fish at {:?}", pos);
    }
    println!("Initialized crate: ecology");
    Ok(Fish { kind: FishKind::Trout, position: pos })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mapgen::generate;

    #[test]
    fn spawn_one_fish() {
        let mut map = generate(0).expect("map");
        let fish = spawn_fish(&mut map).expect("fish");
        assert_eq!(fish.position, Point::new(0, 0));
    }
}
