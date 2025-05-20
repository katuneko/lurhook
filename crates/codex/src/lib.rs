//! Codex system for recording captured fish.

use std::collections::HashMap;
use common::{GameResult};

/// Mapping from fish id to capture count.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Codex {
    records: HashMap<String, u32>,
}

impl Codex {
    /// Loads codex data from a simple JSON map file.
    pub fn load(path: &str) -> GameResult<Self> {
        let data = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e.into()),
        };
        let mut records = HashMap::new();
        for line in data.trim().trim_start_matches('{').trim_end_matches('}').split(',') {
            let line = line.trim();
            if line.is_empty() { continue; }
            let mut parts = line.splitn(2, ':');
            let id = parts.next().unwrap().trim().trim_matches('"');
            let count = parts.next().unwrap().trim();
            let count: u32 = count.parse().unwrap_or(0);
            records.insert(id.to_string(), count);
        }
        Ok(Self { records })
    }

    /// Saves codex data back to disk.
    pub fn save(&self, path: &str) -> GameResult<()> {
        let mut out = String::from("{\n");
        for (i, (id, count)) in self.records.iter().enumerate() {
            out.push_str(&format!("  \"{}\": {}", id, count));
            if i + 1 != self.records.len() { out.push_str(",\n"); } else { out.push('\n'); }
        }
        out.push('}');
        std::fs::write(path, out)?;
        Ok(())
    }

    /// Increments capture count for a fish id and saves immediately.
    pub fn record_capture(&mut self, path: &str, id: &str) -> GameResult<()> {
        *self.records.entry(id.to_string()).or_insert(0) += 1;
        self.save(path)
    }

    /// Returns the capture count for a fish id.
    pub fn count(&self, id: &str) -> u32 {
        *self.records.get(id).unwrap_or(&0)
    }

    /// Returns the total capture count across all fish.
    pub fn total_captures(&self) -> u32 {
        self.records.values().copied().sum()
    }

    #[cfg(test)]
    pub fn set_count(&mut self, id: &str, count: u32) {
        self.records.insert(id.to_string(), count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_nonexistent_returns_empty() {
        let c = Codex::load("/tmp/nope.json").unwrap();
        assert!(c.records.is_empty());
    }

    #[test]
    fn record_and_load() {
        let path = "/tmp/codex_test.json";
        let mut c = Codex::default();
        c.record_capture(path, "A").unwrap();
        let loaded = Codex::load(path).unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(loaded.count("A"), 1);
    }

    #[test]
    fn total_captures_sums_values() {
        let mut c = Codex::default();
        c.records.insert("A".into(), 2);
        c.records.insert("B".into(), 3);
        assert_eq!(c.total_captures(), 5);
    }
}
