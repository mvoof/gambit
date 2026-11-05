//! Measures every move of a saved game, once: the marks are kept with the game.
//! Each position is searched a single time, because the score after one move is
//! the score before the next. Its engine process lives only while it measures,
//! and leaving the replay stops it.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gb_archive::{Archive, StoredReview};
use gb_chess::parse_pgn;
use gb_engine::{evaluation_of, CandidateLine, EngineError, UciEngine};
use gb_model::{EvalPoint, GameReview, GameReviewStatus, MoveMark, ReplayGame};
use tauri::async_runtime::spawn;
use tokio::sync::watch;

use crate::analysis::info_of;
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

    mark_of(
        played.ply,
        played.color,
        &frame.fen,
        best,
        &played.uci,
        reply.score,
    )
}

/// The score of frame `index`, and whether the move to it was the engine's own.
fn point_at(
    replay: &ReplayGame,
    index: usize,
    best: Option<&CandidateLine>,
    line: &CandidateLine,
) -> Option<EvalPoint> {
    let frame = replay.frames.get(index)?;
    let measured = evaluation_of(&frame.fen, &info_of(line));
    let played = index
        .checked_sub(1)
        .and_then(|before| replay.moves.get(before));
    let is_best = played
        .zip(best)
        .is_some_and(|(played, best)| best.moves.first().is_some_and(|uci| *uci == played.uci));

    Some(EvalPoint {
        ply: count(index),
        white_win_percent: measured.white_win_percent,
        centipawns: measured.centipawns,
        mate_in: measured.mate_in,
        is_best,
    })
}

/// `None` when the target changed before the game was finished.
async fn measure(
    target: &ReviewTarget,
    replay: &ReplayGame,
    changes: &watch::Receiver<Option<ReviewTarget>>,
    on_progress: impl Fn(u32, &StoredReview),
) -> Result<Option<StoredReview>, EngineError> {
    let mut engine = UciEngine::start(&target.engine_path).await?;
    let mut review = StoredReview {
        marks: Vec::new(),
        points: Vec::new(),
    };
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
            review.marks.extend(mark_at(replay, index, best, reply));
        }

        if let Some(reached) = &line {
            review
                .points
                .extend(point_at(replay, index, previous.as_ref(), reached));
        }

        previous = line;
        on_progress(count(index + 1), &review);
    }

    Ok(Some(review))
}

type LoadedGame = (ReplayGame, Option<StoredReview>);

fn load(archive: &Mutex<Archive>, id: u32) -> Result<LoadedGame, String> {
    let archive = lock(archive);
    let replay = parse_pgn(&archive.pgn(id).map_err(to_message)?).map_err(to_message)?;
    let review = archive.review(id).map_err(to_message)?;

    Ok((replay, review))
}

async fn review(
    target: &ReviewTarget,
    changes: &watch::Receiver<Option<ReviewTarget>>,
    archive: &Mutex<Archive>,
    sink: &dyn EventSink,
) -> Result<(), String> {
    let (replay, stored) = load(archive, target.game_id)?;
    let total = count(replay.frames.len());
    let report = |status, measured, found: &StoredReview| {
        sink.game_review(&GameReview {
            game_id: target.game_id,
            status,
            measured,
            total,
            marks: found.marks.clone(),
            points: found.points.clone(),
        });
    };

    if let Some(found) = stored {
        report(GameReviewStatus::Done, total, &found);

        return Ok(());
    }

    let nothing_yet = StoredReview {
        marks: Vec::new(),
        points: Vec::new(),
    };

    report(GameReviewStatus::Measuring, 0, &nothing_yet);

    let measured = measure(target, &replay, changes, |measured, found| {
        report(GameReviewStatus::Measuring, measured, found);
    })
    .await
    .map_err(to_message)?;

    if let Some(found) = measured {
        lock(archive)
            .save_review(target.game_id, &found)
            .map_err(to_message)?;
        report(GameReviewStatus::Done, total, &found);
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
                    points: Vec::new(),
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
    fn a_point_knows_when_the_move_was_the_engines_own() {
        let replay = parse_pgn("1. e4 *").unwrap();
        let best = line(Score::Centipawns(20), &["e2e4"]);
        let other = line(Score::Centipawns(20), &["d2d4"]);
        let reached = line(Score::Centipawns(-30), &["e7e5"]);

        let point = point_at(&replay, 1, Some(&best), &reached).unwrap();

        assert!(point.is_best);
        assert_eq!(point.ply, 1);
        // Black is to move after 1. e4, so the score turns to White's side.
        assert_eq!(point.centipawns, Some(30));
        assert!(
            !point_at(&replay, 1, Some(&other), &reached)
                .unwrap()
                .is_best
        );
    }

    #[test]
    fn the_starting_position_has_a_point_but_no_move() {
        let replay = parse_pgn("1. e4 *").unwrap();
        let start = line(Score::Centipawns(20), &["e2e4"]);

        assert!(!point_at(&replay, 0, None, &start).unwrap().is_best);
    }

    #[test]
    fn the_starting_position_has_no_move_to_mark() {
        let replay = parse_pgn("1. e4 *").unwrap();
        let score = line(Score::Centipawns(400), &["e2e4"]);

        assert_eq!(mark_at(&replay, 0, &score, &score), None);
    }
}
