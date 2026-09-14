use std::collections::HashSet;
use std::time::Duration;

use serde::Deserialize;

use crate::models::DetectableGame;

const DETECTABLE_URL: &str = "https://discord.com/api/v9/applications/detectable";

#[derive(Deserialize)]
struct RawApp {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    aliases: Option<Vec<String>>,
    #[serde(default)]
    executables: Option<Vec<RawExecutable>>,
    #[serde(default)]
    third_party_skus: Option<Vec<RawSku>>,
}

#[derive(Deserialize)]
struct RawExecutable {
    #[serde(default)]
    name: String,
    #[serde(default)]
    os: Option<String>,
    #[serde(default)]
    is_launcher: Option<bool>,
}

#[derive(Deserialize)]
struct RawSku {
    #[serde(default)]
    distributor: Option<String>,
    #[serde(default)]
    id: Option<serde_json::Value>,
}

/// Blocking: downloads Discord's detectable list (~12.7MB) and trims it.
pub fn fetch() -> Result<Vec<DetectableGame>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .user_agent("DiscordIdlePicker")
        .build()
        .map_err(|e| e.to_string())?;
    let apps: Vec<RawApp> = client
        .get(DETECTABLE_URL)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let mut seen = HashSet::new();
    let mut games = Vec::new();
    for app in apps {
        if app.name.trim().is_empty() || !seen.insert(app.id.clone()) {
            continue;
        }
        let exe_path = app.executables.as_deref().and_then(select_best_executable);
        let steam_app_id = app.third_party_skus.as_deref().and_then(steam_app_id);
        if exe_path.is_none() && steam_app_id.is_none() {
            continue;
        }
        games.push(DetectableGame {
            id: app.id,
            name: app.name,
            aliases: app.aliases.unwrap_or_default(),
            exe_path,
            steam_app_id,
        });
    }
    games.sort_by_cached_key(|g| g.name.to_lowercase());
    Ok(games)
}

/// Spec §3.1: win32, not a `>` pattern, prefer non-launchers, then the shallowest
/// path, then the shortest name. Paths that cannot be created safely are skipped.
fn select_best_executable(executables: &[RawExecutable]) -> Option<String> {
    executables
        .iter()
        .filter(|e| e.os.as_deref() == Some("win32") && !e.name.starts_with('>'))
        .filter(|e| exe_path_components(&e.name).is_some())
        .min_by_key(|e| {
            (
                e.is_launcher.unwrap_or(false),
                e.name.matches('/').count(),
                e.name.chars().count(),
            )
        })
        .map(|e| e.name.clone())
}

fn steam_app_id(skus: &[RawSku]) -> Option<u32> {
    skus.iter()
        .filter(|s| s.distributor.as_deref() == Some("steam"))
        .find_map(|s| match s.id.as_ref()? {
            serde_json::Value::String(v) => v.parse().ok(),
            serde_json::Value::Number(v) => v.as_u64().and_then(|n| u32::try_from(n).ok()),
            _ => None,
        })
}

/// Splits a registered exe path (`dir/game.exe`) into components that are safe to
/// create under the runtime directory. The path comes from a remote API, so reject
/// anything that could escape the directory or that Windows would silently rename.
pub fn exe_path_components(path: &str) -> Option<Vec<&str>> {
    const FORBIDDEN: &str = "<>:\"\\|?*";
    let valid = |part: &&str| {
        !part.is_empty()
            && *part != "."
            && *part != ".."
            && !part.ends_with('.')
            && !part.ends_with(' ')
            && !part.chars().any(|c| c.is_control() || FORBIDDEN.contains(c))
    };
    let parts: Vec<&str> = path.split('/').collect();
    parts.iter().all(valid).then_some(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exe(name: &str, launcher: bool) -> RawExecutable {
        RawExecutable { name: name.into(), os: Some("win32".into()), is_launcher: Some(launcher) }
    }

    #[test]
    fn picks_shallowest_non_launcher() {
        let list = [exe("launcher.exe", true), exe("terraria/terraria.exe", false), exe("terraria.exe", false)];
        assert_eq!(select_best_executable(&list).as_deref(), Some("terraria.exe"));
    }

    #[test]
    fn rejects_unsafe_paths() {
        assert!(exe_path_components("../evil.exe").is_none());
        assert!(exe_path_components("c:/x.exe").is_none());
        assert!(exe_path_components("a\\b.exe").is_none());
        assert!(exe_path_components("dir//x.exe").is_none());
        assert_eq!(exe_path_components("stardew valley/stardew valley.exe"), Some(vec!["stardew valley", "stardew valley.exe"]));
    }
}
