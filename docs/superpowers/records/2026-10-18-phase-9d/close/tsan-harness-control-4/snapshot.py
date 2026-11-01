from pathlib import Path
import subprocess,json,datetime
p=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/close/tsan-harness-control-4')
command=['timeout','--kill-after=5','25','gdb','-nx','-batch','-ex','set pagination off','-ex','set debuginfod enabled off','-ex','set print frame-arguments none','-ex','thread apply all bt 20','-ex','detach','-p','301014']
r={'purpose':'Read-only stack snapshot of the already failed owned sanitizer diagnostic. No target function calls or target memory writes. Frame argument values disabled.','argv':command,'started':datetime.datetime.now(datetime.timezone.utc).isoformat(),'perturbation':'Debugger attach briefly pauses threads in the already failed diagnostic; not gate acceptance evidence.'}
with (p/'failed-process-stack.stdout').open('xb') as out,(p/'failed-process-stack.stderr').open('xb') as err:
 c=subprocess.run(command,stdout=out,stderr=err)
r.update(actual_exit=c.returncode,finished=datetime.datetime.now(datetime.timezone.utc).isoformat(),detach_seen='detached' in (p/'failed-process-stack.stdout').read_text().lower())
(p/'failed-process-stack.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
