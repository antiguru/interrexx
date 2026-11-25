# Developing interrexx

Design notes are in `docs/design/`; the language specification is `spec/`. The standing decisions
and the reasons for them are in `docs/design/decisions.md`; read it before proposing to change one.

## Setup

A regular build and test run need no submodule and no ooRexx build. `cargo build` and
`cargo nextest run --workspace --profile standalone` read only files of this repository, plus the
ooTest (`ootest/`) and ooRexx documentation (`oodocs/`) checkouts, and the tools some tests run:
`svn` and a C and a C++ compiler. `python3 rust/scripts/test_preflight.py --standalone`
lists whichever of the two checkouts is missing and prints the `svn checkout` commands that fetch
them; it checks the checkouts only, not the tools. `test_preflight.py --standalone -- COMMAND` runs
the command only if nothing is missing.

The submodule and the oracle are needed only for the upstream tier of the tests and for the oracle
comparisons (`docs/design/testing.md`). `scripts/setup-third-party.sh` checks out the pinned
`third_party/ooRexx` submodule, builds it as `third_party/oracle` (CMake `Release`), and checks out
ooTest and the documentation at a fixed revision. Then `python3 rust/scripts/test_preflight.py`
lists the checkouts and the oracle's `build/lib` libraries that are missing; it does not look at
`third_party/ooRexx` or `third_party/oracle`. `test_preflight.py -- COMMAND` runs the command only
if nothing is missing.

Three tables are generated from the pinned sources and checked in: `classes.tsv`, `errors.tsv`
and `builtins.txt`, in the `data/` directories of `interrexx-classes` and `interrexx-inventory`.
A regular build reads these files, never the pinned sources. After bumping the `third_party/ooRexx` submodule, regenerate them and commit the result:

```sh
REXX_UPSTREAM=$PWD/third_party/ooRexx cargo run -p interrexx-upstream -- write
```

`cargo run -p interrexx-upstream -- check` regenerates in memory and exits non-zero, printing the
differing rows, when a checked-in file differs. The `upstream interface` CI job runs it, so a
bump without regeneration and a hand-edited row both fail.

A fourth file, `rust/corpus/docs/directive-options.txt`, is derived from `oodocs` and from the
submodule's `parser/DirectiveParser.cpp`: `cargo run -p interrexx-extract --bin rexx-extract-docs`
reads that source, so regenerating it needs the submodule. A regular build and test run do not.

The same job compares `include/` with the pinned `api/` headers. The ABI dump of
`interrexx-api/tests/abi_dump.rs` is compiled against `api/` (`REXX_UPSTREAM` set, tests
`..._upstream_headers_...`), and `scripts/names-diff.py third_party/ooRexx/api include` diffs every
declared name both ways. Each header of both trees is read twice, as C++17 and as C11, so what sits
under `#ifndef __cplusplus` is compared too. Macros come from the preprocessor with their expansion
(a changed value or body differs, two spellings of one value do not); types, fields, functions and
C++ methods come from clang's syntax tree, a function with its full signature, storage class,
`inline` and default arguments, a member of an anonymous record with the position of that record.
The list of headers is diffed as well. A difference fails unless `EXCEPTIONS` in the script lists
it with its reason. Types are compared with typedef names resolved, so a typedef swapped at a use
site is caught by the ABI dump, not by the diff. The script needs `clang` and `clang++`, which the
CI image has.

The text of each error message is in `rust/crates/interrexx-inventory/messages/<major>.tsv`, written
to the style sheet `STYLE.md` beside them. The build requires a text for every row of `errors.tsv`
and only the inserts the row renders, so a regenerated table that gains a message fails to build
until the message has its text.

These trees are read-only and under their own licences: never modify them. **Clean room:** the API
service, `rxqueue`, `rxsubcom`, `librexxapi` and the extension libraries are written from
behaviour descriptions, public headers (`api/`), `oodocs`, ooTest and running ooRexx, never from
reading ooRexx's `rexxapi/`, `utilities/` or `extensions/` sources, and ooRexx is never traced or
disassembled. Nothing in interrexx copies ooRexx source or documentation text. Error message texts
are interrexx's own data, written to the rules in
`rust/crates/interrexx-inventory/messages/STYLE.md`. A text is written fresh unless it is ANSI's
text for the code, one of the canonical phrases with only names filled in, or the plain statement
anyone given only the meaning of the code would write nearly word for word: a subject, then `must`,
`must not`, `is` or `expected`, then an object, with no adverb, no parenthetical remark, no second
clause and no turn of phrase peculiar to ooRexx.

No text under ooRexx's licence enters the repository, recorded runs included: a recorded oracle
run keeps the code of an error line, not the text after it. No tool in the repository compares
texts with ooRexx's. The check was run once, by hand, with a throwaway script, and the method is
recorded here for whoever repeats it. The script read ooRexx's message catalogue (each text, explanation
and user action, with the inserts as `&k`) and the comments of the pinned `api/` headers, normalised
whitespace, case, quote glyphs and comment markers so that hard-wrapped prose rejoins, and searched
every tracked file outside `third_party/`, `ootest/` and `oodocs/`, fixtures and recorded runs
included. A catalogue text was found by each literal piece of 12 or more characters and by its whole
template with the inserts as wildcards; a header comment was found by sentence. Catalogue texts
identical to our own were skipped, and every hit outside the message files was read by hand,
because short pieces match generic phrases. The limit: a text with no literal piece of 12
characters cannot be found, and neither can a text that was reworded, or quoted in fragments
shorter than that. The scan did not read oodocs prose, so sentences quoted from the documentation
were found by reading. To repeat it, write the script again against the catalogue of the pinned
revision.

`include/` is not hand-maintained where it is generated. After a change to the Rust definitions
in `interrexx-api` (`layout.rs`, `constants.rs`, `exports.rs`), rewrite the generated sections of
the headers and read the diff:

```sh
REXX_HEADERS_REFRESH=1 cargo test -p interrexx-api --test headers
```

Without the switch the same test fails when a header differs from what the definitions render. The
`build` CI job also compiles `rexx.h` and `oorexxapi.h` with clang as C and as C++ under
`-Wall -Wextra -Werror -Wextern-c-compat`.

## What correct means

The specification is normative; ooRexx is evidence. A program's stdout, stderr and exit status
match ooRexx's byte for byte unless a Deviation in `spec/src/deviations.md` says otherwise, or the
specification leaves the difference unspecified: for an error, only the code and the subcode are
specified, not the message text or the items of `ADDITIONAL`. See `docs/design/testing.md`.

## Running ooRexx and interrexx

Run both interpreters only under a memory cap; a runaway program takes the machine with it.

```sh
systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0 COMMAND
```

`ulimit -v` also works, but it caps address space, not memory used: interrexx reserves a 512 MiB
interpreter stack before running anything, so under the same `ulimit -v` it has 512 MiB less to
allocate than ooRexx.

macOS has neither: there is no `systemd-run`, and it refuses a `ulimit -v` below what a process
has already reserved. There, `scripts/rss-cap.sh` stands in for the cap: it runs a command, sums
the resident sets of the command and every process below it twice a second, and kills them all
once the sum passes the bound, exiting 137.

```sh
scripts/rss-cap.sh 8388608 COMMAND
```

It is a watchdog, not a limit: a process that allocates faster than it polls can pass the bound
by up to half a second's allocation before it is killed. Running the `standalone` profile under it
with `--test-threads 4` stays within 8 GiB. interrexx's own bound on memory is #840. macOS also
limits a socket path to 104 bytes, and tests bind sockets under the target directory, so from a
deep worktree set `CARGO_TARGET_DIR` to a shorter path.

* Probe from a fresh empty directory, with absolute paths: the current directory is on the external
  routine search path, and a stray `.rex` there runs instead of a typo'd name.
* Give each ooRexx run its own `XDG_RUNTIME_DIR`, so its `rxapi` daemon is private, and each
  interrexx run its own `REXX_API_DIR`. ooRexx is
  `LD_LIBRARY_PATH=third_party/oracle/lib third_party/oracle/bin/rexx`; interrexx is
  `rust/target/release/rexx`.
* Capture stdout, stderr and the exit status separately, never `2>&1`.
* ooRexx can give a program with more than one activity different results from run to run, so
  one run that matches it does not show agreement. interrexx's simulation mode makes a schedule
  reproducible from its seed; its gate compares against the set of outcomes recorded from ooRexx
  (`docs/design/testing.md`).
* Never `.Package~new` a file in the repository outside the source-line procedure below: it runs
  that file's prolog.
* Do not run a program in `rust/corpus/oracle-crashes.txt` on ooRexx.
* Do not set `NUMERIC DIGITS` above 1000, and check that a controlled `DO` ends at the precision it
  sets: a step that rounds away at that `DIGITS` loops forever.
* A symbol `x` or `b` right before a quoted string makes a hex or binary literal: `say '['x']'` is
  error 15.3. Use other names.

## Checks

From `rust/`, under the memory cap:

* `cargo fmt --all -- --check --config error_on_line_overflow=true` and
  `cargo clippy --workspace --all-targets -- -D warnings`.
* `cargo nextest run --workspace --profile standalone` runs the regular tier, the tests that need
  nothing of `third_party/`. The profile's `default-filter` in `rust/.config/nextest.toml` holds
  the upstream tier out, and the comments above the profile give each entry's tier and the path it
  needs. A test that needs `third_party/` fails loudly when the tree is absent and does not skip
  itself; the filter keeps it out of this profile. Two tests that compare against the oracle only
  in gate mode, `an_unreadable_console_is_end_of_input` and
  `a_program_that_never_finishes_reddens_at_the_deadline_instead_of_hanging`, print `*** SKIPPED`
  and return when their gate variable is unset; `rust/.config/nextest.toml` records them as the
  exception. A new test that reads the submodule or runs the
  oracle is added to the filter with its tier and reason. The `build` CI job checks every name in
  the filter against the test archive (`rust/scripts/check_standalone_filter.py`), so a renamed
  test cannot leave the filter unnoticed. The `build`, `shard`, `doctest`, `clippy` and `jit` CI
  jobs run on a checkout with no submodule, no `third_party/` directory and no `build` link, each
  asserting that before it starts, and `build`, `doctest`, `clippy`, `jit` and `msrv-standalone` keep a
  cargo cache of their own that no job with the submodule saved, where `shard` keeps none. So a
  build script or test that reads the tree fails there. The six shards run this profile over the archive that `build` made without the submodule; the
  `upstream-tier` job runs the complement, `--ignore-default-filter -E 'not default()'`, on a
  checkout with the submodule and the oracle build. The `msrv-standalone` job builds every target,
  runs this profile's unit tests (`--lib`) and the doc tests on the toolchain `rust-version`
  declares, on a checkout with no submodule.
* The `i686` CI job builds every target and runs the `standalone` profile in an i386 Debian
  container (`ci/i686-standalone.sh`), so 32-bit x86 is built and tested on every pull request.
  `cargo clippy --workspace --all-targets --target i686-unknown-linux-gnu -- -D warnings` checks
  the 32-bit `cfg` arms from a 64-bit host. A size pin (`assert!(size_of::<T>() == N)`) holds
  for 64-bit targets only, under `#[cfg(target_pointer_width = "64")]`.
* `cargo nextest run --workspace --profile ci`, then `cargo test --workspace --doc` (nextest does
  not run doc tests). Install nextest with `cargo install cargo-nextest --locked`. Run the gate
  with `REXX_CORPUS_GATE=1 cargo test --workspace --no-fail-fast` in a debug build, not under
  nextest: the group runner's quiet-machine and row bounds hold within one process. Release
  compiles out `debug_assert`, which carries some of the interpreter's invariants. CI runs the
  gate this way in `.github/workflows/gate.yml`: nightly on main, on a change to the pinned
  ooRexx or `scripts/setup-third-party.sh`, on a pull request labelled `gate`, and by hand.
* `cargo test --workspace --no-fail-fast` runs the same tests, one binary after another, and
  stays supported. `cargo nextest run -p interrexx --test NAME` or `-E 'test(NAME)'` runs a
  subset; `cargo nextest list` shows what a filter matches.
* `REXX_SPEC_ORACLE=1 cargo test -p interrexx --test spec` after changing `spec/`.
  `REXX_SPEC_ONLY=native/` runs only the examples of the pages under that path of `spec/src`;
  the other checks still cover every page. The native chapter's examples load fixture libraries
  that `scripts/build-spec-fixture.sh` compiles from `spec/fixture/` against `include/`, so no
  example is skipped.
* ThreadSanitizer runs take `TSAN_OPTIONS=suppressions=rust/tsan.supp`, which hides the dynamic
  linker's `dl_close` race that two interpreters in one process show.
* Mutation runs build with `--profile mutation`, so a mutant never overwrites the release binary.
* `cargo test NAME` exits 0 when nothing matches; check the count of tests that ran.

Editing a `rust/corpus/lang/*.rex` program requires regenerating its recorded source lines; the
module comment of `rust/crates/interrexx-parse/tests/sourceline_oracle.rs` has the procedure.

## Code conventions

* `unsafe_code = "deny"`. The granted modules are pinned by
  `rust/crates/interrexx-core/tests/unsafe_sites.rs`. A new site needs the owner's approval and must
  clear the bar: the unsafety sits in one module behind a safe interface, its invariant is stated
  there and checkable from that module alone, a safe alternative was measured, and a test fails if
  the invariant breaks.
* Macros only for small mechanical code; prefer an `#[inline(always)]` function.
* Tests that are a table of inputs and expected outputs are datadriven files, not cases written
  out in Rust: a new case is a few lines of data, and a rewrite switch re-records the expectations.
* Comments state the contract and properties the code does not show: no history, no task or phase
  references, no counts of a set in prose. A property that matters is asserted in a test.
* Representation is never observable: a fast path or side table that a program can tell apart
  from a full object is a defect.
