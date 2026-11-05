//! Tauri commands. Thin on purpose: validate, delegate, return.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use gb_archive::NewGame;
use gb_chess::parse_pgn;
use gb_engine::{install_stockfish, probe_engine};
use gb_model::{
    AppSettings, EngineInfo, GameSummary, GameView, NewGameOptions, PieceImages, PieceSetView,
    ReplayGame, SheetColumnOrder,
};
use gb_pieces::preview_sheet;
use tauri::State;

use crate::analysis::AnalysisTarget;
use crate::settings::{normalise, save_settings};
use crate::state::{lock, to_message, AppState};

const NO_ENGINE: &str = "no engine installed";

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
pub async fn engine_status(state: State<'_, AppState>) -> Reply<Option<EngineInfo>> {
    let Some(path) = state.engine_path() else {
        return Ok(None);
    };

    probe_engine(&path).await.map(Some).map_err(to_message)
}

#[tauri::command]
pub async fn download_stockfish(state: State<'_, AppState>) -> Reply<EngineInfo> {
    let sink = state.sink.clone();
    let path = install_stockfish(&state.engines_dir, |progress| {
        sink.install_progress(progress);
    })
    .await
    .map_err(to_message)?;

    probe_engine(&path).await.map_err(to_message)
}

#[tauri::command]
pub async fn choose_engine(state: State<'_, AppState>, path: String) -> Reply<EngineInfo> {
    let info = probe_engine(Path::new(&path)).await.map_err(to_message)?;
    let settings = {
        let mut settings = lock(&state.settings);

        settings.engine_path = Some(path);
        settings.clone()
    };

    save_settings(&state.settings_path, &settings).map_err(to_message)?;

    Ok(info)
}

#[tauri::command]
pub async fn new_game(state: State<'_, AppState>, options: NewGameOptions) -> Reply<GameView> {
    let path = state.engine_path().ok_or_else(|| NO_ENGINE.to_owned())?;

    state.session.new_game(options, &path).await
}

#[tauri::command]
pub fn current_game(state: State<'_, AppState>) -> Option<GameView> {
    state.session.current_view()
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
    let target = fen.and_then(|fen| {
        state
            .engine_path()
            .map(|engine_path| AnalysisTarget { fen, engine_path })
    });

    state.analysis.set_target(target);
}

#[tauri::command]
pub fn list_games(state: State<'_, AppState>) -> Reply<Vec<GameSummary>> {
    lock(&state.archive).list().map_err(to_message)
}

#[tauri::command]
pub fn load_replay(state: State<'_, AppState>, id: u32) -> Reply<ReplayGame> {
    let pgn = lock(&state.archive).pgn(id).map_err(to_message)?;

    parse_pgn(&pgn).map_err(to_message)
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
}
