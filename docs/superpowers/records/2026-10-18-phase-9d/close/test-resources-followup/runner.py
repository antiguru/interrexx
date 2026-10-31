from pathlib import Path
import datetime, hashlib, json, os, subprocess, sys, time
ROOT=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
LOG=ROOT/'docs/superpowers/records/2026-10-18-phase-9d/close/test-resources-followup'
TARGET=ROOT/'.superpowers/sdd/2026-10-18-phase-9d/task10-targets/ee52ea5e62daeb6bc35ca4475de4d1e7a96a4bfe'
phase=sys.argv[1]
env=os.environ.copy()
for k in ['RUSTFLAGS','REXX_TEST_THREAD_SANITIZER','TSAN_OPTIONS']:
 env.pop(k,None)
env.update(CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='4',CARGO_TARGET_DIR=str(TARGET/'workspace'))
C=['cargo']; T=['cargo','test','-j','4','-p','rexx-exec','--lib']
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def run(name,args,seconds,cap,updates={}):
 d=LOG/name;d.mkdir(); e=env.copy();e.update(updates)
 argv=['timeout','--kill-after=15',str(seconds),'memcap',cap]+args
 paths=['rust/crates/rexx-exec/src/tests.rs','rust/crates/rexx-exec/src/tests/resources.rs','rust/crates/rexx-exec/src/test_process.rs','rust/crates/rexx-exec/src/lib.rs','rust/tsan.supp','docs/superpowers/records/2026-10-18-phase-9d/close/tsan.sh']
 info={'argv':argv,'cwd':str(ROOT/'rust'),'environment':{k:e.get(k) for k in ['CARGO_INCREMENTAL','CARGO_BUILD_JOBS','CARGO_TARGET_DIR','RUSTFLAGS','REXX_TEST_THREAD_SANITIZER','TSAN_OPTIONS']},'source_sha256':{p:digest(ROOT/p) for p in paths},'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'started':datetime.datetime.now(datetime.timezone.utc).isoformat(),'resident_cap':cap,'bound_seconds':seconds}
 (d/'command.json').write_text(json.dumps(info,indent=2)+'\n')
 print('START '+name,flush=True);start=time.monotonic()
 with (d/'stdout.txt').open('wb') as out,(d/'stderr.txt').open('wb') as err:
  result=subprocess.run(argv,cwd=ROOT/'rust',env=e,stdout=out,stderr=err)
 status={'actual_exit':result.returncode,'elapsed_seconds':time.monotonic()-start,'finished':datetime.datetime.now(datetime.timezone.utc).isoformat()}
 bins={}
 for profile in ['debug','release','x86_64-unknown-linux-gnu/debug']:
  for p in [p for pattern in [profile+'/deps/rexx_exec-*',profile+'/build/rexx-exec/*/out/rexx_exec-*'] for p in Path(e['CARGO_TARGET_DIR']).glob(pattern)]:
   if p.is_file() and p.suffix=='' and os.access(p,os.X_OK): bins[str(p)]=digest(p)
 status['binary_sha256']=bins
 (d/'status.json').write_text(json.dumps(status,indent=2)+'\n');print('END '+name+' exit='+str(status['actual_exit'])+' elapsed='+str(status['elapsed_seconds']),flush=True)
 if result.returncode: sys.exit(result.returncode)
if phase=='normal':
 run('debug-build',T+['--no-run'],600,'16G')
 run('debug-focused',T+['tests::resources::'],300,'8G')
 run('debug-other-marker',T+['tests::resources::'],300,'8G',{'REXX_TEST_THREAD_SANITIZER':'0'})
 run('release-build',T+['--release','--no-run'],600,'16G')
 run('release-focused',T+['--release','tests::resources::'],300,'8G')
 run('ordinary-library',T,2400,'8G')
 run('format',C+['fmt','--all','--check'],600,'8G')
 run('clippy',C+['clippy','-j','4','-p','rexx-exec','--all-targets','--','-D','warnings'],600,'16G')
elif phase=='tsan':
 env.update(CARGO_TARGET_DIR=str(TARGET/'tsan'),RUSTFLAGS='-Zsanitizer=thread',REXX_TEST_THREAD_SANITIZER='1')
 ts=['cargo','+nightly','test','-Zbuild-std','--target','x86_64-unknown-linux-gnu','-p','rexx-exec','--lib']
 run('tsan-build',ts+['--no-run'],600,'16G')
 run('tsan-focused',ts+['tests::resources::'],300,'8G',{'TSAN_OPTIONS':f'log_path={LOG}/tsan-focused/tsan suppressions={ROOT}/rust/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1'})
elif phase=='helper':
 env.update(CARGO_TARGET_DIR=str(TARGET/'tsan'),RUSTFLAGS='-Zsanitizer=thread',REXX_TEST_THREAD_SANITIZER='1')
 run('tsan-helper',['bash',str(ROOT/'docs/superpowers/records/2026-10-18-phase-9d/close/tsan.sh'),str(TARGET/'tsan'),str(LOG/'tsan-helper'/'raw')],3000,'8G')
