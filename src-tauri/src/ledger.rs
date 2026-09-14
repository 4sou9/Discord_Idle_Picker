use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// Temporary Steam registration for an uninstalled game (spec §3.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    pub app_id: u32,
    pub manifest: PathBuf,
    /// `HKCU\Software\Valve\Steam\Apps\<appid>` did not exist and was created by us.
    pub key_created: bool,
    /// `Installed` before we set it to 1, when the key already existed.
    pub prev_installed: Option<u32>,
}

/// Everything placed for one game, so it can be removed after a crash (spec §6.4).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacedEntry {
    pub id: String,
    /// Files to delete (a dummy placed inside a real game folder).
    #[serde(default)]
    pub files: Vec<PathBuf>,
    /// Folders created by this app, deleted recursively.
    #[serde(default)]
    pub dirs: Vec<PathBuf>,
    /// Present only while the registration exists.
    #[serde(default)]
    pub registration: Option<Registration>,
}

/// `%LOCALAPPDATA%\DiscordIdlePicker\placed.json`, rewritten on every change.
pub struct Ledger {
    path: PathBuf,
    entries: Mutex<HashMap<String, PlacedEntry>>,
}

impl Ledger {
    pub fn open(path: PathBuf) -> Self {
        let entries = fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str::<Vec<PlacedEntry>>(&c).ok())
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.id.clone(), e))
            .collect();
        Self {
            path,
            entries: Mutex::new(entries),
        }
    }

    pub fn upsert(&self, entry: PlacedEntry) {
        let mut entries = self.entries.lock().unwrap();
        entries.insert(entry.id.clone(), entry);
        self.save(&entries);
    }

    pub fn remove(&self, id: &str) {
        let mut entries = self.entries.lock().unwrap();
        if entries.remove(id).is_some() {
            self.save(&entries);
        }
    }

    /// Leftovers from the previous run; the ledger is emptied.
    pub fn take_all(&self) -> Vec<PlacedEntry> {
        let mut entries = self.entries.lock().unwrap();
        let all = entries.drain().map(|(_, e)| e).collect();
        self.save(&entries);
        all
    }

    fn save(&self, entries: &HashMap<String, PlacedEntry>) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let list: Vec<&PlacedEntry> = entries.values().collect();
        if let Ok(json) = serde_json::to_string_pretty(&list) {
            let _ = crate::services::storage::write_atomic(&self.path, &json);
        }
    }
}
