"""Cargo target runner for one authorized read-only sanitizer fault capture."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

out = Path(__file__).resolve().parent
binary = Path(sys.argv[1]).resolve()
now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
env_keys = ['CARGO_MANIFEST_DIR', 'CARGO_INCREMENTAL', 'CARGO_BUILD_JOBS', 'CARGO_TARGET_DIR',
            'LD_LIBRARY_PATH', 'RUSTFLAGS', 'RUST_TEST_THREADS', 'REXX_TEST_THREAD_SANITIZER',
            'TSAN_OPTIONS', 'CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER']
argv = ['timeout', '--kill-after=5', '600', 'gdb', '-nx', '-batch', '-x', str(out / 'capture.gdb'),
        '--args', *sys.argv[1:]]
record = dict(purpose='Diagnostic only. Closing TSan gate remains failed.', argv=argv,
              cargo_inherited_argv=sys.argv[1:], cwd=os.getcwd(), started=now(),
              relevant_environment={k: os.environ[k] for k in env_keys if k in os.environ},
              binary_canonical=str(binary), binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
              debugger_script_sha256=hashlib.sha256((out / 'capture.gdb').read_bytes()).hexdigest(),
              perturbation='Debugger startup with ASLR enabled. Actual SIGSEGV intercepted for read-only register/memory capture, then stopped inferior killed for cleanup.')
(out / 'target-command.json').write_text(json.dumps(record, indent=2) + '\n')
with (out / 'debugger.stdout').open('xb') as stdout, (out / 'debugger.stderr').open('xb') as stderr:
    child = subprocess.Popen(argv, stdout=stdout, stderr=stderr)
    record['debugger_timeout_pid'] = child.pid
    (out / 'target-live.json').write_text(json.dumps(record, indent=2) + '\n')
    status = child.wait()
record.update(status=status, finished=now(), reports=[p.name for p in out.glob('tsan.*')])
text = (out / 'debugger.stdout').read_text(errors='replace')
match = re.search(r'^DIAGNOSTIC_INFERIOR_STATUS=(.*)$', text, re.M)
if match:
    record['inferior_observation'] = json.loads(match.group(1))
record['cleanup_kill_recorded'] = 'DIAGNOSTIC_CLEANUP=kill ' in text
(out / 'target-status.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(record), flush=True)
sys.exit(status)
