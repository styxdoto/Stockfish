// Perft (Performance Test) for move generation verification

use crate::movegen::*;
use crate::position::Position;
use std::time::Instant;

pub fn perft(pos: &mut Position, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0u64;
    let mut move_list = MoveList::new();
    generate_legal_moves(pos, &mut move_list);

    for i in 0..move_list.len() {
        let m = move_list.get(i);
        pos.do_move(m);
        nodes += perft(pos, depth - 1);
        pos.undo_move(m);
    }

    nodes
}

pub fn perft_divide(pos: &mut Position, depth: u32) {
    let start = Instant::now();
    let mut total_nodes = 0u64;

    let mut move_list = MoveList::new();
    generate_legal_moves(pos, &mut move_list);

    println!("\nStarting perft divide at depth {}...\n", depth);

    for i in 0..move_list.len() {
        let m = move_list.get(i);
        pos.do_move(m);
        let nodes = if depth > 1 { perft(pos, depth - 1) } else { 1 };
        pos.undo_move(m);

        println!("{}: {}", move_to_string(m), nodes);
        total_nodes += nodes;
    }

    let elapsed = start.elapsed();
    let nps = if elapsed.as_millis() > 0 {
        (total_nodes as u128 * 1000 / elapsed.as_millis()) as u64
    } else {
        0
    };

    println!("\nNodes searched: {}", total_nodes);
    println!("Time: {} ms", elapsed.as_millis());
    println!("Nodes/second: {}", nps);
}

pub fn perft_test() {
    println!("Running Perft tests...\n");

    // Test 1: Starting position
    let mut pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .expect("Valid FEN");

    println!("Position: Starting position");
    println!("FEN: {}", pos.to_fen());

    let test_cases = vec![
        (1, 20),
        (2, 400),
        (3, 8902),
        (4, 197281),
        (5, 4865609),
    ];

    for (depth, expected) in test_cases {
        let result = perft(&mut pos, depth);
        let status = if result == expected { "✓ PASS" } else { "✗ FAIL" };
        println!(
            "Depth {}: {} nodes (expected {}) {}",
            depth, result, expected, status
        );

        if result != expected {
            println!("ERROR: Perft test failed at depth {}", depth);
            return;
        }
    }

    println!("\n=== Testing Position 2 (Kiwipete) ===");
    let mut pos = Position::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    )
    .expect("Valid FEN");

    println!("FEN: {}", pos.to_fen());

    let test_cases = vec![
        (1, 48),
        (2, 2039),
        (3, 97862),
        (4, 4085603),
    ];

    for (depth, expected) in test_cases {
        let start = Instant::now();
        let result = perft(&mut pos, depth);
        let elapsed = start.elapsed();

        let status = if result == expected { "✓ PASS" } else { "✗ FAIL" };
        println!(
            "Depth {}: {} nodes (expected {}) in {} ms {}",
            depth,
            result,
            expected,
            elapsed.as_millis(),
            status
        );

        if result != expected {
            println!("ERROR: Perft test failed at depth {}", depth);
            println!("Running divide to debug...");
            perft_divide(&mut pos, depth);
            return;
        }
    }

    println!("\n✓ All perft tests passed!");
}

fn move_to_string(m: crate::types::Move) -> String {
    use crate::types::*;

    if !m.is_ok() {
        return "0000".to_string();
    }

    let from = m.from_sq();
    let to = m.to_sq();

    let from_str = format!(
        "{}{}",
        char::from_u32('a' as u32 + from.file().index() as u32).unwrap(),
        from.rank().index() + 1
    );

    let to_str = format!(
        "{}{}",
        char::from_u32('a' as u32 + to.file().index() as u32).unwrap(),
        to.rank().index() + 1
    );

    let mut result = format!("{}{}", from_str, to_str);

    if m.is_promotion() {
        if let Some(promo) = m.promotion_type() {
            let promo_char = match promo {
                PieceType::Queen => 'q',
                PieceType::Rook => 'r',
                PieceType::Bishop => 'b',
                PieceType::Knight => 'n',
                _ => 'q',
            };
            result.push(promo_char);
        }
    }

    result
}
