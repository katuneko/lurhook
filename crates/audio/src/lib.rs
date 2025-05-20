//! Simple audio playback utilities.

use common::GameResult;

/// Supported sound effect kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    Hit,
    LineSnap,
    Catch,
    Storm,
}

/// Basic audio manager storing volume level.
#[derive(Debug)]
pub struct AudioManager {
    volume: u8,
}

impl AudioManager {
    /// Creates a new manager with the given volume (0-10).
    pub fn new(volume: u8) -> Self {
        Self {
            volume: volume.min(10),
        }
    }

    /// Sets the playback volume (0-10).
    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume.min(10);
    }

    /// Returns current volume.
    pub fn volume(&self) -> u8 {
        self.volume
    }

    /// Plays the requested sound effect.
    pub fn play(&self, sound: Sound) -> GameResult<()> {
        println!("Play sound {:?} at volume {}", sound, self.volume);
        Ok(())
    }
}

pub fn init() {
    println!("Initialized crate: audio");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_clamped() {
        let m = AudioManager::new(15);
        assert_eq!(m.volume(), 10);
    }

    #[test]
    fn set_volume_clamps() {
        let mut m = AudioManager::new(5);
        m.set_volume(20);
        assert_eq!(m.volume(), 10);
    }

    #[test]
    fn play_runs() {
        let m = AudioManager::new(3);
        assert!(m.play(Sound::Hit).is_ok());
    }
}
