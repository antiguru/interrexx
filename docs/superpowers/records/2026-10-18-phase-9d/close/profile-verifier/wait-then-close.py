"""Wait for a quiet host, start the close at HEAD, drive it, and log host load throughout."""
import subprocess, sys, threading, time
from pathlib import Path

root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
close = root / 'docs/superpowers/records/2026-10-18-phase-9d/close'
pad = Path(__file__).parent
log = pad / 'host-load.log'


def load():
    return float(Path('/proc/loadavg').read_text().split()[0])


def sample():
    while True:
        with log.open('a') as out:
            out.write(f"{time.strftime('%FT%TZ', time.gmtime())} {Path('/proc/loadavg').read_text()}")
        time.sleep(60)


threading.Thread(target=sample, daemon=True).start()
quiet = 0
deadline = time.monotonic() + 4 * 3600
while quiet < 3:
    if time.monotonic() > deadline:
        print('host never quiet; close not started', flush=True)
        sys.exit(3)
    quiet = quiet + 1 if load() < 12 else 0
    time.sleep(60)
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
print('host quiet, starting at ' + head, flush=True)
subprocess.check_call(['bash', str(close / 'run.sh'), 'start', '--head', head], cwd=root)
sys.exit(subprocess.run([sys.executable, str(pad / 'drive-close.py')], cwd=root).returncode)
