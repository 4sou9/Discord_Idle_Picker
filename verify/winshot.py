import ctypes, ctypes.wintypes as w, subprocess, sys, os, time
from PIL import Image
u = ctypes.windll.user32; u.SetProcessDPIAware()
title, out = sys.argv[1], sys.argv[2]
hwnd = u.FindWindowW(None, title)
if not hwnd: sys.exit("window not found")
u.ShowWindow(hwnd, 9); u.SetForegroundWindow(hwnd); time.sleep(0.8)
r = w.RECT(); u.GetWindowRect(hwnd, ctypes.byref(r))
full = out + ".full.png"
subprocess.run(["powershell", "-NoProfile", "-File", "shot.ps1", full])
vl, vt = u.GetSystemMetrics(76), u.GetSystemMetrics(77)
Image.open(full).crop((r.left-vl, r.top-vt, r.right-vl, r.bottom-vt)).save(out)
os.remove(full); print("saved", out, (r.left, r.top, r.right, r.bottom))
