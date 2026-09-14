from harness import *
import winreg
STEAM = r"C:\Program Files (x86)\Steam"
appid = "2601940"
folder = STEAM + r"\steamapps\common\DiscordIdlePicker_" + appid
acf = STEAM + rf"\steamapps\appmanifest_{appid}.acf"
key_path = r"Software\Valve\Steam\Apps\\" + appid
content = f'"AppState"\n{{\n\t"appid"\t\t"{appid}"\n\t"universe"\t\t"1"\n\t"name"\t\t"Pinball Spire"\n\t"StateFlags"\t\t"4"\n\t"installdir"\t\t"DiscordIdlePicker_{appid}"\n}}\n'
path = place(os.path.join(folder, "discord-idle-picker-dummy.exe"))
open(acf, "w", encoding="utf-8").write(content)
k = winreg.CreateKey(winreg.HKEY_CURRENT_USER, key_path)
winreg.SetValueEx(k, "Installed", 0, winreg.REG_DWORD, 1)
winreg.SetValueEx(k, "Name", 0, winreg.REG_SZ, "Pinball Spire")
winreg.CloseKey(k)
time.sleep(3)
off = log_size(); p = start(path, "--style", "tool")
sec, ls = watch(off, 90, "pinball spire")
print(f"[V8-C registry+acf] detected={sec is not None} after={sec}s")
for l in ls[-3:]: print("   ", l[:220])
if os.environ.get("KEEP"):
    print("KEEP: leaving in place, pid", p.pid); sys.exit()
p.terminate(); p.wait(); time.sleep(3)
os.remove(acf); shutil.rmtree(folder); winreg.DeleteKey(winreg.HKEY_CURRENT_USER, key_path)
print("cleaned")
