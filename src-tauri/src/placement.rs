use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

use crate::ledger::{PlacedEntry, Registration};
use crate::models::{GameEntry, Method};
use crate::services::detectable::exe_path_components;
use crate::services::steam_library::{self, GENERATED_DIR_PREFIX};

/// File name of the dummy inside Steam game folders (spec §6.3).
pub const DUMMY_FILE_NAME: &str = "discord-idle-picker-dummy.exe";

/// Where a dummy goes and what has to be cleaned up afterwards. Error strings are
/// codes the frontend localizes.
pub struct Plan {
    pub exe: PathBuf,
    /// Folder to create before staging; `None` means the parent must already exist.
    create_dir: Option<PathBuf>,
    /// `registration` is set for uninstalled Steam games.
    pub entry: PlacedEntry,
}

pub fn plan(game: &GameEntry, runtime_dir: &Path) -> Result<Plan, String> {
    let id = &game.game.id;
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid_path".into());
    }
    let entry = |files: Vec<PathBuf>, dirs: Vec<PathBuf>, registration| PlacedEntry {
        id: id.clone(),
        files,
        dirs,
        registration,
    };

    match game.method {
        // §6.2: runtime\<DiscordID>\<registered path>
        Method::Exe => {
            let parts = game
                .game
                .exe_path
                .as_deref()
                .and_then(exe_path_components)
                .ok_or("invalid_path")?;
            let dir = runtime_dir.join(id);
            let exe = parts.iter().fold(dir.clone(), |path, part| path.join(part));
            Ok(Plan {
                create_dir: exe.parent().map(Path::to_path_buf),
                exe,
                entry: entry(vec![], vec![dir], None),
            })
        }
        Method::Steam => {
            let app_id = game.game.steam_app_id.ok_or("invalid_path")?;

            // §6.3 installed: drop the dummy into the real game folder, never overwriting.
            if let Some(install_dir) = &game.steam_install_dir {
                let install_dir = PathBuf::from(install_dir);
                if !install_dir.is_dir() {
                    return Err("steam_not_found".into());
                }
                let exe = install_dir.join(DUMMY_FILE_NAME);
                if exe.exists() {
                    return Err("file_exists".into());
                }
                return Ok(Plan {
                    exe: exe.clone(),
                    create_dir: None,
                    entry: entry(vec![exe], vec![], None),
                });
            }

            // §6.3 uninstalled: generated folder + temporary registration.
            let steam_path = steam_library::steam_path().ok_or("steam_not_found")?;
            let steamapps = steam_path.join("steamapps");
            let dir = steamapps.join("common").join(format!("{GENERATED_DIR_PREFIX}{app_id}"));
            if dir.exists() {
                return Err("file_exists".into());
            }
            let manifest_name = format!("appmanifest_{app_id}.acf");
            if steam_library::library_dirs(&steam_path)
                .iter()
                .any(|library| library.join(&manifest_name).exists())
            {
                return Err("already_registered".into());
            }
            let (key_created, prev_installed) = probe_registry(app_id)?;
            Ok(Plan {
                exe: dir.join(DUMMY_FILE_NAME),
                create_dir: Some(dir.clone()),
                entry: entry(
                    vec![],
                    vec![dir],
                    Some(Registration {
                        app_id,
                        manifest: steamapps.join(manifest_name),
                        key_created,
                        prev_installed,
                    }),
                ),
            })
        }
    }
}

/// Copies (or hard-links) the dummy to `plan.exe`. Never overwrites an existing file.
pub fn stage(plan: &Plan, dummy_exe: &Path) -> io::Result<()> {
    if let Some(dir) = &plan.create_dir {
        fs::create_dir_all(dir)?;
    }
    if plan.exe.exists() {
        return Err(io::ErrorKind::AlreadyExists.into());
    }
    if fs::hard_link(dummy_exe, &plan.exe).is_err() {
        fs::copy(dummy_exe, &plan.exe)?;
    }
    Ok(())
}

fn apps_key(app_id: u32) -> String {
    format!(r"Software\Valve\Steam\Apps\{app_id}")
}

/// Steam may already keep `Apps\<appid>` for games played before. Refuse only when it
/// says installed; otherwise remember `Installed` so it can be restored.
fn probe_registry(app_id: u32) -> Result<(bool, Option<u32>), String> {
    match RegKey::predef(HKEY_CURRENT_USER).open_subkey(apps_key(app_id)) {
        Err(_) => Ok((true, None)),
        Ok(key) => {
            let installed: Option<u32> = key.get_value("Installed").ok();
            if installed == Some(1) {
                Err("already_registered".into())
            } else {
                Ok((false, installed))
            }
        }
    }
}

/// Spec §3.3: both the appmanifest and the registry value are required for detection.
pub fn register(registration: &Registration, name: &str) -> io::Result<()> {
    let content = format!(
        "\"AppState\"\n{{\n\t\"appid\"\t\t\"{id}\"\n\t\"universe\"\t\t\"1\"\n\t\"name\"\t\t\"{name}\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"{GENERATED_DIR_PREFIX}{id}\"\n}}\n",
        id = registration.app_id,
        name = vdf_escape(name),
    );
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&registration.manifest)
        .and_then(|mut file| io::Write::write_all(&mut file, content.as_bytes()))?;

    let (key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(apps_key(registration.app_id))?;
    key.set_value("Installed", &1u32)?;
    if registration.key_created {
        key.set_value("Name", &name)?;
    }
    Ok(())
}

/// Undoes `register`. Only removes what is recognizably ours, so a real install that
/// happened in the meantime is left alone. Returns whether anything was undone.
pub fn unregister(registration: &Registration) -> bool {
    let mut undone = false;

    let marker = format!("{GENERATED_DIR_PREFIX}{}", registration.app_id);
    if fs::read_to_string(&registration.manifest).is_ok_and(|c| c.contains(&marker))
        && fs::remove_file(&registration.manifest).is_ok()
    {
        undone = true;
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = apps_key(registration.app_id);
    if registration.key_created {
        if hkcu.open_subkey(&key_path).is_ok() && hkcu.delete_subkey_all(&key_path).is_ok() {
            undone = true;
        }
    } else if let Ok(key) = hkcu.open_subkey_with_flags(&key_path, KEY_READ | KEY_WRITE) {
        if key.get_value::<u32, _>("Installed").ok() == Some(1) {
            undone = true;
            let _ = match registration.prev_installed {
                Some(value) => key.set_value("Installed", &value),
                None => key.delete_value("Installed"),
            };
        }
    }
    undone
}

/// Deletes the placed files and generated folders. The exe stays locked for a moment
/// after the process exits, hence the retries.
pub fn remove_files(entry: &PlacedEntry) {
    for file in &entry.files {
        retry(|| fs::remove_file(file));
    }
    for dir in &entry.dirs {
        retry(|| fs::remove_dir_all(dir));
    }
}

fn retry(mut op: impl FnMut() -> io::Result<()>) {
    for _ in 0..20 {
        match op() {
            Ok(()) => return,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return,
            Err(_) => thread::sleep(Duration::from_millis(50)),
        }
    }
}

fn vdf_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
