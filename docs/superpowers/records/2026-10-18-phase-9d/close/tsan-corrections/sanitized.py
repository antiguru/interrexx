# Copyright (c) 2026 Rexx Language Association. All rights reserved.
# This program and the accompanying materials are made available under the
# terms of the Common Public License v1.0 which accompanies this distribution.
# A copy is also available at https://www.oorexx.org/license.html.
from pathlib import Path
import json,os,subprocess
ROOT=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite');OUT=Path(__file__).resolve().parent
TARGET=ROOT/'.superpowers/sdd/2026-10-18-phase-9d/task10-targets/9922e56f3bf6c72442d796f8a43f631a86621abd/tsan'
HEAD=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip()
env=dict(os.environ,CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='4',CARGO_TARGET_DIR=str(TARGET),RUSTFLAGS='-Zsanitizer=thread',REXX_TEST_THREAD_SANITIZER='1')
env.pop('REXX_POOL_FAILURE_CHILD',None)
def run(name,args,seconds=120):
 path=OUT/name;assert not path.with_suffix('.status').exists()
 case=OUT/(name+'-logs');case.mkdir(exist_ok=False)
 env['TSAN_OPTIONS']=f'log_path={case}/tsan suppressions={ROOT}/rust/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1'
 command=['timeout','-k','5',str(seconds),*args]
 with path.with_suffix('.stdout').open('wb') as stdout,path.with_suffix('.stderr').open('wb') as stderr:
  result=subprocess.run(command,cwd=ROOT/'rust',env=env,stdin=subprocess.DEVNULL,stdout=stdout,stderr=stderr)
 reports=[p.name for p in case.glob('tsan.*')]
 path.with_suffix('.status').write_text(str(result.returncode)+'\n')
 path.with_suffix('.json').write_text(json.dumps({'command':command,'status':result.returncode,'source_commit':HEAD,'outer_cap':'8G','marker':env['REXX_TEST_THREAD_SANITIZER'],'child_marker':env.get('REXX_POOL_FAILURE_CHILD'),'TSAN_OPTIONS':env['TSAN_OPTIONS'],'reports':reports},indent=2)+'\n')
 print(name,result.returncode,reports,flush=True)
 assert result.returncode==0 and not reports
 return path.with_suffix('.stdout').read_text()
text=run('tsan-build',['cargo','+nightly','test','-Zbuild-std','--target','x86_64-unknown-linux-gnu','-p','rexx-exec','--lib','--test','signals','--test','concurrency_tests','--no-run','--message-format=json','-j4'],580)
artifacts={}
for line in text.splitlines():
 try:record=json.loads(line)
 except json.JSONDecodeError:continue
 if record.get('reason')=='compiler-artifact' and record.get('executable'):artifacts[record['target']['name']]=record['executable']
(OUT/'artifacts.json').write_text(json.dumps(artifacts,indent=2)+'\n')
names=['input::tests::spawn_failure_on_stdin_reaches_syntax_without_reading','input::tests::spawn_failure_preserves_partial_input_and_requeues_other_posts','input::tests::stdin_recovers_with_a_pooled_worker_after_spawn_failure','input::tests::stdin_recovers_in_configured_inline_mode_after_spawn_failure','scheduler::tests::spawn_failure::native_spawn_failure_reaches_main_and_started_handlers','scheduler::tests::spawn_failure::command_spawn_failure_creates_no_child_and_repeats','scheduler::tests::spawn_failure::guarded_native_spawn_failure_releases_receiver_for_progress','scheduler::tests::spawn_failure::configured_inline_and_fixed_saturation_are_not_spawn_errors','command::tests::a_reader_panic_preserves_the_other_reader_and_child_status','command::tests::a_real_reader_eof_preserves_the_other_pipes_large_output']
for i,name in enumerate(names):
 text=run('tsan-lib-'+str(i),[artifacts['rexx_exec'],'--exact',name,'--nocapture'])
 assert 'test '+name+' ... ok' in text,(name,text)
for i,name in enumerate(['sigint_at_a_nested_redirected_command_is_raised_at_the_outer_clause','sigint_drops_a_condition_a_redirected_target_raised','sigint_keeps_what_a_redirected_command_wrote']):
 text=run('tsan-signal-'+str(i),[artifacts['signals'],'--exact',name,'--nocapture'])
 assert 'test '+name+' ... ok' in text,(name,text)
name='support::arity::binary_selection_tests::arity_runs_the_cargo_owned_binary'
text=run('tsan-selector',[artifacts['concurrency_tests'],'--exact',name,'--nocapture'])
assert 'test '+name+' ... ok' in text
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip()==HEAD
