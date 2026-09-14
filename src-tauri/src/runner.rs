use std::collections::HashMap;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local};
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE};

use crate::discord_log::{self, DetectionTracker, Target};
use crate::ledger::{Ledger, PlacedEntry};
use crate::models::{GameEntry, RunningState};
use crate::placement;
use crate::system::{self, Job};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const CLOSE_TIMEOUT: Duration = Duration::from_secs(1);
const DISCORD_POLL: Duration = Duration::from_secs(2);
const LOG_POLL: Duration = Duration::from_secs(1);
/// Waiting for detection right after starting a dummy (spec §6.3).
const START_DETECT_TIMEOUT: Duration = Duration::from_secs(15);
const START_DETECT_GRACE: Duration = Duration::from_secs(2);
/// Discord needs a while to load after it starts (spec §3.4).
const RESTART_DETECT_TIMEOUT: Duration = Duration::from_secs(90);
const RESTART_DETECT_GRACE: Duration = Duration::from_secs(5);

struct RunningGame {
    child: Child,
    name: String,
    exe: PathBuf,
    /// Always carries the registration for uninstalled Steam games, even when it is
    /// currently removed; `registered` says whether it exists right now.
    entry: PlacedEntry,
    registered: bool,
    /// Distinguishes restarts of the same game for background detection waits.
    token: u64,
    started_at: DateTime<Local>,
}

impl RunningGame {
    /// What the ledger should hold: the registration only while it exists.
    fn ledger_entry(&self) -> PlacedEntry {
        PlacedEntry {
            registration: if self.registered { self.entry.registration.clone() } else { None },
            ..self.entry.clone()
        }
    }
}

struct Inner {
    processes: Mutex<HashMap<String, RunningGame>>,
    dummy_exe: PathBuf,
    runtime_dir: PathBuf,
    ledger: Ledger,
    job: Option<Job>,
    next_token: AtomicU64,
    steam_restart_needed: AtomicBool,
    tracker: DetectionTracker,
}

/// Starts and stops dummy processes. Error strings are codes (`steam_not_found`,
/// `file_exists`, `already_registered`, `dummy_missing`, `invalid_path`,
/// `io:<detail>`) that the frontend localizes.
pub struct Runner {
    inner: Arc<Inner>,
}

impl Runner {
    pub fn new(dummy_exe: PathBuf, runtime_dir: PathBuf, ledger_path: PathBuf) -> Self {
        let inner = Arc::new(Inner {
            processes: Mutex::new(HashMap::new()),
            dummy_exe,
            runtime_dir,
            ledger: Ledger::open(ledger_path),
            job: Job::new(),
            next_token: AtomicU64::new(1),
            steam_restart_needed: AtomicBool::new(false),
            tracker: DetectionTracker::new(),
        });
        watch_discord(Arc::downgrade(&inner));
        follow_log(Arc::downgrade(&inner));
        Self { inner }
    }

    /// Spec §6.4: removes whatever the previous run left behind (crash or kill),
    /// including entries in the ledger of the old data folder `legacy_dir`, which is
    /// then deleted. Nothing can be running yet thanks to single-instance.
    pub fn remove_leftovers(&self, legacy_dir: &Path) {
        let mut entries = self.inner.ledger.take_all();
        if legacy_dir.is_dir() {
            entries.extend(Ledger::open(legacy_dir.join("placed.json")).take_all());
        }

        let mut registration_removed = false;
        for entry in entries {
            if let Some(registration) = &entry.registration {
                registration_removed |= placement::unregister(registration);
            }
            placement::remove_files(&entry);
        }
        let _ = std::fs::remove_dir_all(&self.inner.runtime_dir);
        let _ = std::fs::remove_dir_all(legacy_dir);
        // Steam reads appmanifests at startup; if it is already running it may have
        // picked the leftovers up (V12).
        if registration_removed && system::is_steam_running() {
            self.inner.steam_restart_needed.store(true, Ordering::Relaxed);
        }
    }

    pub fn steam_restart_needed(&self) -> bool {
        self.inner.steam_restart_needed.load(Ordering::Relaxed)
    }

    pub fn start(&self, game: &GameEntry) -> Result<(), String> {
        let inner = &self.inner;
        let id = game.game.id.clone();
        let mut processes = inner.processes.lock().unwrap();
        if let Some(running) = processes.get_mut(&id) {
            if matches!(running.child.try_wait(), Ok(None)) {
                return Ok(());
            }
            let finished = processes.remove(&id).unwrap();
            inner.clean_up(&finished);
        }

        let plan = placement::plan(game, &inner.runtime_dir)?;
        if !inner.dummy_exe.is_file() {
            return Err("dummy_missing".into());
        }
        // Without Discord there is nothing to detect the dummy; register when it starts.
        let register_now = plan.entry.registration.is_some() && system::is_discord_running();

        // Ledger first, so a crash at any later point can still be cleaned up.
        let ledger_entry = PlacedEntry {
            registration: if register_now { plan.entry.registration.clone() } else { None },
            ..plan.entry.clone()
        };
        inner.ledger.upsert(ledger_entry.clone());

        let log_offset = discord_log::current_size();
        let spawned = placement::stage(&plan, &inner.dummy_exe)
            .and_then(|()| match &ledger_entry.registration {
                Some(registration) => placement::register(registration, &game.game.name),
                None => Ok(()),
            })
            .and_then(|()| {
                Command::new(&plan.exe)
                    .current_dir(plan.exe.parent().unwrap_or(&inner.runtime_dir))
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn()
            });

        let child = match spawned {
            Ok(child) => child,
            Err(e) => {
                if let Some(registration) = &ledger_entry.registration {
                    placement::unregister(registration);
                }
                placement::remove_files(&ledger_entry);
                inner.ledger.remove(&id);
                return Err(if e.kind() == std::io::ErrorKind::AlreadyExists {
                    "file_exists".into()
                } else {
                    format!("io:{e}")
                });
            }
        };
        if let Some(job) = &inner.job {
            job.assign(&child);
        }

        let running = RunningGame {
            child,
            name: game.game.name.clone(),
            exe: plan.exe,
            entry: plan.entry,
            registered: register_now,
            token: inner.next_token.fetch_add(1, Ordering::Relaxed),
            started_at: Local::now(),
        };
        inner.tracker.track(&id, &running.exe);
        let wait = register_now.then(|| (running.token, Target::new(&running.exe, &id)));
        processes.insert(id.clone(), running);
        drop(processes);

        if let Some((token, target)) = wait {
            let weak = Arc::downgrade(inner);
            thread::spawn(move || {
                discord_log::wait_for_detection(log_offset, &[target], START_DETECT_TIMEOUT, START_DETECT_GRACE);
                if let Some(inner) = weak.upgrade() {
                    inner.remove_registrations(&[(id, token)]);
                }
            });
        }
        Ok(())
    }

    pub fn stop(&self, id: &str) {
        let removed = self.inner.processes.lock().unwrap().remove(id);
        if let Some(game) = removed {
            self.inner.shutdown(vec![game]);
        }
    }

    pub fn stop_all(&self) {
        let games: Vec<RunningGame> = self.inner.processes.lock().unwrap().drain().map(|(_, g)| g).collect();
        self.inner.shutdown(games);
    }

    pub fn running(&self) -> Vec<RunningState> {
        let inner = &self.inner;
        let mut processes = inner.processes.lock().unwrap();
        let mut finished = Vec::new();
        for (id, game) in processes.iter_mut() {
            if !matches!(game.child.try_wait(), Ok(None)) {
                finished.push(id.clone());
            }
        }
        for id in finished {
            if let Some(game) = processes.remove(&id) {
                inner.clean_up(&game);
            }
        }
        processes
            .iter()
            .map(|(id, g)| RunningState {
                id: id.clone(),
                started_at: g.started_at,
                detection: inner.tracker.detection(id),
            })
            .collect()
    }

    /// Path of a running dummy, for "open location" in the context menu.
    pub fn dummy_path(&self, id: &str) -> Option<PathBuf> {
        self.inner.processes.lock().unwrap().get(id).map(|g| g.exe.clone())
    }
}

impl Inner {
    /// Spec §6.4: ask every dummy to close, give them up to 1 second in total, then
    /// force-kill whatever is left and clean up.
    fn shutdown(&self, mut games: Vec<RunningGame>) {
        if games.is_empty() {
            return;
        }
        for game in &games {
            post_close(game.child.id());
        }
        let deadline = Instant::now() + CLOSE_TIMEOUT;
        for game in &mut games {
            while matches!(game.child.try_wait(), Ok(None)) && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(20));
            }
            if matches!(game.child.try_wait(), Ok(None)) {
                let _ = game.child.kill();
            }
            let _ = game.child.wait();
        }
        for game in &games {
            self.clean_up(game);
        }
    }

    /// Registration → files → ledger (spec §6.3 stop order). The process has exited.
    fn clean_up(&self, game: &RunningGame) {
        if game.registered {
            if let Some(registration) = &game.entry.registration {
                placement::unregister(registration);
            }
        }
        placement::remove_files(&game.entry);
        self.ledger.remove(&game.entry.id);
        self.tracker.untrack(&game.entry.id);
    }

    /// Discord has seen the dummies; the registration is no longer needed (V14).
    fn remove_registrations(&self, games: &[(String, u64)]) {
        let mut processes = self.processes.lock().unwrap();
        for (id, token) in games {
            let Some(game) = processes.get_mut(id) else {
                continue;
            };
            if game.token != *token || !game.registered {
                continue;
            }
            if let Some(registration) = &game.entry.registration {
                placement::unregister(registration);
            }
            game.registered = false;
            self.ledger.upsert(game.ledger_entry());
        }
    }

    /// Spec §3.4-2: Discord (re)started, so it will scan again. Temporarily register
    /// every running uninstalled Steam game until Discord has picked them up.
    fn reregister_for_discord(self: &Arc<Self>) {
        let log_offset = discord_log::current_size();
        let mut waiting = Vec::new();
        let mut targets = Vec::new();
        {
            let mut processes = self.processes.lock().unwrap();
            for (id, game) in processes.iter_mut() {
                let Some(registration) = game.entry.registration.clone() else {
                    continue;
                };
                if game.registered || !matches!(game.child.try_wait(), Ok(None)) {
                    continue;
                }
                game.registered = true;
                self.ledger.upsert(game.ledger_entry());
                if placement::register(&registration, &game.name).is_err() {
                    placement::unregister(&registration);
                    game.registered = false;
                    self.ledger.upsert(game.ledger_entry());
                    continue;
                }
                waiting.push((id.clone(), game.token));
                targets.push(Target::new(&game.exe, id));
            }
        }
        if waiting.is_empty() {
            return;
        }
        discord_log::wait_for_detection(log_offset, &targets, RESTART_DETECT_TIMEOUT, RESTART_DETECT_GRACE);
        self.remove_registrations(&waiting);
    }
}

fn watch_discord(inner: Weak<Inner>) {
    thread::spawn(move || {
        let mut was_running = system::is_discord_running();
        loop {
            thread::sleep(DISCORD_POLL);
            let Some(inner) = inner.upgrade() else {
                return;
            };
            let running = system::is_discord_running();
            if running && !was_running {
                inner.tracker.reset_all();
                inner.reregister_for_discord();
            }
            was_running = running;
        }
    });
}

fn follow_log(inner: Weak<Inner>) {
    thread::spawn(move || loop {
        thread::sleep(LOG_POLL);
        let Some(inner) = inner.upgrade() else {
            return;
        };
        inner.tracker.poll();
    });
}

/// Opens Explorer with the file selected.
pub fn reveal_in_explorer(path: &Path) -> std::io::Result<()> {
    let mut arg = std::ffi::OsString::from("/select,");
    arg.push(path);
    Command::new("explorer.exe").arg(arg).spawn().map(|_| ())
}

fn post_close(pid: u32) {
    unsafe extern "system" fn close_if_owned(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let mut owner = 0u32;
        GetWindowThreadProcessId(hwnd, &mut owner);
        if owner == lparam as u32 {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        }
        1
    }
    unsafe {
        EnumWindows(Some(close_if_owned), pid as LPARAM);
    }
}
