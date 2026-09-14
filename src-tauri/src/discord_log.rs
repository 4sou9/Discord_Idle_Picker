use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(250);

fn log_path() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join("discord").join("logs").join("renderer_js.log")
}

/// Current length of `renderer_js.log`; take this *before* the event to watch for.
pub fn current_size() -> u64 {
    std::fs::metadata(log_path()).map(|m| m.len()).unwrap_or(0)
}

/// What identifies one dummy in the log (spec §7).
pub struct Target {
    /// Full exe path, lowercase with forward slashes (Discord's `newPrimaryKey` form).
    path: String,
    /// `for game <DiscordID>` from the heartbeat line.
    heartbeat: String,
}

impl Target {
    pub fn new(exe: &std::path::Path, discord_id: &str) -> Self {
        Self {
            path: exe.to_string_lossy().to_lowercase().replace('\\', "/"),
            heartbeat: format!("for game {discord_id}"),
        }
    }
}

/// Blocks until every target shows up in the log, or until `grace` has passed since
/// the first `Running Games Changed` (non-primary games never log their path), or
/// until `timeout`. Returns whether Discord reacted at all.
pub fn wait_for_detection(offset: u64, targets: &[Target], timeout: Duration, grace: Duration) -> bool {
    let started = Instant::now();
    let mut pos = offset;
    let mut pending = String::new();
    let mut detected = vec![false; targets.len()];
    let mut changed_at: Option<Instant> = None;

    loop {
        for line in read_new_lines(&mut pos, &mut pending) {
            let line = line.to_lowercase();
            if changed_at.is_none() && line.contains("running games changed") {
                changed_at = Some(Instant::now());
            }
            for (i, target) in targets.iter().enumerate() {
                if !detected[i] && (line.contains(&target.path) || line.contains(&target.heartbeat)) {
                    detected[i] = true;
                }
            }
        }

        if detected.iter().all(|d| *d) {
            return true;
        }
        if changed_at.is_some_and(|t| t.elapsed() >= grace) {
            return true;
        }
        if started.elapsed() >= timeout {
            return changed_at.is_some() || detected.iter().any(|d| *d);
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn read_new_lines(pos: &mut u64, pending: &mut String) -> Vec<String> {
    let Ok(mut file) = File::open(log_path()) else {
        return Vec::new();
    };
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    if len < *pos {
        // Log was rotated.
        *pos = 0;
        pending.clear();
    }
    if file.seek(SeekFrom::Start(*pos)).is_err() {
        return Vec::new();
    }
    let mut buf = Vec::new();
    let Ok(read) = file.read_to_end(&mut buf) else {
        return Vec::new();
    };
    *pos += read as u64;
    pending.push_str(&String::from_utf8_lossy(&buf));

    let mut lines: Vec<String> = pending.split('\n').map(str::to_owned).collect();
    *pending = lines.pop().unwrap_or_default();
    lines
}

/// Detection state shown in the list (spec §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Detection {
    /// Discord has not reacted yet, or reacted without identifying the game.
    Pending,
    /// The dummy's path or Discord ID appeared in the log.
    Detected,
    /// No `Running Games Changed` within `NOT_DETECTED_AFTER` of starting.
    NotDetected,
}

const NOT_DETECTED_AFTER: Duration = Duration::from_secs(15);

struct Track {
    target: Target,
    started: Instant,
    changed: bool,
    detection: Detection,
}

#[derive(Default)]
struct TrackerState {
    pos: u64,
    pending: String,
    tracks: std::collections::HashMap<String, Track>,
}

/// Follows `renderer_js.log` for the whole session. The log format is not a public
/// contract: if nothing matches, games simply stay `Pending` and starting/stopping is
/// unaffected.
pub struct DetectionTracker {
    state: std::sync::Mutex<TrackerState>,
}

impl DetectionTracker {
    pub fn new() -> Self {
        Self {
            state: std::sync::Mutex::new(TrackerState {
                pos: current_size(),
                ..Default::default()
            }),
        }
    }

    pub fn track(&self, id: &str, exe: &std::path::Path) {
        let mut state = self.state.lock().unwrap();
        state.tracks.insert(
            id.to_owned(),
            Track {
                target: Target::new(exe, id),
                started: Instant::now(),
                changed: false,
                detection: Detection::Pending,
            },
        );
    }

    pub fn untrack(&self, id: &str) {
        self.state.lock().unwrap().tracks.remove(id);
    }

    pub fn detection(&self, id: &str) -> Detection {
        self.state
            .lock()
            .unwrap()
            .tracks
            .get(id)
            .map_or(Detection::Pending, |t| t.detection)
    }

    /// Discord (re)started and will detect everything again.
    pub fn reset_all(&self) {
        let mut state = self.state.lock().unwrap();
        for track in state.tracks.values_mut() {
            track.started = Instant::now();
            track.changed = false;
            track.detection = Detection::Pending;
        }
    }

    pub fn poll(&self) {
        let mut guard = self.state.lock().unwrap();
        let state = &mut *guard;
        let lines = read_new_lines(&mut state.pos, &mut state.pending);
        for line in lines {
            let line = line.to_lowercase();
            let changed = line.contains("running games changed");
            for track in state.tracks.values_mut() {
                if line.contains(&track.target.path) || line.contains(&track.target.heartbeat) {
                    track.detection = Detection::Detected;
                } else if changed && track.started.elapsed() <= NOT_DETECTED_AFTER {
                    // Attribute the change only to games started just before it.
                    track.changed = true;
                    if track.detection == Detection::NotDetected {
                        track.detection = Detection::Pending;
                    }
                }
            }
        }
        for track in state.tracks.values_mut() {
            if track.detection == Detection::Pending && !track.changed && track.started.elapsed() > NOT_DETECTED_AFTER {
                track.detection = Detection::NotDetected;
            }
        }
    }
}
