//! The evaluation bar: a second engine at full strength, separate from the
//! Elo-limited opponent. It follows one position until told another.

use std::path::PathBuf;
use std::sync::Arc;

use gb_engine::{evaluation_of, EngineError, SearchLimit, Strength, UciEngine};
use tauri::async_runtime::spawn;
use tokio::sync::watch;

use crate::events::EventSink;

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
        .search(Some(fen), &[], SearchLimit::Infinite, cancel, |info| {
            sink.evaluation(&evaluation_of(fen, info));
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
