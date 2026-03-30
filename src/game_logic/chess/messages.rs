use crate::game_logic::chess::{Color, GameOutcome, MoveError};

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
