#!/usr/bin/env python3
"""Check checkout-local inputs before running Rust workspace gates.

`--standalone` checks only what the `standalone` nextest profile needs: the ooTest and
documentation checkouts. The oracle's `build/lib` libraries belong to the upstream tier.
"""

import os
from pathlib import Path
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


def missing_inputs(root, standalone=False):
    """Return missing files and directories relative to this checkout."""
    native = () if standalone else NATIVE
    return [path for path in (*native, *SUITE_FILES, *DOCS)
            if not (root / path).is_file()] + [
                path for path in SUITE_DIRS if not (root / path).is_dir()]


CHECKOUTS = (
    "  svn checkout -r 13347 https://svn.code.sf.net/p/oorexx/code-0/test/trunk ootest\n"
    "  svn checkout -r 13347 https://svn.code.sf.net/p/oorexx/code-0/docs/trunk/rexxref oodocs/rexxref\n"
)
BUILD = (
    "  systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0 cmake -S . -B build -DCMAKE_POLICY_VERSION_MINIMUM=3.5 -DCMAKE_BUILD_TYPE=Release\n"
    "  systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0 cmake --build build --target rxregexp rxmath orxmethod orxfunction rxsock -j4\n"
)
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
    if missing:
        print(f"Rust test prerequisites missing in {root}:", file=sys.stderr)
        for path in missing:
            print(f"  {root / path}", file=sys.stderr)
        if standalone:
            print("Run from the checkout root:\n" + CHECKOUTS +
                  "Then rerun: python3 rust/scripts/test_preflight.py --standalone", file=sys.stderr)
        else:
            print("Run from the checkout root, under a memory cap:\n" + BUILD + CHECKOUTS +
                  "Keep these checkout-local libraries separate from the read-only oracle build.\n"
                  "Then rerun: python3 rust/scripts/test_preflight.py", file=sys.stderr)
        return 1
    if argv:
        os.execvp(argv[1], argv[1:])
    print(f"Rust test prerequisites present in {root}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
