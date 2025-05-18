//! Game engine entry point.

mod types;

use bracket_lib::prelude::*;

use common::{GameError, GameResult};
use ecology::update_fish;
use ecology::{spawn_fish_population, Fish};
use fishing::{init as fishing_init, TensionMeter};
use mapgen::{generate, Map, TileKind};
use ui::{init as ui_init, UIContext, UILayout};

const VIEW_WIDTH: i32 = 60;
const VIEW_HEIGHT: i32 = 17;
const TIME_SEGMENT_TURNS: u32 = 10;
const TIMES: [&str; 4] = ["Dawn", "Day", "Dusk", "Night"];
use data;

/// Current game mode.
enum GameMode {
    Exploring,
    Fishing { wait: u8 },
    End { score: i32 },
}

pub use types::Player;

/// Basic game state implementing [`GameState`].
pub struct LurhookGame {
    player: Player,
    map: Map,
    fishes: Vec<Fish>,
    fish_types: Vec<data::FishType>,
    ui: UIContext,
    depth: i32,
    time_of_day: &'static str,
    turn: u32,
    rng: RandomNumberGenerator,
    mode: GameMode,
    meter: Option<TensionMeter>,
    reeling: bool,
}

impl LurhookGame {
    /// Creates a new game with a generated map.
    pub fn new(seed: u64) -> GameResult<Self> {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fish.json");
        let fish_types = data::load_fish_types(path)?;
        let mut map = generate(seed)?;
        let fishes = spawn_fish_population(&mut map, &fish_types, 5)?;
        Ok(Self {
            player: Player {
                pos: common::Point::new(map.width as i32 / 2, map.height as i32 / 2),
                hp: 10,
                line: 100,
                bait_bonus: 0.0,
                inventory: Vec::new(),
            },
            map,
            fishes,
            fish_types,
            ui: UIContext::default(),
            depth: 0,
            time_of_day: TIMES[0],
            turn: 0,
            rng: RandomNumberGenerator::seeded(seed),
            mode: GameMode::Exploring,
            meter: None,
            reeling: false,
        })
    }

    fn camera(&self) -> (i32, i32) {
        let half_w = VIEW_WIDTH / 2;
        let half_h = VIEW_HEIGHT / 2;
        let mut x = self.player.pos.x - half_w;
        let mut y = self.player.pos.y - half_h;
        x = x.clamp(0, self.map.width as i32 - VIEW_WIDTH);
        y = y.clamp(0, self.map.height as i32 - VIEW_HEIGHT);
        (x, y)
    }

    fn advance_time(&mut self) {
        self.turn += 1;
        let idx = (self.turn / TIME_SEGMENT_TURNS) % TIMES.len() as u32;
        self.time_of_day = TIMES[idx as usize];
    }
    /// Moves the player by the given delta, clamped to screen bounds.
    fn try_move(&mut self, delta: common::Point) {
        let mut x = self.player.pos.x + delta.x;
        let mut y = self.player.pos.y + delta.y;
        x = x.clamp(0, self.map.width as i32 - 1);
        y = y.clamp(0, self.map.height as i32 - 1);
        self.player.pos.x = x;
        self.player.pos.y = y;
    }

    fn score(&self) -> i32 {
        self
            .player
            .inventory
            .iter()
            .map(|f| ((1.0 / f.rarity) * 10.0) as i32)
            .sum()
    }

    fn end_run(&mut self) {
        let score = self.score();
        self.ui
            .add_log(&format!("Run ended! Final score: {}", score))
            .ok();
        self.mode = GameMode::End { score };
    }

    /// Handles input and updates the player position accordingly.
    fn handle_input(&mut self, ctx: &mut BTerm) {
        self.reeling = false;
        if let Some(key) = ctx.key {
            use VirtualKeyCode::*;
            if key == C && matches!(self.mode, GameMode::Exploring) {
                self.cast();
                return;
            }
            if key == R && matches!(self.mode, GameMode::Fishing { .. }) {
                self.reeling = true;
                return;
            }
            if key == PageUp {
                self.ui.scroll_up();
                return;
            }
            if key == PageDown {
                self.ui.scroll_down();
                return;
            }
            if key == Return && matches!(self.mode, GameMode::Exploring) {
                self.end_run();
                return;
            }
            let delta = match key {
                Left | H => common::Point::new(-1, 0),
                Right | L => common::Point::new(1, 0),
                Up | K => common::Point::new(0, -1),
                Down | J => common::Point::new(0, 1),
                Y => common::Point::new(-1, -1),
                U => common::Point::new(1, -1),
                B => common::Point::new(-1, 1),
                N => common::Point::new(1, 1),
                _ => common::Point::new(0, 0),
            };
            if delta.x != 0 || delta.y != 0 {
                self.try_move(delta);
            }
        }
    }

    fn cast(&mut self) {
        if self.fishes.is_empty() {
            self.ui.add_log("No fish around.").ok();
            return;
        }
        self.ui.add_log("Casting...").ok();
        self.ui.set_layout(UILayout::Fishing);
        self.mode = GameMode::Fishing { wait: 2 };
    }

    fn update_fishing(&mut self) {
        if let GameMode::Fishing { ref mut wait } = self.mode {
            if *wait > 0 {
                *wait -= 1;
                return;
            }

            if self.meter.is_none() {
                let tile = if let Some(f) = self.fishes.first() {
                    self.map.tiles[self.map.idx(f.position)]
                } else {
                    TileKind::ShallowWater
                };
                let chance = fishing::bite_probability(tile, self.player.bait_bonus);
                let bite = self.rng.range(0.0, 1.0) < chance;
                if bite {
                    self.ui.add_log("Hooked a fish!").ok();
                    if let Some(f) = self.fishes.first() {
                        self.meter = Some(TensionMeter::new(f.kind.strength));
                    } else {
                        self.meter = Some(TensionMeter::default());
                    }
                } else {
                    self.ui.add_log("The fish got away...").ok();
                    self.mode = GameMode::Exploring;
                    self.ui.set_layout(UILayout::Standard);
                }
                return;
            }

            if let Some(mut meter) = self.meter.take() {
                use fishing::MeterState;
                match meter.update(self.reeling) {
                    MeterState::Ongoing => {
                        self.meter = Some(meter);
                    }
                    MeterState::Success => {
                        if let Some(fish) = self.fishes.pop() {
                            self.player.inventory.push(fish.kind);
                            self.ui.add_log("Caught a fish!").ok();
                        }
                        self.mode = GameMode::Exploring;
                        self.ui.set_layout(UILayout::Standard);
                    }
                    MeterState::Broken => {
                        self.ui.add_log("Line snapped!").ok();
                        self.mode = GameMode::Exploring;
                        self.ui.set_layout(UILayout::Standard);
                    }
                }
            }
        }
    }

    /// Draws the map to the screen.
    fn draw_map(&self, ctx: &mut BTerm) {
        let (cam_x, cam_y) = self.camera();
        for y in 0..VIEW_HEIGHT {
            for x in 0..VIEW_WIDTH {
                let mx = cam_x + x;
                let my = cam_y + y;
                let idx = self.map.idx(common::Point::new(mx, my));
                let glyph = match self.map.tiles[idx] {
                    TileKind::Land => '.',
                    TileKind::ShallowWater => '~',
                    TileKind::DeepWater => '≈',
                };
                ctx.print(x, y, glyph);
            }
        }
    }

    /// Draws all fish on the map.
    fn draw_fish(&self, ctx: &mut BTerm) {
        let (cam_x, cam_y) = self.camera();
        for fish in &self.fishes {
            if fish.position.x >= cam_x
                && fish.position.x < cam_x + VIEW_WIDTH
                && fish.position.y >= cam_y
                && fish.position.y < cam_y + VIEW_HEIGHT
            {
                ctx.print(fish.position.x - cam_x, fish.position.y - cam_y, 'f');
            }
        }
    }

    /// Saves a minimal game state to a RON-like file at `path`.
    pub fn save_game(&self, path: &str) -> GameResult<()> {
        let content = format!(
            "(player:(pos:(x:{}, y:{}), hp:{}), time_of_day:\"{}\")",
            self.player.pos.x, self.player.pos.y, self.player.hp, self.time_of_day
        );
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Loads a minimal game state from a RON-like file at `path`.
    pub fn load_game(path: &str) -> GameResult<Self> {
        let data = std::fs::read_to_string(path)?;
        // very small parser for the expected format
        fn parse_i32(s: &str, key: &str) -> GameResult<i32> {
            let start = s
                .find(key)
                .ok_or_else(|| GameError::Parse(format!("missing {}", key)))?;
            let s = &s[start + key.len()..];
            let end = s
                .find(|c: char| c == ',' || c == ')')
                .ok_or_else(|| GameError::Parse(format!("malformed {}", key)))?;
            s[..end]
                .trim()
                .parse()
                .map_err(|_| GameError::Parse(format!("invalid {}", key)))
        }

        fn parse_str<'a>(s: &'a str, key: &str) -> GameResult<&'a str> {
            let start = s
                .find(key)
                .ok_or_else(|| GameError::Parse(format!("missing {}", key)))?;
            let s = &s[start + key.len()..];
            let start_quote = s
                .find('"')
                .ok_or_else(|| GameError::Parse(format!("malformed {}", key)))?
                + 1;
            let end_quote = s[start_quote..]
                .find('"')
                .ok_or_else(|| GameError::Parse(format!("malformed {}", key)))?;
            Ok(&s[start_quote..start_quote + end_quote])
        }

        let mut game = Self::new(0)?;
        game.player.pos.x = parse_i32(&data, "x:")?;
        game.player.pos.y = parse_i32(&data, "y:")?;
        game.player.hp = parse_i32(&data, "hp:")?;
        let tod = parse_str(&data, "time_of_day:")?;
        game.time_of_day = match tod {
            "Dawn" => "Dawn",
            "Day" => "Day",
            "Dusk" => "Dusk",
            "Night" => "Night",
            other => return Err(GameError::Parse(format!("invalid time_of_day {}", other))),
        };
        Ok(game)
    }
}

impl Default for LurhookGame {
    fn default() -> Self {
        Self::new(0).expect("game")
    }
}

impl GameState for LurhookGame {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.advance_time();
        self.handle_input(ctx);
        match self.mode {
            GameMode::Exploring => {
                update_fish(&self.map, &mut self.fishes).expect("fish update");
            }
            GameMode::Fishing { .. } => self.update_fishing(),
            GameMode::End { score } => {
                ctx.cls();
                ctx.print_centered(12, "Run Complete!");
                ctx.print_centered(13, format!("Final score: {}", score));
                return;
            }
        }
        ctx.cls();
        self.draw_map(ctx);
        self.draw_fish(ctx);
        let (cam_x, cam_y) = self.camera();
        ctx.print(self.player.pos.x - cam_x, self.player.pos.y - cam_y, "@");
        if let Some(m) = &self.meter {
            self.ui.draw_tension(ctx, m.tension, m.max_tension).ok();
        }
        self.ui.draw_logs(ctx).ok();
        self.ui
            .draw_status(
                ctx,
                self.player.hp,
                self.player.line,
                self.depth,
                self.time_of_day,
            )
            .ok();
    }
}

/// Runs the game loop using [`bracket-lib`].
pub fn run() -> BError {
    println!("Welcome to Lurhook! (engine stub)");
    init_subsystems()?;

    let context = BTermBuilder::simple(80, 25)?
        .with_title("Lurhook")
        .build()?;
    let gs = LurhookGame::new(0).expect("init game");
    main_loop(context, gs)
}

fn init_subsystems() -> GameResult<()> {
    let mut ui = UIContext::default();
    ui_init();
    ui.add_log("UI initialized")?;

    let map = generate(0)?;
    ui.add_log(&format!("Map {}x{} generated", map.width, map.height))?;
    fishing_init();
    let mut meter = TensionMeter::default();
    meter.update(false);
    meter.draw();

    ui.refresh()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_ok() {
        assert!(init_subsystems().is_ok());
    }

    #[test]
    fn default_player_position() {
        let game = LurhookGame::default();
        assert_eq!(
            game.player.pos,
            common::Point::new(game.map.width as i32 / 2, game.map.height as i32 / 2)
        );
        assert!(game.player.inventory.is_empty());
        assert_eq!(game.player.hp, 10);
        assert_eq!(game.player.line, 100);
        assert_eq!(game.player.bait_bonus, 0.0);
        assert_eq!(game.map.width, 120);
        assert_eq!(game.map.height, 80);
        assert_eq!(game.fishes.len(), 5);
        let fish = &game.fishes[0];
        let tile = game.map.tiles[game.map.idx(fish.position)];
        assert!(matches!(tile, TileKind::ShallowWater | TileKind::DeepWater));
    }

    #[test]
    fn movement_clamped_to_bounds() {
        let mut game = LurhookGame::default();
        game.player.pos = common::Point::new(0, 0);
        game.try_move(common::Point::new(-1, -1));
        assert_eq!(game.player.pos, common::Point::new(0, 0));

        game.player.pos = common::Point::new(game.map.width as i32 - 1, game.map.height as i32 - 1);
        game.try_move(common::Point::new(1, 1));
        assert_eq!(
            game.player.pos,
            common::Point::new(game.map.width as i32 - 1, game.map.height as i32 - 1)
        );
    }

    #[test]
    fn diagonal_movement() {
        let mut game = LurhookGame::default();
        let start = game.player.pos;
        game.try_move(common::Point::new(1, 1));
        assert_eq!(
            game.player.pos,
            common::Point::new(start.x + 1, start.y + 1)
        );
    }

    #[test]
    fn cast_enters_fishing_mode() {
        let mut game = LurhookGame::default();
        game.cast();
        assert!(matches!(game.mode, GameMode::Fishing { .. }));
        assert_eq!(game.ui.layout(), UILayout::Fishing);
    }

    #[test]
    fn cast_fails_without_fish() {
        let mut game = LurhookGame::default();
        game.fishes.clear();
        game.cast();
        assert!(matches!(game.mode, GameMode::Exploring));
        assert_eq!(game.ui.layout(), UILayout::Standard);
    }

    #[test]
    fn fishing_resolves_to_exploring() {
        let mut game = LurhookGame::default();
        game.cast();
        if let GameMode::Fishing { ref mut wait } = game.mode {
            *wait = 0;
        }
        game.meter = Some(TensionMeter {
            duration: 1,
            ..Default::default()
        });
        game.update_fishing();
        assert!(matches!(game.mode, GameMode::Exploring));
        assert_eq!(game.ui.layout(), UILayout::Standard);
    }

    #[test]
    fn save_writes_file() {
        let game = LurhookGame::default();
        let path = "test_save_writes.ron";
        game.save_game(path).unwrap();
        assert!(std::fs::metadata(path).is_ok());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn save_and_load_roundtrip() {
        let game = LurhookGame::default();
        let path = "test_save_roundtrip.ron";
        game.save_game(path).unwrap();
        let loaded = LurhookGame::load_game(path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(loaded.player.pos, game.player.pos);
        assert_eq!(loaded.player.hp, game.player.hp);
        assert_eq!(loaded.time_of_day, game.time_of_day);
    }

    #[test]
    fn camera_clamps_to_bounds() {
        let mut game = LurhookGame::default();
        game.player.pos = common::Point::new(0, 0);
        assert_eq!(game.camera(), (0, 0));

        game.player.pos = common::Point::new(game.map.width as i32, game.map.height as i32);
        let cam = game.camera();
        assert!(cam.0 <= game.map.width as i32 - super::VIEW_WIDTH);
        assert!(cam.1 <= game.map.height as i32 - super::VIEW_HEIGHT);
    }

    #[test]
    fn day_night_cycle_progresses() {
        let mut game = LurhookGame::default();
        assert_eq!(game.time_of_day, "Dawn");
        for _ in 0..super::TIME_SEGMENT_TURNS {
            game.advance_time();
        }
        assert_eq!(game.time_of_day, "Day");
        for _ in 0..super::TIME_SEGMENT_TURNS {
            game.advance_time();
        }
        assert_eq!(game.time_of_day, "Dusk");
    }

    #[test]
    fn score_calculation() {
        let mut game = LurhookGame::default();
        let fish = game.fish_types[0].clone();
        game.player.inventory.push(fish.clone());
        let expected = ((1.0 / fish.rarity) * 10.0) as i32;
        assert_eq!(game.score(), expected);
    }

    #[test]
    fn end_run_sets_mode() {
        let mut game = LurhookGame::default();
        let fish = game.fish_types[0].clone();
        game.player.inventory.push(fish);
        game.end_run();
        assert!(matches!(game.mode, GameMode::End { .. }));
    }
}
