// Core chess types and constants

pub type Bitboard = u64;
pub type Key = u64;

pub const MAX_MOVES: usize = 256;
pub const MAX_PLY: usize = 246;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline(always)]
    pub fn flip(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i8)]
pub enum PieceType {
    Pawn = 1,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
}

impl PieceType {
    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Piece {
    WPawn = 0,
    WKnight = 1,
    WBishop = 2,
    WRook = 3,
    WQueen = 4,
    WKing = 5,
    BPawn = 8,
    BKnight = 9,
    BBishop = 10,
    BRook = 11,
    BQueen = 12,
    BKing = 13,
}

impl Piece {
    #[inline(always)]
    pub fn make(color: Color, piece_type: PieceType) -> Self {
        unsafe { std::mem::transmute(((color as u8) << 3) | (piece_type as u8 - 1)) }
    }

    #[inline(always)]
    pub fn color(self) -> Color {
        if (self as u8) < 8 {
            Color::White
        } else {
            Color::Black
        }
    }

    #[inline(always)]
    pub fn piece_type(self) -> PieceType {
        unsafe { std::mem::transmute(((self as u8 & 7) + 1) as i8) }
    }

    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i8)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    #[inline(always)]
    pub fn from_index(idx: usize) -> Self {
        debug_assert!(idx < 64);
        unsafe { std::mem::transmute(idx as i8) }
    }

    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub fn file(self) -> File {
        File::from_index(self.index() & 7)
    }

    #[inline(always)]
    pub fn rank(self) -> Rank {
        Rank::from_index(self.index() >> 3)
    }

    #[inline(always)]
    pub fn make(file: File, rank: Rank) -> Self {
        Self::from_index((rank.index() << 3) | file.index())
    }

    #[inline(always)]
    pub fn flip_rank(self) -> Self {
        Self::from_index(self.index() ^ 56)
    }

    #[inline(always)]
    pub fn relative_square(self, color: Color) -> Self {
        Self::from_index(self.index() ^ (color.index() * 56))
    }

    #[inline(always)]
    pub fn offset(self, offset: i8) -> Option<Self> {
        let new_sq = self.index() as i8 + offset;
        if new_sq >= 0 && new_sq < 64 {
            Some(Self::from_index(new_sq as usize))
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i8)]
pub enum File {
    A, B, C, D, E, F, G, H,
}

impl File {
    #[inline(always)]
    pub fn from_index(idx: usize) -> Self {
        debug_assert!(idx < 8);
        unsafe { std::mem::transmute(idx as i8) }
    }

    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i8)]
pub enum Rank {
    R1, R2, R3, R4, R5, R6, R7, R8,
}

impl Rank {
    #[inline(always)]
    pub fn from_index(idx: usize) -> Self {
        debug_assert!(idx < 8);
        unsafe { std::mem::transmute(idx as i8) }
    }

    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub fn relative_rank(self, color: Color) -> Self {
        Self::from_index(self.index() ^ (color.index() * 7))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CastlingRights {
    None = 0,
    WhiteOO = 1,
    WhiteOOO = 2,
    BlackOO = 4,
    BlackOOO = 8,
}

impl CastlingRights {
    #[inline(always)]
    pub fn from_bits(bits: u8) -> Self {
        unsafe { std::mem::transmute(bits) }
    }

    #[inline(always)]
    pub fn bits(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    pub fn can_castle_kingside(bits: u8, color: Color) -> bool {
        let mask = if color == Color::White { 1 } else { 4 };
        (bits & mask) != 0
    }

    #[inline(always)]
    pub fn can_castle_queenside(bits: u8, color: Color) -> bool {
        let mask = if color == Color::White { 2 } else { 8 };
        (bits & mask) != 0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Move(pub u16);

impl Move {
    #[inline(always)]
    pub fn new(from: Square, to: Square) -> Self {
        Move(((from.index() as u16) << 6) | (to.index() as u16))
    }

    #[inline(always)]
    pub fn new_promotion(from: Square, to: Square, promo: PieceType) -> Self {
        Move(
            (1 << 14)
                | (((promo as u16 - 2) & 3) << 12)
                | ((from.index() as u16) << 6)
                | (to.index() as u16),
        )
    }

    #[inline(always)]
    pub fn new_en_passant(from: Square, to: Square) -> Self {
        Move((2 << 14) | ((from.index() as u16) << 6) | (to.index() as u16))
    }

    #[inline(always)]
    pub fn new_castle(from: Square, to: Square) -> Self {
        Move((3 << 14) | ((from.index() as u16) << 6) | (to.index() as u16))
    }

    #[inline(always)]
    pub fn null() -> Self {
        Move(65)
    }

    #[inline(always)]
    pub fn none() -> Self {
        Move(0)
    }

    #[inline(always)]
    pub fn from_sq(self) -> Square {
        Square::from_index(((self.0 >> 6) & 0x3F) as usize)
    }

    #[inline(always)]
    pub fn to_sq(self) -> Square {
        Square::from_index((self.0 & 0x3F) as usize)
    }

    #[inline(always)]
    pub fn is_promotion(self) -> bool {
        (self.0 & (3 << 14)) == (1 << 14)
    }

    #[inline(always)]
    pub fn is_en_passant(self) -> bool {
        (self.0 & (3 << 14)) == (2 << 14)
    }

    #[inline(always)]
    pub fn is_castle(self) -> bool {
        (self.0 & (3 << 14)) == (3 << 14)
    }

    #[inline(always)]
    pub fn promotion_type(self) -> Option<PieceType> {
        if self.is_promotion() {
            Some(unsafe { std::mem::transmute((((self.0 >> 12) & 3) + 2) as i8) })
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn is_ok(self) -> bool {
        self.0 != 0 && self.0 != 65
    }
}

// Value types for search
pub type Value = i32;

pub const VALUE_ZERO: Value = 0;
pub const VALUE_DRAW: Value = 0;
pub const VALUE_NONE: Value = 32002;
pub const VALUE_INFINITE: Value = 32001;
pub const VALUE_MATE: Value = 32000;
pub const VALUE_MATE_IN_MAX_PLY: Value = VALUE_MATE - MAX_PLY as Value;
pub const VALUE_MATED_IN_MAX_PLY: Value = -VALUE_MATE_IN_MAX_PLY;

// Piece values (centipawns)
pub const PAWN_VALUE: Value = 100;
pub const KNIGHT_VALUE: Value = 320;
pub const BISHOP_VALUE: Value = 330;
pub const ROOK_VALUE: Value = 500;
pub const QUEEN_VALUE: Value = 900;

#[inline(always)]
pub fn piece_value(pt: PieceType) -> Value {
    match pt {
        PieceType::Pawn => PAWN_VALUE,
        PieceType::Knight => KNIGHT_VALUE,
        PieceType::Bishop => BISHOP_VALUE,
        PieceType::Rook => ROOK_VALUE,
        PieceType::Queen => QUEEN_VALUE,
        PieceType::King => 0,
    }
}

#[inline(always)]
pub fn mate_in(ply: usize) -> Value {
    VALUE_MATE - ply as Value
}

#[inline(always)]
pub fn mated_in(ply: usize) -> Value {
    -VALUE_MATE + ply as Value
}

pub type Depth = i32;

pub const DEPTH_ZERO: Depth = 0;
pub const DEPTH_QS: Depth = 0;
pub const DEPTH_NONE: Depth = -6;

// Direction offsets
pub const NORTH: i8 = 8;
pub const SOUTH: i8 = -8;
pub const EAST: i8 = 1;
pub const WEST: i8 = -1;
pub const NORTH_EAST: i8 = 9;
pub const NORTH_WEST: i8 = 7;
pub const SOUTH_EAST: i8 = -7;
pub const SOUTH_WEST: i8 = -9;
