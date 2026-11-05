# Developing interrexx

Design notes are in `docs/design/`; the language specification is `spec/`.

## Setup

`scripts/setup-third-party.sh` checks out the pinned `third_party/ooRexx` submodule, builds it as
`third_party/oracle` (CMake `Release`), and checks out ooTest (`ootest/`) and the ooRexx
documentation (`oodocs/`) at a fixed revision. Then `python3 rust/scripts/test_preflight.py` lists
anything the tests still need; `test_preflight.py -- COMMAND` runs the command only if nothing is
missing.

These trees are read-only and under their own licences: never modify them. **Clean room:** the API
service, `rxqueue`, `rxsubcom`, `librexxapi` and the extension libraries are written from
behaviour descriptions, public headers (`api/`), `oodocs`, ooTest and running ooRexx, never from
reading ooRexx's `rexxapi/`, `utilities/` or `extensions/` sources, and ooRexx is never traced or
disassembled. Nothing in interrexx copies ooRexx source or documentation text.

## What correct means

The specification is normative; ooRexx is evidence. A program's stdout, stderr and exit status
match ooRexx's byte for byte unless a Deviation in `spec/src/deviations.md` says otherwise. See
`docs/design/testing.md`.

## Running ooRexx and interrexx

Run both interpreters only under a memory cap; a runaway program takes the machine with it.

```sh
systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0 COMMAND
```

`ulimit -v` also works, but it caps address space, not memory used: interrexx reserves a 512 MiB
interpreter stack before running anything, so under the same `ulimit -v` it has 512 MiB less to
allocate than ooRexx.

* Probe from a fresh empty directory, with absolute paths: the current directory is on the external
  routine search path, and a stray `.rex` there runs instead of a typo'd name.
* Give each ooRexx run its own `XDG_RUNTIME_DIR`, so its `rxapi` daemon is private, and each
  interrexx run its own `REXX_API_DIR`. ooRexx is
  `LD_LIBRARY_PATH=third_party/oracle/lib third_party/oracle/bin/rexx`; interrexx is
  `rust/target/release/rexx`.
* Capture stdout, stderr and the exit status separately, never `2>&1`.
* Never `.Package~new` a file in the repository outside the source-line procedure below: it runs
  that file's prolog.
* Do not run a program in `rust/corpus/oracle-crashes.txt` on ooRexx.
* Do not set `NUMERIC DIGITS` above 1000, and check that a controlled `DO` ends at the precision it
  sets: a step that rounds away at that `DIGITS` loops forever.
* A symbol `x` or `b` right before a quoted string makes a hex or binary literal: `say '['x']'` is
  error 15.3. Use other names.

## Checks

From `rust/`, under the memory cap:

* `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
* `cargo test --workspace --no-fail-fast`, and the same with `REXX_CORPUS_GATE=1` in a debug build:
  release compiles out `debug_assert`, which carries some of the interpreter's invariants.
* `REXX_SPEC_ORACLE=1 cargo test -p rexx-exec --test spec` after changing `spec/`.
* ThreadSanitizer runs take `TSAN_OPTIONS=suppressions=rust/tsan.supp`, which hides the dynamic
  linker's `dl_close` race that two interpreters in one process show.
* Mutation runs build with `--profile mutation`, so a mutant never overwrites the release binary.
* `cargo test NAME` exits 0 when nothing matches; check the count of tests that ran.

Editing a `rust/corpus/lang/*.rex` program requires regenerating its recorded source lines; the
module comment of `rust/crates/rexx-parse/tests/sourceline_oracle.rs` has the procedure.

## Code conventions

* `unsafe_code = "deny"`. The granted modules are pinned by
  `rust/crates/rexx-core/tests/unsafe_sites.rs`. A new site needs the owner's approval and must
  clear the bar: the unsafety sits in one module behind a safe interface, its invariant is stated
  there and checkable from that module alone, a safe alternative was measured, and a test fails if
  the invariant breaks.
* Macros only for small mechanical code; prefer an `#[inline(always)]` function.
* Comments state the contract and properties the code does not show: no history, no task or phase
  references, no counts of a set in prose. A property that matters is asserted in a test.
* Representation is never observable: a fast path or side table that a program can tell apart
  from a full object is a defect.
