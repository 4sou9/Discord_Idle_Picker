import os, glob, json, winreg, subprocess, sys
APPIDS = sys.argv[1:] or ["4327530", "405820"]
la = os.path.expandvars(r"%LOCALAPPDATA%\com.discordidlepicker.app")
ledger = os.path.join(la, "placed.json")
print("ledger:", json.dumps(json.load(open(ledger, encoding="utf-8")), ensure_ascii=False) if os.path.exists(ledger) else "(none)")
rt = os.path.join(la, "runtime")
print("runtime files:", [os.path.relpath(os.path.join(r, f), rt) for r, _, fs in os.walk(rt) for f in fs] if os.path.exists(rt) else [])
print("generated dirs:", glob.glob(r"C:\Program Files (x86)\Steam\steamapps\common\DiscordIdlePicker_*"))
print("dummy in guild:", os.path.exists(r"D:\SteamLibrary\steamapps\common\WelcomeToTheGuildExplorers\discord-idle-picker-dummy.exe"))
for a in APPIDS:
    acf = [p for p in (rf"C:\Program Files (x86)\Steam\steamapps\appmanifest_{a}.acf",) if os.path.exists(p)]
    try:
        k = winreg.OpenKey(winreg.HKEY_CURRENT_USER, rf"Software\Valve\Steam\Apps\{a}")
        reg = {winreg.EnumValue(k, i)[0]: winreg.EnumValue(k, i)[1] for i in range(winreg.QueryInfoKey(k)[1])}
    except OSError:
        reg = None
    if a != "4327530": print(f"  {a}: acf={bool(acf)} reg={reg}")
out = subprocess.run(["tasklist"], capture_output=True, text=True, encoding="cp932").stdout.lower()
print("procs:", sorted({l.split()[0] for l in out.splitlines() if "dummy" in l or "discord-idle" in l or "factorio" in l}))
