"""Check close evidence integrity using fixtures and a mocked child launcher."""
import ast
import json
from pathlib import Path
import tempfile
import types
from unittest.mock import Mock, patch

source = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/close/runner.py')
module = types.ModuleType('close_runner_selfcheck')
module.__file__ = str(source)
code = source.read_text()
ast.parse(code)
exec(compile(code, str(source), 'exec'), module.__dict__)
fixtures = Path(tempfile.mkdtemp(prefix='p9d-task10-runner-readiness-'))

def fixture_file(path, text='fixture'):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    return path

root = fixtures / 'repo'
here = fixtures / 'close'
oracle = fixtures / 'oracle'
for path in [root / 'build/lib/local.so', root / 'ootest/worker.rex', root / 'oodocs/rexxref/provide.xml',
             here / 'runner.py', here / 'run.sh', oracle / 'lib/librexx.so', oracle / 'bin/rexx',
             oracle / 'bin/rexxc', oracle / 'bin/yaml.cls', oracle / 'samples/factor.rex']:
    fixture_file(path)
pin = fixture_file(fixtures / 'pin')
tsan = fixture_file(fixtures / 'tsan.sh')
with patch.multiple(module, ROOT=root, HERE=here, ORACLE=oracle, PIN=pin, TSAN=tsan):
    baseline = module.static_inputs()
    for path in [oracle / 'bin/rexx', oracle / 'bin/rexxc', oracle / 'bin/yaml.cls', oracle / 'samples/factor.rex']:
        assert str(path) in baseline
        path.write_text('changed')
        assert module.static_inputs() != baseline, path
        path.write_text('fixture')
        assert module.static_inputs() == baseline
    assert baseline[str(root / 'build/lib/local.so')] == module.digest(root / 'build/lib/local.so')
    assert baseline[str(oracle / 'lib/librexx.so')] == module.digest(oracle / 'lib/librexx.so')
    new = fixture_file(oracle / 'bin/new.cls')
    added = module.static_inputs()
    assert added != baseline
    new.rename(fixtures / 'retained-new.cls')
    assert module.static_inputs() == baseline
    directory = oracle / 'samples/empty'
    directory.mkdir()
    assert module.static_inputs() != baseline
    target_a = fixture_file(fixtures / 'targets/a.cls', 'same')
    target_b = fixture_file(fixtures / 'targets/b.cls', 'same')
    link = oracle / 'bin/link.cls'
    link.symlink_to(target_a)
    before = module.static_inputs()
    assert before[str(link)]['symlink_target'] == str(target_a)
    assert before[str(link)]['resolved'] == str(target_a.resolve())
    link.rename(fixtures / 'retained-link-a')
    link.symlink_to(target_b)
    assert module.static_inputs() != before, 'Same-content symlink retargeting went unnoticed'
    linked_directory = fixtures / 'linked-directory'
    fixture_file(linked_directory / 'nested.rex')
    (oracle / 'samples/linked').symlink_to(linked_directory, target_is_directory=True)
    before = module.static_inputs()
    assert str(oracle / 'samples/linked/nested.rex') in before
    (linked_directory / 'nested.rex').write_text('changed target')
    assert module.static_inputs() != before, 'Directory symlink target content went unnoticed'

# Read the selected live paths without starting a freeze or modifying real close outputs.
for base, expected in [(module.ORACLE / 'bin', ['rexx', 'rexxc', 'yaml.cls']),
                       (module.ORACLE / 'samples', ['factor.rex'])]:
    manifest = module.tree_inputs(base)
    assert all(str(base / name) in manifest for name in expected)


def check_status(label, stdout_text, guard_error=None, diagnostic=False):
    output = fixtures / label
    output.mkdir()
    if diagnostic:
        fixture_file(output / 'tsan/tsan.123')
    def fake_popen(argv, **kwargs):
        kwargs['stdout'].write(stdout_text.encode())
        kwargs['stdout'].flush()
        return types.SimpleNamespace(pid=99999999, wait=lambda **kw: 0)
    guard = Mock(side_effect=[None, RuntimeError(guard_error)] if guard_error else [None, None])
    action = dict(name='TSan', argv=['mock-only'], seconds=600, cap='8G', env={})
    with patch.multiple(module, HERE=output, guard=guard, group_alive=lambda pid: False,
                        stop_group=lambda pid: None), patch.object(module.subprocess, 'Popen', fake_popen):
        returned = module.run_action(action, {})
    primary = json.loads((output / 'TSan.json').read_text())
    copied = json.loads((output / 'tsan-status.json').read_text())
    assert copied == primary, 'Copied and primary status disagree'
    assert copied['status'] == 0, 'Raw shell status changed'
    assert returned == copied['validated_status']
    if guard_error:
        assert copied['freeze_error'] == guard_error
    return returned

assert check_status('child-failed', 'api exit 0\nlib exit 1\nint exit 0\n') == 1
assert check_status('freeze-failed', 'api exit 0\nlib exit 0\nint exit 0\n', 'fixture freeze drift') == 1
assert check_status('diagnostic', 'api exit 0\nlib exit 0\nint exit 0\n', diagnostic=True) == 1
assert check_status('passing-control', 'api exit 0\nlib exit 0\nint exit 0\n') == 0
print('PASS: consumed oracle input coverage, membership, symlink targets and content, native manifests, raw/validated/copied TSan status and post-run freeze failure')
print('Fixtures retained:', fixtures)
print('No child processes, gates or actual freeze outputs created.')
