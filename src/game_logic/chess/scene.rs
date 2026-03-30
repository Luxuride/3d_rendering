use glam::Vec3;
use std::collections::HashMap;

use crate::game_logic::chess::{
    Color, GameOutcome, GameState, Move, PieceType, Square, square_to_world, world_to_square,
};

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
    pub moved_models: Vec<PieceMotionUpdate>,
    pub captured_model_index: Option<usize>,
}

pub struct PieceMotionUpdate {
    pub model_index: usize,
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

        let mut moved_models = vec![PieceMotionUpdate {
            model_index: moving_model_index,
            destination_world_position: self.square_to_world(to),
        }];

        self.model_by_square.insert(to, moving_model_index);
        self.square_by_model.insert(moving_model_index, to);

        let is_castling_move = self
            .game_state
            .piece_at(to)
            .is_some_and(|piece| piece.piece_type == PieceType::King)
            && (to.file() as i8 - from.file() as i8).unsigned_abs() == 2;

        if is_castling_move {
            let rank = from.rank();
            let (rook_from, rook_to) = if to.file() > from.file() {
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

            if let Some(rook_model_index) = self.model_by_square.remove(&rook_from) {
                self.square_by_model.remove(&rook_model_index);
                self.model_by_square.insert(rook_to, rook_model_index);
                self.square_by_model.insert(rook_model_index, rook_to);
                moved_models.push(PieceMotionUpdate {
                    model_index: rook_model_index,
                    destination_world_position: self.square_to_world(rook_to),
                });
            }
        }

        Some(ModelMoveUpdate {
            moved_models,
            captured_model_index,
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
