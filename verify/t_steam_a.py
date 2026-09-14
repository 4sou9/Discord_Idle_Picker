from harness import *
import winreg
# V4: インストール済み + ハードリンク
gid = "1473517429639348436"
path = r"D:\SteamLibrary\steamapps\common\WelcomeToTheGuildExplorers\discord-idle-picker-dummy.exe"
place(path, hardlink=True)
off = log_size(); p = start(path, "--style", "tool")
sec, ls = watch(off, 30, path.lower().replace("\\", "/"))
print(f"[V4 installed+hardlink] detected={sec is not None} after={sec}s")
for l in ls[-2:]: print("   ", l[:200])
p.terminate(); p.wait(); time.sleep(6); os.remove(path)

# V8-A: 未インストール + フォルダ + レジストリのみ
appid = "2601940"; gid = "1428178445224906862"
folder = r"C:\Program Files (x86)\Steam\steamapps\common\DiscordIdlePicker_" + appid
key_path = r"Software\Valve\Steam\Apps\\" + appid
path = place(os.path.join(folder, "discord-idle-picker-dummy.exe"))
k = winreg.CreateKey(winreg.HKEY_CURRENT_USER, key_path)
winreg.SetValueEx(k, "Installed", 0, winreg.REG_DWORD, 1)
winreg.SetValueEx(k, "Name", 0, winreg.REG_SZ, "Pinball Spire")
winreg.CloseKey(k)
time.sleep(2)
off = log_size(); p = start(path, "--style", "tool")
sec, ls = watch(off, 90, "pinball spire")
print(f"[V8-A registry only] detected={sec is not None} after={sec}s")
for l in ls[-3:]: print("   ", l[:200])
k = winreg.OpenKey(winreg.HKEY_CURRENT_USER, key_path)
print("   registry still:", [winreg.EnumValue(k, i) for i in range(winreg.QueryInfoKey(k)[1])])
winreg.CloseKey(k)
p.terminate(); p.wait(); time.sleep(3)
shutil.rmtree(folder)
winreg.DeleteKey(winreg.HKEY_CURRENT_USER, key_path)
print("cleaned:", not os.path.exists(folder))
