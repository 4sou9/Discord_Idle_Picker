from harness import *
import winreg, glob
STEAM = r"C:\Program Files (x86)\Steam"
SHOTS = r"D:\Claude\30_DISCORD_IDLE_PICKER\verify\shots"
os.makedirs(SHOTS, exist_ok=True)
def shot(name):
    subprocess.run(["powershell", "-NoProfile", "-File", "shot.ps1", os.path.join(SHOTS, name)])
def nav(appid):
    os.startfile(f"steam://nav/games/details/{appid}")
def logs_snapshot():
    return {f: os.path.getsize(f) for f in glob.glob(STEAM + r"\logs\*.txt")}
def logs_new(snap, needle):
    out = []
    for f, off in snap.items():
        with open(f, "rb") as h:
            h.seek(off); t = h.read().decode("utf-8", "replace")
        out += [os.path.basename(f) + ": " + l for l in t.splitlines() if needle in l]
    return out
def dl():
    return sorted(os.listdir(STEAM + r"\steamapps\downloading"))

appid, name, label = sys.argv[1], sys.argv[2], sys.argv[3]
folder = STEAM + r"\steamapps\common\DiscordIdlePicker_" + appid
acf = STEAM + rf"\steamapps\appmanifest_{appid}.acf"
key_path = r"Software\Valve\Steam\Apps\\" + appid
assert not os.path.exists(acf) and not os.path.exists(folder)
try:
    winreg.OpenKey(winreg.HKEY_CURRENT_USER, key_path); existed = True
except OSError: existed = False
print("reg key existed before:", existed)
if existed:
    k = winreg.OpenKey(winreg.HKEY_CURRENT_USER, key_path)
    print("   before:", [winreg.EnumValue(k, i)[:2] for i in range(winreg.QueryInfoKey(k)[1])])
dl0 = dl(); print("downloading before:", dl0)

nav(appid); time.sleep(5); shot(f"{label}_1_before.png")

snap = logs_snapshot()
path = place(os.path.join(folder, "discord-idle-picker-dummy.exe"))
open(acf, "w", encoding="utf-8").write(f'"AppState"\n{{\n\t"appid"\t\t"{appid}"\n\t"universe"\t\t"1"\n\t"name"\t\t"{name}"\n\t"StateFlags"\t\t"4"\n\t"installdir"\t\t"DiscordIdlePicker_{appid}"\n}}\n')
k = winreg.CreateKey(winreg.HKEY_CURRENT_USER, key_path)
if not existed:
    winreg.SetValueEx(k, "Name", 0, winreg.REG_SZ, name)
prev_installed = None
try: prev_installed = winreg.QueryValueEx(k, "Installed")[0]
except OSError: pass
winreg.SetValueEx(k, "Installed", 0, winreg.REG_DWORD, 1)
winreg.CloseKey(k)
off = log_size(); p = start(path)
aborted = False
try:
    sec, _ = watch(off, 20, name.lower())
    print(f"detected={sec is not None} after={sec}s")
    for i in range(12):   # 60秒監視
        time.sleep(5)
        new_dl = [d for d in dl() if d not in dl0]
        acf_now = open(acf, encoding="utf-8").read() if os.path.exists(acf) else "(消えた)"
        sl = logs_new(snap, appid)
        if i in (1, 6):
            nav(appid); time.sleep(4); shot(f"{label}_2_during_{i}.png")
        if new_dl or acf_now.count("\n") > 8 or sl:
            print(f"  t={5*(i+1)}s CHANGE dl={new_dl} acf_lines={acf_now.count(chr(10))}")
            for l in sl[-15:]: print("   steamlog:", l[:220])
            print(acf_now)
            if new_dl or any("download" in l.lower() or "update" in l.lower() for l in sl):
                aborted = True; break
    print("aborted:", aborted)
finally:
    p.terminate(); p.wait(); time.sleep(2)
    if os.path.exists(acf): os.remove(acf)
    shutil.rmtree(folder, ignore_errors=True)
    if existed:
        k = winreg.OpenKey(winreg.HKEY_CURRENT_USER, key_path, 0, winreg.KEY_SET_VALUE)
        if prev_installed is None: winreg.DeleteValue(k, "Installed")
        else: winreg.SetValueEx(k, "Installed", 0, winreg.REG_DWORD, prev_installed)
        winreg.CloseKey(k)
    else:
        winreg.DeleteKey(winreg.HKEY_CURRENT_USER, key_path)
    time.sleep(5)
    for d in [d for d in dl() if d not in dl0]:
        print("new downloading dir left:", d)
    print("steamlog total:"); [print("   ", l[:220]) for l in logs_new(snap, appid)[-20:]]
    nav(appid); time.sleep(4); shot(f"{label}_3_after.png")
    print("cleaned:", not os.path.exists(acf), not os.path.exists(folder))
