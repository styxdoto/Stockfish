// Bitboard utilities and magic bitboards for sliding piece attack generation

use crate::types::*;
use std::sync::Once;

// File and rank bitboards
pub const FILE_A_BB: Bitboard = 0x0101010101010101;
pub const FILE_B_BB: Bitboard = FILE_A_BB << 1;
pub const FILE_C_BB: Bitboard = FILE_A_BB << 2;
pub const FILE_D_BB: Bitboard = FILE_A_BB << 3;
pub const FILE_E_BB: Bitboard = FILE_A_BB << 4;
pub const FILE_F_BB: Bitboard = FILE_A_BB << 5;
pub const FILE_G_BB: Bitboard = FILE_A_BB << 6;
pub const FILE_H_BB: Bitboard = FILE_A_BB << 7;

pub const RANK_1_BB: Bitboard = 0xFF;
pub const RANK_2_BB: Bitboard = RANK_1_BB << 8;
pub const RANK_3_BB: Bitboard = RANK_1_BB << 16;
pub const RANK_4_BB: Bitboard = RANK_1_BB << 24;
pub const RANK_5_BB: Bitboard = RANK_1_BB << 32;
pub const RANK_6_BB: Bitboard = RANK_1_BB << 40;
pub const RANK_7_BB: Bitboard = RANK_1_BB << 48;
pub const RANK_8_BB: Bitboard = RANK_1_BB << 56;

const FILE_BBS: [Bitboard; 8] = [
    FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB,
    FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
];

const RANK_BBS: [Bitboard; 8] = [
    RANK_1_BB, RANK_2_BB, RANK_3_BB, RANK_4_BB,
    RANK_5_BB, RANK_6_BB, RANK_7_BB, RANK_8_BB,
];

#[inline(always)]
pub fn square_bb(sq: Square) -> Bitboard {
    1u64 << sq.index()
}

#[inline(always)]
pub fn file_bb(file: File) -> Bitboard {
    FILE_BBS[file.index()]
}

#[inline(always)]
pub fn rank_bb(rank: Rank) -> Bitboard {
    RANK_BBS[rank.index()]
}

#[inline(always)]
pub fn popcount(b: Bitboard) -> u32 {
    b.count_ones()
}

#[inline(always)]
pub fn lsb(b: Bitboard) -> Square {
    debug_assert!(b != 0);
    Square::from_index(b.trailing_zeros() as usize)
}

#[inline(always)]
pub fn msb(b: Bitboard) -> Square {
    debug_assert!(b != 0);
    Square::from_index(63 - b.leading_zeros() as usize)
}

#[inline(always)]
pub fn pop_lsb(b: &mut Bitboard) -> Square {
    let sq = lsb(*b);
    *b &= *b - 1;
    sq
}

#[inline(always)]
pub fn more_than_one(b: Bitboard) -> bool {
    b & (b - 1) != 0
}

// Shift operations
#[inline(always)]
pub fn shift_north(b: Bitboard) -> Bitboard {
    b << 8
}

#[inline(always)]
pub fn shift_south(b: Bitboard) -> Bitboard {
    b >> 8
}

#[inline(always)]
pub fn shift_east(b: Bitboard) -> Bitboard {
    (b & !FILE_H_BB) << 1
}

#[inline(always)]
pub fn shift_west(b: Bitboard) -> Bitboard {
    (b & !FILE_A_BB) >> 1
}

#[inline(always)]
pub fn shift_north_east(b: Bitboard) -> Bitboard {
    (b & !FILE_H_BB) << 9
}

#[inline(always)]
pub fn shift_north_west(b: Bitboard) -> Bitboard {
    (b & !FILE_A_BB) << 7
}

#[inline(always)]
pub fn shift_south_east(b: Bitboard) -> Bitboard {
    (b & !FILE_H_BB) >> 7
}

#[inline(always)]
pub fn shift_south_west(b: Bitboard) -> Bitboard {
    (b & !FILE_A_BB) >> 9
}

#[inline(always)]
pub fn pawn_attacks_bb(color: Color, pawns: Bitboard) -> Bitboard {
    match color {
        Color::White => shift_north_east(pawns) | shift_north_west(pawns),
        Color::Black => shift_south_east(pawns) | shift_south_west(pawns),
    }
}

// Magic bitboard structure
#[derive(Copy, Clone)]
struct Magic {
    mask: Bitboard,
    magic: u64,
    attacks: &'static [Bitboard],
    shift: u32,
}

impl Magic {
    #[inline(always)]
    fn index(&self, occupied: Bitboard) -> usize {
        let relevant = occupied & self.mask;
        (relevant.wrapping_mul(self.magic) >> self.shift) as usize
    }

    #[inline(always)]
    fn attacks(&self, occupied: Bitboard) -> Bitboard {
        self.attacks[self.index(occupied)]
    }
}

// Global attack tables
static mut ROOK_MAGICS: [Magic; 64] = [Magic {
    mask: 0,
    magic: 0,
    attacks: &[],
    shift: 0,
}; 64];

static mut BISHOP_MAGICS: [Magic; 64] = [Magic {
    mask: 0,
    magic: 0,
    attacks: &[],
    shift: 0,
}; 64];

static mut ROOK_TABLE: Vec<Bitboard> = Vec::new();
static mut BISHOP_TABLE: Vec<Bitboard> = Vec::new();

static mut PSEUDO_ATTACKS: [[Bitboard; 64]; 8] = [[0; 64]; 8];
static mut LINE_BB: [[Bitboard; 64]; 64] = [[0; 64]; 64];
static mut BETWEEN_BB: [[Bitboard; 64]; 64] = [[0; 64]; 64];
static mut SQUARE_DISTANCE: [[u8; 64]; 64] = [[0; 64]; 64];

static INIT: Once = Once::new();

fn sliding_attack(sq: Square, occupied: Bitboard, directions: &[i8]) -> Bitboard {
    let mut attacks = 0u64;

    for &delta in directions {
        let mut s = sq.index() as i8;
        loop {
            let from_file = (s & 7) as usize;
            s += delta;
            if s < 0 || s >= 64 {
                break;
            }
            let to_file = (s & 7) as usize;
            // Check if we wrapped around the board
            if (delta == 1 || delta == -1 || delta == 9 || delta == -9 ||
                delta == 7 || delta == -7) &&
               (from_file as i8 - to_file as i8).abs() > 1 {
                break;
            }

            attacks |= 1u64 << s;

            if occupied & (1u64 << s) != 0 {
                break;
            }
        }
    }

    attacks
}

fn rook_attack(sq: Square, occupied: Bitboard) -> Bitboard {
    sliding_attack(sq, occupied, &[8, -8, 1, -1])
}

fn bishop_attack(sq: Square, occupied: Bitboard) -> Bitboard {
    sliding_attack(sq, occupied, &[9, -9, 7, -7])
}

// Simple PRNG for magic number generation
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

    fn sparse_rand64(&mut self) -> u64 {
        self.rand64() & self.rand64() & self.rand64()
    }
}

fn init_magics(is_rook: bool) {
    const ROOK_MAGIC_NUMBERS: [u64; 64] = [
        0x0080001020400080, 0x0040001000200040, 0x0080081000200080, 0x0080040800100080,
        0x0080020400080080, 0x0080010200040080, 0x0080008001000200, 0x0080002040800100,
        0x0000800020400080, 0x0000400020005000, 0x0000801000200080, 0x0000800800100080,
        0x0000800400080080, 0x0000800200040080, 0x0000800100020080, 0x0000800040800100,
        0x0000208000400080, 0x0000404000201000, 0x0000808010002000, 0x0000808008001000,
        0x0000808004000800, 0x0000808002000400, 0x0000010100020004, 0x0000020000408104,
        0x0000208080004000, 0x0000200040005000, 0x0000100080200080, 0x0000080080100080,
        0x0000040080080080, 0x0000020080040080, 0x0000010080800200, 0x0000800080004100,
        0x0000204000800080, 0x0000200040401000, 0x0000100080802000, 0x0000080080801000,
        0x0000040080800800, 0x0000020080800400, 0x0000020001010004, 0x0000800040800100,
        0x0000204000808000, 0x0000200040008080, 0x0000100020008080, 0x0000080010008080,
        0x0000040008008080, 0x0000020004008080, 0x0000010002008080, 0x0000004081020004,
        0x0000204000800080, 0x0000200040008080, 0x0000100020008080, 0x0000080010008080,
        0x0000040008008080, 0x0000020004008080, 0x0000800100020080, 0x0000800041000080,
        0x00FFFCDDFCED714A, 0x007FFCDDFCED714A, 0x003FFFCDFFD88096, 0x0000040810002101,
        0x0001000204080011, 0x0001000204000801, 0x0001000082000401, 0x0001FFFAABFAD1A2,
    ];

    const BISHOP_MAGIC_NUMBERS: [u64; 64] = [
        0x0002020202020200, 0x0002020202020000, 0x0004010202000000, 0x0004040080000000,
        0x0001104000000000, 0x0000821040000000, 0x0000410410400000, 0x0000104104104000,
        0x0000040404040400, 0x0000020202020200, 0x0000040102020000, 0x0000040400800000,
        0x0000011040000000, 0x0000008210400000, 0x0000004104104000, 0x0000002082082000,
        0x0004000808080800, 0x0002000404040400, 0x0001000202020200, 0x0000800802004000,
        0x0000800400A00000, 0x0000200100884000, 0x0000400082082000, 0x0000200041041000,
        0x0002080010101000, 0x0001040008080800, 0x0000208004010400, 0x0000404004010200,
        0x0000840000802000, 0x0000404002011000, 0x0000808001041000, 0x0000404000820800,
        0x0001041000202000, 0x0000820800101000, 0x0000104400080800, 0x0000020080080080,
        0x0000404040040100, 0x0000808100020100, 0x0001010100020800, 0x0000808080010400,
        0x0000820820004000, 0x0000410410002000, 0x0000082088001000, 0x0000002011000800,
        0x0000080100400400, 0x0001010101000200, 0x0002020202000400, 0x0001010101000200,
        0x0000410410400000, 0x0000208208200000, 0x0000002084100000, 0x0000000020880000,
        0x0000001002020000, 0x0000040408020000, 0x0004040404040000, 0x0002020202020000,
        0x0000104104104000, 0x0000002082082000, 0x0000000020841000, 0x0000000000208800,
        0x0000000010020200, 0x0000000404080200, 0x0000040404040400, 0x0002020202020200,
    ];

    let magics_array = if is_rook { &ROOK_MAGIC_NUMBERS } else { &BISHOP_MAGIC_NUMBERS };

    let mut table = Vec::with_capacity(if is_rook { 102400 } else { 5248 });

    for sq_idx in 0..64 {
        let sq = Square::from_index(sq_idx);

        // Calculate mask
        let edges = ((RANK_1_BB | RANK_8_BB) & !rank_bb(sq.rank()))
                  | ((FILE_A_BB | FILE_H_BB) & !file_bb(sq.file()));

        let mask = if is_rook {
            rook_attack(sq, 0)
        } else {
            bishop_attack(sq, 0)
        } & !edges;

        let shift = 64 - popcount(mask);

        // Generate all occupancy variations
        let bits = popcount(mask);
        let variations = 1 << bits;
        let mut occupancies = vec![0u64; variations];
        let mut attacks = vec![0u64; variations];

        for i in 0..variations {
            occupancies[i] = index_to_bitboard(i, bits, mask);
            attacks[i] = if is_rook {
                rook_attack(sq, occupancies[i])
            } else {
                bishop_attack(sq, occupancies[i])
            };
        }

        // Use pre-calculated magic
        let magic = magics_array[sq_idx];

        // Build attack table
        let table_size = 1 << bits;
        let start_idx = table.len();
        table.resize(start_idx + table_size, 0);

        for i in 0..variations {
            let idx = ((occupancies[i].wrapping_mul(magic)) >> shift) as usize;
            table[start_idx + idx] = attacks[i];
        }

        // Store magic structure
        let table_slice: &'static [Bitboard] = unsafe {
            std::mem::transmute(&table[start_idx..start_idx + table_size])
        };

        let magic_struct = Magic {
            mask,
            magic,
            attacks: table_slice,
            shift,
        };

        unsafe {
            if is_rook {
                ROOK_MAGICS[sq_idx] = magic_struct;
            } else {
                BISHOP_MAGICS[sq_idx] = magic_struct;
            }
        }
    }

    // Store table
    unsafe {
        if is_rook {
            ROOK_TABLE = table;
        } else {
            BISHOP_TABLE = table;
        }
    }
}

fn index_to_bitboard(index: usize, bits: u32, mut mask: Bitboard) -> Bitboard {
    let mut result = 0u64;
    for i in 0..bits {
        let lsb = mask.trailing_zeros();
        mask &= mask - 1;
        if index & (1 << i) != 0 {
            result |= 1u64 << lsb;
        }
    }
    result
}

pub fn init() {
    INIT.call_once(|| {
        // Initialize distance table
        for sq1 in 0..64 {
            for sq2 in 0..64 {
                let file_dist = ((sq1 & 7) as i8 - (sq2 & 7) as i8).abs() as u8;
                let rank_dist = ((sq1 >> 3) as i8 - (sq2 >> 3) as i8).abs() as u8;
                unsafe {
                    SQUARE_DISTANCE[sq1][sq2] = std::cmp::max(file_dist, rank_dist);
                }
            }
        }

        // Initialize magic bitboards
        init_magics(true);   // Rooks
        init_magics(false);  // Bishops

        // Initialize pseudo attacks
        for sq_idx in 0..64 {
            let sq = Square::from_index(sq_idx);
            let sq_bb = square_bb(sq);

            // Pawn attacks
            unsafe {
                PSEUDO_ATTACKS[Color::White.index()][sq_idx] = pawn_attacks_bb(Color::White, sq_bb);
                PSEUDO_ATTACKS[Color::Black.index()][sq_idx] = pawn_attacks_bb(Color::Black, sq_bb);
            }

            // Knight attacks
            let knight_attacks = {
                let mut attacks = 0u64;
                for &delta in &[-17i8, -15, -10, -6, 6, 10, 15, 17] {
                    if let Some(to) = sq.offset(delta) {
                        let dist = distance(sq, to);
                        if dist <= 2 {
                            attacks |= square_bb(to);
                        }
                    }
                }
                attacks
            };
            unsafe { PSEUDO_ATTACKS[PieceType::Knight.index()][sq_idx] = knight_attacks; }

            // King attacks
            let king_attacks = {
                let mut attacks = 0u64;
                for &delta in &[-9i8, -8, -7, -1, 1, 7, 8, 9] {
                    if let Some(to) = sq.offset(delta) {
                        let dist = distance(sq, to);
                        if dist <= 2 {
                            attacks |= square_bb(to);
                        }
                    }
                }
                attacks
            };
            unsafe { PSEUDO_ATTACKS[PieceType::King.index()][sq_idx] = king_attacks; }

            // Bishop, rook, queen attacks (empty board)
            let bishop_attacks = bishop_attacks_bb(sq, 0);
            let rook_attacks = rook_attacks_bb(sq, 0);
            unsafe {
                PSEUDO_ATTACKS[PieceType::Bishop.index()][sq_idx] = bishop_attacks;
                PSEUDO_ATTACKS[PieceType::Rook.index()][sq_idx] = rook_attacks;
                PSEUDO_ATTACKS[PieceType::Queen.index()][sq_idx] = bishop_attacks | rook_attacks;
            }
        }

        // Initialize line and between bitboards
        for sq1_idx in 0..64 {
            for sq2_idx in 0..64 {
                let sq1 = Square::from_index(sq1_idx);
                let sq2 = Square::from_index(sq2_idx);

                for &is_rook in &[true, false] {
                    let pseudo = if is_rook {
                        unsafe { PSEUDO_ATTACKS[PieceType::Rook.index()][sq1_idx] }
                    } else {
                        unsafe { PSEUDO_ATTACKS[PieceType::Bishop.index()][sq1_idx] }
                    };

                    if pseudo & square_bb(sq2) != 0 {
                        let attacks1 = if is_rook {
                            rook_attacks_bb(sq1, 0)
                        } else {
                            bishop_attacks_bb(sq1, 0)
                        };
                        let attacks2 = if is_rook {
                            rook_attacks_bb(sq2, 0)
                        } else {
                            bishop_attacks_bb(sq2, 0)
                        };

                        unsafe {
                            LINE_BB[sq1_idx][sq2_idx] = (attacks1 & attacks2) | square_bb(sq1) | square_bb(sq2);

                            let attacks1_blocked = if is_rook {
                                rook_attacks_bb(sq1, square_bb(sq2))
                            } else {
                                bishop_attacks_bb(sq1, square_bb(sq2))
                            };
                            let attacks2_blocked = if is_rook {
                                rook_attacks_bb(sq2, square_bb(sq1))
                            } else {
                                bishop_attacks_bb(sq2, square_bb(sq1))
                            };

                            BETWEEN_BB[sq1_idx][sq2_idx] = (attacks1_blocked & attacks2_blocked) | square_bb(sq2);
                        }
                    } else {
                        unsafe {
                            BETWEEN_BB[sq1_idx][sq2_idx] |= square_bb(sq2);
                        }
                    }
                }
            }
        }
    });
}

#[inline(always)]
pub fn bishop_attacks_bb(sq: Square, occupied: Bitboard) -> Bitboard {
    unsafe { BISHOP_MAGICS[sq.index()].attacks(occupied) }
}

#[inline(always)]
pub fn rook_attacks_bb(sq: Square, occupied: Bitboard) -> Bitboard {
    unsafe { ROOK_MAGICS[sq.index()].attacks(occupied) }
}

#[inline(always)]
pub fn queen_attacks_bb(sq: Square, occupied: Bitboard) -> Bitboard {
    bishop_attacks_bb(sq, occupied) | rook_attacks_bb(sq, occupied)
}

#[inline(always)]
pub fn knight_attacks_bb(sq: Square) -> Bitboard {
    unsafe { PSEUDO_ATTACKS[PieceType::Knight.index()][sq.index()] }
}

#[inline(always)]
pub fn king_attacks_bb(sq: Square) -> Bitboard {
    unsafe { PSEUDO_ATTACKS[PieceType::King.index()][sq.index()] }
}

#[inline(always)]
pub fn pawn_attacks_bb_sq(color: Color, sq: Square) -> Bitboard {
    unsafe { PSEUDO_ATTACKS[color.index()][sq.index()] }
}

#[inline(always)]
pub fn line_bb(sq1: Square, sq2: Square) -> Bitboard {
    unsafe { LINE_BB[sq1.index()][sq2.index()] }
}

#[inline(always)]
pub fn between_bb(sq1: Square, sq2: Square) -> Bitboard {
    unsafe { BETWEEN_BB[sq1.index()][sq2.index()] }
}

#[inline(always)]
pub fn distance(sq1: Square, sq2: Square) -> u8 {
    unsafe { SQUARE_DISTANCE[sq1.index()][sq2.index()] }
}

#[inline(always)]
pub fn attacks_bb(pt: PieceType, sq: Square, occupied: Bitboard) -> Bitboard {
    match pt {
        PieceType::Bishop => bishop_attacks_bb(sq, occupied),
        PieceType::Rook => rook_attacks_bb(sq, occupied),
        PieceType::Queen => queen_attacks_bb(sq, occupied),
        PieceType::Knight => knight_attacks_bb(sq),
        PieceType::King => king_attacks_bb(sq),
        _ => 0,
    }
}
