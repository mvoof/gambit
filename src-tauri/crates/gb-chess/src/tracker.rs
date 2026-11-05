//! Stable piece ids, so the frontend animates by key with no chess knowledge.

use gb_model::{BoardPiece, Color, PieceKind};
use shakmaty::{Board, CastlingSide, Move, Square};

use crate::convert::{wire_color, wire_kind};

#[derive(Debug, Clone, Copy)]
struct Tracked {
    id: u32,
    color: Color,
    kind: PieceKind,
}

const SQUARE_COUNT: usize = 64;

#[derive(Debug, Clone)]
pub struct PieceTracker {
    squares: [Option<Tracked>; SQUARE_COUNT],
}

/// What a move took off the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capture {
    pub color: Color,
    pub kind: PieceKind,
}

impl PieceTracker {
    pub fn new(board: &Board) -> Self {
        let mut squares = [None; SQUARE_COUNT];

        for (index, (square, piece)) in board.iter().enumerate() {
            squares[usize::from(square)] = Some(Tracked {
                id: u32::try_from(index).unwrap_or(u32::MAX),
                color: wire_color(piece.color),
                kind: wire_kind(piece.role),
            });
        }

        Self { squares }
    }

    fn take(&mut self, square: Square) -> Option<Tracked> {
        self.squares[usize::from(square)].take()
    }

    fn put(&mut self, square: Square, piece: Option<Tracked>) {
        self.squares[usize::from(square)] = piece;
    }

    /// Follows a legal move and returns the capture, if any.
    pub fn apply(&mut self, chess_move: Move, mover: shakmaty::Color) -> Option<Capture> {
        match chess_move {
            Move::Normal {
                from,
                to,
                promotion,
                ..
            } => {
                let captured = self.take(to);
                let mut moved = self.take(from);

                if let (Some(piece), Some(role)) = (moved.as_mut(), promotion) {
                    piece.kind = wire_kind(role);
                }

                self.put(to, moved);

                captured.map(to_capture)
            }
            Move::EnPassant { from, to } => {
                let captured = self.take(Square::from_coords(to.file(), from.rank()));
                let moved = self.take(from);

                self.put(to, moved);

                captured.map(to_capture)
            }
            Move::Castle { king, rook } => {
                let side = CastlingSide::from_king_side(rook > king);
                let moved_king = self.take(king);
                let moved_rook = self.take(rook);

                self.put(side.king_to(mover), moved_king);
                self.put(side.rook_to(mover), moved_rook);

                None
            }
            Move::Put { .. } => None,
        }
    }

    pub fn pieces(&self) -> Vec<BoardPiece> {
        let mut pieces: Vec<BoardPiece> = Square::ALL
            .iter()
            .filter_map(|square| {
                self.squares[usize::from(*square)].map(|piece| BoardPiece {
                    id: piece.id,
                    color: piece.color,
                    kind: piece.kind,
                    square: square.to_string(),
                })
            })
            .collect();

        // A stable DOM order: reordering keyed nodes would restart animations.
        pieces.sort_by_key(|piece| piece.id);

        pieces
    }
}

fn to_capture(piece: Tracked) -> Capture {
    Capture {
        color: piece.color,
        kind: piece.kind,
    }
}
