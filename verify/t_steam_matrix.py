from harness import *
import winreg, glob
STEAM = r"C:\Program Files (x86)\Steam"
def logs_snapshot():
    return {f: os.path.getsize(f) for f in glob.glob(STEAM + r"\logs\*.txt")}
def logs_grep(snap, needle):
    hits = []
    for f, off in snap.items():
        try:
            with open(f, "rb") as h:
                h.seek(off); t = h.read().decode("utf-8", "replace")
            hits += [os.path.basename(f) + ": " + l for l in t.splitlines() if needle in l]
        except OSError: pass
    return hits

def run(label, appid, name, use_reg, use_acf, wait=60, hold=0):
    folder = STEAM + r"\steamapps\common\DiscordIdlePicker_" + appid
    acf = STEAM + rf"\steamapps\appmanifest_{appid}.acf"
    key_path = r"Software\Valve\Steam\Apps\\" + appid
    assert not os.path.exists(acf) and not os.path.exists(folder)
    snap = logs_snapshot()
    path = place(os.path.join(folder, "discord-idle-picker-dummy.exe"))
    if use_acf:
        open(acf, "w", encoding="utf-8").write(f'"AppState"\n{{\n\t"appid"\t\t"{appid}"\n\t"universe"\t\t"1"\n\t"name"\t\t"{name}"\n\t"StateFlags"\t\t"4"\n\t"installdir"\t\t"DiscordIdlePicker_{appid}"\n}}\n')
    if use_reg:
        k = winreg.CreateKey(winreg.HKEY_CURRENT_USER, key_path)
        winreg.SetValueEx(k, "Installed", 0, winreg.REG_DWORD, 1)
        winreg.SetValueEx(k, "Name", 0, winreg.REG_SZ, name)
        winreg.CloseKey(k)
    off = log_size(); p = start(path, "--style", "tool")   # 登録直後に起動（待ちなし）
    sec, ls = watch(off, wait, name.lower())
    print(f"[{label}] {name} reg={use_reg} acf={use_acf} detected={sec is not None} after={sec}s")
    time.sleep(hold)
    if use_acf:
        now = open(acf, encoding="utf-8").read()
        print("   acf modified by Steam:", "StateFlags\"\t\t\"4\"" not in now or now.count("\n") > 8)
    if use_reg:
        k = winreg.OpenKey(winreg.HKEY_CURRENT_USER, key_path)
        print("   reg:", [winreg.EnumValue(k, i)[:2] for i in range(winreg.QueryInfoKey(k)[1])]); winreg.CloseKey(k)
    for h in logs_grep(snap, appid)[:10]: print("   steamlog:", h[:200])
    p.terminate(); p.wait(); time.sleep(3)
    if use_acf: os.remove(acf)
    shutil.rmtree(folder)
    if use_reg: winreg.DeleteKey(winreg.HKEY_CURRENT_USER, key_path)
    time.sleep(3)
    for h in logs_grep(snap, appid)[10:20]: print("   steamlog(after):", h[:200])

run("A reg only", "2230820", "Sands", True, False)
run("B acf only", "715380", "Those Who Remain", False, True)
run("C both", "1560500", "Badlanders", True, True, hold=30)
run("C both #2", "366320", "Seasons after Fall", True, True)
