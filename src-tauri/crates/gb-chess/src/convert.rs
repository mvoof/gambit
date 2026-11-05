//! Between shakmaty's types and the wire's.

use gb_model::{Color, MoveOption, PieceKind};
use shakmaty::uci::UciMove;
use shakmaty::{CastlingMode, Move, Role};

pub fn wire_color(color: shakmaty::Color) -> Color {
    match color {
        shakmaty::Color::White => Color::White,
        shakmaty::Color::Black => Color::Black,
    }
}

pub fn chess_color(color: Color) -> shakmaty::Color {
    match color {
        Color::White => shakmaty::Color::White,
        Color::Black => shakmaty::Color::Black,
    }
}

pub fn wire_kind(role: Role) -> PieceKind {
    match role {
        Role::King => PieceKind::King,
        Role::Queen => PieceKind::Queen,
        Role::Rook => PieceKind::Rook,
        Role::Bishop => PieceKind::Bishop,
        Role::Knight => PieceKind::Knight,
        Role::Pawn => PieceKind::Pawn,
    }
}

pub fn uci_text(chess_move: Move) -> String {
    UciMove::from_move(chess_move, CastlingMode::Standard).to_string()
}

/// The standard from and to squares: a castling king goes two files, it does
/// not take its own rook.
pub fn move_squares(chess_move: Move) -> (String, String) {
    let uci = uci_text(chess_move);
    let (from, rest) = uci.split_at(2);
    let (to, _promotion) = rest.split_at(2);

    (from.to_owned(), to.to_owned())
}

pub fn move_option(chess_move: Move) -> MoveOption {
    let (from, to) = move_squares(chess_move);

    MoveOption {
        from,
        to,
        uci: uci_text(chess_move),
        promotion: chess_move.promotion().map(wire_kind),
    }
}
