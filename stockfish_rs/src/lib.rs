// Stockfish Rust Port - Core Library

pub mod types;
pub mod bitboard;
pub mod zobrist;
pub mod position;
pub mod movegen;
pub mod evaluate;
pub mod tt;
pub mod search;
pub mod uci;
pub mod perft;

pub use types::*;
pub use bitboard::*;
pub use position::Position;

// Initialize all static data
pub fn init() {
    bitboard::init();
    zobrist::init();
}
