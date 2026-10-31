import importlib.util, json, sys
from pathlib import Path
sys.dont_write_bytecode = True
repo=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
close=repo/'docs/superpowers/records/2026-10-18-phase-9d/close'
spec=importlib.util.spec_from_file_location('phase9d_close_runner', close/'runner.py')
r=importlib.util.module_from_spec(spec); spec.loader.exec_module(r)
state=json.loads((close/'freeze.json').read_text()); r.guard(state)
original=json.loads((close/'G3-build-release.json').read_text())
assert original['status']==124 and not original['orphaned_children'] and not r.group_alive(original['pid'])
assert not (close/'G4-test-release.json').exists()
a=next(a for a in r.actions(state) if a['name']=='G3-build-release')
assert a['cap']=='16G' and '--no-run' in a['argv'] and a['argv']==original['action']['argv']
index=int(sys.argv[1]); assert index>=1
for i in range(1,index):
 prior=json.loads((close/f'G3-build-release-continuation-{i}.json').read_text())
 assert prior['status']==124 and not prior['orphaned_children'] and not r.group_alive(prior['pid'])
a['name']=f'G3-build-release-continuation-{index}'
if index==1:
 r.write(close/'build-continuation-ruling.json', {'ruling':'Continue only the timed-out no-run compilation using its existing cache, preserving every original 600-second bound, 16G cap, command argument, source and environment. No test is executed or retried by this continuation.', 'reason':'The fresh compile exhausted its ten-minute bound while linking targets, with no compiler diagnostic failure. Task10 requires the complete no-run build before tests, and freshness is mandated for Clippy rather than each compile attempt.', 'cost_if_wrong':'Additional compilation time; no test failure or semantic result is replaced.', 'head':state['head'], 'rust_tree':state['source_tree'], 'original_status':124, 'original_record_sha256':r.digest(close/'G3-build-release.json')})
sys.exit(r.run_action(a,state))
