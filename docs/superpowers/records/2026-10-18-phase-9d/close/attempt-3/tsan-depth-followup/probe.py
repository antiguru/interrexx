"""Run one existing exact unit test with original sanitizer checks and a bound."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
out = Path(__file__).resolve().parent
binary = root / '.superpowers/sdd/2026-10-18-phase-9d/task10-targets/ee52ea5e62daeb6bc35ca4475de4d1e7a96a4bfe/tsan/x86_64-unknown-linux-gnu/debug/build/rexx-exec/6c80e238cfcc855d/out/rexx_exec-6c80e238cfcc855d'
name = sys.argv[1]
case = out / (sys.argv[2] if len(sys.argv) > 2 else name.split('::')[-1])
case.mkdir()
env = os.environ.copy()
env['REXX_TEST_THREAD_SANITIZER'] = '1'
env['TSAN_OPTIONS'] = f'log_path={case}/tsan suppressions={root}/rust/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1'
command = ['timeout', '--kill-after=5', '90', str(binary), '--exact', name, '--nocapture']
record = dict(command=command, test=name, binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
              tsan_options=env['TSAN_OPTIONS'], rexx_test_thread_sanitizer='1',
              started=datetime.datetime.now(datetime.timezone.utc).isoformat())
(case / 'command.json').write_text(json.dumps(record, indent=2) + '\n')
captured = False
snapshot_done = False
started_clock = time.monotonic()
with (case / 'stdout').open('xb') as stdout, (case / 'stderr').open('xb') as stderr:
    child = subprocess.Popen(command, cwd=root / 'rust', env=env, stdout=stdout, stderr=stderr)
    record['timeout_pid'] = child.pid
    while child.poll() is None:
        if not captured:
            children = Path(f'/proc/{child.pid}/task/{child.pid}/children').read_text().split()
            for pid in children:
                try:
                    if Path(f'/proc/{pid}/exe').resolve() == binary:
                        (case / 'maps').write_text(Path(f'/proc/{pid}/maps').read_text())
                        record['binary_pid'] = int(pid)
                        captured = True
                except FileNotFoundError:
                    pass
        if case.name == 'join-stack-snapshot-1' and captured and not snapshot_done and time.monotonic() - started_clock >= 5:
            snapshot_done = True
            debugger = ['timeout', '--kill-after=3', '15', 'gdb', '-nx', '-batch',
                        '-ex', 'set pagination off', '-ex', 'set debuginfod enabled off',
                        '-ex', 'set print thread-events off', '-ex', 'info threads',
                        '-ex', 'thread apply all bt 12', '-ex', 'detach',
                        '-p', str(record['binary_pid'])]
            debug_record = dict(argv=debugger, started=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                                perturbation='Read-only ptrace snapshot pauses target scheduling. No target function calls or memory writes.')
            (case / 'debugger.json').write_text(json.dumps(debug_record, indent=2) + '\n')
            with (case / 'gdb.stdout').open('xb') as debug_out, (case / 'gdb.stderr').open('xb') as debug_err:
                result = subprocess.run(debugger, stdout=debug_out, stderr=debug_err)
            debug_record.update(status=result.returncode, finished=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                                detach_confirmed=f'(process {record["binary_pid"]}) detached' in (case / 'gdb.stdout').read_text())
            (case / 'debugger.json').write_text(json.dumps(debug_record, indent=2) + '\n')
            print(json.dumps(debug_record), flush=True)
        time.sleep(0.05)
record.update(status=child.returncode, finished=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              reports=[p.name for p in case.glob('tsan.*')])
(case / 'status.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(record), flush=True)
