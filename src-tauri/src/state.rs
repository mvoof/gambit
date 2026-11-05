//! What the commands share.

use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use gb_archive::Archive;
use gb_engine::stockfish_path;
use gb_model::AppSettings;
use gb_pieces::PieceLibrary;
use tauri::{AppHandle, Manager};

use crate::analysis::Analysis;
use crate::events::{EventSink, TauriSink};
use crate::game_review::GameReviewer;
use crate::session::Session;
use crate::settings::load_settings;

const ARCHIVE_FILE: &str = "games.sqlite";
const SETTINGS_FILE: &str = "settings.json";
const PIECES_FOLDER: &str = "pieces";
const ENGINES_FOLDER: &str = "engines";

pub struct AppState {
    pub session: Arc<Session>,
    pub analysis: Analysis,
    pub game_reviewer: GameReviewer,
    pub archive: Arc<Mutex<Archive>>,
    pub pieces: PieceLibrary,
    pub settings: Mutex<AppSettings>,
    pub settings_path: PathBuf,
    pub engines_dir: PathBuf,
    pub sink: Arc<dyn EventSink>,
}

pub fn to_message(error: impl Display) -> String {
    error.to_string()
}

pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The engine in use: the user's own, or the installed Stockfish.
pub fn resolve_engine(settings: &AppSettings, engines_dir: &Path) -> Option<PathBuf> {
    settings
        .engine_path
        .as_ref()
        .map(PathBuf::from)
        .or_else(|| stockfish_path(engines_dir))
}

impl AppState {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let paths = app.path();
        let data_dir = paths.app_data_dir().map_err(to_message)?;
        let config_dir = paths.app_config_dir().map_err(to_message)?;

        fs::create_dir_all(&data_dir).map_err(to_message)?;

        let archive = Archive::open(&data_dir.join(ARCHIVE_FILE)).map_err(to_message)?;
        let archive = Arc::new(Mutex::new(archive));
        let sink: Arc<dyn EventSink> = Arc::new(TauriSink::new(app.clone()));
        let settings_path = config_dir.join(SETTINGS_FILE);

        Ok(Self {
            session: Session::new(Arc::clone(&archive), Arc::clone(&sink)),
            analysis: Analysis::start(Arc::clone(&sink)),
            game_reviewer: GameReviewer::start(Arc::clone(&archive), Arc::clone(&sink)),
            archive,
            pieces: PieceLibrary::new(data_dir.join(PIECES_FOLDER)),
            settings: Mutex::new(load_settings(&settings_path)),
            settings_path,
            engines_dir: data_dir.join(ENGINES_FOLDER),
            sink,
        })
    }

    pub fn engine_path(&self) -> Option<PathBuf> {
        resolve_engine(&lock(&self.settings), &self.engines_dir)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn the_users_engine_wins_and_nothing_installed_is_none() {
        let folder = tempdir().unwrap();
        let settings = AppSettings {
            engine_path: Some("C:/engines/my.exe".to_owned()),
            ..AppSettings::default()
        };

        assert_eq!(
            resolve_engine(&settings, folder.path()),
            Some(PathBuf::from("C:/engines/my.exe"))
        );
        assert_eq!(resolve_engine(&AppSettings::default(), folder.path()), None);
    }
}
