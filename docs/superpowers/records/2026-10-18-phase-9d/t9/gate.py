import json, os, pathlib, signal, subprocess, sys, time
root=pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
out=root/'docs/superpowers/records/2026-10-18-phase-9d/t9'
name=sys.argv[1]
commands={
'fmt':['cargo','fmt','--all','--check'],
'preflight':['python3','scripts/test_preflight.py'],
'policy':['cargo','test','-j4','-p','rexx-exec','--test','closed_phases','--test','phase_ten_reasons','--test','refusal_dispositions','--test','refusal_sites','--test','representation_refusals','--test','owners','--test','internal_routines','--test','concurrency_tests'],
'core':['cargo','test','-j4','-p','rexx-core','--test','unsafe_sites','--lib'],
'clippy':['cargo','clippy','-j4','-p','rexx-core','-p','rexx-exec','--all-targets','--','-D','warnings'],
'release-build':['cargo','build','--release','-j4','-p','rexx-exec','--bins'],
'corpus':['cargo','test','--release','-j4','-p','rexx-exec','--test','corpus'],
'suite':['cargo','test','--release','-j4','-p','rexx-exec','--test','suite','l3_core_suite','--','--exact','--nocapture'],
'native':['cargo','test','-j4','-p','rexx-exec','--lib','dispatch::native::tests'],
}
env={k:v for k,v in os.environ.items() if not k.startswith('REXX_')}
env.update(CARGO_INCREMENTAL='0',CARGO_TARGET_DIR='/tmp/codex-p9d-t88-target',CARGO_BUILD_JOBS='4')
if name=='corpus':env['REXX_CORPUS_GATE']='1'
if name=='suite':env.update(REXX_SUITE_GATE='1',REXX_SUITE_TABLE=str(out/'suite/table.tsv'),REXX_SUITE_DIAGNOSTICS=str(out/'suite-diagnostics'))
start=time.monotonic();cmd=commands[name]
assert not (out/(name+'.json')).exists(), 'Gate name already recorded'
with (out/(name+'.stdout')).open('x') as stdout,(out/(name+'.stderr')).open('x') as stderr:
 proc=subprocess.Popen(cmd,cwd=root/'rust',env=env,stdout=stdout,stderr=stderr,start_new_session=True)
 try:status=proc.wait(timeout=525)
 except subprocess.TimeoutExpired:
  os.killpg(proc.pid,signal.SIGKILL)
  proc.wait()
  status=124
record={'command':cmd,'cwd':str(root/'rust'),'status':status,'elapsed_seconds':time.monotonic()-start,'bound_seconds':525,'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()}
(out/(name+'.json')).write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record));sys.exit(status)
