from harness import *
import winreg, glob, ctypes, ctypes.wintypes as w
from PIL import Image
STEAM = r"C:\Program Files (x86)\Steam"
STEAM_EXE = STEAM + r"\steam.exe"
SHOTS = r"D:\Claude\30_DISCORD_IDLE_PICKER\verify\shots"
GAMES = [("354240", "Please, Don't Touch Anything"), ("1560500", "Badlanders")]
u = ctypes.windll.user32; u.SetProcessDPIAware()

def steam_running():
    out = subprocess.run(["tasklist", "/FI", "IMAGENAME eq steam.exe"], capture_output=True, text=True, encoding="cp932").stdout
    return "steam.exe" in out.lower()
def steam_rect():
    found = []
    def cb(h, l):
        if u.IsWindowVisible(h):
            n = u.GetWindowTextLengthW(h); b = ctypes.create_unicode_buffer(n + 1); u.GetWindowTextW(h, b, n + 1)
            r = w.RECT(); u.GetWindowRect(h, ctypes.byref(r))
            if b.value == "Steam" and r.right - r.left > 600: found.append((r.left, r.top, r.right, r.bottom))
        return True
    u.EnumWindows(ctypes.WINFUNCTYPE(ctypes.c_bool, w.HWND, w.LPARAM)(cb), 0)
    return found[0] if found else None
def shot(name):
    full = os.path.join(SHOTS, "_full.png")
    subprocess.run(["powershell", "-NoProfile", "-File", "shot.ps1", full])
    r = steam_rect(); vl = u.GetSystemMetrics(76); vt = u.GetSystemMetrics(77)
    if r:
        im = Image.open(full).crop((r[0]-vl, r[1]-vt, r[2]-vl, r[3]-vt)); im.thumbnail((1400, 1400)); im.save(os.path.join(SHOTS, name))
    os.remove(full)
    print("   shot:", name, "steam window:", r)
def paths(appid):
    return (STEAM + r"\steamapps\common\DiscordIdlePicker_" + appid, STEAM + rf"\steamapps\appmanifest_{appid}.acf", r"Software\Valve\Steam\Apps\\" + appid)
def reg_values(key_path):
    try:
        k = winreg.OpenKey(winreg.HKEY_CURRENT_USER, key_path)
        v = {winreg.EnumValue(k, i)[0]: winreg.EnumValue(k, i)[1] for i in range(winreg.QueryInfoKey(k)[1])}; winreg.CloseKey(k); return v
    except OSError: return None
def state(label):
    print(f"--- state: {label}")
    for appid, name in GAMES:
        folder, acf, key = paths(appid)
        acf_txt = open(acf, encoding="utf-8").read() if os.path.exists(acf) else None
        print(f"   {appid}: folder={os.path.exists(folder)} files={os.listdir(folder) if os.path.exists(folder) else None} reg={reg_values(key)}")
        print(f"   {appid}: acf=" + ("(なし)" if acf_txt is None else ("未変更" if acf_txt.count(chr(10)) == 8 else "\n" + acf_txt)))
    print("   downloading:", sorted(os.listdir(STEAM + r"\steamapps\downloading")))
def logs_snapshot():
    return {f: os.path.getsize(f) for f in glob.glob(STEAM + r"\logs\*.txt")}
def logs_new(snap):
    out = []
    for f in glob.glob(STEAM + r"\logs\*.txt"):
        off = snap.get(f, 0)
        if os.path.getsize(f) < off: off = 0
        with open(f, "rb") as h:
            h.seek(off); t = h.read().decode("utf-8", "replace")
        out += [os.path.basename(f) + ": " + l for l in t.splitlines() if any(a in l for a, _ in GAMES)]
    return out

step = sys.argv[1]
if step == "setup":
    for appid, name in GAMES:
        folder, acf, key = paths(appid)
        assert not os.path.exists(acf) and not os.path.exists(folder) and reg_values(key) is None, appid
    shot("v12_0_before.png") if steam_rect() else None
    for appid, name in GAMES:
        folder, acf, key = paths(appid)
        place(os.path.join(folder, "discord-idle-picker-dummy.exe"))
        open(acf, "w", encoding="utf-8").write(f'"AppState"\n{{\n\t"appid"\t\t"{appid}"\n\t"universe"\t\t"1"\n\t"name"\t\t"{name}"\n\t"StateFlags"\t\t"4"\n\t"installdir"\t\t"DiscordIdlePicker_{appid}"\n}}\n')
        k = winreg.CreateKey(winreg.HKEY_CURRENT_USER, key)
        winreg.SetValueEx(k, "Installed", 0, winreg.REG_DWORD, 1); winreg.SetValueEx(k, "Name", 0, winreg.REG_SZ, name); winreg.CloseKey(k)
    state("生成直後")
    json.dump(logs_snapshot(), open("v12_logsnap.json", "w"))
elif step == "restart":
    snap = json.load(open("v12_logsnap.json"))
    subprocess.Popen([STEAM_EXE, "-shutdown"])
    t = time.time()
    while steam_running() and time.time() - t < 90: time.sleep(1)
    print("steam stopped:", not steam_running(), round(time.time() - t), "s")
    state("Steam 終了後")
    subprocess.Popen([STEAM_EXE], creationflags=0x00000008)  # DETACHED_PROCESS
    t = time.time()
    while not steam_rect() and time.time() - t < 180: time.sleep(2)
    print("steam window up after", round(time.time() - t), "s")
    for i in range(12):   # 2分監視
        time.sleep(10)
        dls = sorted(os.listdir(STEAM + r"\steamapps\downloading"))
        new = [a for a, _ in GAMES if a in dls]
        if new:
            print(f"!!! t={10*(i+1)}s download dir created: {new}"); break
    state("Steam 起動から約2分後")
    for l in logs_new(snap)[-40:]: print("   steamlog:", l[:250])
    for appid, _ in GAMES:
        os.startfile(f"steam://nav/games/details/{appid}"); time.sleep(6); shot(f"v12_1_after_restart_{appid}.png")
elif step == "cleanup":
    snap = logs_snapshot()
    for appid, name in GAMES:
        folder, acf, key = paths(appid)
        if os.path.exists(acf): os.remove(acf)
        shutil.rmtree(folder, ignore_errors=True)
        try: winreg.DeleteKey(winreg.HKEY_CURRENT_USER, key)
        except OSError as e: print("reg delete", appid, e)
    time.sleep(15)
    state("後始末から15秒後")
    for l in logs_new(snap)[-20:]: print("   steamlog:", l[:250])
    for appid, _ in GAMES:
        os.startfile(f"steam://nav/games/details/{appid}"); time.sleep(6); shot(f"v12_2_after_cleanup_{appid}.png")
if step == "restore":
    snap = logs_snapshot()
    subprocess.Popen([STEAM_EXE, "-shutdown"])
    t = time.time()
    while steam_running() and time.time() - t < 90: time.sleep(1)
    print("steam stopped:", not steam_running())
    state("Steam 終了後（書き戻し確認）")
    for appid, _ in GAMES:
        folder, acf, key = paths(appid)
        if os.path.exists(acf): print("   Steam が acf を書き戻した → 削除", appid); os.remove(acf)
        if os.path.exists(folder): print("   フォルダが復活 → 削除", appid); shutil.rmtree(folder)
        if reg_values(key) is not None: print("   レジストリあり", appid, reg_values(key))
    subprocess.Popen([STEAM_EXE], creationflags=0x00000008)
    t = time.time()
    while not steam_rect() and time.time() - t < 180: time.sleep(2)
    time.sleep(60)
    state("Steam 再起動から約1分後")
    for l in logs_new(snap)[-20:]: print("   steamlog:", l[:250])
    for appid, _ in GAMES:
        os.startfile(f"steam://nav/games/details/{appid}"); time.sleep(6); shot(f"v12_3_after_restore_{appid}.png")
