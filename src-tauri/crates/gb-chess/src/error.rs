use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChessError {
    InvalidFen(String),
    IllegalMove(String),
    GameOver,
    NothingToUndo,
    InvalidPgn(String),
}

impl fmt::Display for ChessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFen(detail) => write!(formatter, "invalid FEN: {detail}"),
            Self::IllegalMove(detail) => write!(formatter, "illegal move: {detail}"),
            Self::GameOver => write!(formatter, "the game is over"),
            Self::NothingToUndo => write!(formatter, "no move to take back"),
            Self::InvalidPgn(detail) => write!(formatter, "invalid PGN: {detail}"),
        }
    }
}

impl std::error::Error for ChessError {}
