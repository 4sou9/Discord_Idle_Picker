use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

/// Folders this app creates for uninstalled Steam games (spec §3.3). Never treated
/// as a real installation.
pub const GENERATED_DIR_PREFIX: &str = "DiscordIdlePicker_";

pub struct SteamLibrary {
    /// AppID → `<library>\steamapps\common\<installdir>` (only folders that exist).
    pub installed: HashMap<u32, PathBuf>,
}

/// `HKCU\Software\Valve\Steam\SteamPath`, if it points to an existing folder.
pub fn steam_path() -> Option<PathBuf> {
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(r"Software\Valve\Steam").ok()?;
    let value: String = key.get_value("SteamPath").ok()?;
    let path = PathBuf::from(value.replace('/', "\\"));
    path.is_dir().then_some(path)
}

/// Every library's `steamapps` folder, starting with the one inside the Steam install.
pub fn library_dirs(steam_path: &Path) -> Vec<PathBuf> {
    let mut libraries = vec![steam_path.join("steamapps")];
    if let Ok(content) = fs::read_to_string(steam_path.join("steamapps").join("libraryfolders.vdf")) {
        let re = Regex::new(r#""path"\s+"([^"]+)""#).unwrap();
        for cap in re.captures_iter(&content) {
            let dir = PathBuf::from(cap[1].replace("\\\\", "\\")).join("steamapps");
            if dir.is_dir() && !libraries.iter().any(|l| same_path(l, &dir)) {
                libraries.push(dir);
            }
        }
    }
    libraries
}

/// Spec §3.2: scans every library's appmanifest files. Works without Steam running.
pub fn scan() -> Option<SteamLibrary> {
    let steam_path = steam_path()?;

    let id_re = Regex::new(r#""appid"\s+"(\d+)""#).unwrap();
    let dir_re = Regex::new(r#""installdir"\s+"([^"]+)""#).unwrap();

    let mut installed = HashMap::new();
    for library in library_dirs(&steam_path) {
        let Ok(entries) = fs::read_dir(&library) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|f| f.to_str()) else {
                continue;
            };
            if !file_name.starts_with("appmanifest_") || !file_name.ends_with(".acf") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let (Some(id), Some(dir)) = (id_re.captures(&content), dir_re.captures(&content)) else {
                continue;
            };
            let Ok(app_id) = id[1].parse::<u32>() else {
                continue;
            };
            let install_dir_name = dir[1].replace("\\\\", "\\");
            if install_dir_name.to_ascii_lowercase().starts_with(&GENERATED_DIR_PREFIX.to_ascii_lowercase()) {
                continue;
            }
            let install_dir = library.join("common").join(install_dir_name);
            if install_dir.is_dir() {
                installed.insert(app_id, install_dir);
            }
        }
    }

    Some(SteamLibrary { installed })
}

fn same_path(a: &Path, b: &Path) -> bool {
    a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy())
}
