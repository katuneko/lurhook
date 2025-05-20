//! Data loading utilities for Lurhook.

use common::{GameError, GameResult};
use serde::Deserialize;

/// Fighting behavior for a fish.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum FightStyle {
    /// Sudden large tension spikes.
    Aggressive,
    /// Long battle with gradually decreasing strength.
    Endurance,
    /// Tends to flee if the line goes slack.
    Evasive,
}

/// Fish species parameters loaded from JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct FishType {
    pub id: String,
    pub name: String,
    pub rarity: f32,
    pub strength: i32,
    pub min_depth: i32,
    pub max_depth: i32,
    pub fight_style: FightStyle,
    /// Marks extremely rare boss fish.
    pub legendary: bool,
}

/// Loads a list of [`FishType`] from the given JSON file path.
pub fn load_fish_types(path: &str) -> GameResult<Vec<FishType>> {
    let data = std::fs::read_to_string(path)?;
    parse_fish_json(&data)
}

/// Loads [`FishType`] definitions embedded at compile time (used on WASM).
pub fn load_fish_types_embedded() -> GameResult<Vec<FishType>> {
    parse_fish_json(include_str!("../../../assets/fish.json"))
}

fn parse_fish_json(data: &str) -> GameResult<Vec<FishType>> {
    // extremely naive JSON parser sufficient for the test asset
    let mut fishes = Vec::new();
    for obj in data.split('{').skip(1) {
        if let Some(body) = obj.split('}').next() {
            let mut id = String::new();
            let mut name = String::new();
            let mut rarity = 0.0;
            let mut strength = 0;
            let mut min_depth = 0;
            let mut max_depth = 0;
            let mut fight_style = FightStyle::Aggressive;
            let mut legendary = false;
            for line in body.lines() {
                let line = line.trim().trim_end_matches(',');
                if line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, ':');
                let key = parts.next().unwrap().trim().trim_matches('"');
                let val = parts.next().unwrap().trim().trim_matches('"');
                match key {
                    "id" => id = val.to_string(),
                    "name" => name = val.to_string(),
                    "rarity" => rarity = val.parse().unwrap_or(0.0),
                    "strength" => strength = val.parse().unwrap_or(0),
                    "min_depth" => min_depth = val.parse().unwrap_or(0),
                    "max_depth" => max_depth = val.parse().unwrap_or(0),
                    "fight_style" => {
                        fight_style = match val {
                            "Aggressive" => FightStyle::Aggressive,
                            "Endurance" => FightStyle::Endurance,
                            "Evasive" => FightStyle::Evasive,
                            _ => FightStyle::Aggressive,
                        }
                    }
                    "legendary" => {
                        legendary = matches!(val, "true" | "1");
                    }
                    _ => {}
                }
            }
            if !id.is_empty() {
                fishes.push(FishType {
                    id,
                    name,
                    rarity,
                    strength,
                    min_depth,
                    max_depth,
                    fight_style,
                    legendary,
                });
            }
        }
    }
    if fishes.is_empty() {
        return Err(GameError::InvalidOperation);
    }
    Ok(fishes)
}

pub fn init() {
    println!("Initialized crate: data");
}

/// Kind of gear item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Rod,
    Reel,
    Lure,
    Food,
}

/// Gear item parameters loaded from JSON.
#[derive(Clone, Debug)]
pub struct ItemType {
    pub id: String,
    pub name: String,
    pub kind: ItemKind,
    pub tension_bonus: i32,
    pub reel_factor: f32,
    pub bite_bonus: f32,
}

/// Loads a list of [`ItemType`] from the given JSON file path.
pub fn load_item_types(path: &str) -> GameResult<Vec<ItemType>> {
    let data = std::fs::read_to_string(path)?;
    parse_item_json(&data)
}

/// Loads [`ItemType`] definitions embedded at compile time (used on WASM).
pub fn load_item_types_embedded() -> GameResult<Vec<ItemType>> {
    parse_item_json(include_str!("../../../assets/items.json"))
}

fn parse_item_json(data: &str) -> GameResult<Vec<ItemType>> {
    let mut items = Vec::new();
    for obj in data.split('{').skip(1) {
        if let Some(body) = obj.split('}').next() {
            let mut id = String::new();
            let mut name = String::new();
            let mut kind = ItemKind::Rod;
            let mut tension_bonus = 0;
            let mut reel_factor = 1.0;
            let mut bite_bonus = 0.0;
            for line in body.lines() {
                let line = line.trim().trim_end_matches(',');
                if line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, ':');
                let key = parts.next().unwrap().trim().trim_matches('"');
                let val = parts.next().unwrap().trim().trim_matches('"');
                match key {
                    "id" => id = val.to_string(),
                    "name" => name = val.to_string(),
                    "kind" => {
                        kind = match val {
                            "Rod" => ItemKind::Rod,
                            "Reel" => ItemKind::Reel,
                            "Lure" => ItemKind::Lure,
                            "Food" => ItemKind::Food,
                            _ => ItemKind::Rod,
                        }
                    }
                    "tension_bonus" => tension_bonus = val.parse().unwrap_or(0),
                    "reel_factor" => reel_factor = val.parse().unwrap_or(1.0),
                    "bite_bonus" => bite_bonus = val.parse().unwrap_or(0.0),
                    _ => {}
                }
            }
            if !id.is_empty() {
                items.push(ItemType {
                    id,
                    name,
                    kind,
                    tension_bonus,
                    reel_factor,
                    bite_bonus,
                });
            }
        }
    }
    if items.is_empty() {
        return Err(GameError::InvalidOperation);
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_runs() {
        init();
    }

    #[test]
    fn load_sample_data() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fish.json");
        let types = load_fish_types(path).expect("fish types");
        assert!(!types.is_empty());
    }

    #[test]
    fn parse_failure_when_empty() {
        let res = parse_fish_json("");
        assert!(matches!(res, Err(GameError::InvalidOperation)));
    }

    #[test]
    fn parse_simple_data() {
        let json = "[\n  {\n    \"id\": \"A\",\n    \"name\": \"A\",\n    \"rarity\": 1.0,\n    \"strength\": 1,\n    \"min_depth\": 0,\n    \"max_depth\": 1,\n    \"fight_style\": \"Aggressive\",\n    \"legendary\": true\n  }\n]";
        let fishes = parse_fish_json(json).expect("fishes");
        assert_eq!(fishes.len(), 1);
        assert_eq!(fishes[0].id, "A");
        assert_eq!(fishes[0].fight_style, FightStyle::Aggressive);
        assert!(fishes[0].legendary);
    }

    #[test]
    fn load_items() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/items.json");
        let items = load_item_types(path).expect("items");
        assert!(!items.is_empty());
    }

    #[test]
    fn embedded_fish_loads() {
        let fishes = load_fish_types_embedded().expect("fishes");
        assert!(!fishes.is_empty());
    }

    #[test]
    fn embedded_items_load() {
        let items = load_item_types_embedded().expect("items");
        assert!(!items.is_empty());
    }

    #[test]
    fn parse_item_simple() {
        let json = "[\n  {\n    \"id\": \"I\",\n    \"name\": \"Item\",\n    \"kind\": \"Reel\",\n    \"tension_bonus\": 5,\n    \"reel_factor\": 1.5,\n    \"bite_bonus\": 0.1\n  }\n]";
        let items = parse_item_json(json).expect("items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tension_bonus, 5);
        assert!((items[0].bite_bonus - 0.1).abs() < f32::EPSILON);
        assert_eq!(items[0].kind, ItemKind::Reel);
        assert!((items[0].reel_factor - 1.5).abs() < f32::EPSILON);
    }
}
