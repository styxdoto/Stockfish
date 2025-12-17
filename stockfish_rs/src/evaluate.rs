// Evaluation function (simplified version - full NNUE would require extensive implementation)

use crate::bitboard::*;
use crate::position::Position;
use crate::types::*;

// Piece-square tables for basic evaluation
const PAWN_PST: [Value; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     50,  50,  50,  50,  50,  50,  50,  50,
     10,  10,  20,  30,  30,  20,  10,  10,
      5,   5,  10,  25,  25,  10,   5,   5,
      0,   0,   0,  20,  20,   0,   0,   0,
      5,  -5, -10,   0,   0, -10,  -5,   5,
      5,  10,  10, -20, -20,  10,  10,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

const KNIGHT_PST: [Value; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

const BISHOP_PST: [Value; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

const ROOK_PST: [Value; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
      5,  10,  10,  10,  10,  10,  10,   5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
      0,   0,   0,   5,   5,   0,   0,   0,
];

const QUEEN_PST: [Value; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,   5,   5,   5,   0,  -5,
      0,   0,   5,   5,   5,   5,   0,  -5,
    -10,   5,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

const KING_PST_MG: [Value; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -10, -20, -20, -20, -20, -20, -20, -10,
     20,  20,   0,   0,   0,   0,  20,  20,
     20,  30,  10,   0,   0,  10,  30,  20,
];

const KING_PST_EG: [Value; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50,
    -30, -20, -10,   0,   0, -10, -20, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -30,   0,   0,   0,   0, -30, -30,
    -50, -30, -30, -30, -30, -30, -30, -50,
];

#[inline(always)]
fn pst_value(pt: PieceType, sq: Square, color: Color) -> Value {
    let sq_idx = if color == Color::White {
        sq.index()
    } else {
        sq.flip_rank().index()
    };

    match pt {
        PieceType::Pawn => PAWN_PST[sq_idx],
        PieceType::Knight => KNIGHT_PST[sq_idx],
        PieceType::Bishop => BISHOP_PST[sq_idx],
        PieceType::Rook => ROOK_PST[sq_idx],
        PieceType::Queen => QUEEN_PST[sq_idx],
        PieceType::King => KING_PST_MG[sq_idx], // Simplified - would blend MG/EG
    }
}

pub fn evaluate(pos: &Position) -> Value {
    let mut score = 0;

    // Material and piece-square tables
    for sq_idx in 0..64 {
        let sq = Square::from_index(sq_idx);
        if let Some(piece) = pos.piece_on(sq) {
            let color = piece.color();
            let pt = piece.piece_type();
            let piece_value = piece_value(pt);
            let pst = pst_value(pt, sq, color);

            let value = piece_value + pst;

            if color == pos.side_to_move() {
                score += value;
            } else {
                score -= value;
            }
        }
    }

    // Mobility bonus (simplified)
    let us = pos.side_to_move();
    let them = us.flip();

    // Count attacked squares
    let mut our_mobility = 0;
    let mut their_mobility = 0;

    for sq_idx in 0..64 {
        let sq = Square::from_index(sq_idx);
        if let Some(piece) = pos.piece_on(sq) {
            let pt = piece.piece_type();
            if pt != PieceType::Pawn && pt != PieceType::King {
                let attacks = attacks_bb(pt, sq, pos.pieces());
                let mob = popcount(attacks & !pos.pieces_c(piece.color()));

                if piece.color() == us {
                    our_mobility += mob as Value;
                } else {
                    their_mobility += mob as Value;
                }
            }
        }
    }

    score += (our_mobility - their_mobility) * 2;

    // Bishop pair bonus
    if popcount(pos.pieces_cp(us, PieceType::Bishop)) >= 2 {
        score += 30;
    }
    if popcount(pos.pieces_cp(them, PieceType::Bishop)) >= 2 {
        score -= 30;
    }

    // Rook on open file bonus
    for color in [us, them] {
        let mut rooks = pos.pieces_cp(color, PieceType::Rook);
        while rooks != 0 {
            let sq = pop_lsb(&mut rooks);
            let file_bb = file_bb(sq.file());
            let pawns_on_file = pos.pieces_cp(color, PieceType::Pawn) & file_bb;

            if pawns_on_file == 0 {
                let bonus = 20; // Open file bonus
                if color == us {
                    score += bonus;
                } else {
                    score -= bonus;
                }
            }
        }
    }

    // Return score from side to move perspective
    score
}

// Simplified evaluation - in full version this would call NNUE network
pub fn evaluate_nnue(pos: &Position) -> Value {
    // TODO: Full NNUE implementation would go here
    // For now, use classical evaluation
    evaluate(pos)
}
