use crate::game_logic::chess::{Color, Move, MoveError, Piece, PieceType, Square};

#[derive(Clone)]
pub struct GameState {
    board: [Option<Piece>; 64],
    side_to_move: Color,
    en_passant_target: Option<Square>,
    castling_rights: CastlingRights,
}

#[derive(Debug, Copy, Clone)]
struct CastlingRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new_start_position()
    }
}

impl GameState {
    pub fn new_start_position() -> Self {
        let mut game = Self {
            board: [None; 64],
            side_to_move: Color::White,
            en_passant_target: None,
            castling_rights: CastlingRights {
                white_kingside: true,
                white_queenside: true,
                black_kingside: true,
                black_queenside: true,
            },
        };

        game.place_back_rank(Color::White, 0);
        game.place_pawns(Color::White, 1);
        game.place_back_rank(Color::Black, 7);
        game.place_pawns(Color::Black, 6);

        game
    }

    fn place_back_rank(&mut self, color: Color, rank: u8) {
        let setup = [
            PieceType::Rook,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Queen,
            PieceType::King,
            PieceType::Bishop,
            PieceType::Knight,
            PieceType::Rook,
        ];

        for (file, piece_type) in setup.into_iter().enumerate() {
            let square = Square::new(file as u8, rank).expect("valid back rank square");
            self.set_piece(square, Some(Piece { piece_type, color }));
        }
    }

    fn place_pawns(&mut self, color: Color, rank: u8) {
        for file in 0..8 {
            let square = Square::new(file, rank).expect("valid pawn square");
            self.set_piece(
                square,
                Some(Piece {
                    piece_type: PieceType::Pawn,
                    color,
                }),
            );
        }
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.board[square.to_index()]
    }

    pub fn iter_pieces(&self) -> impl Iterator<Item = (Square, Piece)> + '_ {
        self.board
            .iter()
            .enumerate()
            .filter_map(|(idx, piece)| piece.map(|piece| (idx, piece)))
            .filter_map(|(idx, piece)| Square::from_index(idx).map(|square| (square, piece)))
    }

    pub fn apply_move(&mut self, mv: Move) -> Result<(), MoveError> {
        let piece = self.piece_at(mv.from).ok_or(MoveError::NoPieceAtSource)?;

        if piece.color != self.side_to_move {
            return Err(MoveError::WrongTurn);
        }

        if let Some(target) = self.piece_at(mv.to)
            && target.color == piece.color
        {
            return Err(MoveError::DestinationOccupiedByOwnPiece);
        }

        if !self.is_legal_piece_move(piece, mv.from, mv.to) {
            return Err(MoveError::IllegalPieceMovement);
        }

        if self.would_leave_king_in_check(mv, piece.color) {
            return Err(MoveError::KingWouldBeInCheck);
        }

        self.apply_move_unchecked(mv, piece);
        self.side_to_move = self.side_to_move.opposite();
        Ok(())
    }

    pub fn legal_moves_from(&self, from: Square) -> Vec<Square> {
        let Some(piece) = self.piece_at(from) else {
            return Vec::new();
        };

        if piece.color != self.side_to_move {
            return Vec::new();
        }

        self.legal_moves_for_piece(from, piece)
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        let Some(king_square) = self.find_king_square(color) else {
            return false;
        };
        self.is_square_attacked_by(king_square, color.opposite())
    }

    pub fn is_checkmate(&self, color: Color) -> bool {
        if !self.is_in_check(color) {
            return false;
        }

        !self.has_any_legal_move(color)
    }

    pub fn is_stalemate(&self, color: Color) -> bool {
        if self.is_in_check(color) {
            return false;
        }

        !self.has_any_legal_move(color)
    }

    fn set_piece(&mut self, square: Square, piece: Option<Piece>) {
        self.board[square.to_index()] = piece;
    }

    fn has_any_legal_move(&self, color: Color) -> bool {
        for (from, piece) in self.iter_pieces() {
            if piece.color != color {
                continue;
            }

            if self.legal_moves_for_piece(from, piece).is_empty() {
                continue;
            }

            return true;
        }

        false
    }

    fn legal_moves_for_piece(&self, from: Square, piece: Piece) -> Vec<Square> {
        let mut legal_moves = Vec::new();

        for to_index in 0..64 {
            let Some(to) = Square::from_index(to_index) else {
                continue;
            };

            if from == to {
                continue;
            }

            if self
                .piece_at(to)
                .is_some_and(|target| target.color == piece.color)
            {
                continue;
            }

            if !self.is_legal_piece_move(piece, from, to) {
                continue;
            }

            if !self.would_leave_king_in_check(Move { from, to }, piece.color) {
                legal_moves.push(to);
            }
        }

        legal_moves
    }

    fn would_leave_king_in_check(&self, mv: Move, color: Color) -> bool {
        let mut next = self.clone();
        let Some(piece) = next.piece_at(mv.from) else {
            return true;
        };

        next.apply_move_unchecked(mv, piece);
        next.is_in_check(color)
    }

    fn apply_move_unchecked(&mut self, mv: Move, piece: Piece) {
        let captured_on_destination = self.piece_at(mv.to);

        if piece.piece_type == PieceType::King {
            self.clear_castling_rights(piece.color);
        }

        if piece.piece_type == PieceType::Rook {
            self.clear_rook_castling_right(piece.color, mv.from);
        }

        if let Some(captured_piece) = captured_on_destination
            && captured_piece.piece_type == PieceType::Rook
        {
            self.clear_rook_castling_right(captured_piece.color, mv.to);
        }

        let is_en_passant_capture = piece.piece_type == PieceType::Pawn
            && self.en_passant_target.is_some_and(|target| target == mv.to)
            && self.piece_at(mv.to).is_none()
            && mv.from.file() != mv.to.file();

        let is_castling_move = piece.piece_type == PieceType::King
            && mv.from.rank() == mv.to.rank()
            && (mv.to.file() as i8 - mv.from.file() as i8).unsigned_abs() == 2;

        if is_en_passant_capture {
            let captured_square =
                Square::new(mv.to.file(), mv.from.rank()).expect("valid en passant capture square");
            self.set_piece(captured_square, None);
        }

        self.set_piece(mv.from, None);
        self.set_piece(mv.to, Some(piece));

        if is_castling_move {
            let rank = mv.from.rank();
            let (rook_from, rook_to) = if mv.to.file() > mv.from.file() {
                (
                    Square::new(7, rank).expect("valid castling rook source"),
                    Square::new(5, rank).expect("valid castling rook destination"),
                )
            } else {
                (
                    Square::new(0, rank).expect("valid castling rook source"),
                    Square::new(3, rank).expect("valid castling rook destination"),
                )
            };

            let rook = self.piece_at(rook_from);
            self.set_piece(rook_from, None);
            self.set_piece(rook_to, rook);
        }

        self.en_passant_target = if piece.piece_type == PieceType::Pawn
            && (mv.to.rank() as i8 - mv.from.rank() as i8).unsigned_abs() == 2
        {
            let mid_rank = ((mv.from.rank() as u16 + mv.to.rank() as u16) / 2) as u8;
            Square::new(mv.from.file(), mid_rank)
        } else {
            None
        };
    }

    fn clear_castling_rights(&mut self, color: Color) {
        match color {
            Color::White => {
                self.castling_rights.white_kingside = false;
                self.castling_rights.white_queenside = false;
            }
            Color::Black => {
                self.castling_rights.black_kingside = false;
                self.castling_rights.black_queenside = false;
            }
        }
    }

    fn clear_rook_castling_right(&mut self, color: Color, square: Square) {
        let home_rank = match color {
            Color::White => 0,
            Color::Black => 7,
        };

        if square.rank() != home_rank {
            return;
        }

        match square.file() {
            0 => match color {
                Color::White => self.castling_rights.white_queenside = false,
                Color::Black => self.castling_rights.black_queenside = false,
            },
            7 => match color {
                Color::White => self.castling_rights.white_kingside = false,
                Color::Black => self.castling_rights.black_kingside = false,
            },
            _ => {}
        }
    }

    fn find_king_square(&self, color: Color) -> Option<Square> {
        self.iter_pieces().find_map(|(square, piece)| {
            (piece.color == color && piece.piece_type == PieceType::King).then_some(square)
        })
    }

    fn is_square_attacked_by(&self, target: Square, attacker_color: Color) -> bool {
        self.iter_pieces().any(|(from, piece)| {
            if piece.color != attacker_color {
                return false;
            }
            self.can_piece_attack_square(piece, from, target)
        })
    }

    fn can_piece_attack_square(&self, piece: Piece, from: Square, target: Square) -> bool {
        if from == target {
            return false;
        }

        let dx = target.file() as i8 - from.file() as i8;
        let dy = target.rank() as i8 - from.rank() as i8;

        match piece.piece_type {
            PieceType::Pawn => {
                let direction = match piece.color {
                    Color::White => 1,
                    Color::Black => -1,
                };
                dx.unsigned_abs() == 1 && dy == direction
            }
            PieceType::Knight => {
                let adx = dx.unsigned_abs();
                let ady = dy.unsigned_abs();
                (adx == 1 && ady == 2) || (adx == 2 && ady == 1)
            }
            PieceType::Bishop => {
                let adx = dx.unsigned_abs();
                let ady = dy.unsigned_abs();
                adx == ady && self.is_path_clear(from, target)
            }
            PieceType::Rook => (dx == 0 || dy == 0) && self.is_path_clear(from, target),
            PieceType::Queen => {
                let adx = dx.unsigned_abs();
                let ady = dy.unsigned_abs();
                ((adx == ady) || dx == 0 || dy == 0) && self.is_path_clear(from, target)
            }
            PieceType::King => dx.unsigned_abs() <= 1 && dy.unsigned_abs() <= 1,
        }
    }

    fn is_legal_piece_move(&self, piece: Piece, from: Square, to: Square) -> bool {
        if from == to {
            return false;
        }

        let dx = to.file() as i8 - from.file() as i8;
        let dy = to.rank() as i8 - from.rank() as i8;

        match piece.piece_type {
            PieceType::Pawn => self.is_legal_pawn_move(piece.color, from, to, dx, dy),
            PieceType::Knight => {
                let adx = dx.unsigned_abs();
                let ady = dy.unsigned_abs();
                (adx == 1 && ady == 2) || (adx == 2 && ady == 1)
            }
            PieceType::Bishop => {
                let adx = dx.unsigned_abs();
                let ady = dy.unsigned_abs();
                adx == ady && self.is_path_clear(from, to)
            }
            PieceType::Rook => (dx == 0 || dy == 0) && self.is_path_clear(from, to),
            PieceType::Queen => {
                let adx = dx.unsigned_abs();
                let ady = dy.unsigned_abs();
                ((adx == ady) || dx == 0 || dy == 0) && self.is_path_clear(from, to)
            }
            PieceType::King => {
                if dx.unsigned_abs() <= 1 && dy.unsigned_abs() <= 1 {
                    return true;
                }

                dy == 0
                    && dx.unsigned_abs() == 2
                    && self.is_legal_castling_move(piece.color, from, to)
            }
        }
    }

    fn is_legal_castling_move(&self, color: Color, from: Square, to: Square) -> bool {
        let home_rank = match color {
            Color::White => 0,
            Color::Black => 7,
        };
        let king_home = Square::new(4, home_rank).expect("valid king home square");
        if from != king_home || to.rank() != home_rank {
            return false;
        }

        let kingside_target = Square::new(6, home_rank).expect("valid kingside castling target");
        let queenside_target = Square::new(2, home_rank).expect("valid queenside castling target");

        let (is_kingside, rook_from, king_path_squares) = if to == kingside_target {
            (
                true,
                Square::new(7, home_rank).expect("valid kingside rook square"),
                [
                    Square::new(5, home_rank).expect("valid kingside transit square"),
                    Square::new(6, home_rank).expect("valid kingside target square"),
                ],
            )
        } else if to == queenside_target {
            (
                false,
                Square::new(0, home_rank).expect("valid queenside rook square"),
                [
                    Square::new(3, home_rank).expect("valid queenside transit square"),
                    Square::new(2, home_rank).expect("valid queenside target square"),
                ],
            )
        } else {
            return false;
        };

        if !self.has_castling_right(color, is_kingside) {
            return false;
        }

        if self.piece_at(rook_from)
            != Some(Piece {
                piece_type: PieceType::Rook,
                color,
            })
        {
            return false;
        }

        if !self.is_path_clear(from, rook_from) {
            return false;
        }

        if self.is_in_check(color) {
            return false;
        }

        let attacker = color.opposite();
        !king_path_squares
            .iter()
            .copied()
            .any(|square| self.is_square_attacked_by(square, attacker))
    }

    fn has_castling_right(&self, color: Color, kingside: bool) -> bool {
        match (color, kingside) {
            (Color::White, true) => self.castling_rights.white_kingside,
            (Color::White, false) => self.castling_rights.white_queenside,
            (Color::Black, true) => self.castling_rights.black_kingside,
            (Color::Black, false) => self.castling_rights.black_queenside,
        }
    }

    fn is_legal_pawn_move(&self, color: Color, from: Square, to: Square, dx: i8, dy: i8) -> bool {
        let direction = match color {
            Color::White => 1,
            Color::Black => -1,
        };
        let start_rank = match color {
            Color::White => 1,
            Color::Black => 6,
        };

        let target_piece = self.piece_at(to);

        if dx == 0 && dy == direction {
            return target_piece.is_none();
        }

        if dx == 0 && dy == 2 * direction && from.rank() == start_rank {
            let mid_rank = (from.rank() as i8 + direction) as u8;
            let mid_square = Square::new(from.file(), mid_rank).expect("valid intermediate square");
            return self.piece_at(mid_square).is_none() && target_piece.is_none();
        }

        if dx.unsigned_abs() == 1 && dy == direction {
            if target_piece.is_some_and(|target| target.color != color) {
                return true;
            }

            if self.en_passant_target.is_some_and(|target| target == to)
                && self.piece_at(to).is_none()
            {
                let captured_square =
                    Square::new(to.file(), from.rank()).expect("valid en passant capture square");
                return self.piece_at(captured_square).is_some_and(|captured| {
                    captured.piece_type == PieceType::Pawn && captured.color != color
                });
            }

            return false;
        }

        false
    }

    fn is_path_clear(&self, from: Square, to: Square) -> bool {
        let file_step = (to.file() as i8 - from.file() as i8).signum();
        let rank_step = (to.rank() as i8 - from.rank() as i8).signum();

        let mut file = from.file() as i8 + file_step;
        let mut rank = from.rank() as i8 + rank_step;

        while file != to.file() as i8 || rank != to.rank() as i8 {
            let square =
                Square::new(file as u8, rank as u8).expect("path square should be on board");
            if self.piece_at(square).is_some() {
                return false;
            }
            file += file_step;
            rank += rank_step;
        }

        true
    }
}
