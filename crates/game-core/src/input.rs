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
    pub save: VirtualKeyCode,
    pub quit: VirtualKeyCode,
    pub end_run: VirtualKeyCode,
    pub scroll_up: VirtualKeyCode,
    pub scroll_down: VirtualKeyCode,
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
            save: S,
            quit: Q,
            end_run: Return,
            scroll_up: PageUp,
            scroll_down: PageDown,
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
            let val = val.trim().trim_matches('"');
            if let Some(kc) = parse_key(val) {
                match key.trim() {
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
                    "save" => cfg.save = kc,
                    "quit" => cfg.quit = kc,
                    "end_run" => cfg.end_run = kc,
                    "scroll_up" => cfg.scroll_up = kc,
                    "scroll_down" => cfg.scroll_down = kc,
                    _ => {}
                }
            }
        }
        Ok(cfg)
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
        "x" => Some(X),
        "r" => Some(R),
        "i" => Some(I),
        "s" => Some(S),
        "q" => Some(Q),
        "return" => Some(Return),
        "pageup" => Some(PageUp),
        "pagedown" => Some(PageDown),
        _ => None,
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
    }

    #[test]
    fn load_overrides_fields() {
        let mut path = std::env::temp_dir();
        path.push("test_input.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "cast = \"X\"").unwrap();
        let cfg = InputConfig::load(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(cfg.cast, VirtualKeyCode::X);
    }
}
