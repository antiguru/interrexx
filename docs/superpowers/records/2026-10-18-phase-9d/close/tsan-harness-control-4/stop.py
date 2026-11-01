from pathlib import Path
import datetime,json,os,signal,time
p=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/close/tsan-harness-control-4')
d=json.loads((p/'live.json').read_text());pid=d['pid'];assert pid==300897
assert os.getpgid(pid)==pid
assert b'/close/tsan.sh' in Path('/proc',str(pid),'cmdline').read_bytes()
assert any('ThreadSanitizer: SEGV' in f.read_text() for f in (p/'raw').glob('tsan.*'))
assert json.loads((p/'failed-process-stack.json').read_text())['detach_seen']
now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
r={'purpose':'Stop only the conclusively failed diagnostic after its fatal sanitizer report and read-only detached stack snapshot. This is a manual diagnostic stop, not natural timeout or acceptance.','pgid':pid,'started':now(),'signals':[]}
os.killpg(pid,signal.SIGTERM);r['signals'].append('SIGTERM')
def alive():
 try:os.killpg(pid,0);return True
 except ProcessLookupError:return False
until=time.monotonic()+5
while alive() and time.monotonic()<until:time.sleep(.1)
if alive():os.killpg(pid,signal.SIGKILL);r['signals'].append('SIGKILL')
r.update(finished=now(),group_still_visible=alive())
(p/'manual-stop.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
