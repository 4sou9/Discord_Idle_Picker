use crate::models::{GameCache, GameEntry, GamesPayload, Method};

use super::steam_library;

/// Combines the cached detectable list with the current Steam library scan.
/// EXE wins over STEAM when both apply (spec §3).
pub fn build(cache: GameCache) -> GamesPayload {
    let steam = steam_library::scan();

    let games = cache
        .games
        .into_iter()
        .map(|game| {
            let install_dir = game
                .steam_app_id
                .and_then(|id| steam.as_ref()?.installed.get(&id))
                .map(|p| p.to_string_lossy().into_owned());
            let method = if game.exe_path.is_some() { Method::Exe } else { Method::Steam };
            GameEntry {
                game,
                method,
                steam_installed: install_dir.is_some(),
                steam_install_dir: install_dir,
            }
        })
        .collect();

    GamesPayload {
        games,
        fetched_at: cache.fetched_at,
        steam_found: steam.is_some(),
    }
}
