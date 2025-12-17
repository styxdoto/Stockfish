// Test the position module

use stockfish_rs::*;
use stockfish_rs::position::{Position, START_FEN};

fn main() {
    // Initialize static data
    stockfish_rs::init();

    println!("=== Position Module Test ===\n");

    // Test 1: Load starting position
    println!("Test 1: Loading starting position from FEN");
    let pos = Position::from_fen(START_FEN).unwrap();
    println!("FEN: {}", pos.to_fen());
    println!("Side to move: {:?}", pos.side_to_move());
    println!("Key: 0x{:016x}", pos.key());
    println!();

    // Test 2: Piece placement
    println!("Test 2: Verifying piece placement");
    println!("White King on E1: {:?}", pos.piece_on(Square::E1));
    println!("Black King on E8: {:?}", pos.piece_on(Square::E8));
    println!("White Queen on D1: {:?}", pos.piece_on(Square::D1));
    println!("Black Queen on D8: {:?}", pos.piece_on(Square::D8));
    println!();

    // Test 3: Bitboards
    println!("Test 3: Bitboard counts");
    println!("White pieces: {}", popcount(pos.pieces_c(Color::White)));
    println!("Black pieces: {}", popcount(pos.pieces_c(Color::Black)));
    println!("White pawns: {}", popcount(pos.pieces_cp(Color::White, PieceType::Pawn)));
    println!("Black pawns: {}", popcount(pos.pieces_cp(Color::Black, PieceType::Pawn)));
    println!();

    // Test 4: Move making
    println!("Test 4: Making and unmaking moves");
    let mut pos = pos;
    let m1 = Move::new(Square::E2, Square::E4);
    println!("Making move e2-e4");
    pos.do_move(m1);
    println!("FEN after e2-e4: {}", pos.to_fen());
    println!("Side to move: {:?}", pos.side_to_move());

    let m2 = Move::new(Square::E7, Square::E5);
    println!("Making move e7-e5");
    pos.do_move(m2);
    println!("FEN after e7-e5: {}", pos.to_fen());

    println!("Unmaking move e7-e5");
    pos.undo_move(m2);
    println!("FEN after undo: {}", pos.to_fen());

    println!("Unmaking move e2-e4");
    pos.undo_move(m1);
    println!("FEN after undo: {}", pos.to_fen());
    println!();

    // Test 5: Castling rights
    println!("Test 5: Castling rights");
    println!("White can castle: {}", pos.can_castle(Color::White));
    println!("White can castle kingside: {}", pos.can_castle_kingside(Color::White));
    println!("White can castle queenside: {}", pos.can_castle_queenside(Color::White));
    println!();

    // Test 6: FEN parsing of various positions
    println!("Test 6: Parsing various FEN positions");

    let test_fens = vec![
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
    ];

    for fen in test_fens {
        match Position::from_fen(fen) {
            Ok(p) => {
                let generated = p.to_fen();
                if generated == fen {
                    println!("✓ FEN round-trip successful: {}", fen);
                } else {
                    println!("✗ FEN mismatch:");
                    println!("  Input:  {}", fen);
                    println!("  Output: {}", generated);
                }
            }
            Err(e) => println!("✗ Failed to parse FEN '{}': {}", fen, e),
        }
    }
    println!();

    // Test 7: Attackers
    println!("Test 7: Attackers to square");
    let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();
    let attackers_e4 = pos.attackers_to(Square::E4, pos.pieces());
    println!("Attackers to e4: {} pieces", popcount(attackers_e4));
    println!();

    // Test 8: Check detection
    println!("Test 8: Check detection");
    let pos_check = Position::from_fen("rnb1kbnr/pppp1ppp/8/4p3/5PPq/8/PPPPP2P/RNBQKBNR w KQkq - 1 3").unwrap();
    println!("Position in check: {}", pos_check.in_check());
    println!("Checkers: 0x{:016x}", pos_check.checkers());
    println!();

    println!("=== All tests completed! ===");
}
