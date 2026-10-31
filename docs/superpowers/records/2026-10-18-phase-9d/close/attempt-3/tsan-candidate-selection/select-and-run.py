"""Authorized finite Cargo diagnostic selections. Not closing gate acceptance."""
from pathlib import Path
import datetime,hashlib,json,os,re,signal,subprocess,sys,time
ROOT=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
CLOSE=ROOT/'docs/superpowers/records/2026-10-18-phase-9d/close';OUT=Path(__file__).resolve().parent
TARGET=ROOT/'.superpowers/sdd/2026-10-18-phase-9d/task10-targets/ee52ea5e62daeb6bc35ca4475de4d1e7a96a4bfe/tsan'
BINARY=TARGET/'x86_64-unknown-linux-gnu/debug/build/rexx-exec/6c80e238cfcc855d/out/rexx_exec-6c80e238cfcc855d'
sha=lambda path:hashlib.sha256(Path(path).read_bytes()).hexdigest()
pattern=r'\btest (\S+?)(?: - should panic)? \.\.\. (?:ok|FAILED|ignored)\b'
if len(sys.argv)==1:
 normal=(CLOSE/'G6-test-debug.stdout').read_text();start=normal.index('running 1241 tests');end=normal.index('test result: ok. 1241 passed',start);normal_names=set(re.findall(pattern,normal[start:end],re.M));assert len(normal_names)==1241
 helper=(CLOSE/'tsan.sh').read_text();skips=set(re.search('DEPTH="([^"]+)"',helper).group(1).split()+re.search('TIMING="([^"]+)"',helper).group(1).split());assert len(skips)==15
 original_done=set(re.findall(pattern,(CLOSE/'tsan/lib.txt').read_text(),re.M));replay_done=set(re.findall(pattern,(CLOSE/'tsan-library-replay-1/stdout').read_text(),re.M))
 selected=sorted((normal_names-skips-original_done)&(normal_names-skips-replay_done));assert len(selected)==21
 inputs=['G6-test-debug.stdout','tsan.sh','tsan/lib.txt','tsan-library-replay-1/stdout']
 record={'purpose':'Diagnostic selection only. Original full gate remains failed.','normal_names':sorted(normal_names),'exact_historical_skips':sorted(skips),'historical_skips_absent_from_current_normal_names':sorted(skips-normal_names),'original_unfinished':sorted(normal_names-skips-original_done),'replay_unfinished':sorted(normal_names-skips-replay_done),'selected':selected,'input_sha256':{name:sha(CLOSE/name) for name in inputs}}
 (OUT/'selection.json').write_text(json.dumps(record,indent=2)+'\n');label='group-1'
else:
 label=sys.argv[1];selected=json.loads((OUT/(label+'-names.json')).read_text());assert selected and set(selected)<=set(json.loads((OUT/'selection.json').read_text())['selected'])
case=OUT/label;case.mkdir(exist_ok=False)
env=dict(os.environ,CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='4',CARGO_TARGET_DIR=str(TARGET),RUSTFLAGS='-Zsanitizer=thread',REXX_TEST_THREAD_SANITIZER='1')
for key in ['RUST_TEST_THREADS','RUST_TEST_NOCAPTURE','CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER','REXX_POOL_FAILURE_CHILD']:assert not env.get(key),(key,env.get(key))
env['TSAN_OPTIONS']=f'log_path={case}/tsan suppressions={ROOT}/rust/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1'
command=['timeout','--kill-after=15','180','cargo','+nightly','test','-Zbuild-std','--target','x86_64-unknown-linux-gnu','-p','rexx-exec','--lib','--','--exact',*selected]
record={'purpose':'Diagnostic only, not a gate retry or acceptance.','argv':command,'cwd':str(ROOT/'rust'),'selected':selected,'environment':{key:env[key] for key in ['CARGO_INCREMENTAL','CARGO_BUILD_JOBS','CARGO_TARGET_DIR','RUSTFLAGS','REXX_TEST_THREAD_SANITIZER','TSAN_OPTIONS']},'binary_sha256_before':sha(BINARY),'suppression_sha256':sha(ROOT/'rust/tsan.supp'),'started':datetime.datetime.now(datetime.timezone.utc).isoformat(),'resident_cap':'8G'}
(case/'command.json').write_text(json.dumps(record,indent=2)+'\n');start=time.monotonic()
with (case/'stdout').open('wb') as stdout,(case/'stderr').open('wb') as stderr:
 child=subprocess.Popen(command,cwd=ROOT/'rust',env=env,stdin=subprocess.DEVNULL,stdout=stdout,stderr=stderr,start_new_session=True)
 record['pid']=child.pid
 rebuild=False
 try:
  while child.poll() is None:
   if re.search(r'^\s*Compiling ',(case/'stderr').read_text(),re.M):
    rebuild=True;os.killpg(child.pid,signal.SIGKILL);break
   time.sleep(.1)
  status=child.wait()
 finally:
  try:os.killpg(child.pid,signal.SIGKILL)
  except ProcessLookupError:pass
  if child.poll() is None:child.wait()
text=(case/'stdout').read_text();reports=sorted(p.name for p in case.glob('tsan.*'));completed=set(re.findall(pattern,text,re.M))
record.update(status=status,elapsed_seconds=time.monotonic()-start,finished=datetime.datetime.now(datetime.timezone.utc).isoformat(),unexpected_compilation=rebuild,reports=reports,binary_sha256_after=sha(BINARY),stdout_sha256=sha(case/'stdout'),stderr_sha256=sha(case/'stderr'),completed=sorted(completed),unfinished=sorted(set(selected)-completed))
(case/'status.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'group':label,'status':status,'seconds':record['elapsed_seconds'],'reports':reports,'unfinished':record['unfinished'],'unexpected_compilation':rebuild}),flush=True)
