//! Game engine entry point.

mod app;
mod input;
mod types;

use bracket_lib::prelude::*;

use common::{GameError, GameResult, Point};
use ecology::update_fish;
use ecology::{spawn_fish_population, Fish};
use fishing::{init as fishing_init, TensionMeter};
use mapgen::{generate, Map, TileKind};
use ui::{init as ui_init, UIContext, UILayout};

const VIEW_WIDTH: i32 = 60;
const VIEW_HEIGHT: i32 = 17;
const LINE_DAMAGE: i32 = 10;
const MAX_HUNGER: i32 = 100;
const EAT_RAW_FISH: i32 = 20;
const EAT_COOKED_FISH: i32 = 40;
const EAT_CANNED_FOOD: i32 = 60;
const COOK_HP_RESTORE: i32 = 2;
const MAX_HP: i32 = 10;
const TIME_SEGMENT_TURNS: u32 = 10;
const TIDE_TURNS: u32 = 20;
const TIMES: [&str; 4] = ["Dawn", "Day", "Dusk", "Night"];
const SAVE_PATH: &str = "savegame.ron";
const CONFIG_PATH: &str = "lurhook.toml";
pub use app::LurhookApp;
use input::InputConfig;

/// Current game mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameMode {
    Exploring,
    Aiming { target: common::Point },
    Fishing { wait: u8 },
    End { score: i32 },
}

pub use types::Player;

/// Basic game state implementing [`GameState`].
pub struct LurhookGame {
    player: Player,
    map: Map,
    fishes: Vec<Fish>,
    ui: UIContext,
    input: InputConfig,
    depth: i32,
    time_of_day: &'static str,
    turn: u32,
    rng: RandomNumberGenerator,
    mode: GameMode,
    meter: Option<TensionMeter>,
    reeling: bool,
    palette: ui::ColorPalette,
}

impl LurhookGame {
    /// Creates a new game with a generated map.
    pub fn new(seed: u64) -> GameResult<Self> {
        let fish_types = {
            #[cfg(target_arch = "wasm32")]
            {
                data::load_fish_types_embedded()?
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fish.json");
                data::load_fish_types(path)?
            }
        };
        let items = {
            #[cfg(target_arch = "wasm32")]
            {
                data::load_item_types_embedded()?
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let item_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/items.json");
                data::load_item_types(item_path)?
            }
        };
        let equipped = items.first().cloned().unwrap_or(data::ItemType {
            id: String::new(),
            name: String::new(),
            tension_bonus: 0,
            bite_bonus: 0.0,
        });
        let mut map = generate(seed)?;
        let fishes = spawn_fish_population(&mut map, &fish_types, 5)?;
        let input = InputConfig::load(CONFIG_PATH)?;
        let palette = if input.colorblind {
            ui::ColorPalette::colorblind()
        } else {
            ui::ColorPalette::default()
        };
        let start = common::Point::new(map.width as i32 / 2, map.height as i32 / 2);
        let depth = map.depth(start);
        let mut game = Self {
            player: Player {
                pos: start,
                hp: MAX_HP,
                hunger: MAX_HUNGER,
                line: 100,
                bait_bonus: equipped.bite_bonus,
                tension_bonus: equipped.tension_bonus,
                canned_food: 0,
                inventory: Vec::new(),
            },
            map,
            fishes,
            ui: UIContext::default(),
            input,
            depth,
            time_of_day: TIMES[0],
            turn: 0,
            rng: RandomNumberGenerator::seeded(seed),
            mode: GameMode::Exploring,
            meter: None,
            reeling: false,
            palette,
        };
        game.ui.set_layout(ui::UILayout::Help);
        Ok(game)
    }

    /// Returns the current game mode.
    pub(crate) fn mode(&self) -> GameMode {
        self.mode
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
        if self.player.hunger > 0 {
            self.player.hunger -= 1;
            if self.player.hunger == 0 {
                self.ui.add_log("You are starving!").ok();
            }
        } else if self.player.hp > 0 {
            self.player.hp -= 1;
        }
    }

    fn current_drift(&self) -> common::Point {
        if (self.turn / TIDE_TURNS) % 2 == 0 {
            common::Point::new(1, 0)
        } else {
            common::Point::new(-1, 0)
        }
    }

    fn visibility_radius(&self) -> i32 {
        let idx = self.map.idx(self.player.pos);
        match self.map.tiles[idx] {
            TileKind::DeepWater => 5,
            _ => i32::MAX,
        }
    }

    fn is_visible(&self, pt: common::Point) -> bool {
        let r = self.visibility_radius();
        (pt.x - self.player.pos.x).abs() <= r && (pt.y - self.player.pos.y).abs() <= r
    }

    fn tile_style(&self, tile: TileKind, visible: bool) -> (char, RGB) {
        let (glyph, color) = match tile {
            TileKind::Land => ('.', self.palette.land),
            TileKind::ShallowWater => ('~', self.palette.shallow),
            TileKind::DeepWater => ('≈', self.palette.deep),
        };
        let color = if visible { color } else { color * 0.4 }; // darken when hidden
        (glyph, color)
    }
    /// Moves the player by the given delta, clamped to screen bounds.
    fn try_move(&mut self, delta: common::Point) {
        let mut x = self.player.pos.x + delta.x;
        let mut y = self.player.pos.y + delta.y;
        x = x.clamp(0, self.map.width as i32 - 1);
        y = y.clamp(0, self.map.height as i32 - 1);
        self.player.pos.x = x;
        self.player.pos.y = y;
        self.depth = self.map.depth(self.player.pos);
    }

    fn score(&self) -> i32 {
        self.player
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
            self.handle_input_key(Some(key), ctx);
        }
    }

    /// Handles an input key without relying on BTerm.
    fn handle_input_key(&mut self, key: Option<VirtualKeyCode>, ctx: &mut BTerm) {
        self.reeling = false;
        if let Some(key) = key {
            use VirtualKeyCode::*;
            if key == self.input.cast {
                match &mut self.mode {
                    GameMode::Exploring => {
                        self.cast();
                        return;
                    }
                    GameMode::Aiming { .. } => {
                        self.confirm_cast();
                        return;
                    }
                    _ => {}
                }
            }
            if key == self.input.reel && matches!(self.mode, GameMode::Fishing { .. }) {
                self.reeling = true;
                return;
            }
            if key == self.input.scroll_up {
                self.ui.scroll_up();
                return;
            }
            if key == self.input.scroll_down {
                self.ui.scroll_down();
                return;
            }
            if key == self.input.help {
                let next = if self.ui.layout() == UILayout::Help {
                    UILayout::Standard
                } else {
                    UILayout::Help
                };
                self.ui.set_layout(next);
                return;
            }
            if key == self.input.save {
                match self.save_game(SAVE_PATH) {
                    Ok(_) => {
                        self.ui.add_log("Game saved.").ok();
                    }
                    Err(e) => {
                        self.ui.add_log(&format!("Save failed: {}", e)).ok();
                    }
                }
                return;
            }
            if key == self.input.quit {
                ctx.quit();
                return;
            }
            if key == self.input.end_run && matches!(self.mode, GameMode::Exploring) {
                self.end_run();
                return;
            }
            if key == self.input.inventory && matches!(self.mode, GameMode::Exploring) {
                let next = if self.ui.layout() == UILayout::Inventory {
                    UILayout::Standard
                } else {
                    UILayout::Inventory
                };
                self.ui.set_layout(next);
                return;
            }
            if key == self.input.eat && self.ui.layout() == UILayout::Inventory {
                self.eat_fish();
                return;
            }
            if key == self.input.cook && self.ui.layout() == UILayout::Inventory {
                self.cook_fish();
                return;
            }
            if key == self.input.snack && self.ui.layout() == UILayout::Inventory {
                self.eat_canned_food();
                return;
            }
            let delta = match key {
                k if k == Left || k == self.input.left => Point::new(-1, 0),
                k if k == Right || k == self.input.right => Point::new(1, 0),
                k if k == Up || k == self.input.up => Point::new(0, -1),
                k if k == Down || k == self.input.down => Point::new(0, 1),
                k if k == self.input.up_left => Point::new(-1, -1),
                k if k == self.input.up_right => Point::new(1, -1),
                k if k == self.input.down_left => Point::new(-1, 1),
                k if k == self.input.down_right => Point::new(1, 1),
                _ => Point::new(0, 0),
            };
            if delta.x != 0 || delta.y != 0 {
                match &mut self.mode {
                    GameMode::Aiming { target } => {
                        target.x = (target.x + delta.x).clamp(0, self.map.width as i32 - 1);
                        target.y = (target.y + delta.y).clamp(0, self.map.height as i32 - 1);
                    }
                    _ if self.ui.layout() != UILayout::Inventory => {
                        self.try_move(delta);
                    }
                    _ => {}
                }
            }
        }
    }

    fn cast(&mut self) {
        if self.player.line <= 0 {
            self.ui.add_log("Your line is broken!").ok();
            return;
        }
        if self.fishes.is_empty() {
            self.ui.add_log("No fish around.").ok();
            return;
        }
        self.ui.add_log("Select target...").ok();
        self.mode = GameMode::Aiming {
            target: self.player.pos,
        };
    }

    fn confirm_cast(&mut self) {
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
                        let mut m = TensionMeter::new(f.kind.strength);
                        m.max_tension += self.player.tension_bonus;
                        self.meter = Some(m);
                    } else {
                        let mut m = TensionMeter::default();
                        m.max_tension += self.player.tension_bonus;
                        self.meter = Some(m);
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
                        if self.player.line > 0 {
                            self.player.line = (self.player.line - LINE_DAMAGE).max(0);
                            if self.player.line == 0 {
                                self.ui.add_log("Your line is ruined.").ok();
                            }
                        }
                        self.mode = GameMode::Exploring;
                        self.ui.set_layout(UILayout::Standard);
                    }
                    MeterState::Lost => {
                        self.ui.add_log("The fish escaped!").ok();
                        self.mode = GameMode::Exploring;
                        self.ui.set_layout(UILayout::Standard);
                    }
                }
            }
        }
    }

    fn eat_fish(&mut self) {
        if let Some(_fish) = self.player.inventory.pop() {
            self.player.hunger = (self.player.hunger + EAT_RAW_FISH).min(MAX_HUNGER);
            self.ui.add_log("You ate a raw fish.").ok();
        } else {
            self.ui.add_log("No fish to eat.").ok();
        }
    }

    fn cook_fish(&mut self) {
        let idx = self.map.idx(self.player.pos);
        if self.map.tiles[idx] != TileKind::Land {
            self.ui.add_log("You need to be on land to cook.").ok();
            return;
        }
        if let Some(_fish) = self.player.inventory.pop() {
            self.player.hunger = (self.player.hunger + EAT_COOKED_FISH).min(MAX_HUNGER);
            self.player.hp = (self.player.hp + COOK_HP_RESTORE).min(MAX_HP);
            self.ui.add_log("You cooked and ate a fish.").ok();
        } else {
            self.ui.add_log("No fish to cook.").ok();
        }
    }

    fn eat_canned_food(&mut self) {
        if self.player.canned_food > 0 {
            self.player.canned_food -= 1;
            self.player.hunger = (self.player.hunger + EAT_CANNED_FOOD).min(MAX_HUNGER);
            self.ui.add_log("You ate canned food.").ok();
        } else {
            self.ui.add_log("No canned food available.").ok();
        }
    }

    /// Draws the map to the screen.
    fn draw_map(&self, ctx: &mut BTerm) {
        let (cam_x, cam_y) = self.camera();
        for y in 0..VIEW_HEIGHT {
            for x in 0..VIEW_WIDTH {
                let mx = cam_x + x;
                let my = cam_y + y;
                let pt = common::Point::new(mx, my);
                let idx = self.map.idx(pt);
                let tile = self.map.tiles[idx];
                let visible = self.is_visible(pt);
                let (glyph, color) = self.tile_style(tile, visible);
                ctx.set(x, y, color, RGB::named(BLACK), to_cp437(glyph));
            }
        }
        if let GameMode::Aiming { target } = self.mode {
            if target.x >= cam_x
                && target.x < cam_x + VIEW_WIDTH
                && target.y >= cam_y
                && target.y < cam_y + VIEW_HEIGHT
            {
                ctx.set(
                    target.x - cam_x,
                    target.y - cam_y,
                    RGB::named(WHITE),
                    RGB::named(BLACK),
                    to_cp437('*'),
                );
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
                && self.is_visible(fish.position)
            {
                ctx.set(
                    fish.position.x - cam_x,
                    fish.position.y - cam_y,
                    self.palette.fish,
                    RGB::named(BLACK),
                    to_cp437('f'),
                );
            }
        }
    }

    /// Saves a minimal game state to a RON-like file at `path`.
    pub fn save_game(&self, path: &str) -> GameResult<()> {
        let content = format!(
            "(player:(pos:(x:{}, y:{}), hp:{}, hunger:{}, food:{}), time_of_day:\"{}\")",
            self.player.pos.x,
            self.player.pos.y,
            self.player.hp,
            self.player.hunger,
            self.player.canned_food,
            self.time_of_day
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
                .find(|c: char| [',', ')'].contains(&c))
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
        game.player.hunger = parse_i32(&data, "hunger:")?;
        game.player.canned_food = parse_i32(&data, "food:")?;
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
        let key = ctx.key;
        self.handle_input(ctx);
        if key.is_some() {
            self.advance_time();
            match self.mode {
                GameMode::Exploring => {
                    let drift = self.current_drift();
                    update_fish(
                        &self.map,
                        &mut self.fishes,
                        &mut self.rng,
                        self.time_of_day,
                        drift,
                    )
                    .expect("fish update");
                }
                GameMode::Aiming { .. } => {}
                GameMode::Fishing { .. } => self.update_fishing(),
                GameMode::End { score } => {
                    ctx.cls();
                    ctx.print_centered(12, "Run Complete!");
                    ctx.print_centered(13, format!("Final score: {}", score));
                    return;
                }
            }
        } else if matches!(self.mode, GameMode::End { .. }) {
            if let GameMode::End { score } = self.mode {
                ctx.cls();
                ctx.print_centered(12, "Run Complete!");
                ctx.print_centered(13, format!("Final score: {}", score));
                return;
            }
        }
        ctx.cls();
        if self.ui.layout() == UILayout::Help {
            self.ui.draw_help(ctx).ok();
            return;
        }
        self.draw_map(ctx);
        self.draw_fish(ctx);
        let (cam_x, cam_y) = self.camera();
        ctx.set(
            self.player.pos.x - cam_x,
            self.player.pos.y - cam_y,
            self.palette.player,
            RGB::named(BLACK),
            to_cp437('@'),
        );
        if let Some(m) = &self.meter {
            self.ui.draw_tension(ctx, m.tension, m.max_tension).ok();
        }
        self.ui.draw_logs(ctx).ok();
        self.ui
            .draw_status(
                ctx,
                self.player.hp,
                self.player.line,
                self.player.hunger,
                self.depth,
                self.time_of_day,
            )
            .ok();
        self.ui.draw_inventory(ctx, &self.player.inventory).ok();
    }
}

/// Runs the game loop using [`bracket-lib`].
pub fn run() -> BError {
    println!("Welcome to Lurhook! (engine stub)");
    init_subsystems()?;

    let context = BTermBuilder::simple(80, 25)?
        .with_title("Lurhook")
        .build()?;
    let gs = app::LurhookApp::new();
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
    use bracket_lib::prelude::{BTerm, VirtualKeyCode, RGB};

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
        assert_eq!(game.player.hp, MAX_HP);
        assert_eq!(game.player.line, 100);
        assert_eq!(game.player.bait_bonus, 0.0);
        assert_eq!(game.player.tension_bonus, 0);
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
    fn cast_enters_aiming_mode() {
        let mut game = LurhookGame::default();
        game.cast();
        assert!(matches!(game.mode, GameMode::Aiming { .. }));
        assert_eq!(game.ui.layout(), UILayout::Help);
    }

    #[test]
    fn cast_fails_without_fish() {
        let mut game = LurhookGame::default();
        game.fishes.clear();
        game.cast();
        assert!(matches!(game.mode, GameMode::Exploring));
        assert_eq!(game.ui.layout(), UILayout::Help);
    }

    #[test]
    fn fishing_resolves_to_exploring() {
        let mut game = LurhookGame::default();
        game.cast();
        game.confirm_cast();
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
        assert_eq!(loaded.player.hunger, game.player.hunger);
        assert_eq!(loaded.player.canned_food, game.player.canned_food);
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
    fn line_reduces_on_break() {
        let mut game = LurhookGame::default();
        game.cast();
        game.confirm_cast();
        if let GameMode::Fishing { ref mut wait } = game.mode {
            *wait = 0;
        }
        game.meter = Some(TensionMeter {
            max_tension: 1,
            ..Default::default()
        });
        game.update_fishing();
        assert_eq!(game.player.line, 100 - super::LINE_DAMAGE);
    }

    #[test]
    fn lost_fish_returns_to_exploring() {
        let mut game = LurhookGame::default();
        game.cast();
        game.confirm_cast();
        if let GameMode::Fishing { ref mut wait } = game.mode {
            *wait = 0;
        }
        game.meter = Some(TensionMeter {
            tension: 10,
            ..Default::default()
        });
        game.reeling = true;
        game.update_fishing();
        assert!(matches!(game.mode, GameMode::Exploring));
        assert_eq!(game.ui.layout(), UILayout::Standard);
    }

    #[test]
    fn cannot_cast_without_line() {
        let mut game = LurhookGame::default();
        game.player.line = 0;
        game.cast();
        assert!(matches!(game.mode, GameMode::Exploring));
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
    fn hunger_ticks_down() {
        let mut game = LurhookGame::default();
        let start = game.player.hunger;
        game.advance_time();
        assert_eq!(game.player.hunger, start - 1);
    }

    #[test]
    fn starvation_damages_hp() {
        let mut game = LurhookGame::default();
        game.player.hunger = 0;
        let hp_before = game.player.hp;
        game.advance_time();
        assert_eq!(game.player.hp, hp_before - 1);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn score_calculation() {
        let mut game = LurhookGame::default();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fish.json");
        let fish = data::load_fish_types(path).expect("types")[0].clone();
        game.player.inventory.push(fish.clone());
        let expected = ((1.0 / fish.rarity) * 10.0) as i32;
        assert_eq!(game.score(), expected);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn end_run_sets_mode() {
        let mut game = LurhookGame::default();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fish.json");
        let fish = data::load_fish_types(path).expect("types")[0].clone();
        game.player.inventory.push(fish);
        game.end_run();
        assert!(matches!(game.mode, GameMode::End { .. }));
    }

    fn dummy_ctx(key: VirtualKeyCode) -> BTerm {
        BTerm {
            width_pixels: 0,
            height_pixels: 0,
            original_height_pixels: 0,
            original_width_pixels: 0,
            fps: 0.0,
            frame_time_ms: 0.0,
            active_console: 0,
            key: Some(key),
            mouse_pos: (0, 0),
            left_click: false,
            shift: false,
            control: false,
            alt: false,
            web_button: None,
            quitting: false,
            post_scanlines: false,
            post_screenburn: false,
            screen_burn_color: RGB::from_f32(0.0, 0.0, 0.0),
            mouse_visible: true,
        }
    }

    fn dummy_ctx_opt(key: Option<VirtualKeyCode>) -> BTerm {
        BTerm {
            width_pixels: 0,
            height_pixels: 0,
            original_height_pixels: 0,
            original_width_pixels: 0,
            fps: 0.0,
            frame_time_ms: 0.0,
            active_console: 0,
            key,
            mouse_pos: (0, 0),
            left_click: false,
            shift: false,
            control: false,
            alt: false,
            web_button: None,
            quitting: false,
            post_scanlines: false,
            post_screenburn: false,
            screen_burn_color: RGB::from_f32(0.0, 0.0, 0.0),
            mouse_visible: true,
        }
    }

    #[test]
    fn pressing_s_saves_game() {
        let mut game = LurhookGame::default();
        let mut ctx = dummy_ctx(VirtualKeyCode::S);
        game.handle_input(&mut ctx);
        assert!(std::fs::metadata(super::SAVE_PATH).is_ok());
        std::fs::remove_file(super::SAVE_PATH).unwrap();
    }

    #[test]
    fn pressing_q_quits() {
        let mut game = LurhookGame::default();
        let mut ctx = dummy_ctx(VirtualKeyCode::Q);
        game.handle_input(&mut ctx);
        assert!(ctx.quitting);
    }

    #[test]
    fn time_advances_only_on_input() {
        let mut game = LurhookGame::default();
        let mut ctx = dummy_ctx_opt(None);
        game.handle_input_key(None, &mut ctx);
        assert_eq!(game.turn, 0);

        game.handle_input_key(Some(VirtualKeyCode::Right), &mut ctx);
        game.advance_time();
        assert_eq!(game.turn, 1);
    }

    #[test]
    fn tension_bonus_applied_to_meter() {
        let mut game = LurhookGame::default();
        game.player.tension_bonus = 50;
        game.player.bait_bonus = 1.0; // guarantee bite
        game.cast();
        game.confirm_cast();
        if let GameMode::Fishing { ref mut wait } = game.mode {
            *wait = 0;
        }
        // Force meter creation
        game.update_fishing();
        if let Some(m) = &game.meter {
            assert_eq!(m.max_tension, 150);
        } else {
            panic!("meter not created");
        }
    }

    #[test]
    fn visibility_radius_deep_water() {
        let mut game = LurhookGame::default();
        game.map.tiles.fill(TileKind::DeepWater);
        game.player.pos = common::Point::new(0, 0);
        assert!(game.is_visible(common::Point::new(4, 0)));
        assert!(!game.is_visible(common::Point::new(6, 0)));
    }

    #[test]
    fn visibility_unlimited_on_land() {
        let mut game = LurhookGame::default();
        game.map.tiles.fill(TileKind::Land);
        game.player.pos = common::Point::new(0, 0);
        assert!(game.is_visible(common::Point::new(100, 0)));
    }

    #[test]
    fn eat_fish_restores_hunger() {
        let mut game = LurhookGame::default();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fish.json");
        let fish = data::load_fish_types(path).expect("types")[0].clone();
        game.player.inventory.push(fish);
        game.player.hunger = 50;
        game.eat_fish();
        assert!(game.player.hunger > 50);
        assert!(game.player.inventory.is_empty());
    }

    #[test]
    fn eating_caps_hunger() {
        let mut game = LurhookGame::default();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fish.json");
        let fish = data::load_fish_types(path).expect("types")[0].clone();
        game.player.inventory.push(fish);
        game.player.hunger = super::MAX_HUNGER - 5;
        game.eat_fish();
        assert_eq!(game.player.hunger, super::MAX_HUNGER);
    }

    #[test]
    fn eating_without_fish_logs_message() {
        let mut game = LurhookGame::default();
        game.eat_fish();
        assert_eq!(game.player.hunger, super::MAX_HUNGER);
    }

    #[test]
    fn cook_fish_restores_more_hunger_and_hp() {
        let mut game = LurhookGame::default();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fish.json");
        let fish = data::load_fish_types(path).expect("types")[0].clone();
        game.player.inventory.push(fish);
        game.player.hunger = 50;
        game.player.hp = super::MAX_HP - 2;
        // ensure on land
        game.map.tiles.fill(TileKind::Land);
        game.cook_fish();
        assert!(game.player.hunger > 50);
        assert_eq!(game.player.hp, super::MAX_HP);
        assert!(game.player.inventory.is_empty());
    }

    #[test]
    fn tile_style_darkens_when_not_visible() {
        let game = LurhookGame::default();
        let (g1, c1) = game.tile_style(TileKind::ShallowWater, true);
        let (g2, c2) = game.tile_style(TileKind::ShallowWater, false);
        assert_eq!(g1, g2);
        assert!(c2.g < c1.g);
    }

    #[test]
    fn canned_food_restores_hunger() {
        let mut game = LurhookGame::default();
        game.player.canned_food = 1;
        game.player.hunger = 50;
        game.eat_canned_food();
        assert!(game.player.hunger > 50);
        assert_eq!(game.player.canned_food, 0);
    }
}
