"""Synthetic build-proof controls only, with no gate or subprocess execution."""
import copy
import datetime
import json
from pathlib import Path
import tempfile
import types
from unittest.mock import patch

path = Path('/tmp/p9d-close-evidence-audit.py')
m = types.ModuleType('build_proof_check')
m.__file__ = str(path)
exec(compile(path.read_text(), str(path), 'exec'), m.__dict__)
root = Path(tempfile.mkdtemp(prefix='p9d-close-build-proof-controls-'))
action = json.loads((m.RECORDS / 'close/G3-build-release.json').read_text())['action']
head = '1ef21db99b90d55f02e6d12d2e473cee56b6a747'
results = {}


def fixture(name):
    folder = root / name
    folder.mkdir()
    (folder / 'runner.py').write_text('fixture only')
    freeze = dict(head=head, source_tree='fixture-rust-tree', static_inputs={str(folder / 'runner.py'): m.digest(folder / 'runner.py')})
    start = datetime.datetime.fromisoformat('2026-10-31T12:00:00+00:00')
    for index, (label, status, elapsed) in enumerate([('G3-build-release', 124, 600), ('G3-build-release-continuation-1', 0, 20)]):
        began = start + datetime.timedelta(seconds=index * 620)
        finished = began + datetime.timedelta(seconds=elapsed)
        for suffix in ['stdout', 'stderr']:
            (folder / (label + '.' + suffix)).write_text('')
        record = dict(action=dict(action, name=label), argv=['timeout', '--kill-after=15', '600', 'memcap', '16G', *action['argv']], status=status, started=began.isoformat(), finished=finished.isoformat(), interrupted=False, pid=99999999-index, surviving_group=False, orphaned_children=False, stdout_sha256=m.digest(folder / (label + '.stdout')), stderr_sha256=m.digest(folder / (label + '.stderr')))
        (folder / (label + '.json')).write_text(json.dumps(record))
        (folder / (label + '.pid.json')).write_text(json.dumps(dict(pid=record['pid'], started=record['started'])))
    ruling = dict(head=head, rust_tree=freeze['source_tree'], original_status=124, original_record_sha256=m.digest(folder / 'G3-build-release.json'), ruling='Fixture-only explicit ruling', reason='Fixture timeout', cost_if_wrong='Fixture only')
    (folder / 'build-continuation-ruling.json').write_text(json.dumps(ruling))
    (folder / 'G4-test-release.json').write_text(json.dumps(dict(started=(start + datetime.timedelta(seconds=660)).isoformat())))
    return folder, freeze


def rewrite(folder, name, update):
    path = folder / name
    data = json.loads(path.read_text())
    update(data)
    path.write_text(json.dumps(data))


for name in ['valid', 'missing-ruling', 'changed-timeout', 'raw-log-tamper', 'early-g4', 'compile-not-complete', 'interrupted-original', 'semantic-test-retry']:
    folder, freeze = fixture(name)
    if name == 'missing-ruling':
        (folder / 'build-continuation-ruling.json').unlink()
    elif name == 'changed-timeout':
        rewrite(folder, 'G3-build-release-continuation-1.json', lambda r: r['action'].update(seconds=1200))
    elif name == 'raw-log-tamper':
        (folder / 'G3-build-release-continuation-1.stderr').write_text('changed')
    elif name == 'early-g4':
        rewrite(folder, 'G4-test-release.json', lambda r: r.update(started='2026-10-31T12:00:01+00:00'))
    elif name == 'compile-not-complete':
        rewrite(folder, 'G3-build-release-continuation-1.json', lambda r: r.update(status=124))
    elif name == 'interrupted-original':
        rewrite(folder, 'G3-build-release.json', lambda r: r.update(interrupted=True))
        rewrite(folder, 'build-continuation-ruling.json', lambda r: r.update(original_record_sha256=m.digest(folder / 'G3-build-release.json')))
    elif name == 'semantic-test-retry':
        rewrite(folder, 'G3-build-release-continuation-1.json', lambda r: r['action']['argv'].remove('--no-run'))
    with patch.object(m.os, 'killpg', side_effect=ProcessLookupError):
        try:
            m.completed_no_run_build(folder, freeze, [action])
            result = 'accepted'
        except Exception as error:
            result = 'rejected: ' + type(error).__name__ + ': ' + str(error)
    results[name] = result
    assert (result == 'accepted') == (name == 'valid'), (name, result)
(root / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
print(root)
print(json.dumps(results, indent=2))
