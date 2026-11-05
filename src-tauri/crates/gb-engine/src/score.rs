//! Reading `info` lines and turning a score into what the bar shows.

use gb_model::{Color, Evaluation, LastMove, MoveQuality};

/// Lichess' fit of win chance against centipawns.
const WIN_CURVE_STEEPNESS: f64 = -0.003_682_08;
const EVEN_PERCENT: f64 = 50.0;
const ALL_PERCENT: f64 = 100.0;
/// The engine takes a draw unless it stands better than this.
const DRAW_ACCEPT_CENTIPAWNS: i32 = 30;
/// Shallower than this, the first move still jumps around.
pub const MIN_BEST_MOVE_DEPTH: u32 = 10;
/// Win percent a move may lose, as Lichess draws the lines: a tenth, a fifth and
/// three tenths of its win chance scale, which runs from -1 to 1.
const INACCURACY_LOSS: f64 = 5.0;
const MISTAKE_LOSS: f64 = 10.0;
const BLUNDER_LOSS: f64 = 15.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    Centipawns(i32),
    /// Positive when the side to move mates.
    Mate(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchInfo {
    pub depth: u32,
    pub score: Score,
    /// UCI: the first move of the principal variation.
    pub best_move: Option<String>,
}

/// One of the engine's best lines, from `MultiPV`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateLine {
    pub depth: u32,
    /// For the side to move.
    pub score: Score,
    /// UCI, first move first.
    pub moves: Vec<String>,
}

/// The rank and the line of an `info` line with a score. A position with no move
/// left has a score and no variation.
pub fn parse_candidate(line: &str) -> Option<(u32, CandidateLine)> {
    let mut tokens = line.split_whitespace();

    if tokens.next() != Some("info") {
        return None;
    }

    let mut rank = 1;
    let mut depth = 0;
    let mut score = None;

    while let Some(token) = tokens.next() {
        match token {
            "string" => return None,
            "multipv" => rank = tokens.next()?.parse().ok()?,
            "depth" => depth = tokens.next()?.parse().ok()?,
            "score" => {
                let kind = tokens.next();
                let value = tokens.next().and_then(|text| text.parse().ok());

                score = match (kind, value) {
                    (Some("cp"), Some(centipawns)) => Some(Score::Centipawns(centipawns)),
                    (Some("mate"), Some(moves)) => Some(Score::Mate(moves)),
                    _ => None,
                };
            }
            "pv" => {
                let moves = tokens.map(str::to_owned).collect();

                return Some((
                    rank,
                    CandidateLine {
                        depth,
                        score: score?,
                        moves,
                    },
                ));
            }
            _ => {}
        }
    }

    Some((
        rank,
        CandidateLine {
            depth,
            score: score?,
            moves: Vec::new(),
        },
    ))
}

/// `None` for an `info` line that carries no score, such as `info string`.
pub fn parse_info(line: &str) -> Option<SearchInfo> {
    let mut tokens = line.split_whitespace();

    if tokens.next() != Some("info") {
        return None;
    }

    let mut depth = None;
    let mut score = None;
    let mut best_move = None;

    while let Some(token) = tokens.next() {
        match token {
            "string" => break,
            "pv" => {
                best_move = tokens.next().map(str::to_owned);

                break;
            }
            "multipv" if tokens.next() != Some("1") => return None,
            "depth" => depth = tokens.next().and_then(|value| value.parse().ok()),
            "score" => {
                let kind = tokens.next();
                let value = tokens.next().and_then(|text| text.parse().ok());

                score = match (kind, value) {
                    (Some("cp"), Some(centipawns)) => Some(Score::Centipawns(centipawns)),
                    (Some("mate"), Some(moves)) => Some(Score::Mate(moves)),
                    _ => None,
                };
            }
            _ => {}
        }
    }

    Some(SearchInfo {
        depth: depth?,
        score: score?,
        best_move,
    })
}

/// `e7e8q` to its squares; the promotion piece does not change the arrow.
fn arrow_of(uci: &str) -> Option<LastMove> {
    Some(LastMove {
        from: uci.get(..2)?.to_owned(),
        to: uci.get(2..4)?.to_owned(),
    })
}

fn is_white_to_move(fen: &str) -> bool {
    fen.split_whitespace().nth(1) != Some("b")
}

fn win_percent(centipawns: i32) -> f64 {
    let chance = 2.0 / (1.0 + (WIN_CURVE_STEEPNESS * f64::from(centipawns)).exp()) - 1.0;

    EVEN_PERCENT + EVEN_PERCENT * chance
}

fn side_win_percent(score: Score) -> f64 {
    match score {
        Score::Centipawns(centipawns) => win_percent(centipawns),
        Score::Mate(moves) if moves > 0 => ALL_PERCENT,
        Score::Mate(_) => 0.0,
    }
}

/// What a move cost its side. `before` is the mover's score, `after` the
/// opponent's, since the opponent is the side to move by then.
pub fn lost_win_percent(before: Score, after: Score) -> f64 {
    side_win_percent(before) - (ALL_PERCENT - side_win_percent(after))
}

pub fn quality_of(lost_percent: f64) -> Option<MoveQuality> {
    if lost_percent >= BLUNDER_LOSS {
        Some(MoveQuality::Blunder)
    } else if lost_percent >= MISTAKE_LOSS {
        Some(MoveQuality::Mistake)
    } else if lost_percent >= INACCURACY_LOSS {
        Some(MoveQuality::Inaccuracy)
    } else {
        None
    }
}

/// `info` scores are for the side to move; the wire is White's point of view.
pub fn evaluation_of(fen: &str, info: &SearchInfo) -> Evaluation {
    let white_to_move = is_white_to_move(fen);
    let sign = if white_to_move { 1 } else { -1 };
    let turn = if white_to_move {
        Color::White
    } else {
        Color::Black
    };
    let best_move = info
        .best_move
        .as_deref()
        .filter(|_| info.depth >= MIN_BEST_MOVE_DEPTH)
        .and_then(arrow_of);

    match info.score {
        Score::Centipawns(centipawns) => Evaluation {
            fen: fen.to_owned(),
            turn,
            centipawns: Some(centipawns * sign),
            mate_in: None,
            white_win_percent: win_percent(centipawns * sign),
            depth: info.depth,
            best_move,
            lines: Vec::new(),
        },
        Score::Mate(moves) => {
            // `mate 0`: the side to move is already mated.
            let white_mates = if moves == 0 {
                !white_to_move
            } else {
                (moves > 0) == white_to_move
            };

            Evaluation {
                fen: fen.to_owned(),
                turn,
                centipawns: None,
                mate_in: Some(moves * sign),
                white_win_percent: if white_mates { ALL_PERCENT } else { 0.0 },
                depth: info.depth,
                best_move,
                lines: Vec::new(),
            }
        }
    }
}

/// `score` is from the engine's own point of view. No score means no opinion.
pub fn accepts_draw(score: Option<Score>) -> bool {
    match score {
        Some(Score::Centipawns(centipawns)) => centipawns <= DRAW_ACCEPT_CENTIPAWNS,
        Some(Score::Mate(moves)) => moves < 0,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHITE_FEN: &str = "4k3/8/8/8/8/8/8/4K2R w - - 0 1";
    const BLACK_FEN: &str = "4k3/8/8/8/8/8/8/4K2R b - - 0 1";

    #[test]
    fn reads_a_centipawn_line() {
        let line = "info depth 12 seldepth 18 multipv 1 score cp 34 nodes 1000 pv e2e4 e7e5";

        assert_eq!(
            parse_info(line),
            Some(SearchInfo {
                depth: 12,
                score: Score::Centipawns(34),
                best_move: Some("e2e4".to_owned()),
            })
        );
    }

    #[test]
    fn reads_a_mate_line() {
        assert_eq!(
            parse_info("info depth 20 score mate -3 lowerbound").map(|info| info.score),
            Some(Score::Mate(-3))
        );
    }

    #[test]
    fn ignores_lines_with_no_score_and_secondary_lines() {
        assert_eq!(parse_info("info string NNUE evaluation enabled"), None);
        assert_eq!(parse_info("info depth 3 currmove e2e4"), None);
        assert_eq!(parse_info("info depth 9 multipv 2 score cp 10"), None);
        assert_eq!(parse_info("bestmove e2e4"), None);
    }

    #[test]
    fn reads_a_ranked_line() {
        let line = "info depth 12 multipv 2 score cp -14 nodes 900 pv g1f3 b8c6";

        assert_eq!(
            parse_candidate(line),
            Some((
                2,
                CandidateLine {
                    depth: 12,
                    score: Score::Centipawns(-14),
                    moves: vec!["g1f3".to_owned(), "b8c6".to_owned()],
                }
            ))
        );
    }

    #[test]
    fn a_finished_position_is_a_line_with_no_moves() {
        let (rank, line) = parse_candidate("info depth 0 score mate 0").unwrap();

        assert_eq!(rank, 1);
        assert_eq!(line.score, Score::Mate(0));
        assert!(line.moves.is_empty());
    }

    #[test]
    fn a_line_with_no_score_is_no_candidate() {
        assert_eq!(parse_candidate("info depth 3 currmove e2e4"), None);
        assert_eq!(parse_candidate("info string pv a2a3"), None);
        assert_eq!(parse_candidate("bestmove e2e4"), None);
    }

    #[test]
    fn the_best_move_appears_once_the_search_is_deep_enough() {
        let info = |depth| SearchInfo {
            depth,
            score: Score::Centipawns(0),
            best_move: Some("e7e8q".to_owned()),
        };

        assert_eq!(evaluation_of(WHITE_FEN, &info(9)).best_move, None);
        assert_eq!(
            evaluation_of(WHITE_FEN, &info(10)).best_move,
            Some(LastMove {
                from: "e7".to_owned(),
                to: "e8".to_owned(),
            })
        );
    }

    #[test]
    fn keeping_the_evaluation_costs_nothing() {
        let lost = lost_win_percent(Score::Centipawns(40), Score::Centipawns(-40));

        assert!(lost.abs() < 1e-9);
        assert_eq!(quality_of(lost), None);
    }

    #[test]
    fn hanging_a_piece_is_a_blunder_and_a_small_slip_an_inaccuracy() {
        let hung = lost_win_percent(Score::Centipawns(0), Score::Centipawns(300));
        let slip = lost_win_percent(Score::Centipawns(0), Score::Centipawns(60));

        assert_eq!(quality_of(hung), Some(MoveQuality::Blunder));
        assert_eq!(quality_of(slip), Some(MoveQuality::Inaccuracy));
    }

    #[test]
    fn missing_a_mate_costs_the_win() {
        let lost = lost_win_percent(Score::Mate(2), Score::Centipawns(0));

        assert_eq!(quality_of(lost), Some(MoveQuality::Blunder));
    }

    #[test]
    fn walking_into_mate_counts_from_the_movers_side() {
        let lost = lost_win_percent(Score::Centipawns(0), Score::Mate(1));

        assert_eq!(lost, 50.0);
    }

    #[test]
    fn a_score_for_black_flips_to_white_point_of_view() {
        let info = SearchInfo {
            depth: 10,
            score: Score::Centipawns(100),
            best_move: None,
        };

        assert_eq!(evaluation_of(WHITE_FEN, &info).centipawns, Some(100));
        assert_eq!(evaluation_of(BLACK_FEN, &info).centipawns, Some(-100));
    }

    #[test]
    fn the_win_percent_is_symmetric_and_bounded() {
        assert!((win_percent(0) - 50.0).abs() < 1e-9);
        assert!((win_percent(300) + win_percent(-300) - 100.0).abs() < 1e-9);
        assert!(win_percent(i32::MAX / 2) <= 100.0);
        assert!(win_percent(i32::MIN / 2) >= 0.0);
    }

    #[test]
    fn a_mate_fills_the_bar_for_the_mating_side() {
        let mate = |moves| SearchInfo {
            depth: 5,
            score: Score::Mate(moves),
            best_move: None,
        };

        assert_eq!(evaluation_of(WHITE_FEN, &mate(2)).white_win_percent, 100.0);
        assert_eq!(evaluation_of(BLACK_FEN, &mate(2)).white_win_percent, 0.0);
        assert_eq!(evaluation_of(BLACK_FEN, &mate(-2)).white_win_percent, 100.0);
        assert_eq!(evaluation_of(WHITE_FEN, &mate(0)).white_win_percent, 0.0);
        assert_eq!(evaluation_of(BLACK_FEN, &mate(0)).white_win_percent, 100.0);
    }

    #[test]
    fn the_engine_takes_a_draw_only_when_it_is_not_better() {
        assert!(accepts_draw(Some(Score::Centipawns(-200))));
        assert!(accepts_draw(Some(Score::Centipawns(
            DRAW_ACCEPT_CENTIPAWNS
        ))));
        assert!(!accepts_draw(Some(Score::Centipawns(200))));
        assert!(accepts_draw(Some(Score::Mate(-4))));
        assert!(!accepts_draw(Some(Score::Mate(4))));
        assert!(!accepts_draw(None));
    }
}
