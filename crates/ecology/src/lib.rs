//! Ecology system stubs.
use bracket_lib::prelude::RandomNumberGenerator;
use common::{GameError, GameResult, Point};
use mapgen::{Map, TileKind};
use data::FishType;

/// Fish entity placeholder.
#[derive(Clone, Debug)]
pub struct Fish {
    pub kind: FishType,
    pub position: Point,
}

const SCHOOL_RADIUS: i32 = 4;

/// Updates all fish positions with simple AI.
pub fn update_fish(
    map: &Map,
    fishes: &mut [Fish],
    rng: &mut RandomNumberGenerator,
    time_of_day: &str,
) -> GameResult<()> {
    let speed = if time_of_day == "Night" { 2 } else { 1 };
    for i in 0..fishes.len() {
        let (dx_rand, dy_rand) = (rng.range(-speed, speed + 1), rng.range(-speed, speed + 1));
        let mut dx = dx_rand;
        let mut dy = dy_rand;

        // schooling: move towards nearest same-species fish within radius
        let pos = fishes[i].position;
        if let Some(nearest) = fishes
            .iter()
            .enumerate()
            .filter(|(j, f)| *j != i && f.kind.id == fishes[i].kind.id)
            .map(|(_, f)| f.position)
            .filter(|p| (p.x - pos.x).abs() + (p.y - pos.y).abs() <= SCHOOL_RADIUS)
            .min_by_key(|p| (p.x - pos.x).abs() + (p.y - pos.y).abs())
        {
            dx += (nearest.x - pos.x).signum();
            dy += (nearest.y - pos.y).signum();
        }

        dx = dx.clamp(-speed, speed);
        dy = dy.clamp(-speed, speed);

        let mut x = pos.x + dx;
        let mut y = pos.y + dy;
        x = x.clamp(0, map.width as i32 - 1);
        y = y.clamp(0, map.height as i32 - 1);
        let new_pt = Point::new(x, y);
        if matches!(map.tiles[map.idx(new_pt)], TileKind::ShallowWater | TileKind::DeepWater) {
            fishes[i].position = new_pt;
        }
    }
    Ok(())
}

/// Spawns a single fish onto the map.
pub fn spawn_fish(map: &mut Map, fish_types: &[FishType]) -> GameResult<Fish> {
    let mut fishes = spawn_fish_population(map, fish_types, 1)?;
    Ok(fishes.remove(0))
}

/// Spawns `count` fish on water tiles weighted by rarity.
pub fn spawn_fish_population(
    map: &mut Map,
    fish_types: &[FishType],
    count: usize,
) -> GameResult<Vec<Fish>> {
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
    let mut fishes = Vec::new();
    let total: f32 = fish_types.iter().map(|f| f.rarity).sum();
    let max_attempts = count * 10;
    let mut attempts = 0;
    while fishes.len() < count && attempts < max_attempts && !water.is_empty() {
        attempts += 1;

        let mut roll = rng.range(0.0, total);
        let mut chosen = &fish_types[0];
        for ft in fish_types {
            roll -= ft.rarity;
            if roll <= 0.0 {
                chosen = ft;
                break;
            }
        }

        let candidates: Vec<usize> = water
            .iter()
            .enumerate()
            .filter(|(_, pt)| {
                let depth = map.depth(**pt);
                depth >= chosen.min_depth && depth <= chosen.max_depth
            })
            .map(|(i, _)| i)
            .collect();

        if candidates.is_empty() {
            continue;
        }

        let idx = candidates[rng.range(0, candidates.len() as i32) as usize];
        let pos = water.swap_remove(idx);

        fishes.push(Fish {
            kind: chosen.clone(),
            position: pos,
        });
    }

    println!("Spawned {} fish", fishes.len());
    println!("Initialized crate: ecology");
    Ok(fishes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mapgen::generate;
    use data::load_fish_types;
    use bracket_lib::prelude::RandomNumberGenerator;

    #[test]
    fn spawn_one_fish() {
        let mut map = generate(0).expect("map");
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fish.json");
        let types = load_fish_types(path).expect("types");
        let fish = spawn_fish(&mut map, &types).expect("fish");
        let depth = map.depth(fish.position);
        assert!(depth >= fish.kind.min_depth && depth <= fish.kind.max_depth);
    }

    #[test]
    fn spawn_many_fish() {
        let mut map = generate(0).expect("map");
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fish.json");
        let types = load_fish_types(path).expect("types");
        let fishes = spawn_fish_population(&mut map, &types, 5).expect("fishes");
        assert_eq!(fishes.len(), 5);
        for f in fishes {
            let depth = map.depth(f.position);
            assert!(depth >= f.kind.min_depth && depth <= f.kind.max_depth);
        }
    }

    #[test]
    fn fish_moves_within_water_bounds() {
        let mut map = generate(0).expect("map");
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fish.json");
        let types = load_fish_types(path).expect("types");
        let mut fish = spawn_fish(&mut map, &types).expect("fish");
        let mut rng = RandomNumberGenerator::seeded(1);
        for _ in 0..20 {
            update_fish(&map, std::slice::from_mut(&mut fish), &mut rng, "Day")
                .unwrap();
            assert!(fish.position.x >= 0 && fish.position.x < map.width as i32);
            assert!(fish.position.y >= 0 && fish.position.y < map.height as i32);
            let tile = map.tiles[map.idx(fish.position)];
            assert!(matches!(tile, TileKind::ShallowWater | TileKind::DeepWater));
        }
    }

    #[test]
    fn spawn_fails_without_water() {
        let mut map = Map::new(5, 5);
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fish.json");
        let types = load_fish_types(path).expect("types");
        let res = spawn_fish_population(&mut map, &types, 3);
        assert!(matches!(res, Err(GameError::InvalidOperation)));
    }

    #[test]
    fn schooling_moves_fish_closer() {
        let mut map = Map::new(10, 10);
        for t in map.tiles.iter_mut() {
            *t = TileKind::ShallowWater;
        }
        let ft = FishType {
            id: "A".into(),
            name: "A".into(),
            rarity: 1.0,
            strength: 1,
            min_depth: 0,
            max_depth: 10,
        };
        let mut fishes = vec![
            Fish { kind: ft.clone(), position: Point::new(2, 2) },
            Fish { kind: ft.clone(), position: Point::new(5, 2) },
        ];
        let before = (fishes[0].position.x - fishes[1].position.x).abs()
            + (fishes[0].position.y - fishes[1].position.y).abs();
        let mut rng = RandomNumberGenerator::seeded(1);
        update_fish(&map, &mut fishes, &mut rng, "Day").unwrap();
        let after = (fishes[0].position.x - fishes[1].position.x).abs()
            + (fishes[0].position.y - fishes[1].position.y).abs();
        assert!(after < before || after == 0);
    }

    #[test]
    fn night_moves_faster() {
        let mut map = Map::new(10, 10);
        for t in map.tiles.iter_mut() {
            *t = TileKind::ShallowWater;
        }
        let ft = FishType {
            id: "A".into(),
            name: "A".into(),
            rarity: 1.0,
            strength: 1,
            min_depth: 0,
            max_depth: 10,
        };
        let mut day_fish = Fish { kind: ft.clone(), position: Point::new(5, 5) };
        let mut night_fish = Fish { kind: ft.clone(), position: Point::new(5, 5) };
        let mut rng_day = RandomNumberGenerator::seeded(1);
        let mut rng_night = RandomNumberGenerator::seeded(1);
        update_fish(&map, std::slice::from_mut(&mut day_fish), &mut rng_day, "Day").unwrap();
        update_fish(&map, std::slice::from_mut(&mut night_fish), &mut rng_night, "Night").unwrap();
        let day_dist = (day_fish.position.x - 5).abs().max((day_fish.position.y - 5).abs());
        let night_dist = (night_fish.position.x - 5).abs().max((night_fish.position.y - 5).abs());
        assert!(night_dist >= day_dist);
        assert!(night_dist <= 2);
    }
}
