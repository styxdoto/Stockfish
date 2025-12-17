// Stockfish Rust Port - Main Entry Point

use stockfish_rs::{init, uci::UCI};

fn main() {
    // Initialize all static data
    init();

    // Start UCI loop
    let mut uci = UCI::new();
    uci.main_loop();
}
