import fcntl,importlib.util,json,sys
from pathlib import Path
sys.dont_write_bytecode=True
root=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite');close=root/'docs/superpowers/records/2026-10-18-phase-9d/close'
spec=importlib.util.spec_from_file_location('close_runner',close/'runner.py');r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)
with (close/'.runner.lock').open('a') as lock:
 fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 state=json.loads((close/'freeze.json').read_text());r.guard(state)
 failed=json.loads((close/'TSan.json').read_text());assert failed['validated_status']==1 and failed['status']==0 and failed['children']==[['api','0'],['lib','66'],['int','101']]
 records=[json.loads(p.read_text()) for p in close.glob('*.json') if isinstance(json.loads(p.read_text()),dict) and 'pid' in json.loads(p.read_text()) and 'status' in json.loads(p.read_text())]
 assert all(not v.get('orphaned_children',False) and not r.group_alive(v['pid']) for v in records),'Process group still active'
 completed={a['name']:json.loads((close/(a['name']+'.json')).read_text()).get('validated_status',json.loads((close/(a['name']+'.json')).read_text())['status']) for a in r.actions(state) if (close/(a['name']+'.json')).exists()}
 pending=[a['name'] for a in r.actions(state) if not (close/(a['name']+'.json')).exists()];assert pending==['performance-build','performance-copy','callgrind']
 record={'head':state['head'],'source_tree':state['source_tree'],'verdict':'aborted after complete negative TSan run','statuses':completed,'failed':{k:v for k,v in completed.items() if v},'pending':pending,'all_actions_executed':False,'all_child_groups_stopped':True,'diagnostic_probes_completed':True,'finished':r.now()}
 r.write(close/'finished.json',record);r.append('aborted: preserve TSan reports before scoped reader and instrumentation compatibility corrections');r.append('finished');r.guard(state);r.write(close/'negative-attempt-release.json',record);print(json.dumps(record,indent=2))
