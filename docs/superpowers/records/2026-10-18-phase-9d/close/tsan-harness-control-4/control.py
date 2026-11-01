from pathlib import Path
import datetime,hashlib,json,os,re,subprocess,sys
root=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
close=root/'docs/superpowers/records/2026-10-18-phase-9d/close'
out=close/'tsan-harness-control-4';out.mkdir()
raw=out/'raw'
target=root/'.superpowers/sdd/2026-10-18-phase-9d/task10-targets/ee52ea5e62daeb6bc35ca4475de4d1e7a96a4bfe/tsan'
binary=target/'x86_64-unknown-linux-gnu/debug/build/rexx-exec/6c80e238cfcc855d/out/rexx_exec-6c80e238cfcc855d'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
git=lambda *a:subprocess.check_output(['git',*a],cwd=root,text=True).strip()
now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
assert not git('status','--porcelain','--untracked-files=no')
env=dict(os.environ,CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='4',RUST_TEST_THREADS='4')
assert not env.get('CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER')
command=['timeout','--kill-after=15','3000','bash',str(close/'tsan.sh'),str(target),str(raw)]
record=dict(purpose='Diagnostic control of test-harness resource concurrency. Original full-helper failure remains unaccepted; no test removed or source changed.',argv=command,cwd=str(root/'rust'),source_head=git('rev-parse','HEAD'),source_rust_tree=git('rev-parse','HEAD:rust'),binary_sha256_before=sha(binary),helper_sha256=sha(close/'tsan.sh'),suppression_sha256=sha(root/'rust/tsan.supp'),environment_change={'RUST_TEST_THREADS':'4'},cargo_build_jobs='4',resident_cap='8G',timeout_seconds=3000,started=now())
(out/'command.json').write_text(json.dumps(record,indent=2)+'\n')
with (out/'stdout').open('xb') as stdout,(out/'stderr').open('xb') as stderr:
 child=subprocess.Popen(command,cwd=root/'rust',env=env,stdout=stdout,stderr=stderr,start_new_session=True)
 record['pid']=child.pid;(out/'live.json').write_text(json.dumps(record,indent=2)+'\n')
 status=child.wait()
text=(out/'stdout').read_text();components=re.findall(r'^(api|lib|int) exit ([0-9]+)$',text,re.M);reports=sorted(str(p.relative_to(out)) for p in raw.glob('tsan.*'))
record.update(actual_shell_exit=status,components=components,reports=reports,finished=now(),binary_sha256_after=sha(binary),stdout_sha256=sha(out/'stdout'),stderr_sha256=sha(out/'stderr'),source_head_after=git('rev-parse','HEAD'),source_rust_tree_after=git('rev-parse','HEAD:rust'))
valid=status==0 and components==[('api','0'),('lib','0'),('int','0')] and not reports and record['binary_sha256_before']==record['binary_sha256_after'] and record['source_rust_tree']==record['source_rust_tree_after']
record['all_components_passed']=valid;record['gate_acceptance']='pending independent review, not inferred from diagnostic control'
(out/'status.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({'actual_shell_exit':status,'components':components,'reports':reports,'all_components_passed':valid}),flush=True)
sys.exit(0 if valid else 1)
