// Move generation for all piece types

use crate::bitboard::*;
use crate::position::Position;
use crate::types::*;

pub const MAX_MOVES: usize = 256;

pub struct MoveList {
    moves: [Move; MAX_MOVES],
    count: usize,
}

impl MoveList {
    pub fn new() -> Self {
        MoveList {
            moves: [Move::none(); MAX_MOVES],
            count: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, m: Move) {
        debug_assert!(self.count < MAX_MOVES);
        self.moves[self.count] = m;
        self.count += 1;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[inline(always)]
    pub fn get(&self, index: usize) -> Move {
        debug_assert!(index < self.count);
        self.moves[index]
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.count]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GenType {
    Captures,
    Quiets,
    Evasions,
    NonEvasions,
    Legal,
}

// Generate pawn moves
fn generate_pawn_moves<const IS_WHITE: bool, const GEN_TYPE: u8>(
    pos: &Position,
    list: &mut MoveList,
    target: Bitboard,
) {
    let us = if IS_WHITE { Color::White } else { Color::Black };
    let them = us.flip();

    let rank_7 = if IS_WHITE { RANK_7_BB } else { RANK_2_BB };
    let rank_3 = if IS_WHITE { RANK_3_BB } else { RANK_6_BB };

    let up: i8 = if IS_WHITE { 8 } else { -8 };
    let up_right: i8 = if IS_WHITE { 9 } else { -7 };
    let up_left: i8 = if IS_WHITE { 7 } else { -9 };

    let empty_squares = !pos.pieces();
    let enemies = if GEN_TYPE == 2 {
        // Evasions
        pos.checkers()
    } else {
        pos.pieces_c(them)
    };

    let pawns_on_7 = pos.pieces_cp(us, PieceType::Pawn) & rank_7;
    let pawns_not_on_7 = pos.pieces_cp(us, PieceType::Pawn) & !rank_7;

    // Single and double pawn pushes (no promotions)
    if GEN_TYPE != 0 {
        // Not captures-only
        let mut b1 = if IS_WHITE {
            shift_north(pawns_not_on_7)
        } else {
            shift_south(pawns_not_on_7)
        } & empty_squares;

        let mut b2 = if IS_WHITE {
            shift_north(b1 & rank_3)
        } else {
            shift_south(b1 & rank_3)
        } & empty_squares;

        if GEN_TYPE == 2 {
            // Evasions - only blocking squares
            b1 &= target;
            b2 &= target;
        }

        while b1 != 0 {
            let to = pop_lsb(&mut b1);
            list.push(Move::new(Square::from_index((to.index() as i8 - up) as usize), to));
        }

        while b2 != 0 {
            let to = pop_lsb(&mut b2);
            list.push(Move::new(Square::from_index((to.index() as i8 - up - up) as usize), to));
        }
    }

    // Promotions and underpromotions
    if pawns_on_7 != 0 {
        let mut b1 = if IS_WHITE {
            shift_north_east(pawns_on_7)
        } else {
            shift_south_west(pawns_on_7)
        } & enemies;

        let mut b2 = if IS_WHITE {
            shift_north_west(pawns_on_7)
        } else {
            shift_south_east(pawns_on_7)
        } & enemies;

        let mut b3 = if IS_WHITE {
            shift_north(pawns_on_7)
        } else {
            shift_south(pawns_on_7)
        } & empty_squares;

        if GEN_TYPE == 2 {
            b3 &= target;
        }

        while b1 != 0 {
            let to = pop_lsb(&mut b1);
            let from = Square::from_index((to.index() as i8 - up_right) as usize);
            // Always generate queen promotion for captures
            list.push(Move::new_promotion(from, to, PieceType::Queen));
            if GEN_TYPE != 0 {
                // Generate underpromotions for non-captures-only
                list.push(Move::new_promotion(from, to, PieceType::Rook));
                list.push(Move::new_promotion(from, to, PieceType::Bishop));
                list.push(Move::new_promotion(from, to, PieceType::Knight));
            }
        }

        while b2 != 0 {
            let to = pop_lsb(&mut b2);
            let from = Square::from_index((to.index() as i8 - up_left) as usize);
            list.push(Move::new_promotion(from, to, PieceType::Queen));
            if GEN_TYPE != 0 {
                list.push(Move::new_promotion(from, to, PieceType::Rook));
                list.push(Move::new_promotion(from, to, PieceType::Bishop));
                list.push(Move::new_promotion(from, to, PieceType::Knight));
            }
        }

        while b3 != 0 {
            let to = pop_lsb(&mut b3);
            let from = Square::from_index((to.index() as i8 - up) as usize);
            list.push(Move::new_promotion(from, to, PieceType::Queen));
            if GEN_TYPE == 1 || GEN_TYPE == 2 || GEN_TYPE == 3 {
                list.push(Move::new_promotion(from, to, PieceType::Rook));
                list.push(Move::new_promotion(from, to, PieceType::Bishop));
                list.push(Move::new_promotion(from, to, PieceType::Knight));
            }
        }
    }

    // Standard and en passant captures
    if GEN_TYPE == 0 || GEN_TYPE == 2 || GEN_TYPE == 3 {
        let mut b1 = if IS_WHITE {
            shift_north_east(pawns_not_on_7)
        } else {
            shift_south_west(pawns_not_on_7)
        } & enemies;

        let mut b2 = if IS_WHITE {
            shift_north_west(pawns_not_on_7)
        } else {
            shift_south_east(pawns_not_on_7)
        } & enemies;

        while b1 != 0 {
            let to = pop_lsb(&mut b1);
            list.push(Move::new(Square::from_index((to.index() as i8 - up_right) as usize), to));
        }

        while b2 != 0 {
            let to = pop_lsb(&mut b2);
            list.push(Move::new(Square::from_index((to.index() as i8 - up_left) as usize), to));
        }

        // En passant
        if let Some(ep_sq) = pos.ep_square() {
            if GEN_TYPE == 2 {
                // In evasions, skip if it doesn't help
                let ep_target = if IS_WHITE {
                    ep_sq.offset(up).unwrap()
                } else {
                    ep_sq.offset(up).unwrap()
                };
                if (target & square_bb(ep_target)) == 0 {
                    return;
                }
            }

            let mut b1 = pawns_not_on_7 & pawn_attacks_bb_sq(them, ep_sq);

            while b1 != 0 {
                list.push(Move::new_en_passant(pop_lsb(&mut b1), ep_sq));
            }
        }
    }
}

// Generate moves for a specific piece type
fn generate_piece_moves(
    pos: &Position,
    list: &mut MoveList,
    us: Color,
    pt: PieceType,
    target: Bitboard,
) {
    let mut pieces = pos.pieces_cp(us, pt);
    let occupied = pos.pieces();

    while pieces != 0 {
        let from = pop_lsb(&mut pieces);
        let mut attacks = attacks_bb(pt, from, occupied) & target;

        while attacks != 0 {
            list.push(Move::new(from, pop_lsb(&mut attacks)));
        }
    }
}

// Generate all moves for a given type
fn generate_all(pos: &Position, list: &mut MoveList, gen_type: GenType) {
    let us = pos.side_to_move();
    let them = us.flip();
    let ksq = pos.king_square(us);

    // In double check, only king moves are legal
    if gen_type == GenType::Evasions && more_than_one(pos.checkers()) {
        let mut king_moves = king_attacks_bb(ksq) & !pos.pieces_c(us);
        while king_moves != 0 {
            list.push(Move::new(ksq, pop_lsb(&mut king_moves)));
        }
        return;
    }

    let target = match gen_type {
        GenType::Evasions => between_bb(ksq, lsb(pos.checkers())),
        GenType::NonEvasions => !pos.pieces_c(us),
        GenType::Captures => pos.pieces_c(them),
        GenType::Quiets => !pos.pieces(),
        GenType::Legal => !pos.pieces_c(us), // Will filter later
    };

    // Generate pawn moves
    match gen_type {
        GenType::Captures => {
            if us == Color::White {
                generate_pawn_moves::<true, 0>(pos, list, target);
            } else {
                generate_pawn_moves::<false, 0>(pos, list, target);
            }
        }
        GenType::Quiets => {
            if us == Color::White {
                generate_pawn_moves::<true, 1>(pos, list, target);
            } else {
                generate_pawn_moves::<false, 1>(pos, list, target);
            }
        }
        GenType::Evasions => {
            if us == Color::White {
                generate_pawn_moves::<true, 2>(pos, list, target);
            } else {
                generate_pawn_moves::<false, 2>(pos, list, target);
            }
        }
        GenType::NonEvasions | GenType::Legal => {
            if us == Color::White {
                generate_pawn_moves::<true, 3>(pos, list, target);
            } else {
                generate_pawn_moves::<false, 3>(pos, list, target);
            }
        }
    }

    // Generate piece moves
    generate_piece_moves(pos, list, us, PieceType::Knight, target);
    generate_piece_moves(pos, list, us, PieceType::Bishop, target);
    generate_piece_moves(pos, list, us, PieceType::Rook, target);
    generate_piece_moves(pos, list, us, PieceType::Queen, target);

    // Generate king moves
    let king_target = if gen_type == GenType::Evasions {
        !pos.pieces_c(us)
    } else {
        target
    };
    let mut king_moves = king_attacks_bb(ksq) & king_target;
    while king_moves != 0 {
        list.push(Move::new(ksq, pop_lsb(&mut king_moves)));
    }

    // Generate castling moves
    if (gen_type == GenType::Quiets || gen_type == GenType::NonEvasions || gen_type == GenType::Legal)
        && pos.can_castle(us)
    {
        // Kingside
        if pos.can_castle_kingside(us) {
            let rook_sq = pos.castling_rook_square(us, true);
            if !pos.castling_impeded(us, true) {
                list.push(Move::new_castle(ksq, rook_sq));
            }
        }
        // Queenside
        if pos.can_castle_queenside(us) {
            let rook_sq = pos.castling_rook_square(us, false);
            if !pos.castling_impeded(us, false) {
                list.push(Move::new_castle(ksq, rook_sq));
            }
        }
    }
}

pub fn generate_legal_moves(pos: &Position, list: &mut MoveList) {
    let old_count = list.count;

    if pos.checkers() != 0 {
        generate_all(pos, list, GenType::Evasions);
    } else {
        generate_all(pos, list, GenType::NonEvasions);
    }

    // Filter out illegal moves
    let mut i = old_count;
    while i < list.count {
        if !pos.is_legal(list.moves[i]) {
            list.count -= 1;
            list.moves[i] = list.moves[list.count];
        } else {
            i += 1;
        }
    }
}

pub fn generate_captures(pos: &Position, list: &mut MoveList) {
    generate_all(pos, list, GenType::Captures);
}

pub fn generate_quiets(pos: &Position, list: &mut MoveList) {
    generate_all(pos, list, GenType::Quiets);
}

pub fn generate_all_moves(pos: &Position, list: &mut MoveList) {
    if pos.checkers() != 0 {
        generate_all(pos, list, GenType::Evasions);
    } else {
        generate_all(pos, list, GenType::NonEvasions);
    }
}
