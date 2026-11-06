//! Tauri commands. Thin on purpose: validate, delegate, return.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use gb_archive::NewGame;
use gb_chess::parse_pgn;
use gb_engine::{
    catalog_entries, install_engine as install, installed_engine_path, probe_engine,
    remove_engine as remove,
};
use gb_model::{
    AppSettings, ArchiveStats, CatalogEntry, EngineInfo, GameSummary, GameView, NewGameOptions,
    PieceImages, PieceSetView, ReplayFrame, ReplayGame, SheetColumnOrder,
};
use gb_pieces::preview_sheet;
use tauri::State;

use crate::analysis::AnalysisTarget;
use crate::game_review::ReviewTarget;
use crate::settings::{normalise, reset, save_settings};
use crate::state::{lock, to_message, AppState};

const NO_ENGINE: &str = "no engine installed";
const ENGINE_IN_USE: &str = "the engine in use cannot be removed";

type Reply<T> = Result<T, String>;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppSettings {
    lock(&state.settings).clone()
}

#[tauri::command]
pub fn save_app_settings(state: State<'_, AppState>, settings: AppSettings) -> Reply<AppSettings> {
    let normal = normalise(settings);

    save_settings(&state.settings_path, &normal).map_err(to_message)?;
    lock(&state.settings).clone_from(&normal);

    Ok(normal)
}

#[tauri::command]
pub fn reset_app_settings(state: State<'_, AppState>) -> Reply<AppSettings> {
    let fresh = reset(&lock(&state.settings));

    save_settings(&state.settings_path, &fresh).map_err(to_message)?;
    lock(&state.settings).clone_from(&fresh);

    Ok(fresh)
}

#[tauri::command]
pub async fn engine_status(state: State<'_, AppState>) -> Reply<Option<EngineInfo>> {
    let Some(path) = state.engine_path() else {
        return Ok(None);
    };

    probe_engine(&path).await.map(Some).map_err(to_message)
}

#[tauri::command]
pub fn engine_catalog(state: State<'_, AppState>) -> Vec<CatalogEntry> {
    catalog_entries(&state.engines_dir)
}

/// Probes first: an engine that does not answer never becomes the one in use.
async fn activate(state: &AppState, path: &Path) -> Reply<EngineInfo> {
    let info = probe_engine(path).await.map_err(to_message)?;
    let settings = {
        let mut settings = lock(&state.settings);

        settings.engine_path = Some(path.to_string_lossy().into_owned());
        settings.clone()
    };

    save_settings(&state.settings_path, &settings).map_err(to_message)?;

    Ok(info)
}

#[tauri::command]
pub async fn install_engine(state: State<'_, AppState>, id: String) -> Reply<EngineInfo> {
    let sink = state.sink.clone();
    let path = install(&state.engines_dir, &id, |progress| {
        sink.install_progress(progress);
    })
    .await
    .map_err(to_message)?;

    activate(&state, &path).await
}

fn is_in_use(in_use: Option<&Path>, installed: Option<&Path>) -> bool {
    in_use.is_some() && in_use == installed
}

#[tauri::command]
pub fn remove_engine(state: State<'_, AppState>, id: String) -> Reply<()> {
    let installed = installed_engine_path(&state.engines_dir, &id);

    if is_in_use(state.engine_path().as_deref(), installed.as_deref()) {
        return Err(ENGINE_IN_USE.to_owned());
    }

    remove(&state.engines_dir, &id).map_err(to_message)
}

#[tauri::command]
pub async fn choose_engine(state: State<'_, AppState>, path: String) -> Reply<EngineInfo> {
    activate(&state, Path::new(&path)).await
}

#[tauri::command]
pub async fn new_game(state: State<'_, AppState>, options: NewGameOptions) -> Reply<GameView> {
    let path = state.engine_path().ok_or_else(|| NO_ENGINE.to_owned())?;

    state.session.new_game(options, &path).await
}

/// Takes the game off the board. It stays in the archive, to be played on.
#[tauri::command]
pub async fn suspend_game(state: State<'_, AppState>) -> Reply<()> {
    state.session.suspend();

    Ok(())
}

#[tauri::command]
pub async fn resume_game(state: State<'_, AppState>, id: u32) -> Reply<GameView> {
    let path = state.engine_path().ok_or_else(|| NO_ENGINE.to_owned())?;

    state.session.resume_game(id, &path).await
}

#[tauri::command]
pub fn current_game(state: State<'_, AppState>) -> Option<GameView> {
    state.session.current_view()
}

#[tauri::command]
pub fn game_frames(state: State<'_, AppState>) -> Reply<Vec<ReplayFrame>> {
    state.session.frames()
}

// These stay `async` for the runtime they run on: the session spawns tasks.

#[tauri::command]
pub async fn make_move(state: State<'_, AppState>, uci: String) -> Reply<GameView> {
    state.session.make_move(&uci)
}

#[tauri::command]
pub async fn set_premove(state: State<'_, AppState>, uci: String) -> Reply<GameView> {
    state.session.set_premove(&uci)
}

#[tauri::command]
pub async fn clear_premove(state: State<'_, AppState>) -> Reply<GameView> {
    state.session.clear_premove()
}

#[tauri::command]
pub async fn take_back(state: State<'_, AppState>) -> Reply<GameView> {
    state.session.take_back()
}

#[tauri::command]
pub async fn resign(state: State<'_, AppState>) -> Reply<GameView> {
    state.session.resign()
}

#[tauri::command]
pub async fn offer_draw(state: State<'_, AppState>) -> Reply<bool> {
    state.session.offer_draw()
}

#[tauri::command]
pub fn set_analysis_position(state: State<'_, AppState>, fen: Option<String>) {
    let (lines, depth) = {
        let settings = lock(&state.settings);

        (settings.analysis_lines, settings.analysis_depth)
    };
    let target = fen.and_then(|fen| {
        state.engine_path().map(|engine_path| AnalysisTarget {
            fen,
            engine_path,
            lines,
            depth,
        })
    });

    state.analysis.set_target(target);
}

fn review_target(id: Option<u32>, engine_path: Option<PathBuf>) -> Option<ReviewTarget> {
    id.zip(engine_path)
        .map(|(game_id, engine_path)| ReviewTarget {
            game_id,
            engine_path,
        })
}

/// The saved game to measure, or `None` when no replay is on screen.
#[tauri::command]
pub fn set_review_game(state: State<'_, AppState>, id: Option<u32>) {
    state
        .game_reviewer
        .set_target(review_target(id, state.engine_path()));
}

#[tauri::command]
pub fn list_games(state: State<'_, AppState>) -> Reply<Vec<GameSummary>> {
    lock(&state.archive).list().map_err(to_message)
}

#[tauri::command]
pub fn archive_stats(state: State<'_, AppState>) -> Reply<ArchiveStats> {
    lock(&state.archive).stats().map_err(to_message)
}

#[tauri::command]
pub fn set_game_favorite(state: State<'_, AppState>, id: u32, is_favorite: bool) -> Reply<()> {
    lock(&state.archive)
        .set_favorite(id, is_favorite)
        .map_err(to_message)
}

#[tauri::command]
pub fn load_replay(state: State<'_, AppState>, id: u32) -> Reply<ReplayGame> {
    let archive = lock(&state.archive);
    let replay = parse_pgn(&archive.pgn(id).map_err(to_message)?).map_err(to_message)?;

    Ok(ReplayGame {
        human_color: archive.human_color(id).map_err(to_message)?,
        ..replay
    })
}

#[tauri::command]
pub fn delete_game(state: State<'_, AppState>, id: u32) -> Reply<()> {
    lock(&state.archive).delete(id).map_err(to_message)
}

#[tauri::command]
pub fn export_game(state: State<'_, AppState>, id: u32, path: String) -> Reply<()> {
    let pgn = lock(&state.archive).pgn(id).map_err(to_message)?;

    fs::write(path, pgn).map_err(to_message)
}

fn imported_record(pgn: String, replay: &ReplayGame, played_at: u32) -> NewGame {
    NewGame {
        white: replay.white.clone(),
        black: replay.black.clone(),
        result: replay.result.clone(),
        reason: None,
        played_at,
        ply_count: u32::try_from(replay.moves.len()).unwrap_or(u32::MAX),
        time_control: replay.time_control.clone(),
        pgn,
        human_color: None,
        opponent_elo: None,
        final_fen: replay.frames.last().map(|frame| frame.fen.clone()),
        resume: None,
    }
}

#[tauri::command]
pub fn import_game(state: State<'_, AppState>, path: String) -> Reply<Vec<GameSummary>> {
    let pgn = fs::read_to_string(path).map_err(to_message)?;
    let replay = parse_pgn(&pgn).map_err(to_message)?;
    let played_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u32::try_from(elapsed.as_secs()).unwrap_or(u32::MAX)
        });
    let archive = lock(&state.archive);

    archive
        .save(&imported_record(pgn, &replay, played_at))
        .map_err(to_message)?;

    archive.list().map_err(to_message)
}

#[tauri::command]
pub fn list_piece_sets(state: State<'_, AppState>) -> Reply<Vec<PieceSetView>> {
    state.pieces.list().map_err(to_message)
}

#[tauri::command]
pub async fn preview_piece_sheet(path: String, order: SheetColumnOrder) -> Reply<PieceImages> {
    preview_sheet(Path::new(&path), order).map_err(to_message)
}

#[tauri::command]
pub async fn import_piece_sheet(
    state: State<'_, AppState>,
    name: String,
    path: String,
    order: SheetColumnOrder,
) -> Reply<PieceSetView> {
    state
        .pieces
        .import_sheet(&name, Path::new(&path), order)
        .map_err(to_message)
}

#[tauri::command]
pub async fn import_piece_folder(
    state: State<'_, AppState>,
    name: String,
    path: String,
) -> Reply<PieceSetView> {
    state
        .pieces
        .import_folder(&name, Path::new(&path))
        .map_err(to_message)
}

#[tauri::command]
pub fn delete_piece_set(state: State<'_, AppState>, id: String) -> Reply<()> {
    state.pieces.delete(&id).map_err(to_message)
}

#[cfg(test)]
mod tests {
    use gb_archive::Archive;

    use super::*;

    const GAME: &str = "[White \"A\"]\n[Black \"B\"]\n[Result \"1-0\"]\n[TimeControl \"60+1\"]\n\n1. e4 e5 2. Qh5 1-0\n";

    #[test]
    fn an_imported_game_is_listed_and_replays() {
        let archive = Archive::open_in_memory().unwrap();
        let replay = parse_pgn(GAME).unwrap();
        let id = archive
            .save(&imported_record(GAME.to_owned(), &replay, 5))
            .unwrap();
        let listed = archive.list().unwrap();

        assert_eq!(listed[0].white, "A");
        assert_eq!(listed[0].ply_count, 3);
        assert_eq!(listed[0].time_control, "60+1");
        assert_eq!(listed[0].reason, None);
        assert_eq!(
            parse_pgn(&archive.pgn(id).unwrap()).unwrap().frames.len(),
            4
        );
    }

    #[test]
    fn only_the_engine_in_use_is_protected() {
        let active = PathBuf::from("engines/reckless/reckless");
        let other = PathBuf::from("engines/caissa/caissa");

        assert!(is_in_use(Some(&active), Some(&active)));
        assert!(!is_in_use(Some(&active), Some(&other)));
        assert!(!is_in_use(None, None));
    }

    #[test]
    fn a_game_is_measured_only_with_an_engine() {
        let path = PathBuf::from("engine.exe");

        assert_eq!(
            review_target(Some(3), Some(path.clone())),
            Some(ReviewTarget {
                game_id: 3,
                engine_path: path.clone(),
            })
        );
        assert_eq!(review_target(Some(3), None), None);
        assert_eq!(review_target(None, Some(path)), None);
    }
}
