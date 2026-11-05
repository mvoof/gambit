//! Measures what each of the player's moves cost, in the background. It has its
//! own engine process: the opponent is limited in strength, and the evaluation
//! bar follows the position on screen.

use std::path::{Path, PathBuf};
use std::sync::Weak;

use gb_chess::san_line;
use gb_engine::{lost_win_percent, quality_of, EngineError, UciEngine};
use gb_model::MoveMark;
use tauri::async_runtime::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::active_game::PlayedMove;
use crate::session::Session;

const LINES: u32 = 1;
const THINK_MS: u32 = 300;

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
    let Some(reply) = after.first() else {
        return Ok(None);
    };
    let quality = quality_of(lost_win_percent(best.score, reply.score));
    let best_san = san_line(&played.fen_before, best.moves.get(..1).unwrap_or_default()).pop();

    Ok(quality.zip(best_san).map(|(quality, best_san)| MoveMark {
        ply: played.ply,
        quality,
        best_san,
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
