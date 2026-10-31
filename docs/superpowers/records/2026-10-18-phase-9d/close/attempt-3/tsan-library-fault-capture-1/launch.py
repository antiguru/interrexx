"""One original Cargo-selected library invocation with an explicit target runner."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time

root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
out = Path(__file__).resolve().parent
helper = root / 'docs/superpowers/records/2026-10-18-phase-9d/close/tsan.sh'
target = root / '.superpowers/sdd/2026-10-18-phase-9d/task10-targets/ee52ea5e62daeb6bc35ca4475de4d1e7a96a4bfe/tsan'
text = helper.read_text()
skips = text.split('DEPTH="', 1)[1].split('"', 1)[0].splitlines() + text.split('TIMING="', 1)[1].split('"', 1)[0].splitlines()
assert len(skips) == 15
argv = ['timeout', '--kill-after=15', '600', 'cargo', '+nightly', 'test', '-Zbuild-std',
        '--target', 'x86_64-unknown-linux-gnu', '-p', 'rexx-exec', '--lib', '--', '--exact']
for name in skips:
    argv += ['--skip', name]
env = os.environ.copy()
env.update(CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='4', CARGO_TARGET_DIR=str(target),
           RUSTFLAGS='-Zsanitizer=thread', REXX_TEST_THREAD_SANITIZER='1',
           TSAN_OPTIONS=f'log_path={out}/tsan suppressions={root}/rust/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1',
           CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=f'python3 {out}/target-runner.py')
now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
record = dict(purpose='One authorized diagnostic Cargo library fault capture, never acceptance evidence.',
              argv=argv, cwd=str(root / 'rust'), started=now(), memory_cap='8G', timeout_seconds=600,
              environment_changes={k: env[k] for k in ['CARGO_INCREMENTAL', 'CARGO_BUILD_JOBS', 'CARGO_TARGET_DIR', 'RUSTFLAGS', 'REXX_TEST_THREAD_SANITIZER', 'TSAN_OPTIONS', 'CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER']},
              helper_sha256=hashlib.sha256(helper.read_bytes()).hexdigest(),
              original_failed_gate_unchanged=True)
(out / 'command.json').write_text(json.dumps(record, indent=2) + '\n')
unexpected_compile = False
clock = time.monotonic()
last = clock
with (out / 'cargo.stdout').open('xb') as stdout, (out / 'cargo.stderr').open('xb') as stderr:
    child = subprocess.Popen(argv, cwd=root / 'rust', env=env, stdout=stdout, stderr=stderr)
    record['cargo_timeout_pid'] = child.pid
    (out / 'live-process.json').write_text(json.dumps(record, indent=2) + '\n')
    while child.poll() is None:
        if not unexpected_compile and any(line.lstrip().startswith('Compiling ') for line in (out / 'cargo.stderr').read_text(errors='replace').splitlines()):
            unexpected_compile = True
            print('Unexpected compilation observed, refusing diagnostic invocation.', flush=True)
            child.terminate()
        current = time.monotonic()
        if current-last >= 30:
            print(json.dumps(dict(observed=now(), elapsed_seconds=current-clock,
                                  debugger_started=(out / 'target-live.json').exists(),
                                  unexpected_compile=unexpected_compile)), flush=True)
            last = current
        time.sleep(0.1)
record.update(status=child.returncode, finished=now(), unexpected_compile=unexpected_compile,
              elapsed_seconds=time.monotonic()-clock, reports=[p.name for p in out.glob('tsan.*')])
(out / 'status.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(record), flush=True)
