use serde::{Deserialize, Serialize};

/// One entry of Discord's detectable list, trimmed to what the app needs.
/// Entries with neither `exe_path` nor `steam_app_id` are dropped at fetch time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectableGame {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exe_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steam_app_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Exe,
    Steam,
}

/// A detectable game combined with the local Steam library state.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEntry {
    #[serde(flatten)]
    pub game: DetectableGame,
    pub method: Method,
    pub steam_installed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steam_install_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameCache {
    pub fetched_at: chrono::DateTime<chrono::Local>,
    pub games: Vec<DetectableGame>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamesPayload {
    pub games: Vec<GameEntry>,
    pub fetched_at: chrono::DateTime<chrono::Local>,
    pub steam_found: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub language: String,
    pub filter: String,
    pub selected_games: Vec<String>,
    pub favorites: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "ja".into(),
            filter: "all".into(),
            selected_games: Vec::new(),
            favorites: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningState {
    pub id: String,
    pub started_at: chrono::DateTime<chrono::Local>,
    pub detection: crate::discord_log::Detection,
}
