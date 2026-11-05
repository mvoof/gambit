//! The settings file. A missing or damaged file means the defaults: a bad file
//! must not stop the app from starting.

use std::fs;
use std::io;
use std::ops::RangeInclusive;
use std::path::Path;

use gb_model::AppSettings;
use serde_json::Value;

const MAX_SOUND_VOLUME: u32 = 100;
const ANALYSIS_LINES: RangeInclusive<u32> = 1..=5;
const ANALYSIS_DEPTH: RangeInclusive<u32> = 12..=40;
const TEMP_EXTENSION: &str = "tmp";

/// Lays the saved values over the defaults, nested objects included.
fn overlay(base: &mut Value, saved: Value) {
    match (base, saved) {
        (Value::Object(base_fields), Value::Object(saved_fields)) => {
            for (name, value) in saved_fields {
                match base_fields.get_mut(&name) {
                    Some(field) => overlay(field, value),
                    None => {
                        base_fields.insert(name, value);
                    }
                }
            }
        }
        (slot, value) => *slot = value,
    }
}

/// A file from an older version lacks the newer fields: they keep their defaults.
fn merged_with_defaults(text: &str) -> Option<AppSettings> {
    let saved = serde_json::from_str(text).ok()?;
    let mut merged = serde_json::to_value(AppSettings::default()).ok()?;

    overlay(&mut merged, saved);

    serde_json::from_value(merged).ok()
}

pub fn load_settings(path: &Path) -> AppSettings {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| merged_with_defaults(&text))
        .unwrap_or_default()
}

pub fn normalise(mut settings: AppSettings) -> AppSettings {
    settings.sound_volume = settings.sound_volume.min(MAX_SOUND_VOLUME);
    settings.analysis_lines = settings
        .analysis_lines
        .clamp(*ANALYSIS_LINES.start(), *ANALYSIS_LINES.end());
    settings.analysis_depth = settings
        .analysis_depth
        .clamp(*ANALYSIS_DEPTH.start(), *ANALYSIS_DEPTH.end());

    settings
}

/// Through a temporary file, so a crash cannot leave half a settings file.
pub fn save_settings(path: &Path, settings: &AppSettings) -> io::Result<()> {
    let text = serde_json::to_string_pretty(settings)?;
    let temporary = path.with_extension(TEMP_EXTENSION);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&temporary, text)?;
    fs::rename(&temporary, path)
}

/// The defaults, except what the user would have to find again.
pub fn reset(current: &AppSettings) -> AppSettings {
    AppSettings {
        language: current.language.clone(),
        engine_path: current.engine_path.clone(),
        last_game: current.last_game.clone(),
        ..AppSettings::default()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn settings_survive_a_round_trip() {
        let folder = tempdir().unwrap();
        let path = folder.path().join("nested").join("settings.json");
        let settings = AppSettings {
            piece_set: "merida".to_owned(),
            ..AppSettings::default()
        };

        save_settings(&path, &settings).unwrap();

        assert_eq!(load_settings(&path), settings);
    }

    #[test]
    fn a_missing_or_damaged_file_gives_the_defaults() {
        let folder = tempdir().unwrap();
        let path = folder.path().join("settings.json");

        assert_eq!(load_settings(&path), AppSettings::default());

        fs::write(&path, "{ not json").unwrap();

        assert_eq!(load_settings(&path), AppSettings::default());
    }

    #[test]
    fn a_file_from_an_older_version_keeps_the_new_defaults() {
        let settings =
            merged_with_defaults(r#"{"soundVolume":5,"board":{"showCheck":false}}"#).unwrap();

        assert_eq!(settings.sound_volume, 5);
        assert!(!settings.board.show_check);
        assert!(settings.board.show_coordinates);
        assert_eq!(settings.piece_set, AppSettings::default().piece_set);
    }

    #[test]
    fn a_field_this_version_no_longer_has_is_ignored() {
        let settings = merged_with_defaults(r#"{"soundVolume":5,"llm":{"enabled":true}}"#).unwrap();

        assert_eq!(settings.sound_volume, 5);
    }

    #[test]
    fn a_reset_keeps_the_language_and_the_engine() {
        let settings = AppSettings {
            language: "ru".to_owned(),
            engine_path: Some("C:/engines/my.exe".to_owned()),
            sound_volume: 5,
            piece_set: "merida".to_owned(),
            ..AppSettings::default()
        };
        let fresh = reset(&settings);

        assert_eq!(fresh.language, "ru");
        assert_eq!(fresh.engine_path, settings.engine_path);
        assert_eq!(fresh.sound_volume, AppSettings::default().sound_volume);
        assert_eq!(fresh.piece_set, AppSettings::default().piece_set);
    }

    #[test]
    fn the_analysis_settings_stay_in_range() {
        let settings = AppSettings {
            analysis_lines: 0,
            analysis_depth: 900,
            ..AppSettings::default()
        };
        let normal = normalise(settings);

        assert_eq!(normal.analysis_lines, *ANALYSIS_LINES.start());
        assert_eq!(normal.analysis_depth, *ANALYSIS_DEPTH.end());
    }

    #[test]
    fn the_volume_is_capped() {
        let settings = AppSettings {
            sound_volume: 900,
            ..AppSettings::default()
        };

        assert_eq!(normalise(settings).sound_volume, MAX_SOUND_VOLUME);
    }
}
