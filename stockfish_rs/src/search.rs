// Search engine with alpha-beta and iterative deepening

use crate::evaluate::evaluate;
use crate::movegen::*;
use crate::position::Position;
use crate::tt::*;
use crate::types::*;
use std::time::{Duration, Instant};

const MAX_DEPTH: Depth = 64;
const MATE_SCORE: Value = 30000;

pub struct SearchLimits {
    pub depth: Option<Depth>,
    pub nodes: Option<u64>,
    pub movetime: Option<Duration>,
    pub infinite: bool,
}

impl Default for SearchLimits {
    fn default() -> Self {
        SearchLimits {
            depth: Some(6),
            nodes: None,
            movetime: None,
            infinite: false,
        }
    }
}

pub struct SearchInfo {
    pub nodes: u64,
    pub start_time: Instant,
    pub limits: SearchLimits,
    pub stopped: bool,
}

impl SearchInfo {
    pub fn new(limits: SearchLimits) -> Self {
        SearchInfo {
            nodes: 0,
            start_time: Instant::now(),
            limits,
            stopped: false,
        }
    }

    pub fn should_stop(&mut self) -> bool {
        if self.stopped {
            return true;
        }

        // Check node limit
        if let Some(node_limit) = self.limits.nodes {
            if self.nodes >= node_limit {
                self.stopped = true;
                return true;
            }
        }

        // Check time limit
        if let Some(movetime) = self.limits.movetime {
            if self.start_time.elapsed() >= movetime {
                self.stopped = true;
                return true;
            }
        }

        false
    }
}

pub struct Search {
    pub tt: TranspositionTable,
}

impl Search {
    pub fn new(hash_mb: usize) -> Self {
        Search {
            tt: TranspositionTable::new(hash_mb),
        }
    }

    pub fn search(&mut self, pos: &mut Position, limits: SearchLimits) -> (Move, Value) {
        let mut info = SearchInfo::new(limits);
        let max_depth = info.limits.depth.unwrap_or(MAX_DEPTH);

        let mut best_move = Move::none();
        let mut best_score = -VALUE_INFINITE;

        self.tt.new_search();

        // Iterative deepening
        for depth in 1..=max_depth {
            if info.should_stop() {
                break;
            }

            let score = self.negamax(pos, depth, -VALUE_INFINITE, VALUE_INFINITE, &mut info);

            if info.stopped {
                break;
            }

            // Get best move from TT
            if let Some(tt_entry) = self.tt.probe(pos.key()) {
                let m = tt_entry.get_move();
                if m.is_ok() {
                    best_move = m;
                    best_score = score;
                }
            }

            // Print info
            let elapsed = info.start_time.elapsed();
            let nps = if elapsed.as_millis() > 0 {
                (info.nodes as u128 * 1000 / elapsed.as_millis()) as u64
            } else {
                0
            };

            println!(
                "info depth {} score cp {} nodes {} nps {} time {} pv {}",
                depth,
                score,
                info.nodes,
                nps,
                elapsed.as_millis(),
                move_to_uci(best_move)
            );
        }

        (best_move, best_score)
    }

    fn negamax(
        &mut self,
        pos: &mut Position,
        depth: Depth,
        mut alpha: Value,
        beta: Value,
        info: &mut SearchInfo,
    ) -> Value {
        if info.should_stop() {
            return 0;
        }

        info.nodes += 1;

        // Check for draw
        if pos.is_repetition() || pos.fifty_move_rule() {
            return VALUE_DRAW;
        }

        // Probe transposition table
        let tt_hit = self.tt.probe(pos.key());
        if let Some(entry) = tt_hit {
            if entry.depth() >= depth {
                let tt_value = entry.value();
                match entry.bound() {
                    Bound::Exact => return tt_value,
                    Bound::Lower if tt_value >= beta => return tt_value,
                    Bound::Upper if tt_value <= alpha => return tt_value,
                    _ => {}
                }
            }
        }

        // Quiescence search at leaf nodes
        if depth <= 0 {
            return self.quiescence(pos, alpha, beta, info);
        }

        // Generate moves
        let mut move_list = MoveList::new();
        generate_legal_moves(pos, &mut move_list);

        if move_list.is_empty() {
            // Checkmate or stalemate
            return if pos.in_check() {
                -MATE_SCORE + pos.ply() as Value
            } else {
                VALUE_DRAW
            };
        }

        // Order moves (TT move first)
        let tt_move = tt_hit.map(|e| e.get_move()).unwrap_or(Move::none());
        self.order_moves(&mut move_list, tt_move);

        let mut best_score = -VALUE_INFINITE;
        let mut best_move = Move::none();
        let mut bound = Bound::Upper;

        for i in 0..move_list.len() {
            let m = move_list.get(i);

            pos.do_move(m);
            let score = -self.negamax(pos, depth - 1, -beta, -alpha, info);
            pos.undo_move(m);

            if score > best_score {
                best_score = score;
                best_move = m;

                if score > alpha {
                    alpha = score;
                    bound = Bound::Exact;

                    if score >= beta {
                        bound = Bound::Lower;
                        break; // Beta cutoff
                    }
                }
            }
        }

        // Store in transposition table
        let eval = evaluate(pos);
        self.tt.store(pos.key(), best_score, eval, depth, best_move, bound);

        best_score
    }

    fn quiescence(
        &mut self,
        pos: &mut Position,
        mut alpha: Value,
        beta: Value,
        info: &mut SearchInfo,
    ) -> Value {
        if info.should_stop() {
            return 0;
        }

        info.nodes += 1;

        let stand_pat = evaluate(pos);

        if stand_pat >= beta {
            return beta;
        }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        // Generate captures only
        let mut move_list = MoveList::new();
        generate_captures(pos, &mut move_list);

        for i in 0..move_list.len() {
            let m = move_list.get(i);

            pos.do_move(m);
            let score = -self.quiescence(pos, -beta, -alpha, info);
            pos.undo_move(m);

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    fn order_moves(&self, _move_list: &mut MoveList, _tt_move: Move) {
        // Simple move ordering: TT move first, then MVV-LVA for captures
        // In a full implementation, we'd use killer moves, history heuristic, etc.

        // This is a placeholder - in a real implementation, we'd sort the moves
        // based on various heuristics
    }
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
