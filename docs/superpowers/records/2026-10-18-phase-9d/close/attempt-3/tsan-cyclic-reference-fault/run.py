"""One authorized existing-test debugger diagnostic with bounded cleanup."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
out = Path(__file__).resolve().parent
target = root / '.superpowers/sdd/2026-10-18-phase-9d/task10-targets/ee52ea5e62daeb6bc35ca4475de4d1e7a96a4bfe/tsan'
binary = target / 'x86_64-unknown-linux-gnu/debug/build/rexx-exec/6c80e238cfcc855d/out/rexx_exec-6c80e238cfcc855d'
name = 'tests::a_cyclic_reference_is_insufficient_stack_in_every_conversion'
env = os.environ.copy()
env.update(CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='4', CARGO_TARGET_DIR=str(target),
           RUSTFLAGS='-Zsanitizer=thread', REXX_TEST_THREAD_SANITIZER='1',
           TSAN_OPTIONS=f'log_path={out}/tsan suppressions={root}/rust/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1')
argv = ['timeout', '--kill-after=5', '90', 'gdb', '-nx', '-batch', '-x', str(out / 'capture.gdb'),
        '--args', str(binary), '--exact', name, '--nocapture']
now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
record = dict(purpose='One distinct explicitly authorized fault-capture diagnostic, never acceptance evidence.',
              argv=argv, cwd=str(root / 'rust/crates/rexx-exec'), started=now(), memory_cap='8G', timeout_seconds=90,
              sha256={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in [binary, out / 'capture.gdb', out / 'run.py']},
              relevant_environment={k: env[k] for k in ['CARGO_INCREMENTAL', 'CARGO_BUILD_JOBS', 'CARGO_TARGET_DIR', 'RUSTFLAGS', 'REXX_TEST_THREAD_SANITIZER', 'TSAN_OPTIONS']},
              perturbation='Debugger launches inferior with ASLR enabled and intercepts SIGSEGV. Reads only registers/memory, then kills any still-stopped inferior for cleanup.')
(out / 'command.json').write_text(json.dumps(record, indent=2) + '\n')
with (out / 'stdout').open('xb') as stdout, (out / 'stderr').open('xb') as stderr:
    child = subprocess.Popen(argv, cwd=root / 'rust/crates/rexx-exec', env=env, stdout=stdout, stderr=stderr)
    record['timeout_pid'] = child.pid
    (out / 'live-process.json').write_text(json.dumps(record, indent=2) + '\n')
    status = child.wait()
record.update(status=status, finished=now(), reports=[p.name for p in out.glob('tsan.*')])
text = (out / 'stdout').read_text(errors='replace')
match = re.search(r'^DIAGNOSTIC_INFERIOR_STATUS=(.*)$', text, re.M)
if match:
    record['inferior_observation'] = json.loads(match.group(1))
record['cleanup_kill_recorded'] = 'DIAGNOSTIC_CLEANUP=kill stopped inferior' in text
(out / 'status.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(record), flush=True)
