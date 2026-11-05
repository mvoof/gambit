//! Application wiring: builds the Tauri app and registers what it needs.

mod active_game;
mod analysis;
#[cfg(feature = "dev")]
mod bindings;
mod civil_date;
mod commands;
mod events;
mod game_review;
mod move_review;
mod session;
mod settings;
mod state;

use tauri::Manager;
use tracing_subscriber::EnvFilter;

use crate::state::AppState;

const DEFAULT_LOG_FILTER: &str = "gambit_lib=info,gb_engine=info";

fn init_logging() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build());

    #[cfg(feature = "dev")]
    let builder = builder.plugin(tauri_plugin_mcp_bridge::init());

    let result = builder
        .setup(|app| {
            #[cfg(feature = "dev")]
            {
                app.add_capability(include_str!("../capabilities-dev/mcp-bridge.json"))?;

                if let Err(error) = bindings::export() {
                    tracing::warn!("cannot export the contract: {error}");
                }
            }

            app.manage(AppState::new(app.handle())?);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_app_settings,
            commands::reset_app_settings,
            commands::engine_status,
            commands::download_stockfish,
            commands::choose_engine,
            commands::new_game,
            commands::suspend_game,
            commands::resume_game,
            commands::current_game,
            commands::game_frames,
            commands::make_move,
            commands::set_premove,
            commands::clear_premove,
            commands::take_back,
            commands::resign,
            commands::offer_draw,
            commands::set_analysis_position,
            commands::set_review_game,
            commands::list_games,
            commands::archive_stats,
            commands::set_game_favorite,
            commands::load_replay,
            commands::delete_game,
            commands::export_game,
            commands::import_game,
            commands::list_piece_sets,
            commands::preview_piece_sheet,
            commands::import_piece_sheet,
            commands::import_piece_folder,
            commands::delete_piece_set,
        ])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        tracing::error!("error while running tauri application: {error}");
    }
}
