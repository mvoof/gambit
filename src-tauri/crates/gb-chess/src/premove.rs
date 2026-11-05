//! Moves worth offering as a premove: what a piece could do on an empty board,
//! because the opponent's reply may clear the way. Legality is decided when the
//! premove is played.

use gb_model::{MoveOption, PieceKind};
use shakmaty::attacks;
use shakmaty::{Bitboard, CastlingSide, Chess, Color, Position, Rank, Role, Square};

const PROMOTION_SUFFIX: &str = "q";

fn pawn_targets(color: Color, from: Square) -> Vec<Square> {
    let mut targets: Vec<Square> = attacks::pawn_attacks(color, from).into_iter().collect();
    let step: i32 = if color == Color::White { 8 } else { -8 };
    let home = if color == Color::White {
        Rank::Second
    } else {
        Rank::Seventh
    };

    if let Some(one) = from.offset(step) {
        targets.push(one);

        if from.rank() == home {
            targets.extend(one.offset(step));
        }
    }

    targets
}

fn castling_targets(position: &Chess, color: Color, from: Square) -> Vec<Square> {
    [CastlingSide::KingSide, CastlingSide::QueenSide]
        .into_iter()
        .filter(|side| position.castles().has(color, *side))
        .filter(|_| position.board().king_of(color) == Some(from))
        .map(|side| side.king_to(color))
        .collect()
}

fn targets_of(position: &Chess, color: Color, role: Role, from: Square) -> Vec<Square> {
    let empty = Bitboard::EMPTY;

    match role {
        Role::Pawn => pawn_targets(color, from),
        Role::Knight => attacks::knight_attacks(from).into_iter().collect(),
        Role::Bishop => attacks::bishop_attacks(from, empty).into_iter().collect(),
        Role::Rook => attacks::rook_attacks(from, empty).into_iter().collect(),
        Role::Queen => attacks::queen_attacks(from, empty).into_iter().collect(),
        Role::King => {
            let mut targets: Vec<Square> = attacks::king_attacks(from).into_iter().collect();

            targets.extend(castling_targets(position, color, from));

            targets
        }
    }
}

fn is_promotion(role: Role, color: Color, target: Square) -> bool {
    role == Role::Pawn && target.rank() == color.fold_wb(Rank::Eighth, Rank::First)
}

pub fn premove_options(position: &Chess, color: Color) -> Vec<MoveOption> {
    let board = position.board();
    let mut options = Vec::new();

    for from in board.by_color(color) {
        let Some(role) = board.role_at(from) else {
            continue;
        };

        for target in targets_of(position, color, role, from) {
            if board.color_at(target) == Some(color) {
                continue;
            }

            let promotes = is_promotion(role, color, target);
            let suffix = if promotes { PROMOTION_SUFFIX } else { "" };

            options.push(MoveOption {
                from: from.to_string(),
                to: target.to_string(),
                uci: format!("{from}{target}{suffix}"),
                promotion: promotes.then_some(PieceKind::Queen),
            });
        }
    }

    options
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has(options: &[MoveOption], uci: &str) -> bool {
        options.iter().any(|option| option.uci == uci)
    }

    #[test]
    fn offers_moves_blocked_right_now() {
        let options = premove_options(&Chess::default(), Color::White);

        assert!(has(&options, "e2e4"));
        assert!(has(&options, "g1f3"));
        assert!(has(&options, "e2d3"));
        assert!(!has(&options, "a1a2"), "own piece on the target");
    }

    #[test]
    fn a_pawn_premove_to_the_last_rank_promotes_to_a_queen() {
        let position: Chess = "8/4P3/8/8/8/8/8/k1K5 w - - 0 1"
            .parse::<shakmaty::fen::Fen>()
            .unwrap()
            .into_position(shakmaty::CastlingMode::Standard)
            .unwrap();
        let options = premove_options(&position, Color::White);

        assert!(has(&options, "e7e8q"));
    }
}
