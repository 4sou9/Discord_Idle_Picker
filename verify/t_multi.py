from harness import *
import re
games = [("358422126602223616","factorio.exe"),("1402416901551816837","celeste.exe"),("363431029484027904","hollow_knight.exe"),("1209665818464358430","balatro/balatro.exe"),("1402418342127472751","hades.exe")]
off0 = log_size()
procs = []
for gid, reg in games:
    path = os.path.join(RUNTIME, gid, *reg.split("/"))
    place(path)
    off = log_size()
    procs.append((gid, path, start(path, "--style", "tool")))
    time.sleep(6)
    ls = read_from(off)
    print(f"start {reg}: RunningGamesChanged={sum('Running Games Changed' in l for l in ls)} primary={[re.search(r'visibleGame=(\S+)', l).group(1) for l in ls if 'visibleGame=' in l]}")
print("waiting for heartbeat...")
time.sleep(330)
hb = [l for l in read_from(off0) if "HeartbeatManager" in l]
ids = {}
for l in hb:
    m = re.search(r"for game (\d+)", l)
    if m: ids[m.group(1)] = ids.get(m.group(1), 0) + 1
print("heartbeat per game:", ids)
for gid, path, p in reversed(procs):
    off = log_size()
    p.terminate(); p.wait()
    time.sleep(7)
    ls = read_from(off)
    print(f"stop {os.path.basename(path)}: primary->{[re.search(r'visibleGame=(\S+)', l).group(1) for l in ls if 'visibleGame=' in l]}")
    shutil.rmtree(os.path.join(RUNTIME, gid), ignore_errors=True)
