// UCI (Universal Chess Interface) protocol implementation

use crate::perft::{perft, perft_divide, perft_test};
use crate::position::Position;
use crate::search::{Search, SearchLimits};
use crate::types::*;
use std::io::{self, BufRead};
use std::time::Duration;

const ENGINE_NAME: &str = "Stockfish-RS";
const ENGINE_AUTHOR: &str = "Rust Port";
const ENGINE_VERSION: &str = "0.1.0";

pub struct UCI {
    search: Search,
    position: Position,
}

impl UCI {
    pub fn new() -> Self {
        UCI {
            search: Search::new(16), // 16 MB default hash
            position: Position::startpos(),
        }
    }

    pub fn main_loop(&mut self) {
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines();

        while let Some(Ok(line)) = lines.next() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "uci" => self.uci(),
                "isready" => self.isready(),
                "ucinewgame" => self.ucinewgame(),
                "position" => self.position(&parts[1..]),
                "go" => self.go(&parts[1..]),
                "quit" => break,
                "stop" => {} // TODO: Implement search stopping
                "perft" => self.perft(&parts[1..]),
                "perfttest" => perft_test(),
                "d" | "display" => self.display(),
                "setoption" => self.setoption(&parts[1..]),
                _ => println!("Unknown command: {}", parts[0]),
            }
        }
    }

    fn uci(&self) {
        println!("id name {} {}", ENGINE_NAME, ENGINE_VERSION);
        println!("id author {}", ENGINE_AUTHOR);
        println!("option name Hash type spin default 16 min 1 max 1024");
        println!("option name Threads type spin default 1 min 1 max 1");
        println!("uciok");
    }

    fn isready(&self) {
        println!("readyok");
    }

    fn ucinewgame(&mut self) {
        self.search.tt.clear();
        self.position = Position::startpos();
    }

    fn position(&mut self, args: &[&str]) {
        if args.is_empty() {
            return;
        }

        let mut idx = 0;

        // Parse position
        match args[0] {
            "startpos" => {
                self.position = Position::startpos();
                idx = 1;
            }
            "fen" => {
                // Find "moves" keyword or end of FEN
                let mut fen_end = args.len();
                for (i, &arg) in args.iter().enumerate().skip(1) {
                    if arg == "moves" {
                        fen_end = i;
                        break;
                    }
                }

                let fen = args[1..fen_end].join(" ");
                match Position::from_fen(&fen) {
                    Ok(pos) => {
                        self.position = pos;
                        idx = fen_end;
                    }
                    Err(e) => {
                        println!("Error parsing FEN: {}", e);
                        return;
                    }
                }
            }
            _ => return,
        }

        // Parse moves
        if idx < args.len() && args[idx] == "moves" {
            for &move_str in &args[idx + 1..] {
                match parse_uci_move(&self.position, move_str) {
                    Ok(m) => {
                        self.position.do_move(m);
                    }
                    Err(e) => {
                        println!("Error parsing move {}: {}", move_str, e);
                        return;
                    }
                }
            }
        }
    }

    fn go(&mut self, args: &[&str]) {
        let mut limits = SearchLimits::default();

        let mut i = 0;
        while i < args.len() {
            match args[i] {
                "depth" => {
                    if i + 1 < args.len() {
                        if let Ok(d) = args[i + 1].parse::<i32>() {
                            limits.depth = Some(d);
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "nodes" => {
                    if i + 1 < args.len() {
                        if let Ok(n) = args[i + 1].parse::<u64>() {
                            limits.nodes = Some(n);
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "movetime" => {
                    if i + 1 < args.len() {
                        if let Ok(ms) = args[i + 1].parse::<u64>() {
                            limits.movetime = Some(Duration::from_millis(ms));
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "infinite" => {
                    limits.infinite = true;
                    i += 1;
                }
                "perft" => {
                    if i + 1 < args.len() {
                        if let Ok(d) = args[i + 1].parse::<u32>() {
                            let nodes = perft(&mut self.position, d);
                            println!("Nodes: {}", nodes);
                            return;
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }

        let (best_move, _score) = self.search.search(&mut self.position, limits);
        println!("bestmove {}", move_to_uci(best_move));
    }

    fn perft(&mut self, args: &[&str]) {
        if args.is_empty() {
            return;
        }

        if let Ok(depth) = args[0].parse::<u32>() {
            perft_divide(&mut self.position, depth);
        }
    }

    fn display(&self) {
        println!("{}", self.position.to_fen());
        // TODO: Display the board visually
    }

    fn setoption(&mut self, args: &[&str]) {
        // Parse setoption name <name> value <value>
        let mut name_start = None;
        let mut value_start = None;

        for (i, &arg) in args.iter().enumerate() {
            if arg == "name" {
                name_start = Some(i + 1);
            } else if arg == "value" {
                value_start = Some(i + 1);
                break;
            }
        }

        if let (Some(ns), Some(vs)) = (name_start, value_start) {
            let name = args[ns..vs - 1].join(" ").to_lowercase();
            let value = args[vs..].join(" ");

            match name.as_str() {
                "hash" => {
                    if let Ok(mb) = value.parse::<usize>() {
                        self.search.tt.resize(mb);
                    }
                }
                _ => {}
            }
        }
    }
}

fn parse_uci_move(pos: &Position, move_str: &str) -> Result<Move, String> {
    if move_str.len() < 4 {
        return Err("Move string too short".to_string());
    }

    let from_file = move_str.chars().nth(0).unwrap() as usize - 'a' as usize;
    let from_rank = move_str.chars().nth(1).unwrap() as usize - '1' as usize;
    let to_file = move_str.chars().nth(2).unwrap() as usize - 'a' as usize;
    let to_rank = move_str.chars().nth(3).unwrap() as usize - '1' as usize;

    if from_file > 7 || from_rank > 7 || to_file > 7 || to_rank > 7 {
        return Err("Invalid square".to_string());
    }

    let from = Square::make(File::from_index(from_file), Rank::from_index(from_rank));
    let to = Square::make(File::from_index(to_file), Rank::from_index(to_rank));

    // Check for promotion
    let promo = if move_str.len() >= 5 {
        match move_str.chars().nth(4).unwrap() {
            'q' => Some(PieceType::Queen),
            'r' => Some(PieceType::Rook),
            'b' => Some(PieceType::Bishop),
            'n' => Some(PieceType::Knight),
            _ => None,
        }
    } else {
        None
    };

    // Generate legal moves and find matching move
    use crate::movegen::*;
    let mut move_list = MoveList::new();
    generate_legal_moves(pos, &mut move_list);

    for i in 0..move_list.len() {
        let m = move_list.get(i);
        if m.from_sq() == from && m.to_sq() == to {
            if let Some(pt) = promo {
                if m.is_promotion() && m.promotion_type() == Some(pt) {
                    return Ok(m);
                }
            } else if !m.is_promotion() {
                return Ok(m);
            }
        }
    }

    Err(format!("Move {} not legal in current position", move_str))
}

fn move_to_uci(m: Move) -> String {
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
