//! Game engine entry point.

mod types;

use bracket_lib::prelude::*;

use common::GameResult;
use ecology::update_fish;
use ecology::{spawn_fish, Fish};
use fishing::{init as fishing_init, TensionMeter};
use mapgen::{generate, Map, TileKind};
use ui::{init as ui_init, UIContext, UILayout};

/// Current game mode.
enum GameMode {
    Exploring,
    Fishing { wait: u8 },
}

pub use types::Player;

/// Basic game state implementing [`GameState`].
pub struct LurhookGame {
    player: Player,
    map: Map,
    fishes: Vec<Fish>,
    ui: UIContext,
    depth: i32,
    time_of_day: &'static str,
    rng: RandomNumberGenerator,
    mode: GameMode,
    meter: Option<TensionMeter>,
    reeling: bool,
}

impl LurhookGame {
    /// Creates a new game with a generated map.
    pub fn new(seed: u64) -> GameResult<Self> {
        let mut map = generate(seed)?;
        let fish = spawn_fish(&mut map)?;
        Ok(Self {
            player: Player {
                pos: common::Point::new(40, 12),
                hp: 10,
                line: 100,
                inventory: Vec::new(),
            },
            map,
            fishes: vec![fish],
            ui: UIContext::default(),
            depth: 0,
            time_of_day: "Dawn",
            rng: RandomNumberGenerator::seeded(seed),
            mode: GameMode::Exploring,
            meter: None,
            reeling: false,
        })
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
                let bite = self.rng.range(0, 100) < 50;
                if bite {
                    self.ui.add_log("Hooked a fish!").ok();
                    self.meter = Some(TensionMeter::default());
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
        for y in 0..self.map.height {
            for x in 0..self.map.width {
                let idx = self.map.idx(common::Point::new(x as i32, y as i32));
                let glyph = match self.map.tiles[idx] {
                    TileKind::Land => '.',
                    TileKind::ShallowWater => '~',
                    TileKind::DeepWater => '≈',
                };
                ctx.print(x as i32, y as i32, glyph);
            }
        }
    }

    /// Draws all fish on the map.
    fn draw_fish(&self, ctx: &mut BTerm) {
        for fish in &self.fishes {
            ctx.print(fish.position.x, fish.position.y, 'f');
        }
    }
}

impl Default for LurhookGame {
    fn default() -> Self {
        Self::new(0).expect("game")
    }
}

impl GameState for LurhookGame {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.handle_input(ctx);
        match self.mode {
            GameMode::Exploring => {
                update_fish(&self.map, &mut self.fishes).expect("fish update");
            }
            GameMode::Fishing { .. } => self.update_fishing(),
        }
        ctx.cls();
        self.draw_map(ctx);
        self.draw_fish(ctx);
        ctx.print(self.player.pos.x, self.player.pos.y, "@");
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
        assert_eq!(game.player.pos, common::Point::new(40, 12));
        assert!(game.player.inventory.is_empty());
        assert_eq!(game.player.hp, 10);
        assert_eq!(game.player.line, 100);
        assert_eq!(game.map.width, 80);
        assert_eq!(game.map.height, 25);
        assert_eq!(game.fishes.len(), 1);
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
}
