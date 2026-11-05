//! Measures every move of a saved game, once: the marks are kept with the game.
//! Each position is searched a single time, because the score after one move is
//! the score before the next. Its engine process lives only while it measures,
//! and leaving the replay stops it.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gb_archive::Archive;
use gb_chess::parse_pgn;
use gb_engine::{CandidateLine, EngineError, UciEngine};
use gb_model::{GameReview, GameReviewStatus, MoveMark, ReplayGame};
use tauri::async_runtime::spawn;
use tokio::sync::watch;

use crate::events::EventSink;
use crate::move_review::{mark_of, LINES, THINK_MS};
use crate::state::{lock, to_message};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewTarget {
    pub game_id: u32,
    pub engine_path: PathBuf,
}

/// The sender side: setting a target replaces the game being measured.
pub struct GameReviewer {
    target: watch::Sender<Option<ReviewTarget>>,
}

fn count(length: usize) -> u32 {
    u32::try_from(length).unwrap_or(u32::MAX)
}

/// The mark of the move that led to frame `index`, whose search gave `reply`.
fn mark_at(
    replay: &ReplayGame,
    index: usize,
    best: &CandidateLine,
    reply: &CandidateLine,
) -> Option<MoveMark> {
    let before = index.checked_sub(1)?;
    let played = replay.moves.get(before)?;
    let frame = replay.frames.get(before)?;

    mark_of(played.ply, &frame.fen, best, &played.uci, reply.score)
}

/// `None` when the target changed before the game was finished.
async fn measure(
    target: &ReviewTarget,
    replay: &ReplayGame,
    changes: &watch::Receiver<Option<ReviewTarget>>,
    on_progress: impl Fn(u32, &[MoveMark]),
) -> Result<Option<Vec<MoveMark>>, EngineError> {
    let mut engine = UciEngine::start(&target.engine_path).await?;
    let mut marks = Vec::new();
    let mut previous: Option<CandidateLine> = None;

    for (index, frame) in replay.frames.iter().enumerate() {
        if changes.has_changed().unwrap_or(true) {
            return Ok(None);
        }

        let line = engine
            .candidates(&frame.fen, LINES, THINK_MS)
            .await?
            .into_iter()
            .next();

        if let (Some(best), Some(reply)) = (&previous, &line) {
            marks.extend(mark_at(replay, index, best, reply));
        }

        previous = line;
        on_progress(count(index + 1), &marks);
    }

    Ok(Some(marks))
}

type LoadedGame = (ReplayGame, Option<Vec<MoveMark>>);

fn load(archive: &Mutex<Archive>, id: u32) -> Result<LoadedGame, String> {
    let archive = lock(archive);
    let replay = parse_pgn(&archive.pgn(id).map_err(to_message)?).map_err(to_message)?;
    let marks = archive.marks(id).map_err(to_message)?;

    Ok((replay, marks))
}

async fn review(
    target: &ReviewTarget,
    changes: &watch::Receiver<Option<ReviewTarget>>,
    archive: &Mutex<Archive>,
    sink: &dyn EventSink,
) -> Result<(), String> {
    let (replay, stored) = load(archive, target.game_id)?;
    let total = count(replay.frames.len());
    let report = |status, measured, marks: &[MoveMark]| {
        sink.game_review(&GameReview {
            game_id: target.game_id,
            status,
            measured,
            total,
            marks: marks.to_vec(),
        });
    };

    if let Some(marks) = stored {
        report(GameReviewStatus::Done, total, &marks);

        return Ok(());
    }

    report(GameReviewStatus::Measuring, 0, &[]);

    let measured = measure(target, &replay, changes, |measured, marks| {
        report(GameReviewStatus::Measuring, measured, marks);
    })
    .await
    .map_err(to_message)?;

    if let Some(marks) = measured {
        lock(archive)
            .save_marks(target.game_id, &marks)
            .map_err(to_message)?;
        report(GameReviewStatus::Done, total, &marks);
    }

    Ok(())
}

async fn run(
    mut changes: watch::Receiver<Option<ReviewTarget>>,
    archive: Arc<Mutex<Archive>>,
    sink: Arc<dyn EventSink>,
) {
    loop {
        let target = changes.borrow_and_update().clone();

        if let Some(target) = target {
            if let Err(error) = review(&target, &changes, &archive, sink.as_ref()).await {
                tracing::warn!("game review: {error}");
                sink.game_review(&GameReview {
                    game_id: target.game_id,
                    status: GameReviewStatus::Failed,
                    measured: 0,
                    total: 0,
                    marks: Vec::new(),
                });
            }
        }

        if changes.changed().await.is_err() {
            return;
        }
    }
}

impl GameReviewer {
    pub fn start(archive: Arc<Mutex<Archive>>, sink: Arc<dyn EventSink>) -> Self {
        let (target, changes) = watch::channel(None);

        spawn(run(changes, archive, sink));

        Self { target }
    }

    pub fn set_target(&self, target: Option<ReviewTarget>) {
        self.target.send_if_modified(|current| {
            let is_new = *current != target;

            *current = target;

            is_new
        });
    }
}

#[cfg(test)]
mod tests {
    use gb_engine::Score;
    use gb_model::MoveQuality;

    use super::*;

    fn line(score: Score, moves: &[&str]) -> CandidateLine {
        CandidateLine {
            depth: 12,
            score,
            moves: moves.iter().map(|text| (*text).to_owned()).collect(),
        }
    }

    #[test]
    fn a_frame_marks_the_move_that_led_to_it() {
        let replay = parse_pgn("1. e4 f6 2. d4 *").unwrap();
        let best = line(Score::Centipawns(20), &["e7e5"]);
        let reply = line(Score::Centipawns(400), &["d2d4"]);
        let mark = mark_at(&replay, 2, &best, &reply).unwrap();

        assert_eq!(mark.ply, 2);
        assert_eq!(mark.quality, MoveQuality::Blunder);
        assert_eq!(mark.best_san, "e5");
    }

    #[test]
    fn the_starting_position_has_no_move_to_mark() {
        let replay = parse_pgn("1. e4 *").unwrap();
        let score = line(Score::Centipawns(400), &["e2e4"]);

        assert_eq!(mark_at(&replay, 0, &score, &score), None);
    }
}
