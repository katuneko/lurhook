use bracket_lib::prelude::VirtualKeyCode;
use common::GameResult;

/// Configuration for keyboard controls.
#[derive(Clone, Debug)]
pub struct InputConfig {
    pub left: VirtualKeyCode,
    pub right: VirtualKeyCode,
    pub up: VirtualKeyCode,
    pub down: VirtualKeyCode,
    pub up_left: VirtualKeyCode,
    pub up_right: VirtualKeyCode,
    pub down_left: VirtualKeyCode,
    pub down_right: VirtualKeyCode,
    pub cast: VirtualKeyCode,
    pub reel: VirtualKeyCode,
    pub inventory: VirtualKeyCode,
    pub eat: VirtualKeyCode,
    pub cook: VirtualKeyCode,
    pub snack: VirtualKeyCode,
    pub save: VirtualKeyCode,
    pub quit: VirtualKeyCode,
    pub end_run: VirtualKeyCode,
    pub scroll_up: VirtualKeyCode,
    pub scroll_down: VirtualKeyCode,
    pub help: VirtualKeyCode,
    pub options: VirtualKeyCode,
    pub colorblind: bool,
}

impl Default for InputConfig {
    fn default() -> Self {
        use VirtualKeyCode::*;
        Self {
            left: H,
            right: L,
            up: K,
            down: J,
            up_left: Y,
            up_right: U,
            down_left: B,
            down_right: N,
            cast: C,
            reel: R,
            inventory: I,
            eat: X,
            cook: F,
            snack: G,
            save: S,
            quit: Q,
            end_run: Return,
            scroll_up: PageUp,
            scroll_down: PageDown,
            help: F1,
            options: O,
            colorblind: false,
        }
    }
}

impl InputConfig {
    /// Loads configuration from a file if it exists.
    pub fn load(path: &str) -> GameResult<Self> {
        let mut cfg = Self::default();
        let data = match std::fs::read_to_string(path) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(cfg),
            Err(e) => return Err(e.into()),
        };
        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, val) = match line.split_once('=') {
                Some(v) => v,
                None => continue,
            };
            let key = key.trim();
            let val = val.trim().trim_matches('"');
            if key == "colorblind" {
                cfg.colorblind = val.parse().unwrap_or(false);
                continue;
            }
            if let Some(kc) = parse_key(val) {
                match key {
                    "left" => cfg.left = kc,
                    "right" => cfg.right = kc,
                    "up" => cfg.up = kc,
                    "down" => cfg.down = kc,
                    "up_left" => cfg.up_left = kc,
                    "up_right" => cfg.up_right = kc,
                    "down_left" => cfg.down_left = kc,
                    "down_right" => cfg.down_right = kc,
                    "cast" => cfg.cast = kc,
                    "reel" => cfg.reel = kc,
                    "inventory" => cfg.inventory = kc,
                    "eat" => cfg.eat = kc,
                    "cook" => cfg.cook = kc,
                    "snack" => cfg.snack = kc,
                    "save" => cfg.save = kc,
                    "quit" => cfg.quit = kc,
                    "end_run" => cfg.end_run = kc,
                    "scroll_up" => cfg.scroll_up = kc,
                    "scroll_down" => cfg.scroll_down = kc,
                    "help" => cfg.help = kc,
                    "options" => cfg.options = kc,
                    _ => {}
                }
            }
        }
        Ok(cfg)
    }

    /// Saves the configuration to `path`.
    pub fn save(&self, path: &str) -> GameResult<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        macro_rules! write_key {
            ($key:expr, $name:expr) => {
                writeln!(file, "{} = \"{}\"", $name, key_name($key))?;
            };
        }
        write_key!(self.left, "left");
        write_key!(self.right, "right");
        write_key!(self.up, "up");
        write_key!(self.down, "down");
        write_key!(self.up_left, "up_left");
        write_key!(self.up_right, "up_right");
        write_key!(self.down_left, "down_left");
        write_key!(self.down_right, "down_right");
        write_key!(self.cast, "cast");
        write_key!(self.reel, "reel");
        write_key!(self.inventory, "inventory");
        write_key!(self.eat, "eat");
        write_key!(self.cook, "cook");
        write_key!(self.snack, "snack");
        write_key!(self.save, "save");
        write_key!(self.quit, "quit");
        write_key!(self.end_run, "end_run");
        write_key!(self.scroll_up, "scroll_up");
        write_key!(self.scroll_down, "scroll_down");
        write_key!(self.help, "help");
        write_key!(self.options, "options");
        writeln!(file, "colorblind = {}", self.colorblind)?;
        Ok(())
    }
}

fn parse_key(name: &str) -> Option<VirtualKeyCode> {
    use VirtualKeyCode::*;
    match name.to_ascii_lowercase().as_str() {
        "left" => Some(Left),
        "right" => Some(Right),
        "up" => Some(Up),
        "down" => Some(Down),
        "y" => Some(Y),
        "u" => Some(U),
        "h" => Some(H),
        "j" => Some(J),
        "k" => Some(K),
        "l" => Some(L),
        "b" => Some(B),
        "n" => Some(N),
        "c" => Some(C),
        "f" => Some(F),
        "g" => Some(G),
        "x" => Some(X),
        "e" => Some(E),
        "r" => Some(R),
        "i" => Some(I),
        "s" => Some(S),
        "q" => Some(Q),
        "return" => Some(Return),
        "pageup" => Some(PageUp),
        "pagedown" => Some(PageDown),
        "f1" => Some(F1),
        "o" => Some(O),
        _ => None,
    }
}

fn key_name(key: VirtualKeyCode) -> &'static str {
    use VirtualKeyCode::*;
    match key {
        Left => "Left",
        Right => "Right",
        Up => "Up",
        Down => "Down",
        Y => "Y",
        U => "U",
        H => "H",
        J => "J",
        K => "K",
        L => "L",
        B => "B",
        N => "N",
        C => "C",
        F => "F",
        G => "G",
        X => "X",
        E => "E",
        R => "R",
        I => "I",
        S => "S",
        Q => "Q",
        Return => "Return",
        PageUp => "PageUp",
        PageDown => "PageDown",
        F1 => "F1",
        O => "O",
        other => panic!("unsupported key {:?}", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_nonexistent_returns_default() {
        let cfg = InputConfig::load("/no/such/file.toml").unwrap();
        assert_eq!(cfg.cast, VirtualKeyCode::C);
        assert_eq!(cfg.eat, VirtualKeyCode::X);
        assert_eq!(cfg.cook, VirtualKeyCode::F);
        assert_eq!(cfg.snack, VirtualKeyCode::G);
        assert_eq!(cfg.help, VirtualKeyCode::F1);
        assert_eq!(cfg.options, VirtualKeyCode::O);
        assert!(!cfg.colorblind);
    }

    #[test]
    fn load_overrides_fields() {
        let mut path = std::env::temp_dir();
        path.push("test_input.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "cast = \"X\"").unwrap();
        writeln!(file, "eat = \"E\"").unwrap();
        writeln!(file, "cook = \"G\"").unwrap();
        writeln!(file, "snack = \"H\"").unwrap();
        let cfg = InputConfig::load(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(cfg.cast, VirtualKeyCode::X);
        assert_eq!(cfg.eat, VirtualKeyCode::E);
        assert_eq!(cfg.cook, VirtualKeyCode::G);
        assert_eq!(cfg.snack, VirtualKeyCode::H);
        assert!(!cfg.colorblind);
    }

    #[test]
    fn load_colorblind_flag() {
        let mut path = std::env::temp_dir();
        path.push("test_input_colorblind.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "colorblind = true").unwrap();
        let cfg = InputConfig::load(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(path).unwrap();
        assert!(cfg.colorblind);
    }

    #[test]
    fn help_key_parsed() {
        let mut path = std::env::temp_dir();
        path.push("test_help.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "help = \"F1\"").unwrap();
        let cfg = InputConfig::load(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(cfg.help, VirtualKeyCode::F1);
    }

    #[test]
    fn options_key_parsed() {
        let mut path = std::env::temp_dir();
        path.push("test_options.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "options = \"O\"").unwrap();
        let cfg = InputConfig::load(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(cfg.options, VirtualKeyCode::O);
    }

    #[test]
    fn save_round_trip() {
        let cfg = InputConfig::default();
        let mut path = std::env::temp_dir();
        path.push("test_save_round_trip.toml");
        cfg.save(path.to_str().unwrap()).unwrap();
        let loaded = InputConfig::load(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(loaded.left, cfg.left);
        assert_eq!(loaded.colorblind, cfg.colorblind);
    }
}
