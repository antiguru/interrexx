# Copyright (c) 2026 Rexx Language Association. All rights reserved.
# This program and the accompanying materials are made available under the
# terms of the Common Public License v1.0 which accompanies this distribution.
# A copy is also available at https://www.oorexx.org/license.html.

from pathlib import Path
import json
import os
import subprocess

ROOT = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
RECORDS = ROOT / 'docs/superpowers/records/2026-10-18-phase-9d/close/guard-followup'
HEAD = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip()
ENV = os.environ.copy()
ENV.update(CARGO_INCREMENTAL='0', CARGO_TARGET_DIR='/tmp/codex-p9d-t88-target')
checks = [
    ('tests', ['cargo', 'test', '-p', 'rexx-exec', '--lib', '--test', 'dispatch_seam', '--test', 'close_corrections', '--test', 'semantic_boundaries', '--test', 'refusal_sites', '--test', 'refusal_dispositions', '--test', 'closed_phases', '-j4']),
    ('fmt', ['cargo', 'fmt', '--all', '--check']),
    ('clippy', ['cargo', 'clippy', '-p', 'rexx-exec', '--all-targets', '-j4', '--', '-D', 'warnings']),
]
for name, arguments in checks:
    output = RECORDS / name
    assert not output.with_suffix('.status').exists()
    command = ['timeout', '-k', '5', '580', *arguments]
    with output.with_suffix('.stdout').open('wb') as stdout, output.with_suffix('.stderr').open('wb') as stderr:
        result = subprocess.run(command, cwd=ROOT / 'rust', env=ENV, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr)
    output.with_suffix('.status').write_text(str(result.returncode) + '\n')
    output.with_suffix('.json').write_text(json.dumps({'command': command, 'cwd': str(ROOT / 'rust'), 'status': result.returncode, 'source_commit': HEAD, 'outer_memory_cap': '8G', 'environment': {'CARGO_INCREMENTAL': '0', 'CARGO_TARGET_DIR': ENV['CARGO_TARGET_DIR']}}, indent=2) + '\n')
    assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip() == HEAD
    print(name, result.returncode, flush=True)
    if result.returncode:
        raise SystemExit(result.returncode)
