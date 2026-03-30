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
