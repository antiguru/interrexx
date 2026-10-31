# Copyright (c) 2026 Rexx Language Association. All rights reserved.
# This program and the accompanying materials are made available under the
# terms of the Common Public License v1.0 which accompanies this distribution.
# A copy is also available at https://www.oorexx.org/license.html.

from pathlib import Path
import json
import os
import subprocess

ROOT = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
RECORDS = ROOT / 'docs/superpowers/records/2026-10-18-phase-9d/close/corrections'
env = os.environ.copy()
env.update(CARGO_INCREMENTAL='0', CARGO_TARGET_DIR='/tmp/codex-p9d-t88-target')
checks = [
    ('refusal-verify', ['cargo', 'test', '-p', 'rexx-exec', '--test', 'refusal_sites', '-j4', '--', '--test-threads=1']),
    ('sourceline-verify', ['cargo', 'test', '-p', 'rexx-parse', '--test', 'sourceline_oracle', '-j4']),
    ('fmt-final', ['cargo', 'fmt', '--all', '--check']),
    ('clippy-touched', ['cargo', 'clippy', '-p', 'rexx-exec', '--all-targets', '-j4', '--', '-D', 'warnings']),
    ('build-cli', ['cargo', 'build', '-p', 'rexx-exec', '--bin', 'rexx', '-j4']),
]
for name, arguments in checks:
    output = RECORDS / (name + os.environ.get('CHECK_RECORD_SUFFIX', ''))
    assert not output.with_suffix('.status').exists()
    command = ['timeout', '-k', '5', '580', *arguments]
    with output.with_suffix('.stdout').open('wb') as stdout, output.with_suffix('.stderr').open('wb') as stderr:
        result = subprocess.run(command, cwd=ROOT / 'rust', env=env, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr)
    output.with_suffix('.status').write_text(str(result.returncode) + '\n')
    output.with_suffix('.json').write_text(json.dumps({'command': command, 'cwd': str(ROOT / 'rust'), 'status': result.returncode}, indent=2) + '\n')
    print(name, result.returncode, flush=True)
    if result.returncode:
        raise SystemExit(result.returncode)
