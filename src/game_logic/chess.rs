use glam::Vec3;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Square {
    file: u8,
    rank: u8,
}

impl Square {
    pub fn new(file: u8, rank: u8) -> Option<Self> {
        if file < 8 && rank < 8 {
            Some(Self { file, rank })
        } else {
            None
        }
    }

    pub fn file(self) -> u8 {
        self.file
    }

    pub fn rank(self) -> u8 {
        self.rank
    }

    pub fn from_index(index: usize) -> Option<Self> {
        if index < 64 {
            let file = (index % 8) as u8;
            let rank = (index / 8) as u8;
            Self::new(file, rank)
        } else {
            None
        }
    }

    pub fn to_index(self) -> usize {
        self.rank as usize * 8 + self.file as usize
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MoveError {
    NoPieceAtSource,
    WrongTurn,
    DestinationOccupiedByOwnPiece,
    IllegalPieceMovement,
    KingWouldBeInCheck,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GameOutcome {
    Checkmate { winner: Color },
    Stalemate,
}

pub struct ChessSceneState {
    pub game_state: GameState,
    pub game_outcome: Option<GameOutcome>,
    pub board_model_index: usize,
    pub board_min: Vec3,
    pub board_max: Vec3,
    pub model_by_square: HashMap<Square, usize>,
    pub square_by_model: HashMap<usize, Square>,
    pub highlight_model_indices: Vec<usize>,
    pub selected_square: Option<Square>,
    pub last_error: Option<String>,
}

pub struct ModelMoveUpdate {
    pub moving_model_index: usize,
    pub captured_model_index: Option<usize>,
    pub destination_world_position: Vec3,
}

impl ChessSceneState {
    pub fn new(
        game_state: GameState,
        board_model_index: usize,
        board_min: Vec3,
        board_max: Vec3,
        model_by_square: HashMap<Square, usize>,
        square_by_model: HashMap<usize, Square>,
    ) -> Self {
        Self {
            game_state,
            game_outcome: None,
            board_model_index,
            board_min,
            board_max,
            model_by_square,
            square_by_model,
            highlight_model_indices: Vec::new(),
            selected_square: None,
            last_error: None,
        }
    }

    pub fn clear_last_error(&mut self) {
        self.last_error = None;
    }

    pub fn clear_selection(&mut self) {
        self.selected_square = None;
    }

    pub fn is_highlight_model(&self, model_index: usize) -> bool {
        self.highlight_model_indices.contains(&model_index)
    }

    pub fn try_select_piece_model(&mut self, model_index: usize) -> Option<Square> {
        let square = self.square_by_model.get(&model_index).copied()?;
        let piece = self.game_state.piece_at(square)?;
        if piece.color == self.game_state.side_to_move() {
            self.selected_square = Some(square);
            Some(square)
        } else {
            None
        }
    }

    pub fn try_build_click_move(
        &self,
        model_index: usize,
        hit_point: Option<Vec3>,
    ) -> Option<Move> {
        if model_index != self.board_model_index {
            return None;
        }

        let from = self.selected_square?;
        let point = hit_point?;
        let to = self.world_to_square(point)?;
        Some(Move { from, to })
    }

    pub fn apply_mapping_after_move(
        &mut self,
        from: Square,
        to: Square,
    ) -> Option<ModelMoveUpdate> {
        let moving_model_index = self.model_by_square.remove(&from)?;
        self.square_by_model.remove(&moving_model_index);

        let mut captured_model_index = self.model_by_square.remove(&to);

        if captured_model_index.is_none()
            && from.file() != to.file()
            && self
                .game_state
                .piece_at(to)
                .is_some_and(|piece| piece.piece_type == PieceType::Pawn)
            && let Some(en_passant_captured_square) = Square::new(to.file(), from.rank())
        {
            captured_model_index = self.model_by_square.remove(&en_passant_captured_square);
        }

        if let Some(captured) = captured_model_index {
            self.square_by_model.remove(&captured);
        }

        self.model_by_square.insert(to, moving_model_index);
        self.square_by_model.insert(moving_model_index, to);

        Some(ModelMoveUpdate {
            moving_model_index,
            captured_model_index,
            destination_world_position: self.square_to_world(to),
        })
    }

    pub fn square_to_world(&self, square: Square) -> Vec3 {
        square_to_world(square, self.board_min, self.board_max)
    }

    pub fn world_to_square(&self, point: Vec3) -> Option<Square> {
        world_to_square(point, self.board_min, self.board_max)
    }
}

pub fn parse_piece_template_name(name: &str) -> Option<(PieceType, Color)> {
    let normalized = name.trim().to_ascii_lowercase();
    let (piece_name, color_suffix) = normalized.split_once('.')?;

    let piece_type = match piece_name {
        "pawn" => PieceType::Pawn,
        "rook" => PieceType::Rook,
        "knight" => PieceType::Knight,
        "bishop" => PieceType::Bishop,
        "queen" | "queeen" => PieceType::Queen,
        "king" => PieceType::King,
        _ => return None,
    };

    let color = match color_suffix {
        "000" => Color::Black,
        "001" => Color::White,
        _ => return None,
    };

    Some((piece_type, color))
}

pub fn square_to_world(square: Square, board_min: Vec3, board_max: Vec3) -> Vec3 {
    let square_width = (board_max.x - board_min.x) / 8.0;
    let square_depth = (board_max.z - board_min.z) / 8.0;
    let x = board_min.x + (square.file() as f32 + 0.5) * square_width;
    let z = board_min.z + (square.rank() as f32 + 0.5) * square_depth;
    Vec3::new(x, board_max.y + 0.01, z)
}

pub fn world_to_square(point: Vec3, board_min: Vec3, board_max: Vec3) -> Option<Square> {
    if point.x < board_min.x
        || point.x > board_max.x
        || point.z < board_min.z
        || point.z > board_max.z
    {
        return None;
    }

    let width = board_max.x - board_min.x;
    let depth = board_max.z - board_min.z;
    if width <= 0.0 || depth <= 0.0 {
        return None;
    }

    let rel_x = ((point.x - board_min.x) / width).clamp(0.0, 0.999_999);
    let rel_z = ((point.z - board_min.z) / depth).clamp(0.0, 0.999_999);

    let file = (rel_x * 8.0).floor() as u8;
    let rank = (rel_z * 8.0).floor() as u8;
    Square::new(file, rank)
}

pub fn move_error_message(err: MoveError) -> String {
    match err {
        MoveError::NoPieceAtSource => "No piece selected".to_owned(),
        MoveError::WrongTurn => "Piece cannot move this turn".to_owned(),
        MoveError::DestinationOccupiedByOwnPiece => "Destination occupied by own piece".to_owned(),
        MoveError::IllegalPieceMovement => "Illegal move for selected piece".to_owned(),
        MoveError::KingWouldBeInCheck => "King in check".to_owned(),
    }
}

pub fn game_outcome_message(outcome: GameOutcome) -> String {
    match outcome {
        GameOutcome::Checkmate {
            winner: Color::White,
        } => "Checkmate: White wins".to_owned(),
        GameOutcome::Checkmate {
            winner: Color::Black,
        } => "Checkmate: Black wins".to_owned(),
        GameOutcome::Stalemate => "Stalemate: Draw".to_owned(),
    }
}

#[derive(Clone)]
pub struct GameState {
    board: [Option<Piece>; 64],
    side_to_move: Color,
    en_passant_target: Option<Square>,
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
        let is_en_passant_capture = piece.piece_type == PieceType::Pawn
            && self.en_passant_target.is_some_and(|target| target == mv.to)
            && self.piece_at(mv.to).is_none()
            && mv.from.file() != mv.to.file();

        if is_en_passant_capture {
            let captured_square =
                Square::new(mv.to.file(), mv.from.rank()).expect("valid en passant capture square");
            self.set_piece(captured_square, None);
        }

        self.set_piece(mv.from, None);
        self.set_piece(mv.to, Some(piece));

        self.en_passant_target = if piece.piece_type == PieceType::Pawn
            && (mv.to.rank() as i8 - mv.from.rank() as i8).unsigned_abs() == 2
        {
            let mid_rank = ((mv.from.rank() as u16 + mv.to.rank() as u16) / 2) as u8;
            Square::new(mv.from.file(), mid_rank)
        } else {
            None
        };
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
            PieceType::King => dx.unsigned_abs() <= 1 && dy.unsigned_abs() <= 1,
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
