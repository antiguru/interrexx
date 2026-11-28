#!/usr/bin/env python3
"""Check inputs and external tools before running Rust workspace gates.

`--standalone` omits the oracle's `build/lib` libraries from the input check.
"""

import os
from pathlib import Path
import shutil
import sys


NATIVE = tuple(
    f"build/lib/lib{name}.so"
    for name in ("rexx", "rexxapi", "rxregexp", "rxmath", "orxmethod", "orxfunction", "rxsock")
)
SUITE_FILES = (
    "ootest/ooTest.frm", "ootest/framework/OOREXXUNIT.CLS",
    "ootest/testOORexx.rex", "ootest/worker.rex",
)
SUITE_DIRS = tuple(f"ootest/ooRexx/{name}" for name in (
    "base/expressions", "base/keyword", "base/bif", "API",
))
DOCS = tuple(f"oodocs/rexxref/en-US/{name}.xml" for name in (
    "provide", "fundclasses", "collclasses", "utilityclasses", "streamclasses",
))
TOOLS = ("svn", "cc", "c++", "g++")


def missing_inputs(root, standalone=False):
    """Return missing files and directories relative to this checkout."""
    native = () if standalone else NATIVE
    return [path for path in (*native, *SUITE_FILES, *DOCS)
            if not (root / path).is_file()] + [
                path for path in SUITE_DIRS if not (root / path).is_dir()]


USAGE = "usage: test_preflight.py [--standalone] [-- COMMAND ARG ...]"


def main(argv):
    root = Path(__file__).resolve().parents[2]
    standalone = bool(argv) and argv[0] == "--standalone"
    if standalone:
        argv = argv[1:]
    if argv and (argv[0] != "--" or len(argv) == 1):
        print(USAGE, file=sys.stderr)
        return 2
    missing = missing_inputs(root, standalone)
    missing_tools = [name for name in TOOLS if shutil.which(name) is None]
    if missing_tools:
        print(f"Rust test tools missing from PATH: {', '.join(missing_tools)}",
              file=sys.stderr)
    if missing:
        print(f"Rust test prerequisites missing in {root}:", file=sys.stderr)
        for path in missing:
            print(f"  {root / path}", file=sys.stderr)
        option = " --standalone" if standalone else ""
        print(f"Run from the checkout root:\n  scripts/setup-third-party.sh{option}\n"
              f"Then rerun: python3 rust/scripts/test_preflight.py{option}", file=sys.stderr)
    if missing or missing_tools:
        return 1
    if argv:
        os.execvp(argv[1], argv[1:])
    print(f"Rust test prerequisites present in {root}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
