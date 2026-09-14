use std::fs;
use std::path::PathBuf;

use crate::models::{AppSettings, GameCache};

/// Writes to a temporary file and renames it over the target, so an interrupted
/// write (crash, power loss) never leaves a truncated JSON file behind.
pub fn write_atomic(path: &std::path::Path, contents: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, path)
}

fn app_data_dir() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join("com.discordidlepicker.app")
}

fn cache_path() -> PathBuf {
    app_data_dir().join("detectable_cache.json")
}

fn settings_path() -> PathBuf {
    app_data_dir().join("settings.json")
}

fn local_appdata() -> PathBuf {
    PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into()))
}

/// Named after the bundle identifier so the uninstaller's "delete app data" option
/// removes it (the WebView2 profile lives here too).
fn local_data_dir() -> PathBuf {
    local_appdata().join("com.discordidlepicker.app")
}

/// Where dummy executables are staged.
pub fn runtime_dir() -> PathBuf {
    local_data_dir().join("runtime")
}

/// Ledger of everything placed for running games (spec §6.4).
pub fn ledger_path() -> PathBuf {
    local_data_dir().join("placed.json")
}

/// `%LOCALAPPDATA%\DiscordIdlePicker`, used before the move above. Its ledger may
/// still list leftovers, so it is cleaned up once and then deleted.
pub fn legacy_local_dir() -> PathBuf {
    local_appdata().join("DiscordIdlePicker")
}

pub fn load_cache() -> Option<GameCache> {
    let content = fs::read_to_string(cache_path()).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_cache(cache: &GameCache) {
    let _ = fs::create_dir_all(app_data_dir());
    // Compact on purpose: roughly 19k entries.
    if let Ok(json) = serde_json::to_string(cache) {
        let _ = write_atomic(&cache_path(), &json);
    }
}

pub fn load_settings() -> AppSettings {
    let Ok(content) = fs::read_to_string(settings_path()) else {
        return AppSettings::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save_settings(settings: &AppSettings) {
    let _ = fs::create_dir_all(app_data_dir());
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = write_atomic(&settings_path(), &json);
    }
}
