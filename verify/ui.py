import ctypes, ctypes.wintypes as w, sys, time
u = ctypes.windll.user32; u.SetProcessDPIAware()
class KI(ctypes.Structure): _fields_=[("wVk",w.WORD),("wScan",w.WORD),("dwFlags",w.DWORD),("time",w.DWORD),("dwExtraInfo",ctypes.c_size_t)]
class MI(ctypes.Structure): _fields_=[("dx",w.LONG),("dy",w.LONG),("mouseData",w.DWORD),("dwFlags",w.DWORD),("time",w.DWORD),("dwExtraInfo",ctypes.c_size_t)]
class U(ctypes.Union): _fields_=[("ki",KI),("mi",MI),("pad",ctypes.c_byte*32)]
class INPUT(ctypes.Structure): _fields_=[("type",w.DWORD),("u",U)]
def send(*inputs):
    arr=(INPUT*len(inputs))(*inputs); u.SendInput(len(inputs), arr, ctypes.sizeof(INPUT))
def rect():
    import os; h=u.FindWindowW(None, os.environ.get("UI_TITLE", "Discord Idle Picker")); u.SetForegroundWindow(h); r=w.RECT(); u.GetWindowRect(h,ctypes.byref(r)); return r
def click(x,y):
    r=rect(); u.SetCursorPos(r.left+x, r.top+y); time.sleep(0.1)
    a=INPUT(type=0); a.u.mi=MI(0,0,0,0x2,0,0); b=INPUT(type=0); b.u.mi=MI(0,0,0,0x4,0,0); send(a,b); time.sleep(0.3)
def typ(text):
    for ch in text:
        a=INPUT(type=1); a.u.ki=KI(0,ord(ch),0x4,0,0); b=INPUT(type=1); b.u.ki=KI(0,ord(ch),0x6,0,0); send(a,b); time.sleep(0.02)
    time.sleep(0.4)
def key(vk):
    a=INPUT(type=1); a.u.ki=KI(vk,0,0,0,0); b=INPUT(type=1); b.u.ki=KI(vk,0,2,0,0); send(a,b); time.sleep(0.2)
for cmd in sys.argv[1:]:
    op,_,arg=cmd.partition("=")
    if op=="click": x,y=map(int,arg.split(",")); click(x,y)
    elif op=="rclick":
        x,y=map(int,arg.split(",")); r=rect(); u.SetCursorPos(r.left+x, r.top+y); time.sleep(0.1)
        a=INPUT(type=0); a.u.mi=MI(0,0,0,0x8,0,0); b=INPUT(type=0); b.u.mi=MI(0,0,0,0x10,0,0); send(a,b); time.sleep(0.4)
    elif op=="move":
        x,y=map(int,arg.split(",")); r=rect(); u.SetCursorPos(r.left+x, r.top+y); time.sleep(0.5)
    elif op=="type": typ(arg)
    elif op=="selectall": ctrl=INPUT(type=1); ctrl.u.ki=KI(0x11,0,0,0,0); key_a=INPUT(type=1); key_a.u.ki=KI(0x41,0,0,0,0); ku=INPUT(type=1); ku.u.ki=KI(0x41,0,2,0,0); cu=INPUT(type=1); cu.u.ki=KI(0x11,0,2,0,0); send(ctrl,key_a,ku,cu); time.sleep(0.2)
    elif op=="del": key(0x2E)
    elif op=="vk": key(int(arg, 16))
    elif op=="sleep": time.sleep(float(arg))
