use std::collections::HashMap;
use std::sync::Mutex;

use tauri::State;

use crate::models::{AppSettings, GameCache, GameEntry, GamesPayload, RunningState};
use crate::runner::{self, Runner};
use crate::services::{catalog, detectable, storage};
use crate::system;

/// The last list sent to the frontend, so commands can take just a Discord ID.
#[derive(Default)]
pub struct GameIndex(Mutex<HashMap<String, GameEntry>>);

impl GameIndex {
    fn replace(&self, games: &[GameEntry]) {
        let mut map = self.0.lock().unwrap();
        map.clear();
        map.extend(games.iter().map(|g| (g.game.id.clone(), g.clone())));
    }

    fn get(&self, id: &str) -> Option<GameEntry> {
        self.0.lock().unwrap().get(id).cloned()
    }
}

#[tauri::command]
pub async fn load_games(index: State<'_, GameIndex>) -> Result<Option<GamesPayload>, String> {
    let payload = tauri::async_runtime::spawn_blocking(|| storage::load_cache().map(catalog::build))
        .await
        .map_err(|e| e.to_string())?;
    if let Some(payload) = &payload {
        index.replace(&payload.games);
    }
    Ok(payload)
}

#[tauri::command]
pub async fn refresh_games(index: State<'_, GameIndex>) -> Result<GamesPayload, String> {
    let payload = tauri::async_runtime::spawn_blocking(|| {
        let games = detectable::fetch()?;
        let cache = GameCache {
            fetched_at: chrono::Local::now(),
            games,
        };
        storage::save_cache(&cache);
        Ok::<_, String>(catalog::build(cache))
    })
    .await
    .map_err(|e| e.to_string())??;
    index.replace(&payload.games);
    Ok(payload)
}

#[tauri::command]
pub fn start_game(runner: State<Runner>, index: State<GameIndex>, id: String) -> Result<(), String> {
    let game = index.get(&id).ok_or("unknown_game")?;
    runner.start(&game)
}

#[tauri::command]
pub fn stop_game(runner: State<Runner>, id: String) {
    runner.stop(&id);
}

#[tauri::command]
pub fn stop_all(runner: State<Runner>) {
    runner.stop_all();
}

#[tauri::command]
pub fn get_running(runner: State<Runner>) -> Vec<RunningState> {
    runner.running()
}

#[tauri::command]
pub fn is_discord_running() -> bool {
    system::is_discord_running()
}

/// Set when leftover registrations were removed while Steam was running (spec §3.4).
#[tauri::command]
pub fn steam_restart_needed(runner: State<Runner>) -> bool {
    runner.steam_restart_needed()
}

#[tauri::command]
pub fn open_steam_store(app_id: u32) -> Result<(), String> {
    // Built here from the number so arbitrary URLs cannot be opened.
    let url = format!("https://store.steampowered.com/app/{app_id}/");
    std::process::Command::new("explorer.exe")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("io:{e}"))
}

#[tauri::command]
pub fn reveal_dummy(runner: State<Runner>, id: String) -> Result<(), String> {
    let path = runner.dummy_path(&id).ok_or("unknown_game")?;
    runner::reveal_in_explorer(&path).map_err(|e| format!("io:{e}"))
}

#[tauri::command]
pub fn load_settings() -> AppSettings {
    storage::load_settings()
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) {
    storage::save_settings(&settings);
}
