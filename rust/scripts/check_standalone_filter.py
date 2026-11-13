#!/usr/bin/env python3
"""Check that every name in the `standalone` nextest filter exists in a test list.

The `standalone` profile keeps the tests that need `third_party/` out of its run with a
`default-filter` (`.config/nextest.toml`). A test that is renamed or removed would silently
leave that filter and, with it, the upstream set, so this check resolves each entry against the
list of the built test archive:

    cargo nextest list --archive-file ARCHIVE --message-format json | check_standalone_filter.py CONFIG

A filter entry is `binary_id(=B)` (a whole binary), `(binary_id(=B) & test(=T))` (one test of a
binary) or `test(=T)` (one test compiled into several binaries). The list must come from a
profile without a default filter, as `default` and `ci` are.
"""

import json
import re
import sys
import tomllib

PROFILE = "standalone"

BINARY = re.compile(r"binary_id\(=([^()&|]+)\)")
TEST = re.compile(r"test\(=([^()&|]+)\)")
# One entry per line, after the `not (` line and before the closing `)`.
ENTRY = re.compile(
    r"\s*\|?\s*\(?\s*(?:binary_id\(=[^()&|]+\))?\s*&?\s*(?:test\(=[^()&|]+\))?\s*\)?\s*"
)


def entries(expression):
    """Return the filter's entries as (binary id or None, test name or None) pairs.

    Raises ValueError for a line that is not one of the three entry shapes.
    """
    lines = [line for line in expression.splitlines() if line.strip()]
    if len(lines) < 3 or lines[0].strip() != "not (" or lines[-1].strip() != ")":
        raise ValueError("the filter must be `not (` entries `)`, one entry per line")
    found = []
    for line in lines[1:-1]:
        binary, test = BINARY.search(line), TEST.search(line)
        if (binary is None and test is None) or not ENTRY.fullmatch(line):
            raise ValueError(f"not an entry of the filter: {line.strip()!r}")
        found.append((binary and binary.group(1), test and test.group(1)))
    return found


def unresolved(found, suites):
    """Return the entries that name no binary or test of `suites` (binary id -> test names)."""
    missing = []
    for binary, test in found:
        if binary is None:
            ok = any(test in tests for tests in suites.values())
        elif binary not in suites:
            ok = False
        else:
            ok = test is None or test in suites[binary]
        if not ok:
            missing.append((binary, test))
    return missing


def suites_of(listing):
    """Return `{binary id: set of test names}` from `cargo nextest list --message-format json`."""
    return {
        binary: set(suite["testcases"])
        for binary, suite in listing["rust-suites"].items()
    }


def main(argv):
    if len(argv) != 1:
        print("usage: check_standalone_filter.py NEXTEST_TOML < list.json", file=sys.stderr)
        return 2
    with open(argv[0], "rb") as config:
        expression = tomllib.load(config)["profile"][PROFILE]["default-filter"]
    listing = json.load(sys.stdin)
    found = entries(expression)
    missing = unresolved(found, suites_of(listing))
    if missing:
        print(f"The {PROFILE} filter names tests that are not in the list:", file=sys.stderr)
        for binary, test in missing:
            print(f"  binary {binary or '(any)'}, test {test or '(whole binary)'}", file=sys.stderr)
        return 1
    print(f"All {len(found)} entries of the {PROFILE} filter name a test of the list")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
