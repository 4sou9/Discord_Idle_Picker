import winreg, os
def dump(root, path):
    try:
        k = winreg.OpenKey(root, path)
    except OSError:
        print(path, "-> なし"); return
    i = 0
    print(path)
    while True:
        try:
            n, v, t = winreg.EnumValue(k, i); print("   ", n, "=", v); i += 1
        except OSError: break
dump(winreg.HKEY_CURRENT_USER, r"Software\Valve\Steam\Apps\4327530")
dump(winreg.HKEY_CURRENT_USER, r"Software\Valve\Steam\Apps\2601940")
dump(winreg.HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Valve\Steam\Apps\4327530")
print(open(r"D:\SteamLibrary\steamapps\appmanifest_4327530.acf", encoding="utf-8").read())
for p in [r"C:\Program Files (x86)\Steam\steamapps\appmanifest_2601940.acf", r"D:\SteamLibrary\steamapps\appmanifest_2601940.acf", r"C:\Program Files (x86)\Steam\steamapps\common\DiscordIdlePicker_2601940"]:
    print(p, os.path.exists(p))
for d in [r"C:\Program Files (x86)\Steam\steamapps\common", r"C:\Program Files (x86)\Steam\steamapps"]:
    f = os.path.join(d, "__dip_write_test.tmp")
    try:
        open(f, "w").write("x"); os.remove(f); print(d, "writable")
    except Exception as e:
        print(d, "NOT writable", e)
