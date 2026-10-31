import fcntl, importlib.util, json, os, sys
from pathlib import Path
sys.dont_write_bytecode = True
root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
close = root / 'docs/superpowers/records/2026-10-18-phase-9d/close'
spec = importlib.util.spec_from_file_location('close_runner', close / 'runner.py')
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)
with (close / '.runner.lock').open('a') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    state = json.loads((close / 'freeze.json').read_text())
    r.guard(state)
    failed = json.loads((close / 'TSan.json').read_text())
    assert failed['status'] == 0 and failed['validated_status'] == 1
    assert failed['children'] == [['api', '0'], ['lib', '66'], ['int', '0']]
    assert not (close / 'finished.json').exists()
    actual = [json.loads((close / (a['name'] + '.json')).read_text()) for a in r.actions(state) if (close / (a['name'] + '.json')).exists()]
    assert all(not v.get('orphaned_children', False) and not v.get('surviving_group', False) and not r.group_alive(v['pid']) for v in actual if 'pid' in v)
    for name, expected in [('group-1', 124), ('group-2', 124), ('group-3', 66), ('group-4', 101)]:
        d = json.loads((close / 'tsan-candidate-selection' / name / 'status.json').read_text())
        assert d['status'] == expected and not d['unexpected_compilation']
        assert not r.group_alive(d['pid'])
    diagnostics = ['tsan-depth-followup', 'tsan-library-replay-1', 'tsan-library-fault-capture-1', 'tsan-cyclic-reference-fault', 'tsan-candidate-selection']
    ids = set()
    def collect(value):
        if isinstance(value, dict):
            for key, item in value.items():
                if key.endswith('pid') and isinstance(item, int) and item > 0:
                    ids.add(item)
                collect(item)
        elif isinstance(value, list):
            for item in value:
                collect(item)
    for directory in diagnostics:
        for path in (close / directory).rglob('*.json'):
            collect(json.loads(path.read_text()))
    alive = [pid for pid in sorted(ids) if Path('/proc', str(pid)).exists()]
    assert not alive, f'Recorded diagnostic process remains: {alive}'
    completed = {a['name']: json.loads((close / (a['name'] + '.json')).read_text()).get('validated_status', json.loads((close / (a['name'] + '.json')).read_text())['status']) for a in r.actions(state) if (close / (a['name'] + '.json')).exists()}
    pending = [a['name'] for a in r.actions(state) if not (close / (a['name'] + '.json')).exists()]
    assert pending == ['performance-build', 'performance-copy', 'callgrind']
    record = dict(head=state['head'], source_tree=state['source_tree'], verdict='aborted after complete negative TSan run', statuses=completed, failed={k:v for k,v in completed.items() if v}, pending=pending, all_actions_executed=False, all_child_groups_stopped=True, diagnostic_probes_completed=True, recorded_diagnostic_pids_absent=sorted(ids), exact_join_probe_status=66, exact_stem_probe_status=101, original_write_fault_identity='not proven identical to standalone join read fault', finished=r.now())
    r.write(close / 'finished.json', record)
    r.append('aborted: preserve failed TSan and diagnostic resource-boundary evidence before test-only corrections')
    r.append('finished')
    r.guard(state)
    r.write(close / 'negative-attempt-release.json', record)
    print(json.dumps(record, indent=2))
