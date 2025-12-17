// Chess position representation with board state and move making/unmaking

use crate::bitboard::*;
use crate::types::*;
use crate::zobrist;

#[derive(Clone)]
pub struct StateInfo {
    pub key: Key,
    pub checkers_bb: Bitboard,
    pub castling_rights: u8,
    pub ep_square: Option<Square>,
    pub rule50: u32,
    pub ply: u32,
    pub captured_piece: Option<Piece>,
    pub blockers_for_king: [Bitboard; 2],
    pub pinners: [Bitboard; 2],
    pub check_squares: [Bitboard; 7],
}

impl StateInfo {
    fn new() -> Self {
        StateInfo {
            key: 0,
            checkers_bb: 0,
            castling_rights: 0,
            ep_square: None,
            rule50: 0,
            ply: 0,
            captured_piece: None,
            blockers_for_king: [0; 2],
            pinners: [0; 2],
            check_squares: [0; 7],
        }
    }
}

pub struct Position {
    board: [Option<Piece>; 64],
    by_type: [Bitboard; 7],
    by_color: [Bitboard; 2],
    piece_count: [u8; 16],
    side_to_move: Color,
    state: StateInfo,
    state_history: Vec<StateInfo>,
    castling_rook_sq: [[Option<Square>; 2]; 2],
    castling_path: [[Bitboard; 2]; 2],
}

impl Position {
    /// Create an empty position
    pub fn new() -> Self {
        Position {
            board: [None; 64],
            by_type: [0; 7],
            by_color: [0; 2],
            piece_count: [0; 16],
            side_to_move: Color::White,
            state: StateInfo::new(),
            state_history: Vec::new(),
            castling_rook_sq: [[None; 2]; 2],
            castling_path: [[0; 2]; 2],
        }
    }

    /// Create position from starting position
    pub fn startpos() -> Self {
        Position::from_fen(START_FEN).expect("Starting position FEN should be valid")
    }

    /// Create position from FEN string
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut pos = Position::new();
        let parts: Vec<&str> = fen.split_whitespace().collect();

        if parts.is_empty() {
            return Err("Empty FEN string".to_string());
        }

        // Parse board position
        let ranks: Vec<&str> = parts[0].split('/').collect();
        if ranks.len() != 8 {
            return Err("FEN must have 8 ranks".to_string());
        }

        for (rank_idx, rank_str) in ranks.iter().enumerate() {
            let rank = Rank::from_index(7 - rank_idx);
            let mut file = 0;

            for c in rank_str.chars() {
                if file >= 8 {
                    return Err(format!("Rank {} has too many files", rank_idx + 1));
                }

                if c.is_ascii_digit() {
                    let skip = c.to_digit(10).unwrap() as usize;
                    file += skip;
                } else {
                    let piece = match c {
                        'P' => Piece::make(Color::White, PieceType::Pawn),
                        'N' => Piece::make(Color::White, PieceType::Knight),
                        'B' => Piece::make(Color::White, PieceType::Bishop),
                        'R' => Piece::make(Color::White, PieceType::Rook),
                        'Q' => Piece::make(Color::White, PieceType::Queen),
                        'K' => Piece::make(Color::White, PieceType::King),
                        'p' => Piece::make(Color::Black, PieceType::Pawn),
                        'n' => Piece::make(Color::Black, PieceType::Knight),
                        'b' => Piece::make(Color::Black, PieceType::Bishop),
                        'r' => Piece::make(Color::Black, PieceType::Rook),
                        'q' => Piece::make(Color::Black, PieceType::Queen),
                        'k' => Piece::make(Color::Black, PieceType::King),
                        _ => return Err(format!("Invalid piece character: {}", c)),
                    };

                    let sq = Square::make(File::from_index(file), rank);
                    pos.put_piece(piece, sq);
                    file += 1;
                }
            }

            if file != 8 {
                return Err(format!("Rank {} has {} files instead of 8", rank_idx + 1, file));
            }
        }

        // Parse side to move
        if parts.len() > 1 {
            pos.side_to_move = match parts[1] {
                "w" => Color::White,
                "b" => Color::Black,
                _ => return Err(format!("Invalid side to move: {}", parts[1])),
            };
        }

        // Parse castling rights
        if parts.len() > 2 {
            for c in parts[2].chars() {
                match c {
                    'K' => {
                        pos.state.castling_rights |= CastlingRights::WhiteOO as u8;
                        // Find the rook on h1
                        if let Some(p) = pos.piece_on(Square::H1) {
                            if p.piece_type() == PieceType::Rook && p.color() == Color::White {
                                pos.castling_rook_sq[Color::White.index()][0] = Some(Square::H1);
                            }
                        }
                    }
                    'Q' => {
                        pos.state.castling_rights |= CastlingRights::WhiteOOO as u8;
                        // Find the rook on a1
                        if let Some(p) = pos.piece_on(Square::A1) {
                            if p.piece_type() == PieceType::Rook && p.color() == Color::White {
                                pos.castling_rook_sq[Color::White.index()][1] = Some(Square::A1);
                            }
                        }
                    }
                    'k' => {
                        pos.state.castling_rights |= CastlingRights::BlackOO as u8;
                        // Find the rook on h8
                        if let Some(p) = pos.piece_on(Square::H8) {
                            if p.piece_type() == PieceType::Rook && p.color() == Color::Black {
                                pos.castling_rook_sq[Color::Black.index()][0] = Some(Square::H8);
                            }
                        }
                    }
                    'q' => {
                        pos.state.castling_rights |= CastlingRights::BlackOOO as u8;
                        // Find the rook on a8
                        if let Some(p) = pos.piece_on(Square::A8) {
                            if p.piece_type() == PieceType::Rook && p.color() == Color::Black {
                                pos.castling_rook_sq[Color::Black.index()][1] = Some(Square::A8);
                            }
                        }
                    }
                    '-' => {}
                    _ => return Err(format!("Invalid castling character: {}", c)),
                }
            }
        }

        // Calculate castling paths
        pos.update_castling_paths();

        // Parse en passant square
        if parts.len() > 3 && parts[3] != "-" {
            let ep_str = parts[3];
            if ep_str.len() != 2 {
                return Err(format!("Invalid en passant square: {}", ep_str));
            }
            let file = match ep_str.chars().next().unwrap() {
                'a' => File::A,
                'b' => File::B,
                'c' => File::C,
                'd' => File::D,
                'e' => File::E,
                'f' => File::F,
                'g' => File::G,
                'h' => File::H,
                _ => return Err(format!("Invalid en passant file: {}", ep_str)),
            };
            let rank = match ep_str.chars().nth(1).unwrap() {
                '1' => Rank::R1,
                '2' => Rank::R2,
                '3' => Rank::R3,
                '4' => Rank::R4,
                '5' => Rank::R5,
                '6' => Rank::R6,
                '7' => Rank::R7,
                '8' => Rank::R8,
                _ => return Err(format!("Invalid en passant rank: {}", ep_str)),
            };
            pos.state.ep_square = Some(Square::make(file, rank));
        }

        // Parse halfmove clock
        if parts.len() > 4 {
            pos.state.rule50 = parts[4].parse().unwrap_or(0);
        }

        // Parse fullmove number
        if parts.len() > 5 {
            let fullmove: u32 = parts[5].parse().unwrap_or(1);
            pos.state.ply = 2 * (fullmove - 1) + if pos.side_to_move == Color::Black { 1 } else { 0 };
        }

        // Compute derived state
        pos.state.key = pos.compute_key();
        pos.set_check_info();

        Ok(pos)
    }

    /// Generate FEN string from position
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // Board position
        for rank_idx in (0..8).rev() {
            let rank = Rank::from_index(rank_idx);
            let mut empty = 0;

            for file_idx in 0..8 {
                let file = File::from_index(file_idx);
                let sq = Square::make(file, rank);

                if let Some(piece) = self.piece_on(sq) {
                    if empty > 0 {
                        fen.push_str(&empty.to_string());
                        empty = 0;
                    }
                    let c = match (piece.color(), piece.piece_type()) {
                        (Color::White, PieceType::Pawn) => 'P',
                        (Color::White, PieceType::Knight) => 'N',
                        (Color::White, PieceType::Bishop) => 'B',
                        (Color::White, PieceType::Rook) => 'R',
                        (Color::White, PieceType::Queen) => 'Q',
                        (Color::White, PieceType::King) => 'K',
                        (Color::Black, PieceType::Pawn) => 'p',
                        (Color::Black, PieceType::Knight) => 'n',
                        (Color::Black, PieceType::Bishop) => 'b',
                        (Color::Black, PieceType::Rook) => 'r',
                        (Color::Black, PieceType::Queen) => 'q',
                        (Color::Black, PieceType::King) => 'k',
                    };
                    fen.push(c);
                } else {
                    empty += 1;
                }
            }

            if empty > 0 {
                fen.push_str(&empty.to_string());
            }

            if rank_idx > 0 {
                fen.push('/');
            }
        }

        // Side to move
        fen.push(' ');
        fen.push(if self.side_to_move == Color::White { 'w' } else { 'b' });

        // Castling rights
        fen.push(' ');
        let mut castling = String::new();
        if CastlingRights::can_castle_kingside(self.state.castling_rights, Color::White) {
            castling.push('K');
        }
        if CastlingRights::can_castle_queenside(self.state.castling_rights, Color::White) {
            castling.push('Q');
        }
        if CastlingRights::can_castle_kingside(self.state.castling_rights, Color::Black) {
            castling.push('k');
        }
        if CastlingRights::can_castle_queenside(self.state.castling_rights, Color::Black) {
            castling.push('q');
        }
        fen.push_str(if castling.is_empty() { "-" } else { &castling });

        // En passant square
        fen.push(' ');
        if let Some(ep_sq) = self.state.ep_square {
            let file_char = (b'a' + ep_sq.file().index() as u8) as char;
            let rank_char = (b'1' + ep_sq.rank().index() as u8) as char;
            fen.push(file_char);
            fen.push(rank_char);
        } else {
            fen.push('-');
        }

        // Halfmove clock
        fen.push(' ');
        fen.push_str(&self.state.rule50.to_string());

        // Fullmove number
        fen.push(' ');
        fen.push_str(&(self.state.ply / 2 + 1).to_string());

        fen
    }

    /// Get side to move
    #[inline(always)]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// Get all pieces bitboard
    #[inline(always)]
    pub fn pieces(&self) -> Bitboard {
        self.by_color[Color::White.index()] | self.by_color[Color::Black.index()]
    }

    /// Get pieces for a specific color
    #[inline(always)]
    pub fn pieces_c(&self, color: Color) -> Bitboard {
        self.by_color[color.index()]
    }

    /// Get specific piece type for a color
    #[inline(always)]
    pub fn pieces_cp(&self, color: Color, pt: PieceType) -> Bitboard {
        self.by_type[pt.index()] & self.by_color[color.index()]
    }

    /// Get pieces of a specific type (both colors)
    #[inline(always)]
    pub fn pieces_p(&self, pt: PieceType) -> Bitboard {
        self.by_type[pt.index()]
    }

    /// Get piece on square
    #[inline(always)]
    pub fn piece_on(&self, sq: Square) -> Option<Piece> {
        self.board[sq.index()]
    }

    /// Get king square for a color
    #[inline(always)]
    pub fn king_square(&self, color: Color) -> Square {
        lsb(self.pieces_cp(color, PieceType::King))
    }

    /// Get checkers bitboard
    #[inline(always)]
    pub fn checkers(&self) -> Bitboard {
        self.state.checkers_bb
    }

    /// Get en passant square
    #[inline(always)]
    pub fn ep_square(&self) -> Option<Square> {
        self.state.ep_square
    }

    /// Check if color can castle
    #[inline(always)]
    pub fn can_castle(&self, color: Color) -> bool {
        let mask = if color == Color::White { 3 } else { 12 };
        (self.state.castling_rights & mask) != 0
    }

    /// Check if color can castle kingside
    #[inline(always)]
    pub fn can_castle_kingside(&self, color: Color) -> bool {
        CastlingRights::can_castle_kingside(self.state.castling_rights, color)
    }

    /// Check if color can castle queenside
    #[inline(always)]
    pub fn can_castle_queenside(&self, color: Color) -> bool {
        CastlingRights::can_castle_queenside(self.state.castling_rights, color)
    }

    /// Get castling rook square
    #[inline(always)]
    pub fn castling_rook_square(&self, color: Color, kingside: bool) -> Square {
        self.castling_rook_sq[color.index()][if kingside { 0 } else { 1 }].unwrap()
    }

    /// Check if castling path is blocked
    #[inline(always)]
    pub fn castling_impeded(&self, color: Color, kingside: bool) -> bool {
        let path = self.castling_path[color.index()][if kingside { 0 } else { 1 }];
        (self.pieces() & path) != 0
    }

    /// Check if current side is in check
    #[inline(always)]
    pub fn in_check(&self) -> bool {
        self.state.checkers_bb != 0
    }

    /// Place piece on board
    fn put_piece(&mut self, piece: Piece, sq: Square) {
        self.board[sq.index()] = Some(piece);
        self.by_type[piece.piece_type().index()] |= square_bb(sq);
        self.by_color[piece.color().index()] |= square_bb(sq);
        self.piece_count[piece.index()] += 1;
    }

    /// Remove piece from board
    fn remove_piece(&mut self, sq: Square) -> Option<Piece> {
        let piece = self.board[sq.index()];
        if let Some(p) = piece {
            self.board[sq.index()] = None;
            self.by_type[p.piece_type().index()] ^= square_bb(sq);
            self.by_color[p.color().index()] ^= square_bb(sq);
            self.piece_count[p.index()] -= 1;
        }
        piece
    }

    /// Move piece from one square to another
    fn move_piece(&mut self, from: Square, to: Square) {
        let piece = self.board[from.index()].unwrap();
        let sq_bb = square_bb(from) | square_bb(to);
        self.board[from.index()] = None;
        self.board[to.index()] = Some(piece);
        self.by_type[piece.piece_type().index()] ^= sq_bb;
        self.by_color[piece.color().index()] ^= sq_bb;
    }

    /// Compute zobrist hash for current position
    fn compute_key(&self) -> Key {
        let mut key = 0u64;

        for sq_idx in 0..64 {
            if let Some(piece) = self.board[sq_idx] {
                key ^= zobrist::piece_key(piece, Square::from_index(sq_idx));
            }
        }

        if let Some(ep_sq) = self.state.ep_square {
            key ^= zobrist::enpassant_key(ep_sq.file());
        }

        key ^= zobrist::castling_key(self.state.castling_rights);

        if self.side_to_move == Color::Black {
            key ^= zobrist::side_key();
        }

        key
    }

    /// Get all attackers to a square
    pub fn attackers_to(&self, sq: Square, occupied: Bitboard) -> Bitboard {
        (pawn_attacks_bb_sq(Color::Black, sq) & self.pieces_cp(Color::White, PieceType::Pawn))
            | (pawn_attacks_bb_sq(Color::White, sq) & self.pieces_cp(Color::Black, PieceType::Pawn))
            | (knight_attacks_bb(sq) & self.pieces_p(PieceType::Knight))
            | (king_attacks_bb(sq) & self.pieces_p(PieceType::King))
            | (bishop_attacks_bb(sq, occupied) & (self.pieces_p(PieceType::Bishop) | self.pieces_p(PieceType::Queen)))
            | (rook_attacks_bb(sq, occupied) & (self.pieces_p(PieceType::Rook) | self.pieces_p(PieceType::Queen)))
    }

    /// Set check and pin information
    fn set_check_info(&mut self) {
        let us = self.side_to_move;
        let them = us.flip();
        let ksq = self.king_square(us);

        self.state.checkers_bb = self.attackers_to(ksq, self.pieces()) & self.pieces_c(them);

        // Update blockers and pinners
        self.update_slider_blockers(Color::White);
        self.update_slider_blockers(Color::Black);

        // Update check squares
        let occupied = self.pieces();
        self.state.check_squares[PieceType::Pawn.index()] = pawn_attacks_bb_sq(us, self.king_square(them));
        self.state.check_squares[PieceType::Knight.index()] = knight_attacks_bb(self.king_square(them));
        self.state.check_squares[PieceType::Bishop.index()] = bishop_attacks_bb(self.king_square(them), occupied);
        self.state.check_squares[PieceType::Rook.index()] = rook_attacks_bb(self.king_square(them), occupied);
        self.state.check_squares[PieceType::Queen.index()] =
            self.state.check_squares[PieceType::Bishop.index()] | self.state.check_squares[PieceType::Rook.index()];
        self.state.check_squares[PieceType::King.index()] = 0;
    }

    /// Update slider blockers and pinners
    fn update_slider_blockers(&mut self, color: Color) {
        let ksq = self.king_square(color);
        let snipers = ((self.pieces_p(PieceType::Rook) | self.pieces_p(PieceType::Queen)) & rook_attacks_bb(ksq, 0))
                    | ((self.pieces_p(PieceType::Bishop) | self.pieces_p(PieceType::Queen)) & bishop_attacks_bb(ksq, 0));

        let mut blockers = 0u64;
        let mut pinners = 0u64;

        let mut snipers_for_color = snipers & self.pieces_c(color.flip());
        while snipers_for_color != 0 {
            let sniper_sq = pop_lsb(&mut snipers_for_color);
            let between = between_bb(ksq, sniper_sq) & self.pieces();

            if between != 0 && !more_than_one(between) {
                blockers |= between;
                if (between & self.pieces_c(color)) != 0 {
                    pinners |= square_bb(sniper_sq);
                }
            }
        }

        self.state.blockers_for_king[color.index()] = blockers;
        self.state.pinners[color.index()] = pinners;
    }

    /// Update castling paths based on current rook positions
    fn update_castling_paths(&mut self) {
        for &color in &[Color::White, Color::Black] {
            let ksq = self.king_square(color);

            // Kingside
            if let Some(rook_sq) = self.castling_rook_sq[color.index()][0] {
                let (king_to, rook_to) = if color == Color::White {
                    (Square::G1, Square::F1)
                } else {
                    (Square::G8, Square::F8)
                };

                let mut path = 0u64;
                let mut sq = std::cmp::min(ksq.index(), king_to.index());
                let end = std::cmp::max(rook_sq.index(), rook_to.index());
                while sq <= end {
                    if sq != ksq.index() && sq != rook_sq.index() {
                        path |= square_bb(Square::from_index(sq));
                    }
                    sq += 1;
                }
                self.castling_path[color.index()][0] = path;
            }

            // Queenside
            if let Some(rook_sq) = self.castling_rook_sq[color.index()][1] {
                let (king_to, rook_to) = if color == Color::White {
                    (Square::C1, Square::D1)
                } else {
                    (Square::C8, Square::D8)
                };

                let mut path = 0u64;
                let mut sq = std::cmp::min(rook_sq.index(), rook_to.index());
                let end = std::cmp::max(ksq.index(), king_to.index());
                while sq <= end {
                    if sq != ksq.index() && sq != rook_sq.index() {
                        path |= square_bb(Square::from_index(sq));
                    }
                    sq += 1;
                }
                self.castling_path[color.index()][1] = path;
            }
        }
    }

    /// Check if move gives check
    pub fn gives_check(&self, m: Move) -> bool {
        let us = self.side_to_move;
        let from = m.from_sq();
        let to = m.to_sq();

        // Direct check
        let piece = self.piece_on(from).unwrap();
        if (self.state.check_squares[piece.piece_type().index()] & square_bb(to)) != 0 {
            return true;
        }

        // Discovered check
        if (self.state.blockers_for_king[us.flip().index()] & square_bb(from)) != 0
            && !line_bb(from, to).contains(&self.king_square(us.flip()))
        {
            return true;
        }

        match piece.piece_type() {
            PieceType::Pawn => {
                if m.is_en_passant() {
                    // En passant can give discovered check
                    let captured_sq = Square::from_index((to.index() as i8 + if us == Color::White { -8 } else { 8 }) as usize);
                    let occupied = (self.pieces() ^ square_bb(from) ^ square_bb(captured_sq)) | square_bb(to);
                    let ksq = self.king_square(us.flip());
                    return ((rook_attacks_bb(ksq, occupied) & (self.pieces_cp(us, PieceType::Rook) | self.pieces_cp(us, PieceType::Queen)))
                         | (bishop_attacks_bb(ksq, occupied) & (self.pieces_cp(us, PieceType::Bishop) | self.pieces_cp(us, PieceType::Queen)))) != 0;
                }
                if m.is_promotion() {
                    // Promotion can give check
                    let promo_type = m.promotion_type().unwrap();
                    let occupied = (self.pieces() ^ square_bb(from)) | square_bb(to);
                    return (attacks_bb(promo_type, to, occupied) & square_bb(self.king_square(us.flip()))) != 0;
                }
            }
            PieceType::King => {
                if m.is_castle() {
                    // Castling can give check via the rook
                    let (rook_from, rook_to) = if to.file().index() > from.file().index() {
                        // Kingside
                        (to, Square::from_index(to.index() - 1))
                    } else {
                        // Queenside
                        (to, Square::from_index(to.index() + 1))
                    };
                    let occupied = (self.pieces() ^ square_bb(from) ^ square_bb(rook_from)) | square_bb(to) | square_bb(rook_to);
                    return (rook_attacks_bb(rook_to, occupied) & square_bb(self.king_square(us.flip()))) != 0;
                }
            }
            _ => {}
        }

        false
    }

    /// Check if move is legal
    pub fn is_legal(&self, m: Move) -> bool {
        let us = self.side_to_move;
        let from = m.from_sq();
        let to = m.to_sq();

        // Handle en passant legality
        if m.is_en_passant() {
            let ksq = self.king_square(us);
            let captured_sq = Square::from_index((to.index() as i8 + if us == Color::White { -8 } else { 8 }) as usize);
            let occupied = (self.pieces() ^ square_bb(from) ^ square_bb(captured_sq)) | square_bb(to);

            return ((rook_attacks_bb(ksq, occupied) & (self.pieces_cp(us.flip(), PieceType::Rook) | self.pieces_cp(us.flip(), PieceType::Queen)))
                 | (bishop_attacks_bb(ksq, occupied) & (self.pieces_cp(us.flip(), PieceType::Bishop) | self.pieces_cp(us.flip(), PieceType::Queen)))) == 0;
        }

        // Handle castling legality
        if m.is_castle() {
            // Check if squares between king and destination are attacked
            let step = if to.index() > from.index() { 1 } else { -1 };
            let mut sq = from.index() as i8;
            let end = to.index() as i8;

            while sq != end {
                let test_sq = Square::from_index(sq as usize);
                if (self.attackers_to(test_sq, self.pieces()) & self.pieces_c(us.flip())) != 0 {
                    return false;
                }
                sq += step;
            }
            // Check destination square
            if (self.attackers_to(Square::from_index(end as usize), self.pieces()) & self.pieces_c(us.flip())) != 0 {
                return false;
            }
            return true;
        }

        // King moves must not go to attacked square
        if self.piece_on(from).unwrap().piece_type() == PieceType::King {
            let occupied = self.pieces() ^ square_bb(from);
            return (self.attackers_to(to, occupied) & self.pieces_c(us.flip())) == 0;
        }

        // Pinned piece must move along pin ray
        if (self.state.blockers_for_king[us.index()] & square_bb(from)) != 0 {
            return line_bb(from, self.king_square(us)).contains(&to);
        }

        true
    }

    /// Make a move
    pub fn do_move(&mut self, m: Move) {
        // Save current state
        self.state_history.push(self.state.clone());

        let us = self.side_to_move;
        let them = us.flip();
        let from = m.from_sq();
        let to = m.to_sq();
        let piece = self.piece_on(from).unwrap();
        let captured = self.piece_on(to);

        // Update hash
        let mut new_key = self.state.key;

        // Remove old en passant
        if let Some(ep_sq) = self.state.ep_square {
            new_key ^= zobrist::enpassant_key(ep_sq.file());
        }

        // Update castling rights
        let old_castling = self.state.castling_rights;
        let mut new_castling = old_castling;

        // Remove castling rights if king or rook moves
        if piece.piece_type() == PieceType::King {
            new_castling &= if us == Color::White { 0b1100 } else { 0b0011 };
        } else if piece.piece_type() == PieceType::Rook {
            if Some(from) == self.castling_rook_sq[us.index()][0] {
                new_castling &= if us == Color::White { 0b1110 } else { 0b1011 };
            } else if Some(from) == self.castling_rook_sq[us.index()][1] {
                new_castling &= if us == Color::White { 0b1101 } else { 0b0111 };
            }
        }

        // Remove castling rights if rook is captured
        if let Some(cap) = captured {
            if cap.piece_type() == PieceType::Rook {
                if Some(to) == self.castling_rook_sq[them.index()][0] {
                    new_castling &= if them == Color::White { 0b1110 } else { 0b1011 };
                } else if Some(to) == self.castling_rook_sq[them.index()][1] {
                    new_castling &= if them == Color::White { 0b1101 } else { 0b0111 };
                }
            }
        }

        if old_castling != new_castling {
            new_key ^= zobrist::castling_key(old_castling);
            new_key ^= zobrist::castling_key(new_castling);
            self.state.castling_rights = new_castling;
        }

        // Handle special moves
        if m.is_castle() {
            // Castling
            let (rook_from, rook_to) = if to.file().index() > from.file().index() {
                // Kingside
                (to, Square::from_index(to.index() - 1))
            } else {
                // Queenside
                (to, Square::from_index(to.index() + 1))
            };

            let king_to = if to.file().index() > from.file().index() {
                Square::from_index(from.index() + 2)
            } else {
                Square::from_index(from.index() - 2)
            };

            // Remove pieces
            new_key ^= zobrist::piece_key(piece, from);
            new_key ^= zobrist::piece_key(Piece::make(us, PieceType::Rook), rook_from);
            self.remove_piece(from);
            self.remove_piece(rook_from);

            // Place pieces
            new_key ^= zobrist::piece_key(piece, king_to);
            new_key ^= zobrist::piece_key(Piece::make(us, PieceType::Rook), rook_to);
            self.put_piece(piece, king_to);
            self.put_piece(Piece::make(us, PieceType::Rook), rook_to);

            self.state.captured_piece = None;
        } else if m.is_en_passant() {
            // En passant
            let captured_sq = Square::from_index((to.index() as i8 + if us == Color::White { -8 } else { 8 }) as usize);
            let captured_pawn = self.remove_piece(captured_sq).unwrap();

            new_key ^= zobrist::piece_key(captured_pawn, captured_sq);
            new_key ^= zobrist::piece_key(piece, from);
            new_key ^= zobrist::piece_key(piece, to);

            self.move_piece(from, to);
            self.state.captured_piece = Some(captured_pawn);
        } else if m.is_promotion() {
            // Promotion
            let promo_piece = Piece::make(us, m.promotion_type().unwrap());

            new_key ^= zobrist::piece_key(piece, from);
            self.remove_piece(from);

            if let Some(cap) = captured {
                new_key ^= zobrist::piece_key(cap, to);
                self.remove_piece(to);
            }

            new_key ^= zobrist::piece_key(promo_piece, to);
            self.put_piece(promo_piece, to);

            self.state.captured_piece = captured;
        } else {
            // Normal move
            if let Some(cap) = captured {
                new_key ^= zobrist::piece_key(cap, to);
                self.remove_piece(to);
            }

            new_key ^= zobrist::piece_key(piece, from);
            new_key ^= zobrist::piece_key(piece, to);
            self.move_piece(from, to);

            self.state.captured_piece = captured;
        }

        // Update en passant square
        self.state.ep_square = None;
        if piece.piece_type() == PieceType::Pawn {
            let delta = (to.index() as i8 - from.index() as i8).abs();
            if delta == 16 {
                let ep_sq = Square::from_index(((from.index() + to.index()) / 2) as usize);
                self.state.ep_square = Some(ep_sq);
                new_key ^= zobrist::enpassant_key(ep_sq.file());
            }
        }

        // Update rule50
        if piece.piece_type() == PieceType::Pawn || captured.is_some() {
            self.state.rule50 = 0;
        } else {
            self.state.rule50 += 1;
        }

        // Update ply
        self.state.ply += 1;

        // Switch side to move
        new_key ^= zobrist::side_key();
        self.state.key = new_key;
        self.side_to_move = them;

        // Update check info
        self.set_check_info();
    }

    /// Undo a move
    pub fn undo_move(&mut self, m: Move) {
        let from = m.from_sq();
        let to = m.to_sq();

        // Switch side to move back (it was flipped by do_move)
        self.side_to_move = self.side_to_move.flip();
        let us = self.side_to_move;

        // IMPORTANT: Save captured piece BEFORE restoring old state
        // because do_move() stored it in the current state
        let captured = self.state.captured_piece;

        // Restore previous state from history (this was saved by do_move before making changes)
        self.state = self.state_history.pop().unwrap();

        // Undo the move on the board
        if m.is_castle() {
            // Castling: undo king and rook movements
            // Move is encoded as king_from -> rook_square
            let (rook_from, rook_to) = if to.file().index() > from.file().index() {
                // Kingside castling
                (to, Square::from_index(to.index() - 1))
            } else {
                // Queenside castling
                (to, Square::from_index(to.index() + 1))
            };

            let king_to = if to.file().index() > from.file().index() {
                Square::from_index(from.index() + 2)  // Kingside: king moved 2 squares right
            } else {
                Square::from_index(from.index() - 2)  // Queenside: king moved 2 squares left
            };

            // Remove pieces from their current positions
            self.remove_piece(king_to);
            self.remove_piece(rook_to);

            // Put them back to original positions
            self.put_piece(Piece::make(us, PieceType::King), from);
            self.put_piece(Piece::make(us, PieceType::Rook), rook_from);
        } else if m.is_en_passant() {
            // En passant: move pawn back and restore captured pawn
            // The capturing pawn is at 'to', needs to go back to 'from'
            self.move_piece(to, from);

            // The captured pawn was on a different square
            let captured_sq = Square::from_index(
                (to.index() as i8 + if us == Color::White { -8 } else { 8 }) as usize
            );
            self.put_piece(captured.unwrap(), captured_sq);
        } else if m.is_promotion() {
            // Promotion: remove promoted piece, restore pawn, restore captured piece if any
            // The promoted piece is at 'to'
            self.remove_piece(to);

            // Put the original pawn back at 'from'
            self.put_piece(Piece::make(us, PieceType::Pawn), from);

            // Restore captured piece if there was one (promotion with capture)
            if let Some(cap) = captured {
                self.put_piece(cap, to);
            }
        } else {
            // Normal move or capture: move piece back from 'to' to 'from'
            // After do_move, the piece is at 'to', we need to move it back to 'from'
            self.move_piece(to, from);

            // Restore captured piece if there was one
            if let Some(cap) = captured {
                self.put_piece(cap, to);
            }
        }
    }

    /// Get position key
    #[inline(always)]
    pub fn key(&self) -> Key {
        self.state.key
    }

    /// Get rule50 counter
    #[inline(always)]
    pub fn rule50(&self) -> u32 {
        self.state.rule50
    }

    /// Get ply
    #[inline(always)]
    pub fn ply(&self) -> u32 {
        self.state.ply
    }

    /// Check for three-fold repetition
    pub fn is_repetition(&self) -> bool {
        let current_key = self.state.key;
        let mut count = 1;

        // Look back through state history for matching keys
        // We only need to check positions where the 50-move counter allows reversibility
        let max_lookback = std::cmp::min(self.state.rule50 as usize, self.state_history.len());

        for i in (0..max_lookback).step_by(2) {
            let idx = self.state_history.len() - 1 - i;
            if self.state_history[idx].key == current_key {
                count += 1;
                if count >= 3 {
                    return true;
                }
            }
        }

        false
    }

    /// Check if fifty move rule applies (draw)
    pub fn fifty_move_rule(&self) -> bool {
        self.state.rule50 >= 100
    }

    /// Get piece count
    #[inline(always)]
    pub fn count(&self, piece: Piece) -> u8 {
        self.piece_count[piece.index()]
    }

    /// Get blockers for king
    #[inline(always)]
    pub fn blockers_for_king(&self, color: Color) -> Bitboard {
        self.state.blockers_for_king[color.index()]
    }

    /// Get pinners
    #[inline(always)]
    pub fn pinners(&self, color: Color) -> Bitboard {
        self.state.pinners[color.index()]
    }

    /// Get check squares for a piece type
    #[inline(always)]
    pub fn check_squares(&self, pt: PieceType) -> Bitboard {
        self.state.check_squares[pt.index()]
    }
}

impl Default for Position {
    fn default() -> Self {
        Position::new()
    }
}

// Helper trait for bitboard contains check
trait BitboardContains {
    fn contains(&self, sq: &Square) -> bool;
}

impl BitboardContains for Bitboard {
    #[inline(always)]
    fn contains(&self, sq: &Square) -> bool {
        (self & square_bb(*sq)) != 0
    }
}

/// Standard starting position FEN
pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startpos_fen() {
        crate::bitboard::init();
        crate::zobrist::init();

        let pos = Position::from_fen(START_FEN).unwrap();
        assert_eq!(pos.to_fen(), START_FEN);
    }

    #[test]
    fn test_piece_placement() {
        crate::bitboard::init();
        crate::zobrist::init();

        let pos = Position::from_fen(START_FEN).unwrap();

        // Check white pieces
        let wking = Piece::make(Color::White, PieceType::King);
        let wqueen = Piece::make(Color::White, PieceType::Queen);
        let wrook = Piece::make(Color::White, PieceType::Rook);
        assert_eq!(pos.piece_on(Square::E1), Some(wking));
        assert_eq!(pos.piece_on(Square::D1), Some(wqueen));
        assert_eq!(pos.piece_on(Square::A1), Some(wrook));
        assert_eq!(pos.piece_on(Square::H1), Some(wrook));

        // Check black pieces
        let bking = Piece::make(Color::Black, PieceType::King);
        let bqueen = Piece::make(Color::Black, PieceType::Queen);
        assert_eq!(pos.piece_on(Square::E8), Some(bking));
        assert_eq!(pos.piece_on(Square::D8), Some(bqueen));

        // Check pawns
        let wpawn = Piece::make(Color::White, PieceType::Pawn);
        let bpawn = Piece::make(Color::Black, PieceType::Pawn);
        assert_eq!(pos.piece_on(Square::E2), Some(wpawn));
        assert_eq!(pos.piece_on(Square::E7), Some(bpawn));
    }

    #[test]
    fn test_move_making() {
        crate::bitboard::init();
        crate::zobrist::init();

        let mut pos = Position::from_fen(START_FEN).unwrap();
        let m = Move::new(Square::E2, Square::E4);
        let wpawn = Piece::make(Color::White, PieceType::Pawn);

        let original_fen = pos.to_fen();

        pos.do_move(m);
        assert_eq!(pos.piece_on(Square::E4), Some(wpawn));
        assert_eq!(pos.piece_on(Square::E2), None);
        assert_eq!(pos.side_to_move(), Color::Black);

        pos.undo_move(m);
        assert_eq!(pos.piece_on(Square::E2), Some(wpawn));
        assert_eq!(pos.piece_on(Square::E4), None);
        assert_eq!(pos.side_to_move(), Color::White);
        assert_eq!(pos.to_fen(), original_fen, "Position should be restored to original FEN");
    }

    #[test]
    fn test_multiple_moves_undo() {
        crate::bitboard::init();
        crate::zobrist::init();

        let mut pos = Position::from_fen(START_FEN).unwrap();
        let original_fen = pos.to_fen();

        // Make a sequence of moves
        let m1 = Move::new(Square::E2, Square::E4);
        let m2 = Move::new(Square::E7, Square::E5);

        pos.do_move(m1);
        pos.do_move(m2);

        // Undo in reverse order
        pos.undo_move(m2);
        pos.undo_move(m1);

        assert_eq!(pos.to_fen(), original_fen, "Position should be restored after undoing all moves");
    }

    #[test]
    fn test_search_pattern_undo() {
        crate::bitboard::init();
        crate::zobrist::init();

        let mut pos = Position::from_fen(START_FEN).unwrap();
        let original_fen = pos.to_fen();

        // Simulate search pattern: try different moves from same position
        let moves = vec![
            Move::new(Square::E2, Square::E4),
            Move::new(Square::D2, Square::D4),
            Move::new(Square::B1, Square::C3),
        ];

        for m in &moves {
            eprintln!("Trying move {:?} -> {:?}", m.from_sq(), m.to_sq());
            let before_fen = pos.to_fen();

            pos.do_move(*m);
            eprintln!("After do_move: {}", pos.to_fen());

            pos.undo_move(*m);
            eprintln!("After undo_move: {}", pos.to_fen());

            assert_eq!(pos.to_fen(), before_fen, "Position should be restored after undo");
            assert_eq!(pos.to_fen(), original_fen, "Position should match original");
        }
    }
}
