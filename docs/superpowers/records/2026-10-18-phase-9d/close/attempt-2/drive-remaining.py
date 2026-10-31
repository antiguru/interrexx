import importlib.util,json,subprocess,sys
from pathlib import Path
sys.dont_write_bytecode=True
root=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
close=root/'docs/superpowers/records/2026-10-18-phase-9d/close'
spec=importlib.util.spec_from_file_location('close_runner',close/'runner.py');r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)
state=json.loads((close/'freeze.json').read_text());r.guard(state)
g4=json.loads((close/'G4-test-release.json').read_text());assert g4['status']==0 and not g4['orphaned_children'] and not g4['surviving_group']
while True:
 r.guard(state)
 pending=[a for a in r.actions(state) if not (close/(a['name']+'.json')).exists()]
 if not pending: print('All closing actions recorded; controller must inspect and finish',flush=True);break
 name=pending[0]['name'];print('Executing '+name,flush=True)
 status=subprocess.run(['bash',str(close/'run.sh'),'step'],cwd=root).returncode
 print(name+' command exit '+str(status),flush=True)
 if status: sys.exit(status)
