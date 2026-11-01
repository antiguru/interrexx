from pathlib import Path
import subprocess,json,datetime
p=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/close/tsan-longchain-observation/solo')
r={'purpose':'Read-only bounded native-stack sample of the owned long-chain diagnostic, not acceptance evidence.','started':datetime.datetime.now(datetime.timezone.utc).isoformat(),'perturbation':'Debugger pauses native execution briefly; actual pause is not separately measured. No target function calls or memory writes; frame argument values hidden.'}
if (p/'status.json').exists():
 r['skipped']='Test completed before snapshot';(p/'stack-snapshot.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r));raise SystemExit(0)
d=json.loads((p/'live.json').read_text());pgid=d['pid'];matches=[]
for line in subprocess.check_output(['ps','-eo','pid,pgid,comm'],text=True).splitlines()[1:]:
 a=line.split()
 if len(a)==3 and a[1]==str(pgid) and a[2].startswith('rexx_exec'):
  try:
   if 'task10-targets/ee52ea5e' in str(Path('/proc',a[0],'exe').resolve()):matches.append(int(a[0]))
  except FileNotFoundError:pass
assert len(matches)==1,matches
pid=matches[0];command=['timeout','--kill-after=5','25','gdb','-nx','-batch','-ex','set pagination off','-ex','set debuginfod enabled off','-ex','set print frame-arguments none','-ex','thread apply all bt 128','-ex','detach','-p',str(pid)]
r.update(argv=command,pid=pid,owned_pgid=pgid)
with (p/'stack-snapshot.stdout').open('xb') as out,(p/'stack-snapshot.stderr').open('xb') as err:c=subprocess.run(command,stdout=out,stderr=err)
r.update(actual_exit=c.returncode,finished=datetime.datetime.now(datetime.timezone.utc).isoformat(),detach_seen='detached' in (p/'stack-snapshot.stdout').read_text().lower())
(p/'stack-snapshot.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
