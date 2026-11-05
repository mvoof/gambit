//! Application wiring: builds the Tauri app and registers what it needs.

#[cfg(feature = "dev")]
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());

    #[cfg(feature = "dev")]
    let builder = builder
        .plugin(tauri_plugin_mcp_bridge::init())
        .setup(|app| {
            app.add_capability(include_str!("../capabilities-dev/mcp-bridge.json"))?;

            Ok(())
        });

    if let Err(error) = builder.run(tauri::generate_context!()) {
        eprintln!("error while running tauri application: {error}");
    }
}
