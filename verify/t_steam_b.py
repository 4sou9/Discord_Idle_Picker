from harness import *
STEAM = r"C:\Program Files (x86)\Steam"
appid = "2601940"
folder = STEAM + r"\steamapps\common\DiscordIdlePicker_" + appid
acf = STEAM + rf"\steamapps\appmanifest_{appid}.acf"
clog = STEAM + r"\logs\content_log.txt"
clog_off = os.path.getsize(clog)
content = f'''"AppState"
{{
\t"appid"\t\t"{appid}"
\t"universe"\t\t"1"
\t"name"\t\t"Pinball Spire"
\t"StateFlags"\t\t"4"
\t"installdir"\t\t"DiscordIdlePicker_{appid}"
}}
'''
path = place(os.path.join(folder, "discord-idle-picker-dummy.exe"))
assert not os.path.exists(acf)
open(acf, "w", encoding="utf-8").write(content)
t_acf = time.time()
time.sleep(2)
off = log_size(); p = start(path, "--style", "tool")
sec, ls = watch(off, 150, "pinball spire")
print(f"[V8-B acf] detected={sec is not None} after={sec}s (acf作成から {round(time.time()-t_acf)}s)")
for l in ls[-3:]: print("   ", l[:220])
print("--- acf now:"); print(open(acf, encoding="utf-8").read() if os.path.exists(acf) else "(消えた)")
print("--- steamapps/downloading:", os.listdir(STEAM + r"\steamapps\downloading") if os.path.exists(STEAM + r"\steamapps\downloading") else None)
with open(clog, "rb") as f:
    f.seek(clog_off); print("--- content_log new:"); print(f.read().decode("utf-8","replace")[-3000:])
input_wait = os.environ.get("KEEP")
p.terminate(); p.wait(); time.sleep(3)
os.remove(acf) if os.path.exists(acf) else None
shutil.rmtree(folder)
time.sleep(5)
with open(clog, "rb") as f:
    f.seek(clog_off); tail = f.read().decode("utf-8","replace")
print("--- content_log after cleanup (last 1500):"); print(tail[-1500:])
print("cleaned:", not os.path.exists(folder), not os.path.exists(acf))
