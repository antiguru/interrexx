"""One authorized selected-library diagnostic replay, never acceptance evidence."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
out = Path(__file__).resolve().parent
helper = root / 'docs/superpowers/records/2026-10-18-phase-9d/close/tsan.sh'
target = root / '.superpowers/sdd/2026-10-18-phase-9d/task10-targets/ee52ea5e62daeb6bc35ca4475de4d1e7a96a4bfe/tsan'
binary = target / 'x86_64-unknown-linux-gnu/debug/build/rexx-exec/6c80e238cfcc855d/out/rexx_exec-6c80e238cfcc855d'
text = helper.read_text()
skips = text.split('DEPTH="', 1)[1].split('"', 1)[0].splitlines()
skips += text.split('TIMING="', 1)[1].split('"', 1)[0].splitlines()
assert len(skips) == 15
argv = ['timeout', '--kill-after=15', '600', str(binary), '--exact']
for name in skips:
    argv += ['--skip', name]
env = os.environ.copy()
env.update(CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='4', CARGO_TARGET_DIR=str(target),
           RUSTFLAGS='-Zsanitizer=thread', REXX_TEST_THREAD_SANITIZER='1',
           TSAN_OPTIONS=f'log_path={out}/tsan suppressions={root}/rust/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1')

def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()

record = dict(purpose='Diagnostic only. Original closing TSan gate remains failed regardless of this result.',
              argv=argv, cwd=str(root / 'rust'), started=now(),
              head=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
              sha256={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in [binary, helper]},
              relevant_environment={k: v for k, v in env.items() if k.startswith(('CARGO_', 'REXX_', 'RUST_')) or k in ('RUSTFLAGS', 'TSAN_OPTIONS')},
              memory_cap='8G', timeout_seconds=600, default_test_concurrency=True, default_output_capture=True,
              cgroup=Path('/proc/self/cgroup').read_text())
(out / 'command.json').write_text(json.dumps(record, indent=2) + '\n')
captured = False
began = time.monotonic()
last_progress = began
with (out / 'stdout').open('xb') as stdout, (out / 'stderr').open('xb') as stderr:
    child = subprocess.Popen(argv, cwd=root / 'rust', env=env, stdout=stdout, stderr=stderr)
    record['timeout_pid'] = child.pid
    while child.poll() is None:
        if not captured:
            children = Path(f'/proc/{child.pid}/task/{child.pid}/children').read_text().split()
            for pid in children:
                try:
                    if Path(f'/proc/{pid}/exe').resolve() == binary:
                        (out / 'maps').write_text(Path(f'/proc/{pid}/maps').read_text())
                        record.update(binary_pid=int(pid), maps_captured=now())
                        (out / 'live-process.json').write_text(json.dumps(record, indent=2) + '\n')
                        captured = True
                except FileNotFoundError:
                    pass
        clock = time.monotonic()
        if clock - last_progress >= 30:
            progress = dict(observed=now(), elapsed_seconds=clock-began,
                            binary_pid=record.get('binary_pid'), reports=[p.name for p in out.glob('tsan.*')])
            (out / 'progress.json').write_text(json.dumps(progress, indent=2) + '\n')
            print(json.dumps(progress), flush=True)
            last_progress = clock
        time.sleep(0.01 if not captured else 0.1)
record.update(status=child.returncode, finished=now(), elapsed_seconds=time.monotonic()-began,
              reports=[p.name for p in out.glob('tsan.*')],
              stdout_sha256=hashlib.sha256((out / 'stdout').read_bytes()).hexdigest(),
              stderr_sha256=hashlib.sha256((out / 'stderr').read_bytes()).hexdigest())
(out / 'status.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(record), flush=True)
