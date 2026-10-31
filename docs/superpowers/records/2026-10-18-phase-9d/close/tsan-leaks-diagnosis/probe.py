#!/usr/bin/env python3
"""Bounded probes using existing binaries without changing repository sources."""
import hashlib
import json
import os
from pathlib import Path
import select
import signal
import subprocess
import time

ROOT = Path(__file__).resolve().parents[6]
OUT = Path(__file__).resolve().parent
TARGET = ROOT / '.superpowers/sdd/2026-10-18-phase-9d/task10-targets/9922e56f3bf6c72442d796f8a43f631a86621abd'
BINARIES = {'tsan': TARGET / 'tsan/x86_64-unknown-linux-gnu/debug/rexx',
            'release': TARGET / 'workspace/release/rexx'}
BASE = "call on halt name h\ns.0 = 5; s.1 = 'old'\nsay 'a'\naddress system 'echo first; sleep 5; echo late' with output stem s.\nsay 'b' rc s.0 s.1\nexit\nh: say 'halted' sigl; return\n"
CASES = [
    ('basic-command', 'tsan', "address system 'printf out; printf err >&2'\nsay rc\n", False),
    ('redirected-normal', 'tsan', BASE.replace('sleep 5', 'sleep 0.1'), False),
    ('redirected-halt', 'tsan', BASE, True),
    ('redirected-halt-drained', 'tsan', BASE.replace("say 'b' rc s.0 s.1\nexit", "say 'b' rc s.0 s.1\ncall syssleep 6\nexit"), True),
    ('redirected-halt-both-pipes', 'tsan', BASE.replace('with output stem s.', 'with output stem s. error stem e.'), True),
    ('redirected-halt-release', 'release', BASE, True),
]

results = []
for name, profile, source, interrupt in CASES:
    case = OUT / name
    case.mkdir(exist_ok=True)
    script = case / 'program.rex'
    script.write_text(source)
    env = os.environ.copy()
    env['TSAN_OPTIONS'] = (f'log_path={case}/tsan suppressions={ROOT}/rust/tsan.supp '
                           'allocator_may_return_null=1 second_deadlock_stack=1')
    argv = ['env', '--default-signal=INT,TERM,HUP', str(BINARIES[profile]), str(script)]
    start = time.monotonic()
    child = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE, env=env, start_new_session=True)
    captured = b''
    if interrupt:
        ready_until = start + 10
        while not captured.endswith(b'a\n') and time.monotonic() < ready_until:
            if select.select([child.stdout], [], [], 0.1)[0]:
                data = os.read(child.stdout.fileno(), 4096)
                if not data:
                    break
                captured += data
        assert captured.endswith(b'a\n'), (name, captured)
        time.sleep(0.5)
        os.kill(child.pid, signal.SIGINT)
    status = child.wait(timeout=20)
    process_ended = time.monotonic() - start
    stdout, stderr = child.communicate(timeout=12)
    (case / 'stdout').write_bytes(captured + stdout)
    (case / 'stderr').write_bytes(stderr)
    row = dict(name=name, profile=profile, argv=argv, tsan_options=env['TSAN_OPTIONS'],
               pid=child.pid, status=status, process_seconds=process_ended,
               pipe_seconds=time.monotonic() - start,
               binary_sha256=hashlib.sha256(BINARIES[profile].read_bytes()).hexdigest(),
               reports=[p.name for p in case.glob('tsan.*')])
    (case / 'status.json').write_text(json.dumps(row, indent=2) + '\n')
    results.append(row)
    print(json.dumps(row), flush=True)
(OUT / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
