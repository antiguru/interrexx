#!/usr/bin/env python3
"""Read-only Phase 9d close evidence verifier. Never runs a profiled program.

Exit 0 means valid evidence, even with budget OVER diagnostics. Exit 2 means
invalid/incomplete evidence. Neither exit approves performance acceptance.
Uses the repository cgsum.py self-count algorithm, with completion checks.
"""
# Copyright (c) 2026 Rexx Language Association. All rights reserved.
# Distributed under the Common Public License v1.0.
import os
import argparse
import csv
import hashlib
import json
from pathlib import Path
import re
import shlex
import statistics
import sys

DEFAULT_ROOT = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
PIN_ROOT = Path('/home/moritz/dev/repos/ooRexx-p9-pin')
HEAD = 'ee52ea5e62daeb6bc35ca4475de4d1e7a96a4bfe'
PIN_HASH = 'c4b4e8c4ce7f1f12e4a73fa37cb15eca262717092c2c1aed727febf437523c67'
TIMED = {'heapshape', 'rexxcps'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=DEFAULT_ROOT)
    parser.add_argument('--profiles', type=Path, default=PIN_ROOT / 'cg-9d/close')
    parser.add_argument('--output', type=Path)
    parser.add_argument('--head', default=HEAD)
    parser.add_argument('--pin-binary', type=Path, default=PIN_ROOT / 'rexx-run')
    parser.add_argument('--pin-sha256', default=PIN_HASH)
    parser.add_argument('--head-binary', type=Path)
    parser.add_argument('--head-sha256', help='Otherwise read successful close/performance-copy.json')
    args = parser.parse_args()
    root = args.root.resolve()
    profiles = args.profiles.resolve()
    output = (args.output or root / 'docs/superpowers/records/2026-10-18-phase-9d/close/performance-analysis').resolve()
    if output == profiles or profiles in output.parents:
        parser.error('Output must be outside the immutable profile directory')
    output.mkdir(parents=True, exist_ok=False)
    errors, diagnostics, manifest, counts, streams = [], [], {}, {}, {}

    def fail(context, message):
        errors.append({'context': str(context), 'error': str(message)})

    def fingerprint(path):
        path = Path(path)
        digest = hashlib.sha256()
        size = 0
        with path.open('rb') as stream:
            for block in iter(lambda: stream.read(1 << 20), b''):
                digest.update(block)
                size += len(block)
        entry = {'sha256': digest.hexdigest(), 'bytes': size}
        manifest[str(path)] = entry
        return entry

    def text(path):
        fingerprint(path)
        return Path(path).read_text()

    def require(condition, message):
        if not condition:
            raise ValueError(message)

    programs, paths, expected_hashes, binaries = [], {}, {}, {}
    instrument_version = ''
    try:
        bench = root / 'rust/bench-programs'
        instrument = text(bench / 'callgrind.sh')
        fingerprint(bench / 'cgsum.py')
        fingerprint(Path(__file__).resolve())
        ping = re.search(r'^PINGPONG="([^"]+)"', instrument, re.M).group(1).split()
        programs = re.search(r'^PROGRAMS="([^"]+)"', instrument, re.M).group(1).replace('$PINGPONG', ' '.join(ping)).split()
        require(len(programs) == 33 and len(set(programs)) == 33, 'Expected exactly 33 distinct programs')
        disk = [p.stem for p in bench.glob('*.rex')] + [p.stem for p in (bench / 'pingpong').glob('*.rex')] + ['rexxcps']
        require(sorted(disk) == sorted(programs), 'Instrument/directory program enumeration mismatch')
        require(set(re.search(r'^TIMED="([^"]+)"', instrument, re.M).group(1).split()) == TIMED, 'TIMED exemption changed')
        for name in programs:
            paths[name] = ((root / 'rust/bench-rexxcps/rexxcps.rex') if name == 'rexxcps' else bench / 'pingpong' / (name + '.rex') if name in ping else bench / (name + '.rex')).resolve()
            fingerprint(paths[name])
        binaries = {'pin': args.pin_binary.resolve(strict=True), 'head': (args.head_binary or PIN_ROOT / ('rexx-9d-' + args.head)).resolve(strict=True)}
        head_hash = args.head_sha256
        if head_hash is None:
            copied = json.loads(text(root / 'docs/superpowers/records/2026-10-18-phase-9d/close/performance-copy.json'))
            require(copied['status'] == 0, 'Performance copy did not succeed')
            head_hash = copied['sha256']
        expected_hashes = {'pin': args.pin_sha256, 'head': head_hash}
        for name, path in binaries.items():
            require(fingerprint(path)['sha256'] == expected_hashes[name], f'{name} executable hash mismatch')
        header = text(profiles / 'binaries.txt')
        require(re.search(r'\brounds=3\b', header) is not None, 'Expected three recorded rounds')
        require(re.search(r'^# valgrind-\S+$', header, re.M) is not None, 'Missing instrument version')
        instrument_version = re.search(r'^# valgrind-(\S+)$', header, re.M).group(1)
        declared = {}
        for line in header.splitlines():
            match = re.fullmatch(r'# (\S+) (.+) ([0-9a-f]{64})', line)
            if match:
                name, path, digest = match.groups()
                require(name not in declared, 'Duplicate binary manifest entry')
                declared[name] = (Path(path).resolve(), digest)
        require(declared == {name: (path, expected_hashes[name]) for name, path in binaries.items()}, 'Unexpected binary manifest paths/hashes')
    except (OSError, ValueError, KeyError, AttributeError) as error:
        fail('inputs', error)

    rows = {}
    try:
        for line in text(profiles / 'summary.tsv').splitlines():
            cells = line.split('\t')
            require(len(cells) == 8, 'Summary row must have eight fields')
            key = (cells[0], cells[1], cells[2])
            require(key not in rows, f'Duplicate summary tuple {key}')
            rows[key] = cells
    except (OSError, ValueError) as error:
        fail('summary.tsv', error)
    expected = {(binary, program, f'r{round_}') for program in programs for binary in ['pin', 'head'] for round_ in range(1, 4)}
    if set(rows) != expected:
        fail('summary.tsv', {'missing': sorted(expected - set(rows)), 'unexpected': sorted(set(rows) - expected)})
    expected_files = {f'{program}.{binary}.{round_}.{suffix}' for binary, program, round_ in expected for suffix in ['cg', 'vg', 'out', 'err', 'rc', 'row']}
    actual_files = {p.name for suffix in ['cg', 'vg', 'out', 'err', 'rc', 'row'] for p in profiles.glob('*.' + suffix)}
    if actual_files != expected_files:
        fail('artifacts', {'missing': sorted(expected_files - actual_files), 'unexpected': sorted(actual_files - expected_files)})

    for binary, program, round_ in sorted(expected):
        prefix = profiles / f'{program}.{binary}.{round_}'
        try:
            suffix = lambda name: Path(str(prefix) + '.' + name)
            require(text(suffix('rc')).strip() == '0', 'Nonzero interpreter/Valgrind status')
            row = text(suffix('row')).strip().split('\t')
            require(row == rows[(binary, program, round_)], 'Raw row/summary disagreement')
            require(row[:3] == [binary, program, round_] and row[7] == 'rc=0', 'Invalid row identity/status')
            by_object, fields, current, skip, digest, size = {}, {}, None, False, hashlib.sha256(), 0
            terminated = False
            with suffix('cg').open('rb') as stream:
                for raw in stream:
                    digest.update(raw)
                    size += len(raw)
                    line = raw.decode('utf-8')
                    if line.strip() == 'desc: Trigger: Program termination':
                        terminated = True
                    if line.startswith(('summary:', 'totals:', 'events:', 'positions:', 'cmd:', 'creator:')):
                        name, value = line.split(':', 1)
                        require(name not in fields, f'Duplicate profile field {name}')
                        fields[name] = value.strip()
                    if line.startswith('ob='):
                        current = line[3:].strip()
                    elif line.startswith('calls='):
                        skip = True
                    elif line[:1].isdigit() or line[:1] in '+-*':
                        parts = line.split()
                        if len(parts) >= 2:
                            if skip:
                                skip = False
                            else:
                                by_object[current] = by_object.get(current, 0) + int(parts[-1])
            manifest[str(suffix('cg'))] = {'sha256': digest.hexdigest(), 'bytes': size}
            require(fields.get('creator') == 'callgrind-' + instrument_version, 'Profile instrument version differs')
            require(terminated, 'Missing normal profile termination marker')
            require(fields.get('events') == 'Ir' and fields.get('positions') == 'line', 'Unexpected cost format')
            require([os.path.normpath(part) for part in shlex.split(fields['cmd'])] == [os.path.normpath(str(binaries[binary])), os.path.normpath(str(paths[program]))], 'Actual profiled command/program differs')
            whole = int(fields['summary'])
            require(whole > 0 and int(fields['totals']) == whole and sum(by_object.values()) == whole, 'Truncated/inconsistent profile totals or self counts')
            libc = sum(value for obj, value in by_object.items() if obj and obj.endswith('libc.so.6'))
            loader = sum(value for obj, value in by_object.items() if obj and 'ld-linux' in obj)
            own = whole - libc - loader
            require([int(value) for value in row[3:7]] == [whole, libc, loader, own], 'Raw Ir/object counts disagree with row')
            vg = text(suffix('vg'))
            clean_lines = [re.sub(r'^==\d+==\s?', '', line).strip() for line in vg.splitlines()]
            allowed = r'^(?:Callgrind,.*|Copyright.*|Using Valgrind-.*|Command:.*|Parent PID: \d+|For interactive control,.*|Events\s*:\s*Ir|Collected\s*:\s*[0-9,]+|I\s+refs:\s*[0-9,]+)?$'
            unexpected = [line for line in clean_lines if not re.fullmatch(allowed, line)]
            commands = [line.removeprefix('Command:').strip() for line in clean_lines if line.startswith('Command:')]
            require(len(commands) == 1 and [os.path.normpath(part) for part in shlex.split(commands[0])] == [os.path.normpath(str(binaries[binary])), os.path.normpath(str(paths[program]))], 'Valgrind log command differs')
            require(any(line.startswith('Using Valgrind-' + instrument_version + ' ') for line in clean_lines), 'Valgrind log version differs')
            collected = re.findall(r'^Collected\s*:\s*([0-9,]+)$', '\n'.join(clean_lines), re.M)
            refs = re.findall(r'^I\s+refs:\s*([0-9,]+)$', '\n'.join(clean_lines), re.M)
            require(len(collected) == 1 and int(collected[0].replace(',', '')) == whole and len(refs) == 1 and int(refs[0].replace(',', '')) == whole, 'Missing/truncated Valgrind completion or Ir mismatch')
            out, err = fingerprint(suffix('out')), fingerprint(suffix('err'))
            with suffix('err').open('rb') as stream:
                preview = stream.read(4096).decode('utf-8', errors='backslashreplace')
            diagnostics.append({'profile': str(prefix), 'stderr': err, 'stderr_preview': preview, 'stderr_preview_truncated': err['bytes'] > 4096, 'vg_diagnostics': unexpected})
            require(not unexpected, 'Unexpected Valgrind diagnostics: ' + repr(unexpected))
            require(err['bytes'] == 0, 'Nonempty semantic stderr; inspect retained raw artifact')
            counts[(binary, program, round_)] = (whole, own)
            streams[(binary, program, round_)] = out['sha256']
        except (OSError, ValueError, KeyError, UnicodeError) as error:
            fail(prefix, error)

    # Keep a manifest even for artifacts rejected before their parser ran.
    for filename in sorted(actual_files):
        path = profiles / filename
        if str(path) not in manifest:
            try:
                fingerprint(path)
            except OSError as error:
                fail(path, error)
    for name, path in binaries.items():
        try:
            require(fingerprint(path)['sha256'] == expected_hashes[name], f'{name} executable changed during verification')
        except (OSError, ValueError, KeyError) as error:
            fail(path, error)

    for key, digest in streams.items():
        binary, program, round_ = key
        if program not in TIMED and digest != streams.get(('pin', program, 'r1')):
            fail(key, 'Semantic stdout mismatch against pin r1')
    metrics, budget = [], []
    table = ['program\tpin\thead\thead d%\tpin -libc\thead -libc\thead -libc d%\tpin spread%\thead spread%']
    for program in programs:
        if not all((binary, program, f'r{r}') in counts for binary in binaries for r in range(1, 4)) or set(binaries) != {'pin', 'head'}:
            continue
        medians, spreads = {}, {}
        for binary in binaries:
            whole = [counts[(binary, program, f'r{r}')][0] for r in range(1, 4)]
            own = [counts[(binary, program, f'r{r}')][1] for r in range(1, 4)]
            medians[binary] = (statistics.median(whole), statistics.median(own))
            spreads[binary] = 100 * (max(whole) - min(whole)) / medians[binary][0]
            if spreads[binary] > .01:
                fail(program, f'SPREAD {binary} {spreads[binary]:.8f}% > 0.01%')
        if medians['pin'][1] <= 0:
            fail(program, 'Nonpositive own-cost denominator')
            continue
        whole_delta, own_delta = [100 * (medians['head'][i] - medians['pin'][i]) / medians['pin'][i] for i in [0, 1]]
        flag = 'OVER' if whole_delta > .5 else ''
        if flag:
            budget.append({'program': program, 'whole_delta_percent': whole_delta, 'flag': flag, 'threshold_percent': .5})
        metrics.append([program, *medians['pin'], *medians['head'], whole_delta, own_delta, spreads['pin'], spreads['head'], flag])
        table.append(f"{program}\t{medians['pin'][0]:.0f}\t{medians['head'][0]:.0f}\t{whole_delta:+.4f}\t{medians['pin'][1]:.0f}\t{medians['head'][1]:.0f}\t{own_delta:+.4f}\t{spreads['pin']:.4f}\t{spreads['head']:.4f}")
    try:
        require(text(profiles / 'table.txt') == '\n'.join(table) + '\n', 'Original table differs from independent medians/spreads')
    except (OSError, ValueError) as error:
        fail('table.txt', error)
    with (output / 'metrics.tsv').open('x') as stream:
        writer = csv.writer(stream, delimiter='\t', lineterminator='\n')
        writer.writerow(['program', 'pin_whole', 'pin_own', 'head_whole', 'head_own', 'whole_delta_percent', 'own_delta_percent', 'pin_spread_percent', 'head_spread_percent', 'whole_budget_flag'])
        writer.writerows(metrics)
    result = {'valid': not errors, 'expected_profiles': 198, 'validated_profiles': len(counts), 'complete_programs': len(metrics), 'errors': errors, 'diagnostics': diagnostics, 'budget_diagnostics': budget, 'adjudication': 'pending, no automatic acceptance', 'timed_stdout_exemptions': sorted(TIMED), 'expected_binary_hashes': expected_hashes, 'source_head': args.head, 'limits': ['No fresh profile execution or reruns.', 'Own cost excludes only libc/loader and is not repository-only cost.', 'No cross-cohort historical residual attribution.', 'TIMED stdout receives no automatic semantic equivalence claim.', 'Unknown Valgrind log lines conservatively fail validity.', 'Runtime library and frozen-source provenance require the close runner records.', 'Rotation order is inferred from the committed instrument, not independently timed.', 'The output manifest hashes other generated files, not itself.']}
    (output / 'validation.json').write_text(json.dumps(result, indent=2) + '\n')
    (output / 'input-sha256.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
    generated = {p.name: fingerprint(p) for p in output.iterdir() if p.is_file()}
    (output / 'output-sha256.json').write_text(json.dumps(generated, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'valid': not errors, 'budget_OVER': len(budget), 'output': str(output)}))
    return 2 if errors else 0


if __name__ == '__main__':
    sys.exit(main())
