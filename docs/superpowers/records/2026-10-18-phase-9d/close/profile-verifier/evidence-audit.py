#!/usr/bin/env python3
"""Derive close evidence without modifying inputs or adjudicating new licences."""
import argparse
import csv
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
RECORDS = ROOT / 'docs/superpowers/records/2026-10-18-phase-9d'
FIELDS = ['group', 'test', 'row', 'oracle', 'crate']
TABLE_FIELDS = ['group', 'side', 'expected', 'started', 'passed', 'failed', 'blocked', 'blocked at load', 'ended before a test', 'iterations', 'ended normally', 'cap', 'tests ran']
VERDICT_FIELDS = ['group', 'verdict', 'crate', 'oracle denominator']
DIFF_FIELDS = ['group', 'test', 'oracle', 'crate']
INPUTS = {}
ERRORS = []


def check(condition, message):
    if not condition:
        ERRORS.append(message)


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(chunk)
    return value.hexdigest()


def consume(path):
    path = Path(path)
    INPUTS[str(path)] = digest(path)
    return path.read_text()


def load(path, fields, suite=True):
    # Keep Task 9's schema and revision checks, rather than accepting counts alone.
    lines = consume(path).splitlines()
    if suite:
        assert '# ootest revision 13178' in lines, path
    reader = csv.DictReader((line for line in lines if not line.startswith('#')), delimiter='\t')
    assert reader.fieldnames == fields, (path, reader.fieldnames)
    rows = list(reader)
    assert all(set(row) == set(fields) and None not in row.values() for row in rows), path
    return lines, rows


def tsv(out, name, fields, rows):
    with (out / name).open('x') as stream:
        writer = csv.writer(stream, delimiter='\t', lineterminator='\n')
        writer.writerow(fields)
        writer.writerows(rows)


def keyed_suite(root):
    lines, rows = load(root / 'tests.tsv', FIELDS)
    assert '# groups all' in lines, root
    keyed = {(row['group'], row['test']): row for row in rows}
    assert len(keyed) == len(rows) and rows, root
    return keyed


def suite_structure(root, tests):
    _, tables = load(root / 'table.tsv', TABLE_FIELDS)
    _, verdicts = load(root / 'verdicts.tsv', VERDICT_FIELDS)
    _, differences = load(root / 'oracle-differences.tsv', DIFF_FIELDS)
    groups = {group for group, _ in tests}
    assert len({(r['group'], r['side']) for r in tables}) == len(tables)
    assert {(r['group'], r['side']) for r in tables} == {(g, s) for g in groups for s in ['oracle', 'crate']}
    assert len({r['group'] for r in verdicts}) == len(verdicts) == len(groups)
    assert {r['group'] for r in verdicts} == groups
    check(all(row['verdict'] == 'green' for row in verdicts), 'Non-green suite group verdict')
    check(all(row['oracle denominator'] == 'equal' for row in verdicts), 'Suite oracle denominator differs')
    for row in tables:
        check(row['ended normally'] == 'yes', 'Abnormal suite end: ' + row['group'] + '/' + row['side'])
    check(len({(r['group'], r['test']) for r in differences}) == len(differences), 'Duplicate oracle difference key')
    # The harness report is an unfiltered semantic difference report. Preserve it exactly.
    for row in differences:
        key = row['group'], row['test']
        check(key in tests and all(tests[key][f] == row[f] for f in ['oracle', 'crate']), 'Oracle difference report inconsistent: ' + str(key))
    expected = {k for k, row in tests.items() if row['oracle'] != row['crate']}
    check({(r['group'], r['test']) for r in differences} == expected, 'Oracle difference report is incomplete')
    return tables, verdicts, differences


def delta(out, name, before, after):
    changes = []
    for group, test in sorted(before.keys() | after.keys()):
        a, b = before.get((group, test)), after.get((group, test))
        if a != b:
            changes.append([group, test, 'added' if a is None else 'removed' if b is None else 'changed', *[(a or {}).get(k, '<absent>') for k in FIELDS[2:]], *[(b or {}).get(k, '<absent>') for k in FIELDS[2:]]])
    tsv(out, name + '.tsv', ['group', 'test', 'change', 'old row', 'old oracle', 'old crate', 'new row', 'new oracle', 'new crate'], changes)
    check(before.keys() == after.keys(), name + ': key population changed')
    return dict(old_tests=len(before), new_tests=len(after), changed_rows=len(changes), added=len(after.keys() - before.keys()), removed=len(before.keys() - after.keys()))


def suite_derivations(out, suite):
    old9c = keyed_suite(ROOT / 'docs/superpowers/records/2026-10-15-phase-9c/close/suite')
    old9 = keyed_suite(RECORDS / 't9/suite')
    current = keyed_suite(suite)
    tables, verdicts, differences = suite_structure(suite, current)
    summary = {name: delta(out, name, before, current) for name, before in [('delta-vs-9c', old9c), ('delta-vs-t9', old9)]}
    check({g for g, _ in current} == {g for g, _ in old9} == {g for g, _ in old9c}, 'Group population changed')
    # Task 9 already records the approved 9c-to-9d licences and hostemu refusal.
    # New results must preserve those row owners and oracle answers, or improve crate output.
    for key in sorted(old9.keys() & current.keys()):
        a, b = old9[key], current[key]
        check(a['row'] == b['row'], 'Unreviewed licence/owner change: ' + str(key))
        check(a['oracle'] == b['oracle'], 'Changed oracle result requires review: ' + str(key))
        check(a['crate'] == b['crate'] or b['crate'] == 'passed', 'Unapproved crate regression/change versus Task 9: ' + str(key))
    for key in sorted(old9c.keys() & current.keys()):
        a, b = old9c[key], current[key]
        check(a['crate'] != 'passed' or b['crate'] == 'passed', 'Passed-to-nonpassed regression versus 9c: ' + str(key))
    header = consume(ROOT / 'rust/corpus/suite-9d-set.txt')
    command = next(line[1:].strip() for line in header.splitlines() if line.startswith('#') and 'awk -F' in line)
    assert command == r'''awk -F'\t' '$3=="-" && $4=="passed" && $5=="failed" {print $1"\t"$2}' tests.tsv''', command
    # Execute the committed literal command with the selected suite as cwd.
    derived = subprocess.run(['bash', '-c', command], cwd=suite, capture_output=True, check=False)
    (out / 'set-header.stdout').write_bytes(derived.stdout)
    (out / 'set-header.stderr').write_bytes(derived.stderr)
    check(derived.returncode == 0 and not derived.stdout and not derived.stderr, 'Literal set-header derivation did not exit zero with empty output')
    (out / 'set-header-command.json').write_text(json.dumps(dict(command=command, cwd=str(suite), status=derived.returncode), indent=2) + '\n')
    extensions = [row for row in current.values() if row['group'].startswith(('extensions/json/', 'extensions/yaml/', 'extensions/dateparser/'))]
    groups = {'extensions/json/json', 'extensions/json/json_02', 'extensions/yaml/yaml', 'extensions/dateparser/DateParser', 'extensions/dateparser/DateFormatter'}
    check({r['group'] for r in extensions} == groups, 'Expected extension group membership differs')
    check(all(r['oracle'] == r['crate'] == 'passed' and r['row'] == '-' for r in extensions), 'Pure-Rexx extension assertion did not pass without exclusion')
    tsv(out, 'extensions.tsv', FIELDS, ([r[f] for f in FIELDS] for r in sorted(extensions, key=lambda r: (r['group'], r['test']))))
    binary_groups = {'utilities/rexx/rexx_command', 'utilities/rexxc/rexxc', 'API/oo/METHOD', 'base/class/Method', 'base/class/Routine'}
    binaries = [r for r in current.values() if r['group'] in binary_groups]
    check({r['group'] for r in binaries} == binary_groups, 'Parent binary group absent')
    exception = ('utilities/rexx/rexx_command', 'TEST_PASSINGARGS7')
    for row in binaries:
        key = row['group'], row['test']
        if key == exception:
            check(row['row'] == 'Deviation(40)' and row['oracle'] == 'passed' and row['crate'] in {'passed', 'failed'}, 'Deviation 40 binary row changed unexpectedly')
        else:
            check(row['oracle'] == row['crate'] == 'passed', 'Binary group assertion did not pass: ' + str(key))
    required = {('API/oo/METHOD', 'TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA')}
    required |= {(g, t) for g in ['base/class/Method', 'base/class/Routine'] for t in ['TEST_NEW_ARRAY_FROM_FILE', 'TEST_NEW_FILE_COMPILED']}
    check(required <= current.keys(), 'Parent compiled/native test key absent')
    tsv(out, 'parent-binary-rows.tsv', FIELDS, ([r[f] for f in FIELDS] for r in sorted(binaries, key=lambda r: (r['group'], r['test']))))
    tsv(out, 'oracle-differences.tsv', DIFF_FIELDS, ([r[f] for f in DIFF_FIELDS] for r in differences))
    return dict(deltas=summary, tests=len(current), groups=len(verdicts), extension_rows=len(extensions), oracle_differences=len(differences), binary_exception=current.get(exception))


def dispositions(out, pending):
    fields = ['site', 'class', 'method', 'receiver', 'reached', 'probe', 'oracle', 'disposition']
    _, refusals = load(RECORDS / 'refusals.tsv', fields, suite=False)
    check(len(refusals) == 15, 'Historical refusal enumeration population changed')
    check(len({tuple(r[f] for f in fields[:4]) for r in refusals}) == len(refusals), 'Duplicate refusal identity')
    for r in refusals:
        check(r['disposition'] in {'done (Task 7.1)', 'guard', 'oracle-crash'} and bool(r['site']) and bool(r['receiver']) and bool(r['oracle']), 'Undisposed refusal: ' + str(r))
    tsv(out, 'refusal-dispositions.tsv', fields, ([r[f] for f in fields] for r in refusals))
    fields = ['item', 'kind', 'probe', 'proposed', 'reason', 'ruled']
    _, inventory = load(RECORDS / 'queued-inventory.tsv', fields, suite=False)
    keyed = {r['item']: r for r in inventory}
    check(len(keyed) == len(inventory), 'Duplicate inventory item')
    queued = sorted(p for p in (ROOT / '.superpowers/sdd/queued').iterdir() if p.is_file())
    firstlines = []
    for path in queued:
        lines = consume(path).splitlines()
        first = lines[0] if lines else ''
        firstlines.append([path.name, first, keyed.get(path.name, {}).get('ruled', '<missing>')])
        check(path.name in keyed, 'Queued file absent from inventory: ' + path.name)
        check(bool(first) and re.search(r'(?i)closed|open|ruled|deviation|phase|not (?:a divergence|copied)', first), 'No disposition first line: ' + path.name)
        check(not re.search(r'(?i)not yet ruled|awaiting (?:a )?ruling', first), 'Unruled queued item: ' + path.name)
    check(len(queued) == 245, 'Queued filename population differs from frozen 245-file baseline')
    for row in inventory:
        check(bool(row['ruled']) and row['ruled'] not in {'pending', 'unruled'}, 'Unruled inventory row: ' + row['item'])
        if row['ruled'] == '9d':
            check(pending == 'allow' and row['item'] == 'gate:phase-9a-gate.md:466', 'Pending Phase 9d inventory obligation: ' + row['item'])
    tsv(out, 'queued-first-lines.tsv', ['item', 'first line', 'ruled'], firstlines)
    tsv(out, 'inventory.tsv', fields, ([r[f] for f in fields] for r in inventory))
    for name in ['refusal-sites.tsv', 'refusal-dispositions.tsv', 'suite-licences.txt', 'suite-not-run.txt']:
        consume(ROOT / 'rust/corpus' / name)
    consume(ROOT / 'docs/superpowers/plans/phase-9-exclusions.txt')
    return dict(refusal_rows=len(refusals), inventory_rows=len(inventory), queued_files=len(queued), pending_gate466=pending)


POLICIES = {
    'closed_phases': ['no_refusal_names_a_closed_phase', 'no_owner_table_names_a_closed_phase', 'no_open_exclusions_row_names_a_closed_phase', 'the_literal_scan_finds_a_phase_inside_a_longer_text'],
    'owners': ['every_excluded_builtin_names_a_phase_that_owes_it'],
    'phase_ten_reasons': ['no_production_literal_names_phase_10', 'every_excluded_builtin_names_a_reason_the_enum_knows'],
    'refusal_dispositions': ['every_ownerless_refusal_has_a_disposition'],
    'refusal_sites': ['the_table_holds_every_constructor_the_source_defines', 'every_send_surface_row_is_walked'],
    'representation_refusals': ['every_representation_refusal_is_on_the_allow_list', 'every_row_names_an_owner'],
    'rexxc': ['rexxc_agrees_with_every_recorded_parse_error_row', 'registered_is_refused_with_the_rxapi_reason', 'the_command_line_answers_as_the_oracles_does', 'the_oracle_refuses_this_crates_images_as_another_versions'],
    'compiled': ['each_route_runs_a_restored_program_as_the_oracles_does', 'a_foreign_or_damaged_image_is_refused'],
    'errors': ['every_message_the_parser_names_is_raised_by_a_corpus_row_or_recorded_as_unreached', 'every_secondary_message_matches_the_oracle', 'every_unreached_reason_still_holds'],
}


def gate_logs(out, close):
    rows = []
    for gate in ['G4-test-release', 'G6-test-debug']:
        record = json.loads(consume(close / (gate + '.json')))
        check(record.get('validated_status', record['status']) == 0 and not record.get('orphaned_children'), gate + ' did not pass')
        stdout = consume(close / (gate + '.stdout'))
        stderr = consume(close / (gate + '.stderr'))
        for target, names in POLICIES.items():
            source = ROOT / ('rust/crates/rexx-parse/tests/errors.rs' if target == 'errors' else 'rust/crates/rexx-exec/tests/' + target + '.rs')
            source_text = consume(source)
            for name in names:
                check('fn ' + name + '(' in source_text, 'Named policy is not in current source: ' + target + '/' + name)
                passed = bool(re.search(r'^test ' + re.escape(name) + r' \.\.\. ok\s*$', stdout, re.M))
                target_seen = bool(re.search(r'Running tests/' + re.escape(target) + r'\.rs\b', stderr))
                check(passed and target_seen, gate + ': missing passed policy/target evidence ' + target + '/' + name)
                rows.append([gate, target, name, passed, target_seen])
    tsv(out, 'policy-and-binary-checks.tsv', ['gate', 'target', 'test', 'passed line', 'target launch'], rows)



def completed_no_run_build(close, freeze, commands):
    """Prove completion without replacing an original timed-out build result."""
    original_path = close / 'G3-build-release.json'
    original = json.loads(consume(original_path))
    canonical = next(a for a in commands if a['name'] == 'G3-build-release')
    assert original['action'] == canonical, 'Original build differs from frozen action manifest'
    assert canonical['argv'] == ['cargo', 'test', '-j', '4', '--workspace', '--release', '--no-run']
    assert canonical['cap'] == '16G' and canonical['seconds'] == 600
    assert canonical['env']['CARGO_INCREMENTAL'] == '0'
    assert canonical['env']['CARGO_BUILD_JOBS'] == '4'
    assert canonical['env']['REXX_CORPUS_GATE'] == '1'
    assert freeze['head'] in canonical['env']['CARGO_TARGET_DIR']
    continuation_paths = {}
    for path in close.glob('G3-build-release-continuation-*.json'):
        match = re.fullmatch(r'G3-build-release-continuation-([1-9][0-9]*)\.json', path.name)
        if match:
            continuation_paths[int(match[1])] = path
    if original['status'] == 0:
        assert not continuation_paths, 'A successful no-run build must not be retried'
        return dict(original_status=0, completed_compile_status=0, continuation_used=False)
    assert original['status'] == 124, 'Only an actual no-run compilation timeout can be continued'
    ruling = json.loads(consume(close / 'build-continuation-ruling.json'))
    assert ruling['head'] == freeze['head'] and ruling['rust_tree'] == freeze['source_tree'], 'Continuation ruling source differs from frozen source'
    assert ruling['original_status'] == 124 and ruling['original_record_sha256'] == digest(original_path), 'Original timeout record differs from the ruling'
    assert all(isinstance(ruling.get(k), str) and ruling[k].strip() for k in ['ruling', 'reason', 'cost_if_wrong']), 'Missing explicit build continuation ruling'
    runner = close / 'runner.py'
    consume(runner)
    assert digest(runner) == freeze['static_inputs'][str(runner)], 'Guarded action runner changed after freeze'
    assert continuation_paths, 'Timed-out no-run build has no completed continuation proof'
    assert sorted(continuation_paths) == list(range(1, len(continuation_paths) + 1)), 'Continuation attempt indices are not contiguous'
    attempts = [('G3-build-release', original)]
    for index, path in sorted(continuation_paths.items()):
        record = json.loads(consume(path))
        expected = dict(canonical, name=f'G3-build-release-continuation-{index}')
        assert record['action'] == expected, 'Continuation command, environment, cap or bound changed'
        attempts.append((expected['name'], record))
    assert attempts[-1][1]['status'] == 0, 'No-run compilation has not completed successfully'
    proof = []
    previous_finish = None
    for index, (name, record) in enumerate(attempts):
        expected_status = 0 if index == len(attempts) - 1 else 124
        assert record['status'] == expected_status, 'Only terminal compile timeouts may precede the successful continuation'
        assert record.get('validated_status', record['status']) == record['status'], 'Build attempt has a validation failure'
        assert not record.get('freeze_error') and record.get('interrupted') is False, 'Interrupted or drifted build is not a continuation candidate'
        assert record.get('surviving_group') is False and record.get('orphaned_children') is False, 'Build process group was not stopped cleanly'
        assert record['argv'] == original['argv'], 'Build launcher, timeout or cap changed'
        assert record['argv'] == ['timeout', '--kill-after=15', '600', 'memcap', '16G', *canonical['argv']]
        pid_record = json.loads(consume(close / (name + '.pid.json')))
        assert pid_record['pid'] == record['pid'] and pid_record['started'] == record['started'], 'Build PID provenance differs'
        try:
            os.killpg(record['pid'], 0)
        except ProcessLookupError:
            pass
        else:
            raise AssertionError('Build process group still exists: ' + name)
        started = datetime.datetime.fromisoformat(record['started'])
        finished = datetime.datetime.fromisoformat(record['finished'])
        assert finished >= started and (previous_finish is None or started >= previous_finish), 'Build attempts overlap or have invalid chronology'
        if expected_status == 124:
            assert 600 <= (finished - started).total_seconds() <= 660, 'Timeout was not the actual bounded compile terminal result'
        previous_finish = finished
        logs = {}
        for suffix in ['stdout', 'stderr']:
            path = close / (name + '.' + suffix)
            content = consume(path)
            assert digest(path) == record[suffix + '_sha256'], 'Build raw log hash differs: ' + name + '/' + suffix
            assert not re.search(r'^error(?:\[[^]]+\])?:|^error: could not compile', content, re.M), 'Compiler failure cannot be hidden by a continuation: ' + name
            assert not re.search(r'^running [0-9]+ tests?$|^test result:', content, re.M), 'No-run compile evidence contains a semantic test execution'
            logs[suffix + '_sha256'] = digest(path)
        proof.append(dict(name=name, status=record['status'], started=record['started'], finished=record['finished'], record_sha256=digest(close / (name + '.json')), **logs))
    g4 = json.loads(consume(close / 'G4-test-release.json'))
    assert previous_finish <= datetime.datetime.fromisoformat(g4['started']), 'G4 began before the successful no-run compilation completed'
    return dict(original_status=124, completed_compile_status=0, continuation_used=True, frozen_head=freeze['head'], frozen_rust_tree=freeze['source_tree'], ruling_sha256=digest(close / 'build-continuation-ruling.json'), attempts=proof, g4_started=g4['started'], limitation='Original G3 raw timeout remains a canonical gate failure. This proves only completed no-run compilation before G4.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--suite-root', type=Path, default=RECORDS / 'close/suite')
    parser.add_argument('--output', type=Path, default=RECORDS / 'close/derived')
    parser.add_argument('--pending-gate466', choices=['forbid', 'allow'], default='forbid')
    parser.add_argument('--allow-postfreeze-metadata', action='store_true', help='Allow descendant metadata commits only when the Rust tree still equals the frozen source')
    parser.add_argument('--fixture-only', action='store_true', help='Skip close completion/log validation and label all output as fixture evidence')
    args = parser.parse_args()
    if args.fixture_only:
        assert str(args.output.resolve()).startswith('/tmp/'), 'Fixture output must be under /tmp'
    else:
        finished = RECORDS / 'close/finished.json'
        assert finished.exists(), 'Close has not finished. Refuse derivation during freeze'
        assert 'finished' in (RECORDS / 'close/status.txt').read_text().split(), 'Freeze completion marker absent'
    args.output.mkdir(parents=True, exist_ok=False)
    summary = dict(mode='fixture-only, no close verdict' if args.fixture_only else 'close evidence derivation', suite_root=str(args.suite_root.resolve()), pending_gate466=args.pending_gate466, limitations=['No new licence, oracle cause or performance ruling is inferred', 'Named policy success must come from actual G4/G6 logs', 'A whole-phase gate verdict remains the controller responsibility'])
    try:
        consume(Path(__file__).resolve())
        consume(RECORDS / 't9/suite-delta.py')
        summary['git_head'] = subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', 'HEAD'], text=True).strip()
        summary['rust_tree'] = subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', 'HEAD:rust'], text=True).strip()
        summary['suite'] = suite_derivations(args.output, args.suite_root.resolve())
        summary['dispositions'] = dispositions(args.output, args.pending_gate466)
        if not args.fixture_only:
            close = RECORDS / 'close'
            consume(close / 'status.txt')
            finish = json.loads(consume(close / 'finished.json'))
            freeze = json.loads(consume(close / 'freeze.json'))
            check(finish['head'] == freeze['head'], 'Finish HEAD differs from frozen close')
            summary['frozen_head'] = freeze['head']
            if args.allow_postfreeze_metadata:
                ancestry = subprocess.run(['git', '-C', str(ROOT), 'merge-base', '--is-ancestor', freeze['head'], summary['git_head']], check=False)
                check(ancestry.returncode == 0, 'Metadata HEAD is not descended from frozen close')
            else:
                check(freeze['head'] == summary['git_head'], 'Derivation HEAD differs from frozen close, requires explicit --allow-postfreeze-metadata')
            check(freeze['source_tree'] == summary['rust_tree'], 'Derivation Rust tree differs from frozen close')
            commands = json.loads(consume(close / 'commands.json'))
            expected = {a['name'] for a in commands}
            check(len(commands) == len(expected) == 17, 'Close action manifest differs from reviewed 17 actions')
            check(set(finish.get('statuses', {})) == expected, 'Finish action coverage differs from commands')
            observed = {}
            for name in sorted(expected):
                action = json.loads(consume(close / (name + '.json')))
                status = action.get('validated_status', action['status']) or int(action.get('orphaned_children', False))
                observed[name] = status
                check(status == finish.get('statuses', {}).get(name), 'Action/finish status differs: ' + name)
            check(finish.get('failed', {}) == {n: s for n, s in observed.items() if s}, 'Finish failure summary differs from action records')
            failures = finish.get('failed', {})
            summary['retained_close_failures'] = failures
            summary['canonical_finished_statuses'] = finish.get('statuses', {})
            build_proof = completed_no_run_build(close, freeze, commands)
            summary['no_run_compile_completion'] = build_proof
            permitted = {'callgrind'} | ({'G3-build-release'} if build_proof['continuation_used'] else set())
            check(set(failures) <= permitted, 'Unresolved functional close actions failed: ' + str(failures))
            summary['retained_close_failures'] = failures
            summary['performance_adjudicated'] = False
            gate_logs(args.output, close)
        else:
            summary['omitted_checks'] = ['Close completion and freeze identity', 'Actual G4/G6 policy and binary/parser test log checks', 'All other functional gates and performance adjudication']
    except Exception as error:
        ERRORS.append(type(error).__name__ + ': ' + str(error))
    summary['errors'] = ERRORS
    summary['derivation_passed'] = not ERRORS
    (args.output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    (args.output / 'inputs-sha256.json').write_text(json.dumps(INPUTS, indent=2, sort_keys=True) + '\n')
    outputs = {p.name: digest(p) for p in sorted(args.output.iterdir()) if p.is_file()}
    (args.output / 'outputs-sha256.json').write_text(json.dumps(outputs, indent=2, sort_keys=True) + '\n')
    print(json.dumps(dict(output=str(args.output), errors=ERRORS, mode=summary['mode']), indent=2))
    return int(bool(ERRORS))


if __name__ == '__main__':
    sys.exit(main())
