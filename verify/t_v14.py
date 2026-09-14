from harness import *
import winreg, re
STEAM = r"C:\Program Files (x86)\Steam"
appid, name, gid = "405820", "Turok", "1402416831914053733"
folder = STEAM + r"\steamapps\common\DiscordIdlePicker_" + appid
acf = STEAM + rf"\steamapps\appmanifest_{appid}.acf"
key_path = r"Software\Valve\Steam\Apps\\" + appid
def log(msg): print(time.strftime("%H:%M:%S"), msg, flush=True)
def register():
    open(acf, "w", encoding="utf-8").write(f'"AppState"\n{{\n\t"appid"\t\t"{appid}"\n\t"universe"\t\t"1"\n\t"name"\t\t"{name}"\n\t"StateFlags"\t\t"4"\n\t"installdir"\t\t"DiscordIdlePicker_{appid}"\n}}\n')
    k = winreg.CreateKey(winreg.HKEY_CURRENT_USER, key_path)
    winreg.SetValueEx(k, "Installed", 0, winreg.REG_DWORD, 1); winreg.SetValueEx(k, "Name", 0, winreg.REG_SZ, name); winreg.CloseKey(k)
def unregister():
    if os.path.exists(acf): os.remove(acf)
    try: winreg.DeleteKey(winreg.HKEY_CURRENT_USER, key_path)
    except OSError: pass
def relevant(lines):
    return [l for l in lines if gid in l or "turok" in l.lower() or "Running Games Changed" in l or "became null" in l]

assert not os.path.exists(acf) and not os.path.exists(folder)
path = place(os.path.join(folder, "discord-idle-picker-dummy.exe"))
p = None
try:
    register()
    off = log_size(); p = start(path)
    sec, _ = watch(off, 20, "turok")
    log(f"detected={sec is not None} after={sec}s")
    if sec is None: raise SystemExit("not detected")
    time.sleep(1)
    unregister()
    log("登録情報を削除（ダミーは起動したまま）")
    off_del = log_size()
    t0 = time.time()
    seen = 0
    while time.time() - t0 < 660:      # 11分監視
        time.sleep(5)
        lines = relevant(read_from(off_del))
        for l in lines[seen:]: log("   log: " + l[:200])
        seen = len(lines)
    hb = [l for l in read_from(off_del) if "HeartbeatManager" in l and gid in l]
    lost = [l for l in read_from(off_del) if "became null" in l]
    log(f"削除後11分: heartbeat({gid})={len(hb)} visible-null={len(lost)}")

    off_re = log_size()
    register()
    log("登録情報を作り直し")
    time.sleep(30)
    for l in relevant(read_from(off_re)): log("   log: " + l[:200])
    log(f"作り直し後30秒: 変化={len(relevant(read_from(off_re)))}件")
    unregister()
    log("再度削除")
finally:
    off_stop = log_size()
    if p: p.terminate(); p.wait()
    unregister()
    time.sleep(8)
    for l in relevant(read_from(off_stop)): log("   stop log: " + l[:200])
    shutil.rmtree(folder, ignore_errors=True)
    log(f"cleaned: acf={not os.path.exists(acf)} folder={not os.path.exists(folder)}")
