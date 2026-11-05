//! The evaluation bar: a second engine at full strength, separate from the
//! Elo-limited opponent. It follows one position until told another.

use std::path::PathBuf;
use std::sync::Arc;

use gb_chess::san_line;
use gb_engine::{evaluation_of, CandidateLine, EngineError, SearchInfo, Strength, UciEngine};
use gb_model::{Evaluation, EvaluationLine};
use tauri::async_runtime::spawn;
use tokio::sync::watch;

use crate::events::EventSink;

const LINES: u32 = 3;
const LINE_PLIES: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisTarget {
    pub fen: String,
    pub engine_path: PathBuf,
}

/// The sender side: setting a target replaces the one being analysed.
pub struct Analysis {
    target: watch::Sender<Option<AnalysisTarget>>,
}

struct Analyst {
    engine: UciEngine,
    path: PathBuf,
}

async fn start_analyst(path: PathBuf) -> Option<Analyst> {
    let started = async {
        let mut engine = UciEngine::start(&path).await?;

        engine.set_strength(Strength::Full).await?;

        Ok::<_, EngineError>(engine)
    }
    .await;

    match started {
        Ok(engine) => Some(Analyst { engine, path }),
        Err(error) => {
            tracing::warn!("analysis engine: {error}");

            None
        }
    }
}

fn line_view(fen: &str, line: &CandidateLine) -> Option<EvaluationLine> {
    let shown = line.moves.len().min(LINE_PLIES);
    let moves = san_line(fen, &line.moves[..shown]);
    let measured = evaluation_of(fen, &info_of(line));

    (!moves.is_empty()).then_some(EvaluationLine {
        moves,
        centipawns: measured.centipawns,
        mate_in: measured.mate_in,
    })
}

fn info_of(line: &CandidateLine) -> SearchInfo {
    SearchInfo {
        depth: line.depth,
        score: line.score,
        best_move: line.moves.first().cloned(),
    }
}

fn evaluation_with_lines(fen: &str, lines: &[CandidateLine]) -> Option<Evaluation> {
    let best = lines.first()?;

    Some(Evaluation {
        lines: lines
            .iter()
            .filter_map(|line| line_view(fen, line))
            .collect(),
        ..evaluation_of(fen, &info_of(best))
    })
}

async fn analyse_until_changed(
    analyst: &mut Analyst,
    fen: &str,
    changes: &watch::Receiver<Option<AnalysisTarget>>,
    sink: &dyn EventSink,
) -> bool {
    let mut watcher = changes.clone();
    let cancel = async move {
        // An error means the app is closing, which also ends the search.
        let _ = watcher.changed().await;
    };
    let searched = analyst
        .engine
        .search_lines(fen, LINES, cancel, |lines| {
            if let Some(evaluation) = evaluation_with_lines(fen, lines) {
                sink.evaluation(&evaluation);
            }
        })
        .await;

    if let Err(error) = &searched {
        tracing::warn!("analysis stopped: {error}");
    }

    searched.is_ok()
}

async fn run(mut changes: watch::Receiver<Option<AnalysisTarget>>, sink: Arc<dyn EventSink>) {
    let mut analyst: Option<Analyst> = None;

    loop {
        let target = changes.borrow_and_update().clone();

        if let Some(wanted) = target {
            let is_running = analyst
                .as_ref()
                .is_some_and(|current| current.path == wanted.engine_path);

            if !is_running {
                analyst = start_analyst(wanted.engine_path.clone()).await;
            }

            if let Some(current) = analyst.as_mut() {
                if !analyse_until_changed(current, &wanted.fen, &changes, sink.as_ref()).await {
                    analyst = None;
                }
            }
        } else {
            // The bar is off: no process should burn a core for nothing.
            analyst = None;
        }

        // A finished search (a mated position) waits here for the next target.
        if changes.changed().await.is_err() {
            return;
        }
    }
}

impl Analysis {
    pub fn start(sink: Arc<dyn EventSink>) -> Self {
        let (target, changes) = watch::channel(None);

        spawn(run(changes, sink));

        Self { target }
    }

    pub fn set_target(&self, target: Option<AnalysisTarget>) {
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
    use gb_model::LastMove;

    use super::*;

    const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    const MATED_FEN: &str = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3";

    fn line(score: Score, moves: &[&str]) -> CandidateLine {
        CandidateLine {
            depth: 12,
            score,
            moves: moves.iter().map(|text| (*text).to_owned()).collect(),
        }
    }

    #[test]
    fn the_first_line_measures_and_every_line_is_san() {
        let lines = [
            line(Score::Centipawns(30), &["e2e4", "e7e5"]),
            line(Score::Centipawns(20), &["d2d4"]),
        ];
        let evaluation = evaluation_with_lines(START_FEN, &lines).unwrap();

        assert_eq!(evaluation.centipawns, Some(30));
        assert_eq!(
            evaluation.best_move,
            Some(LastMove {
                from: "e2".to_owned(),
                to: "e4".to_owned(),
            })
        );
        assert_eq!(evaluation.lines[0].moves, vec!["e4", "e5"]);
        assert_eq!(evaluation.lines[1].centipawns, Some(20));
    }

    #[test]
    fn a_mated_position_still_reaches_the_bar_with_no_lines() {
        let lines = [line(Score::Mate(0), &[])];
        let evaluation = evaluation_with_lines(MATED_FEN, &lines).unwrap();

        assert_eq!(evaluation.white_win_percent, 0.0);
        assert!(evaluation.lines.is_empty());
        assert_eq!(evaluation.best_move, None);
    }

    #[test]
    fn no_lines_yet_is_no_evaluation() {
        assert_eq!(evaluation_with_lines(START_FEN, &[]), None);
    }
}
