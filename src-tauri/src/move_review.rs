//! Measures what each of the player's moves cost, in the background. It has its
//! own engine process: the opponent is limited in strength, and the evaluation
//! bar follows the position on screen.

use std::path::{Path, PathBuf};
use std::sync::Weak;

use gb_chess::san_line;
use gb_engine::{lost_win_percent, quality_of, CandidateLine, EngineError, Score, UciEngine};
use gb_model::MoveMark;
use tauri::async_runtime::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::active_game::PlayedMove;
use crate::session::Session;

pub const LINES: u32 = 1;
pub const THINK_MS: u32 = 300;

pub struct ReviewJob {
    pub game_id: u64,
    pub played: PlayedMove,
    pub engine_path: PathBuf,
}

pub struct MoveReviewer {
    jobs: UnboundedSender<ReviewJob>,
}

struct Reviewer {
    engine: UciEngine,
    path: PathBuf,
}

/// The mark of a move, from the best line before it and the score after it.
/// `reply` is the opponent's score, and `None` here means nothing to mark.
pub fn mark_of(
    ply: u32,
    fen_before: &str,
    best: &CandidateLine,
    played_uci: &str,
    reply: Score,
) -> Option<MoveMark> {
    if best.moves.first().map(String::as_str) == Some(played_uci) {
        return None;
    }

    let quality = quality_of(lost_win_percent(best.score, reply))?;
    let best_san = san_line(fen_before, best.moves.get(..1).unwrap_or_default()).pop()?;

    Some(MoveMark {
        ply,
        quality,
        best_san,
    })
}

async fn review(
    engine: &mut UciEngine,
    played: &PlayedMove,
) -> Result<Option<MoveMark>, EngineError> {
    let before = engine
        .candidates(&played.fen_before, LINES, THINK_MS)
        .await?;
    let Some(best) = before.first() else {
        return Ok(None);
    };

    if best.moves.first() == Some(&played.uci) {
        return Ok(None);
    }

    // A move that ended the game still has a score here: mate, or a draw that a
    // better move would have avoided.
    let after = engine
        .candidates(&played.fen_after, LINES, THINK_MS)
        .await?;

    Ok(after.first().and_then(|reply| {
        mark_of(
            played.ply,
            &played.fen_before,
            best,
            &played.uci,
            reply.score,
        )
    }))
}

async fn review_with(
    slot: &mut Option<Reviewer>,
    path: &Path,
    played: &PlayedMove,
) -> Result<Option<MoveMark>, EngineError> {
    if slot.as_ref().is_none_or(|reviewer| reviewer.path != path) {
        *slot = Some(Reviewer {
            engine: UciEngine::start(path).await?,
            path: path.to_owned(),
        });
    }

    let Some(reviewer) = slot.as_mut() else {
        return Err(EngineError::Closed);
    };

    review(&mut reviewer.engine, played).await
}

async fn run(mut jobs: UnboundedReceiver<ReviewJob>, session: Weak<Session>) {
    let mut slot = None;

    while let Some(job) = jobs.recv().await {
        match review_with(&mut slot, &job.engine_path, &job.played).await {
            Ok(Some(mark)) => {
                if let Some(session) = session.upgrade() {
                    session.add_move_mark(job.game_id, &job.played.uci, mark);
                }
            }
            Ok(None) => {}
            Err(error) => {
                // A mark is a bonus: the game goes on without it.
                tracing::warn!("move review: {error}");
                slot = None;
            }
        }
    }
}

impl MoveReviewer {
    pub fn start(session: Weak<Session>) -> Self {
        let (jobs, received) = unbounded_channel();

        spawn(run(received, session));

        Self { jobs }
    }

    pub fn submit(&self, job: ReviewJob) {
        if self.jobs.send(job).is_err() {
            tracing::warn!("move review has stopped");
        }
    }
}

#[cfg(test)]
mod tests {
    use gb_model::MoveQuality;

    use super::*;

    const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    fn best_line() -> CandidateLine {
        CandidateLine {
            depth: 12,
            score: Score::Centipawns(30),
            moves: vec!["e2e4".to_owned()],
        }
    }

    #[test]
    fn the_engines_own_move_has_no_mark() {
        let mark = mark_of(1, START_FEN, &best_line(), "e2e4", Score::Centipawns(900));

        assert_eq!(mark, None);
    }

    #[test]
    fn a_move_that_keeps_the_score_has_no_mark() {
        let mark = mark_of(1, START_FEN, &best_line(), "d2d4", Score::Centipawns(-25));

        assert_eq!(mark, None);
    }

    #[test]
    fn a_costly_move_is_marked_with_the_better_move_in_san() {
        let mark = mark_of(1, START_FEN, &best_line(), "f2f3", Score::Mate(3));

        assert_eq!(
            mark,
            Some(MoveMark {
                ply: 1,
                quality: MoveQuality::Blunder,
                best_san: "e4".to_owned(),
            })
        );
    }
}
