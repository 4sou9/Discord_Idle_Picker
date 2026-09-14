from harness import *
tests = [
  # (label, discord_id, reg_path, style, title)
  ("V6+app/exe名タイトル", "358422126602223616", "factorio.exe", "app", None),
  ("V1 tool", "1402416901551816837", "celeste.exe", "tool", None),
  ("hidden", "363431029484027904", "hollow_knight.exe", "hidden", None),
  ("V2 app/ゲーム名タイトル", "1209665818464358430", "balatro/balatro.exe", "app", "Balatro"),
]
only = sys.argv[1:] 
for label, gid, reg, style, title in tests:
    if only and label.split()[0] not in only: continue
    path = os.path.join(RUNTIME, gid, *reg.split("/"))
    place(path)
    off = log_size()
    args = ["--style", style] + (["--title", title] if title else [])
    p = start(path, *args)
    want = path.lower().replace("\\", "/")
    sec, lines = watch(off, 20, want)
    print(f"[{label}] detected={sec is not None} after={sec}s")
    for l in lines[-4:]: print("   ", l[:230])
    off2 = log_size(); t = time.time()
    p.terminate(); p.wait()
    sec2, lines2 = watch(off2, 30, "visible game became null")
    print(f"   stop -> null after={sec2}s")
    time.sleep(2)
    shutil.rmtree(os.path.join(RUNTIME, gid), ignore_errors=True)
