#!/usr/bin/env python3
"""Execute the committed close gates, one foreground action per invocation."""

import argparse
import datetime
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
RUST = ROOT / 'rust'
TARGETS = ROOT / '.superpowers/sdd/2026-10-18-phase-9d/task10-targets'
PIN_ROOT = Path('/home/moritz/dev/repos/ooRexx-p9-pin')
PIN = PIN_ROOT / 'rexx-run'
PIN_HASH = 'c4b4e8c4ce7f1f12e4a73fa37cb15eca262717092c2c1aed727febf437523c67'
TSAN = ROOT / '.superpowers/sdd/2026-10-07-phase-6-1/p61-gates/tsan.sh'


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def digest(path):
    h = hashlib.sha256()
    with open(path, 'rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def git(*args):
    return subprocess.check_output(['git', '-C', str(ROOT), *args], timeout=30).decode().strip()


def write(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')


def append(text):
    with (HERE / 'status.txt').open('a') as stream:
        stream.write(f'{text} at {now()}\n')


def clean(head):
    assert git('rev-parse', 'HEAD') == head, 'HEAD changed during freeze'
    assert not git('diff', '--name-only', head), 'Tracked working tree changed'
    assert not git('diff', '--cached', '--name-only'), 'Index changed'
    assert not git('ls-files', '--others', '--exclude-standard', '--', 'rust'), 'Untracked Rust inputs'


def static_inputs():
    # Hash the external runtime inputs without changing the read-only checkouts.
    paths = [PIN, TSAN, HERE / 'runner.py', HERE / 'run.sh']
    for base in [ROOT / 'build/lib', Path('/home/moritz/dev/repos/ooRexx/build/lib')]:
        paths.extend(path for path in base.glob('*') if path.is_file())
    paths.append(Path('/home/moritz/dev/repos/ooRexx/build/bin/rexx'))
    for base in [ROOT / 'ootest', ROOT / 'oodocs/rexxref']:
        paths.extend(path for path in base.rglob('*') if path.is_file() and '.svn' not in path.parts)
    return {str(path): digest(path) for path in sorted(set(paths))}


def guard(state):
    clean(state['head'])
    assert digest(PIN) == PIN_HASH, 'Phase 9 pin changed'
    assert static_inputs() == state['static_inputs'], 'External or runner inputs changed'
    assert state['source_tree'] == git('rev-parse', 'HEAD:rust')


def actions(state):
    target = TARGETS / state['head']
    workspace = target / 'workspace'
    common = {'CARGO_INCREMENTAL': '0', 'CARGO_BUILD_JOBS': '4', 'CARGO_TARGET_DIR': str(workspace)}
    corpus = {'REXX_CORPUS_GATE': '1'}
    cargo = ['cargo']
    test = cargo + ['test', '-j', '4']
    result = []

    def add(name, argv, cap='16G', seconds=600, env=None, purpose=None):
        environment = common | (env or {})
        if purpose:
            environment['CARGO_TARGET_DIR'] = str(target / purpose)
        result.append(dict(name=name, argv=argv, cap=cap, seconds=seconds, env=environment))

    add('preflight', ['python3', 'scripts/test_preflight.py'], '8G')
    add('G1-fmt', cargo + ['fmt', '--all', '--check'], '8G')
    add('G2-clippy', cargo + ['clippy', '-j', '4', '--workspace', '--all-targets', '--', '-D', 'warnings'], purpose='clippy')
    add('G3-build-release', test + ['--workspace', '--release', '--no-run'], env=corpus)
    tables = lambda name: {key: str(HERE / f'{name}-{suffix}.txt') for key, suffix in [
        ('REXX_TIMER_TABLE', 'timer-table'), ('REXX_CRITERION_ONE_TABLE', 'criterion-one-table'),
        ('REXX_CRITERION_ONE_S3_TABLE', 'criterion-one-s3-table')]}
    add('G4-test-release', test + ['--workspace', '--release', '--no-fail-fast'], '8G', 2400,
        corpus | tables('G4') | {'REXX_SUITE_DIAGNOSTICS': str(HERE / 'release-diagnostics')})
    add('whole-suite', test + ['--release', '-p', 'rexx-exec', '--test', 'suite', 'l3_core_suite', '--', '--nocapture'], '8G', 600,
        {'REXX_SUITE_GATE': '1', 'REXX_SUITE_TABLE': str(HERE / 'suite/table.tsv'),
         'REXX_SUITE_DIAGNOSTICS': str(HERE / 'suite-diagnostics')})
    add('G5-build-debug', test + ['--workspace', '--no-fail-fast', '--no-run'], env=corpus)
    add('G6-test-debug', test + ['--workspace', '--no-fail-fast'], '8G', 3000,
        corpus | tables('G6') | {'REXX_SUITE_DIAGNOSTICS': str(HERE / 'debug-diagnostics')})
    add('G7-clippy-pinning', cargo + ['clippy', '-j', '4', '-p', 'rexx-exec', '--all-targets', '--features', 'pinning', '--', '-D', 'warnings'], purpose='pinning')
    pinning = ['-p', 'rexx-exec', '--features', 'pinning', '--test', 'concurrency_tests']
    add('G8-build-pinning', test + pinning + ['--no-run'], purpose='pinning')
    add('G8-test-pinning', test + pinning + ['--', 'measured::a_', 'measured::an_'], '8G', purpose='pinning')
    loom = ['-p', 'rexx-exec', '--test', 'loom']
    add('G9-build-loom', test + loom + ['--no-run'], env={'RUSTFLAGS': '--cfg loom'}, purpose='loom')
    add('G9-loom', test + loom, '8G', env={'RUSTFLAGS': '--cfg loom'}, purpose='loom')
    add('TSan', ['bash', str(TSAN), str(target / 'tsan'), str(HERE / 'tsan')], '8G', 3000, purpose='tsan')
    add('performance-build', ['cargo', 'build', '-j', '4', '--release', '-p', 'rexx-exec', '--bin', 'rexx'])
    add('performance-copy', ['internal-copy'], '8G')
    add('callgrind', ['bench-programs/callgrind.sh', '-B', '0.5', '-o', str(PIN_ROOT / 'cg-9d/close'),
                     f'pin={PIN}', f'head={PIN_ROOT / ("rexx-9d-" + state["head"])}'], '8G', 43200)
    return result


def environment(extra):
    env = {k: v for k, v in os.environ.items() if not k.startswith('REXX_')}
    for key in list(env):
        if key in {'RUSTFLAGS', 'RUSTDOCFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_ENCODED_RUSTDOCFLAGS',
                   'CARGO_BUILD_RUSTFLAGS', 'CARGO_BUILD_RUSTDOCFLAGS', 'CARGO_BUILD_TARGET', 'RUSTC_WRAPPER',
                   'RUSTC_WORKSPACE_WRAPPER', 'TSAN_OPTIONS'} or key.startswith('CARGO_PROFILE_') or (
                       key.startswith('CARGO_TARGET_') and key.endswith(('_RUNNER', '_RUSTFLAGS', '_RUSTDOCFLAGS', '_LINKER'))):
            env.pop(key)
    env.update(extra)
    return env


def group_alive(pid):
    try:
        os.killpg(pid, 0)
        return True
    except ProcessLookupError:
        return False


def stop_group(pid):
    if group_alive(pid):
        os.killpg(pid, signal.SIGTERM)
        deadline = time.monotonic() + 5
        while group_alive(pid) and time.monotonic() < deadline:
            time.sleep(0.1)
        if group_alive(pid):
            os.killpg(pid, signal.SIGKILL)


def run_action(action, state):
    name = action['name']
    prefix = HERE / name
    assert not prefix.with_suffix('.json').exists(), 'Action already recorded, no retries'
    guard(state)
    if name == 'performance-copy':
        assert json.loads((HERE / 'performance-build.json').read_text())['status'] == 0, 'Performance build failed'
        source = TARGETS / state['head'] / 'workspace/release/rexx'
        dest = PIN_ROOT / ('rexx-9d-' + state['head'])
        with source.open('rb') as src, dest.open('xb') as dst:
            shutil.copyfileobj(src, dst)
        dest.chmod(0o555)
        write(prefix.with_suffix('.json'), dict(action=action, status=0, sha256=digest(dest), finished=now()))
        append(f'{name} exit 0')
        return 0
    if name == 'callgrind':
        assert not (PIN_ROOT / 'cg-9d/close').exists(), 'Callgrind output exists, no overwrite'
        assert (HERE / 'performance-build.json').exists()
        assert json.loads((HERE / 'performance-build.json').read_text())['status'] == 0
        assert digest(PIN_ROOT / ('rexx-9d-' + state['head'])) == json.loads((HERE / 'performance-copy.json').read_text())['sha256']
    argv = ['timeout', '--kill-after=15', str(action['seconds']), 'memcap', action['cap'], *action['argv']]
    started = now()
    append(f'{name} started')
    proc = None
    interrupted = False
    def interrupted_signal(signum, frame):
        raise KeyboardInterrupt(f'signal {signum}')
    signal.signal(signal.SIGTERM, interrupted_signal)
    try:
        with prefix.with_suffix('.stdout').open('xb') as stdout, prefix.with_suffix('.stderr').open('xb') as stderr:
            proc = subprocess.Popen(argv, cwd=RUST, env=environment(action['env']), stdout=stdout,
                                    stderr=stderr, start_new_session=True)
            write(HERE / f'{name}.pid.json', {'pid': proc.pid, 'started': started})
            try:
                status = proc.wait(timeout=action['seconds'] + 45)
            except (KeyboardInterrupt, subprocess.TimeoutExpired):
                interrupted = True
                stop_group(proc.pid)
                proc.wait()
                status = 124
    finally:
        if proc is not None:
            leaked = group_alive(proc.pid)
            stop_group(proc.pid)
            proc.wait()
        signal.signal(signal.SIGTERM, signal.SIG_DFL)
    record = dict(action=action, argv=argv, status=status, started=started, finished=now(),
                  interrupted=interrupted, pid=proc.pid, surviving_group=group_alive(proc.pid), orphaned_children=leaked,
                  stdout_sha256=digest(prefix.with_suffix('.stdout')), stderr_sha256=digest(prefix.with_suffix('.stderr')))
    if name == 'TSan':
        output = prefix.with_suffix('.stdout').read_text()
        children = re.findall(r'^(api|lib|int) exit (\d+)$', output, re.M)
        diagnostics = sorted(str(p) for p in (HERE / 'tsan').glob('tsan.*'))
        record.update(children=children, diagnostics=diagnostics)
        write(HERE / 'tsan-status.json', record)
        # The historical script prints each child exit but returns its final ls/grep status.
        record['validated_status'] = 0 if status == 0 and [c[0] for c in children] == ['api', 'lib', 'int'] and all(c[1] == '0' for c in children) and not diagnostics else 1
    if name == 'callgrind':
        with (HERE / 'callgrind.txt').open('xb') as stream:
            stream.write(prefix.with_suffix('.stdout').read_bytes())
    try:
        guard(state)
    except Exception as error:
        record['freeze_error'] = str(error)
        record['validated_status'] = 1
    write(prefix.with_suffix('.json'), record)
    append(f'{name} exit {status} validated {record.get("validated_status", status)}')
    return record.get('validated_status', status) or int(leaked)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['plan', 'start', 'step', 'finish'])
    parser.add_argument('--head', help='Full committed revision required at start')
    args = parser.parse_args()
    if args.mode == 'plan':
        print(json.dumps(actions({'head': args.head or '<close-commit>'}), indent=2))
        return 0
    with (HERE / '.runner.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        if args.mode == 'start':
            assert args.head and re.fullmatch('[a-f0-9]{40}', args.head)
            clean(args.head)
            assert digest(PIN) == PIN_HASH
            assert shutil.disk_usage(ROOT).free > 200 * 1024**3, 'Require 200 GiB free for fresh targets'
            assert not (TARGETS / args.head).exists(), 'Target exists, not fresh'
            assert not (HERE / 'status.txt').exists(), 'Run exists, no restart'
            for command in ['cargo', 'rustc', 'rustup', 'memcap', 'timeout', 'valgrind', 'svn']:
                assert shutil.which(command), f'Missing {command}'
            state = dict(head=args.head, source_tree=git('rev-parse', 'HEAD:rust'), static_inputs=static_inputs(), started=now())
            (TARGETS / args.head).mkdir(parents=True)
            (HERE / 'suite').mkdir()
            write(HERE / 'freeze.json', state)
            with (HERE / 'status.txt').open('x') as status:
                status.write(f'head {args.head}\nstart {now()}\n')
            for name, argv in [('rustc', ['rustc', '-vV']), ('cargo', ['cargo', '-vV']),
                               ('toolchains', ['rustup', 'toolchain', 'list']), ('components', ['rustup', 'component', 'list', '--installed', '--toolchain', 'nightly']),
                               ('ootest', ['svn', 'info', str(ROOT / 'ootest')]), ('oodocs', ['svn', 'info', str(ROOT / 'oodocs/rexxref')])]:
                done = subprocess.run(argv, capture_output=True, timeout=30)
                write(HERE / f'provenance-{name}.json', dict(argv=argv, status=done.returncode, stdout=done.stdout.decode(), stderr=done.stderr.decode()))
                assert done.returncode == 0, f'Provenance failed: {name}'
            env = environment({})
            write(HERE / 'environment.json', {k: v for k, v in env.items() if k.startswith(('CARGO_', 'RUST', 'REXX_')) or k in {'PATH', 'LD_LIBRARY_PATH', 'LIBRARY_PATH', 'TMPDIR'}})
            write(HERE / 'commands.json', actions(state))
            print('started; tree frozen until finish')
            return 0
        state = json.loads((HERE / 'freeze.json').read_text())
        assert 'finished' not in (HERE / 'status.txt').read_text().split(), 'Already finished'
        guard(state)
        planned = actions(state)
        pending = [a for a in planned if not (HERE / (a['name'] + '.json')).exists()]
        if args.mode == 'step':
            assert pending, 'No pending action, use finish'
            return run_action(pending[0], state)
        assert not pending, f'Unexecuted actions: {[a["name"] for a in pending]}'
        records = [json.loads((HERE / (a['name'] + '.json')).read_text()) for a in planned]
        assert not any(group_alive(r['pid']) for r in records if 'pid' in r), 'Child process group survives'
        statuses = {a['name']: (r.get('validated_status', r['status']) or int(r.get('orphaned_children', False))) for a, r in zip(planned, records)}
        failed = {name: status for name, status in statuses.items() if status}
        write(HERE / 'finished.json', dict(head=state['head'], statuses=statuses, failed=failed, finished=now()))
        append('finished')
        print(json.dumps(statuses, indent=2))
        return int(bool(failed))


if __name__ == '__main__':
    sys.exit(main())
