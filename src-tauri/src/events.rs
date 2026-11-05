//! What the backend tells the frontend, behind a trait so the session never
//! names Tauri's emitter.

use gb_model::{
    CompanionView, EngineInstallProgress, Evaluation, GameView, EVENT_COMPANION,
    EVENT_ENGINE_INSTALL, EVENT_EVALUATION, EVENT_GAME_UPDATED,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub trait EventSink: Send + Sync {
    fn game_updated(&self, view: &GameView);
    fn evaluation(&self, evaluation: &Evaluation);
    fn install_progress(&self, progress: EngineInstallProgress);
    fn companion_updated(&self, view: &CompanionView);
}

pub struct TauriSink {
    app: AppHandle,
}

impl TauriSink {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    fn emit(&self, event: &str, payload: impl Serialize + Clone) {
        if let Err(error) = self.app.emit(event, payload) {
            tracing::warn!("cannot emit {event}: {error}");
        }
    }
}

impl EventSink for TauriSink {
    fn game_updated(&self, view: &GameView) {
        self.emit(EVENT_GAME_UPDATED, view);
    }

    fn evaluation(&self, evaluation: &Evaluation) {
        self.emit(EVENT_EVALUATION, evaluation);
    }

    fn install_progress(&self, progress: EngineInstallProgress) {
        self.emit(EVENT_ENGINE_INSTALL, progress);
    }

    fn companion_updated(&self, view: &CompanionView) {
        self.emit(EVENT_COMPANION, view);
    }
}
