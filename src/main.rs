fn main() {
    // Entry point - delegate to game core
    if let Err(e) = game_core::run() {
        eprintln!("Game error: {}", e);
    }
}
