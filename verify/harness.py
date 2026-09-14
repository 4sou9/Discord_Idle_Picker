import os, sys, time, shutil, subprocess, json
LOG = os.path.expandvars(r"%APPDATA%\discord\logs\renderer_js.log")
DUMMY = r"D:\Claude\30_DISCORD_IDLE_PICKER\verify\dummy\target\release\dummy.exe"
RUNTIME = os.path.expandvars(r"%LOCALAPPDATA%\DiscordIdlePicker\runtime")
KEYS = ("Running Games Changed", "handleRunningGamesChange", "RunningGameHeartbeatManager")

def log_size():
    return os.path.getsize(LOG)

def read_from(off):
    with open(LOG, "rb") as f:
        f.seek(off)
        data = f.read().decode("utf-8", "replace")
    return [l for l in data.splitlines() if any(k in l for k in KEYS)]

def place(path, hardlink=False):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    if os.path.exists(path):
        raise SystemExit("exists: " + path)
    if hardlink:
        os.link(DUMMY, path)
    else:
        shutil.copy2(DUMMY, path)
    return path

def start(path, *args):
    return subprocess.Popen([path, *args], cwd=os.path.dirname(path), creationflags=0x08000000)

def watch(off, seconds, want=None):
    """ログを seconds 秒監視。want（小文字パス片 or ゲームID）が出たら検出までの秒数を返す"""
    t0 = time.time()
    while time.time() - t0 < seconds:
        lines = read_from(off)
        if want:
            for l in lines:
                if want in l.lower():
                    return round(time.time() - t0, 1), lines
        time.sleep(0.5)
    return None, read_from(off)
