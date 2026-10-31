# Copyright (c) 2026 Rexx Language Association. All rights reserved.
# This program and the accompanying materials are made available under the
# terms of the Common Public License v1.0 which accompanies this distribution.
# A copy is also available at https://www.oorexx.org/license.html.

from pathlib import Path
import hashlib
import json
import os
import resource
import shutil
import subprocess
import tempfile

ROOT = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
RECORDS = ROOT / ('docs/superpowers/records/2026-10-18-phase-9d/close/corrections/' + os.environ.get('PAIR_RECORD_DIR', 'paired'))
NAMES = [
    'constant_result_cleanup', 'raise_additional_items', 'raise_additional_return_mutation',
    'define_methods_supplier_override', 'define_methods_protected_supplier',
    'raise_additional_modes', 'method_table_supplier_boundaries',
    'define_methods_supplier', 'class_mutators_user_class', 'constant_expression_trace',
    'class_constant_expression_self', 'constant_expression_trace_labels',
    'raise_array_additional_objects', 'raise_syntax_string_additional',
]
BINARIES = {
    'crate': Path('/tmp/codex-p9d-t88-target/debug/rexx'),
    'oracle': Path('/home/moritz/dev/repos/ooRexx/build/bin/rexx'),
}

def cap_oracle():
    resource.setrlimit(resource.RLIMIT_AS, (1073741824, 1073741824))

head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip()
for name in NAMES:
    record = RECORDS / name
    record.mkdir(parents=True, exist_ok=False)
    scratch = Path(tempfile.mkdtemp(prefix='p9d-correction-pair-'))
    source = scratch / 'probe.rex'
    shutil.copyfile(ROOT / 'rust/corpus/lang' / f'{name}.rex', source)
    results = {}
    for side, binary in BINARIES.items():
        env = {key: value for key, value in os.environ.items() if not key.startswith('REXX_')}
        env.pop('LD_PRELOAD', None)
        env['LD_LIBRARY_PATH'] = '/home/moritz/dev/repos/ooRexx/build/lib'
        cwd = scratch if side == 'oracle' else ROOT / 'rust'
        command = ['timeout', '-k', '5', '30', str(binary), str(source)]
        result = subprocess.run(command, cwd=cwd, env=env, preexec_fn=cap_oracle if side == 'oracle' else None, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        results[side] = result
        (record / f'{side}.stdout').write_bytes(result.stdout)
        (record / f'{side}.stderr').write_bytes(result.stderr)
        (record / f'{side}.status').write_text(str(result.returncode) + '\n')
        (record / f'{side}.json').write_text(json.dumps({
            'command': command, 'cwd': str(cwd), 'status': result.returncode,
            'source_head': head, 'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
            'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
            'outer_memory_cap': '2G', 'oracle_address_space_bytes': 1073741824 if side == 'oracle' else None,
        }, indent=2) + '\n')
    a, b = results['crate'], results['oracle']
    matches = {channel: getattr(a, channel) == getattr(b, channel) for channel in ['stdout', 'stderr', 'returncode']}
    (record / 'comparison.json').write_text(json.dumps(matches, indent=2) + '\n')
    print(name, matches, flush=True)
    if not all(matches.values()):
        raise SystemExit(1)
