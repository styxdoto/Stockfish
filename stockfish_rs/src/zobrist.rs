// Zobrist hashing for position keys

use crate::types::*;
use std::sync::Once;

static mut ZOBRIST_PIECE: [[Key; 64]; 16] = [[0; 64]; 16];
static mut ZOBRIST_ENPASSANT: [Key; 8] = [0; 8];
static mut ZOBRIST_CASTLING: [Key; 16] = [0; 16];
static mut ZOBRIST_SIDE: Key = 0;

static INIT: Once = Once::new();

// Simple PRNG for key generation
struct PRNG {
    seed: u64,
}

impl PRNG {
    fn new(seed: u64) -> Self {
        PRNG { seed }
    }

    fn rand64(&mut self) -> u64 {
        self.seed ^= self.seed >> 12;
        self.seed ^= self.seed << 25;
        self.seed ^= self.seed >> 27;
        self.seed.wrapping_mul(2685821657736338717u64)
    }
}

pub fn init() {
    INIT.call_once(|| {
        let mut rng = PRNG::new(1070372);

        // Initialize piece keys
        for piece in 0..16 {
            for sq in 0..64 {
                unsafe {
                    ZOBRIST_PIECE[piece][sq] = rng.rand64();
                }
            }
        }

        // Initialize en passant keys
        for file in 0..8 {
            unsafe {
                ZOBRIST_ENPASSANT[file] = rng.rand64();
            }
        }

        // Initialize castling keys
        for cr in 0..16 {
            unsafe {
                ZOBRIST_CASTLING[cr] = rng.rand64();
            }
        }

        // Initialize side to move key
        unsafe {
            ZOBRIST_SIDE = rng.rand64();
        }
    });
}

#[inline(always)]
pub fn piece_key(piece: Piece, sq: Square) -> Key {
    unsafe { ZOBRIST_PIECE[piece.index()][sq.index()] }
}

#[inline(always)]
pub fn enpassant_key(file: File) -> Key {
    unsafe { ZOBRIST_ENPASSANT[file.index()] }
}

#[inline(always)]
pub fn castling_key(castling_rights: u8) -> Key {
    unsafe { ZOBRIST_CASTLING[castling_rights as usize] }
}

#[inline(always)]
pub fn side_key() -> Key {
    unsafe { ZOBRIST_SIDE }
}
