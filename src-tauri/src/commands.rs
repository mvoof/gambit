//! The Tauri command surface: everything the frontend may call, in one module.
//! Keep commands thin — they validate input and delegate, they do not hold
//! business logic.

use serde::Serialize;

/// Returned to the frontend as `AppInfo` in `src/lib/tauri.ts`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub tauri_version: String,
}

#[tauri::command]
pub fn greet(name: String) -> String {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return "Hello from Rust!".to_string();
    }

    format!("Hello, {trimmed}! You have been greeted from Rust.")
}

#[tauri::command]
pub fn get_app_info(app: tauri::AppHandle) -> AppInfo {
    let package_info = app.package_info();

    AppInfo {
        name: package_info.name.clone(),
        version: package_info.version.to_string(),
        tauri_version: tauri::VERSION.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets_a_named_user() {
        assert_eq!(
            greet("  Ada  ".to_string()),
            "Hello, Ada! You have been greeted from Rust."
        );
    }

    #[test]
    fn falls_back_when_the_name_is_blank() {
        assert_eq!(greet("   ".to_string()), "Hello from Rust!");
    }
}
