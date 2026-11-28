#!/usr/bin/env python3
"""Render a Markdown conformance summary from a checkout's tracked inventories and gate results."""

import argparse
from collections import Counter, defaultdict
from html import escape
from pathlib import Path
import re
import subprocess
import sys


CORPUS = "rust/corpus"
SUITE_HEADER = "group\ttest\trow\toracle\tcrate"
CORPUS_REPORT = re.compile(
    r"^interrexx differential corpus report -- rust/corpus/differential\.txt -- "
    r"shard (\d+) of (\d+)\n"
    r"mode: STRICT \(the gate\) -- REXX_CORPUS_GATE is set\n"
    r"(\d+) of (\d+) matching$",
    re.MULTILINE,
)
STREAM_GROUPS = {
    "base/bif/charin", "base/bif/charout", "base/bif/chars",
    "base/bif/linein", "base/bif/lineout", "base/bif/lines",
    "base/bif/stream", "base/class/stream",
}
CONCURRENCY_GROUPS = {
    "base/class/alarm", "base/class/eventsemaphore", "base/class/message",
    "base/class/mutexsemaphore", "base/class/ticker", "base/keyword/guard",
    "base/keyword/reply",
}
SUITE_AREAS = (
    ("api/", "Native API"),
    ("base/bif/", "Built-in functions"),
    ("base/class/", "Classes"),
    ("base/directives/", "Directives"),
    ("base/keyword/", "Keywords"),
    ("base/rexxutil/", "RexxUtil"),
    ("extensions/platform/", "Platform extensions"),
    ("extensions/rxsock/", "RxSock"),
)
SUITE_STATUSES = (
    "passed", "failed", "blocked", "started", "blocked at load", "not started",
)


def git(root, *args):
    result = subprocess.run(
        ["git", "-C", str(root), *args], capture_output=True, encoding="utf-8", check=True,
    )
    return result.stdout


def data_lines(content):
    return (line for line in content.splitlines() if line.strip() and not line.startswith("#"))


def records(content, width, source):
    """Read the common tab-separated layout without counting comments or headings."""
    for line in data_lines(content):
        fields = line.split("\t")
        if len(fields) != width or any(not field.strip() for field in fields):
            raise ValueError(f"{source} has a malformed row: {line!r}")
        yield fields


def metadata(content, prefix, source):
    values = [line[len(prefix):] for line in content.splitlines() if line.startswith(prefix)]
    if len(values) != 1 or not values[0]:
        raise ValueError(f"{source} must contain one {prefix.strip()!r} line")
    return values[0]


def markdown_table(headers, rows):
    def row(cells):
        escaped = (
            escape(str(cell), quote=False).replace("|", "&#124;").replace("\n", " ")
            for cell in cells
        )
        return "| " + " | ".join(escaped) + " |"

    return [row(headers), row(["---"] * len(headers)), *(row(cells) for cells in rows)]


def suite_area(group):
    name = group.lower()
    if name in STREAM_GROUPS:
        return "Streams"
    if name in CONCURRENCY_GROUPS:
        return "Concurrency"
    return next((area for prefix, area in SUITE_AREAS if name.startswith(prefix)), "Other")


def suite_exclusions(read, revision):
    reasons = Counter()
    counts = defaultdict(Counter)
    for source in ("suite-licences.txt", "suite-not-run.txt"):
        content = read(source)
        if metadata(content, "# ootest revision ", source) != revision:
            raise ValueError(f"{source} does not match the ooTest pin r{revision}")
        for group, _test, owner, reason in records(content, 4, source):
            area = suite_area(group)
            kind = {"NotRun": "Not run", "Unspecified": "Assertion allowance"}.get(
                owner, "Test licence",
            )
            counts[area][kind] += 1
            reasons[area, owner, reason] += 1
    return counts, reasons


def suite_results(content, revision):
    source = "ooTest tests.tsv"
    if metadata(content, "# ootest revision ", source) != revision:
        raise ValueError(f"{source} does not match the ooTest pin r{revision}")
    selected = metadata(content, "# groups ", source)
    data = list(data_lines(content))
    if not data or data[0] != SUITE_HEADER:
        raise ValueError(f"{source} has no per-test header")
    seen = set()
    counts = defaultdict(Counter)
    for group, test, _row, oracle, crate in records("\n".join(data[1:]), 5, source):
        if (group, test) in seen:
            raise ValueError(f"{source} repeats {group} {test}")
        seen.add((group, test))
        for engine, result in (("interrexx", crate), ("ooRexx", oracle)):
            status = result.partition(": ")[0]
            if status not in SUITE_STATUSES:
                raise ValueError(f"{source} has an unknown verdict: {result!r}")
            counts[suite_area(group), engine][status] += 1
    if not seen:
        raise ValueError(f"{source} has no test results")
    scope = "all groups" if selected == "all" else f"selected groups ({escape(selected)})"
    return [
        f"ooTest r{revision}, {scope}. These are per-test outcomes as recorded by the suite gate.",
        "A test blocked at load never started. A started test can lack a final verdict.",
        "",
        *markdown_table(
            ("Area", "Engine", "Started", "Passed", "Failed", "Blocked in test",
             "Started without verdict", "Blocked at load", "Not started"),
            (
                (area, engine, sum(outcomes[s] for s in SUITE_STATUSES[:4]),
                 *(outcomes[s] for s in SUITE_STATUSES))
                for (area, engine), outcomes in sorted(counts.items())
            ),
        ),
        "",
    ]


def corpus_results(content, inventory_size):
    reports = [tuple(map(int, row)) for row in CORPUS_REPORT.findall(content)]
    if not reports:
        raise ValueError("corpus log has no strict differential report")
    width = reports[0][1]
    seen = set()
    for index, count, matched, total in reports:
        if width < 1 or count != width or index >= width or index in seen:
            raise ValueError("corpus log has repeated or inconsistent shard indices")
        expected = len(range(index, inventory_size, width))
        if total != expected or matched > total:
            raise ValueError(f"corpus shard {index} does not match the tracked inventory size")
        seen.add(index)
    matched = sum(row[2] for row in reports)
    total = sum(row[3] for row in reports)
    return [
        f"Strict differential reports cover {len(seen)} of {width} shards.",
        "Counts describe those shards only. Programs that did not finish are among the non-matches.",
        "",
        *markdown_table(
            ("Program comparisons reported", "Matching", "Did not match"),
            [(total, matched, total - matched)],
        ),
        "",
    ]


def render(read, commit, oracle, suite_report=None, corpus_log=None):
    revision = read("suite-revision.txt").strip()
    if not revision.isascii() or not revision.isdecimal():
        raise ValueError("suite-revision.txt does not contain an SVN revision")
    inventory_size = sum(1 for _ in data_lines(read("differential.txt")))
    unfiled = list(records(read("unfiled.txt"), 2, "unfiled.txt"))
    builtin = defaultdict(list)
    for name, status in records(read("builtin-status.txt"), 2, "builtin-status.txt"):
        builtin[status].append(name)
    deviations = Counter(
        status for _number, status, _tables, _witnesses
        in records(read("deviations.tsv"), 4, "deviations.tsv")
    )
    counts, reasons = suite_exclusions(read, revision)
    lines = [
        "# Interrexx conformance",
        "",
        f"- Interrexx inventory commit `{commit}`",
        f"- ooRexx pin `{oracle}` from `third_party/ooRexx`",
        f"- ooTest pin r{revision}",
        "",
        "## Tracked inventories",
        "",
        "These counts describe recorded coverage and exclusions.",
        "",
        f"- {inventory_size} programs are listed in `differential.txt`.",
        f"- {len(unfiled)} programs are outside the differential in `unfiled.txt`.",
        f"- `deviations.tsv` lists {deviations['active']} active and "
        f"{deviations['withdrawn']} withdrawn Deviations.",
        "",
        *markdown_table(
            ("Recorded built-in probe status", "Names", "Detail"),
            ((status, len(names), "" if status == "implemented" else ", ".join(names))
             for status, names in sorted(builtin.items())),
        ),
        "",
        "The suite inventory groups ooTest paths by area. Test licences describe why a test",
        "is outside the required pass set. Assertion allowances leave the test in that set.",
        "`suite-not-run.txt` records framework exclusions on the gate host.",
        "",
        *markdown_table(
            ("ooTest area", "Test licences", "Assertion allowances", "Framework exclusions"),
            ((area, row["Test licence"], row["Assertion allowance"], row["Not run"])
             for area, row in sorted(counts.items())),
        ),
        "",
        "## Observed results",
        "",
        "### Differential corpus",
        "",
    ]
    if corpus_log is None:
        lines.extend(["No strict corpus log supplied.", ""])
    else:
        lines.extend(corpus_results(corpus_log, inventory_size))
    lines.extend(["### ooTest suite", ""])
    if suite_report is None:
        lines.extend(["No ooTest per-test report supplied.", ""])
    else:
        lines.extend(suite_results(suite_report, revision))
    lines.extend([
        "## Recorded exclusions and allowances",
        "",
        "### ooTest",
        "",
        *markdown_table(
            ("Area", "Category", "Reason", "Tests"),
            ((area, kind, reason, count) for (area, kind, reason), count in sorted(reasons.items())),
        ),
        "",
        "### Corpus programs outside the differential",
        "",
        *markdown_table(("Program", "Reason"), unfiled),
        "",
    ])
    return "\n".join(lines)


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus-log", type=Path, help="combined output of a strict corpus run")
    parser.add_argument("--suite-report", type=Path, help="tests.tsv from the ooTest suite gate")
    parser.add_argument("--run-commit", help="full interrexx commit recorded for the supplied results")
    args = parser.parse_args(argv)
    root = Path(__file__).resolve().parents[2]
    try:
        commit = git(root, "rev-parse", "HEAD").strip()
        oracle = git(root, "rev-parse", f"{commit}:third_party/ooRexx").strip()
        if (args.corpus_log or args.suite_report) and args.run_commit != commit:
            raise ValueError(f"observations require --run-commit {commit}")
        summary = render(
            lambda name: git(root, "show", f"{commit}:{CORPUS}/{name}"),
            commit,
            oracle,
            args.suite_report.read_text(encoding="utf-8") if args.suite_report else None,
            args.corpus_log.read_text(encoding="utf-8") if args.corpus_log else None,
        )
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f"conformance summary: {error}", file=sys.stderr)
        return 1
    sys.stdout.write(summary)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
