use super::*;

impl LurhookGame {
    pub(super) fn advance_time(&mut self) {
        if self.storm_turns > 0 {
            self.storm_turns -= 1;
        }
        self.turn += 1;
        let idx = (self.turn / TIME_SEGMENT_TURNS) % TIMES.len() as u32;
        self.time_of_day = TIMES[idx as usize];
        if self.player.hunger > 0 {
            let loss = self.difficulty.hunger_loss(self.turn);
            if loss > 0 {
                self.player.hunger = (self.player.hunger - loss).max(0);
                if self.player.hunger == 0 {
                    self.ui.add_log("You are starving!").ok();
                }
            }
        } else if self.player.hp > 0 {
            self.player.hp -= 1;
        }
        let idx = self.map.idx(self.player.pos);
        let tile = self.map.tiles[idx];
        match tile {
            TileKind::Land => {
                if self.rng.range(0, 100) < 10 {
                    if self.rng.range(0, 2) == 0 && self.player.hp < MAX_HP {
                        self.player.hp += 1;
                        self.ui.add_log("You rest on the shore.").ok();
                    } else {
                        self.player.canned_food += 1;
                        self.ui.add_log("You found canned food!").ok();
                    }
                }
            }
            TileKind::DeepWater => {
                if self.rng.range(0, 100) < 5 {
                    self.storm_turns = 5;
                    self.ui.add_log("A storm reduces visibility!").ok();
                }
                if self.rng.range(0, 100) < self.difficulty.hazard_chance(self.area) {
                    self.hazards.push(Hazard {
                        pos: self.player.pos,
                        turns: HAZARD_DURATION,
                    });
                    self.ui.add_log("A jellyfish appears!").ok();
                }
            }
            _ => {}
        }
    }

    pub(super) fn current_drift(&self) -> common::Point {
        if (self.turn / TIDE_TURNS) % 2 == 0 {
            common::Point::new(1, 0)
        } else {
            common::Point::new(-1, 0)
        }
    }

    pub(super) fn visibility_radius(&self) -> i32 {
        let idx = self.map.idx(self.player.pos);
        match self.map.tiles[idx] {
            TileKind::DeepWater => {
                let base = 5;
                if self.storm_turns > 0 {
                    base.min(3)
                } else {
                    base
                }
            }
            _ => i32::MAX,
        }
    }

    pub(super) fn is_visible(&self, pt: common::Point) -> bool {
        let r = self.visibility_radius();
        (pt.x - self.player.pos.x).abs() <= r && (pt.y - self.player.pos.y).abs() <= r
    }

    pub(super) fn update_hazards(&mut self) {
        for hazard in self.hazards.iter_mut() {
            if hazard.turns > 0 {
                hazard.turns -= 1;
            }
        }
        for hazard in &self.hazards {
            if hazard.pos == self.player.pos {
                if self.player.hp > 0 {
                    self.player.hp -= HAZARD_DAMAGE;
                    self.ui.add_log("A jellyfish stings you!").ok();
                }
                if self.player.line > 0 {
                    self.player.line = (self.player.line - LINE_DAMAGE).max(0);
                }
            }
        }
        self.hazards.retain(|h| h.turns > 0);
    }
}
