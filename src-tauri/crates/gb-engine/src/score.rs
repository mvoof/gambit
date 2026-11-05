//! Reading `info` lines and turning a score into what the bar shows.

use gb_model::Evaluation;

/// Lichess' fit of win chance against centipawns.
const WIN_CURVE_STEEPNESS: f64 = -0.003_682_08;
const EVEN_PERCENT: f64 = 50.0;
const ALL_PERCENT: f64 = 100.0;
/// The engine takes a draw unless it stands better than this.
const DRAW_ACCEPT_CENTIPAWNS: i32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    Centipawns(i32),
    /// Positive when the side to move mates.
    Mate(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchInfo {
    pub depth: u32,
    pub score: Score,
}

/// `None` for an `info` line that carries no score, such as `info string`.
pub fn parse_info(line: &str) -> Option<SearchInfo> {
    let mut tokens = line.split_whitespace();

    if tokens.next() != Some("info") {
        return None;
    }

    let mut depth = None;
    let mut score = None;

    while let Some(token) = tokens.next() {
        match token {
            "string" => break,
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
    })
}

fn is_white_to_move(fen: &str) -> bool {
    fen.split_whitespace().nth(1) != Some("b")
}

fn win_percent(centipawns: i32) -> f64 {
    let chance = 2.0 / (1.0 + (WIN_CURVE_STEEPNESS * f64::from(centipawns)).exp()) - 1.0;

    EVEN_PERCENT + EVEN_PERCENT * chance
}

/// `info` scores are for the side to move; the wire is White's point of view.
pub fn evaluation_of(fen: &str, info: SearchInfo) -> Evaluation {
    let white_to_move = is_white_to_move(fen);
    let sign = if white_to_move { 1 } else { -1 };

    match info.score {
        Score::Centipawns(centipawns) => Evaluation {
            fen: fen.to_owned(),
            centipawns: Some(centipawns * sign),
            mate_in: None,
            white_win_percent: win_percent(centipawns * sign),
            depth: info.depth,
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
                centipawns: None,
                mate_in: Some(moves * sign),
                white_win_percent: if white_mates { ALL_PERCENT } else { 0.0 },
                depth: info.depth,
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
    fn a_score_for_black_flips_to_white_point_of_view() {
        let info = SearchInfo {
            depth: 10,
            score: Score::Centipawns(100),
        };

        assert_eq!(evaluation_of(WHITE_FEN, info).centipawns, Some(100));
        assert_eq!(evaluation_of(BLACK_FEN, info).centipawns, Some(-100));
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
        };

        assert_eq!(evaluation_of(WHITE_FEN, mate(2)).white_win_percent, 100.0);
        assert_eq!(evaluation_of(BLACK_FEN, mate(2)).white_win_percent, 0.0);
        assert_eq!(evaluation_of(BLACK_FEN, mate(-2)).white_win_percent, 100.0);
        assert_eq!(evaluation_of(WHITE_FEN, mate(0)).white_win_percent, 0.0);
        assert_eq!(evaluation_of(BLACK_FEN, mate(0)).white_win_percent, 100.0);
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
