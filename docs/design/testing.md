# Testing

## What correct means

The language specification under `spec/` is normative. ooRexx is the differential instrument: a
program's stdout, stderr and exit status on interrexx must match ooRexx's byte for byte, compared
as separate streams. Every intended difference is a numbered Deviation in
`spec/src/deviations.md`; a difference with no Deviation is a defect on one side, settled in the
specification before it is settled in code. A new Deviation needs the project owner's ruling.

Where ooRexx is defective (a crash, an unbounded loop, an answer that depends on memory layout),
interrexx does not reproduce the defect. `rust/corpus/oracle-crashes.txt` lists the programs that
crash ooRexx; no differential case may run one.

Checks against the ANSI X3.274 standard record three values per finding: what ANSI says, what
ooRexx does, what interrexx does. Where ooRexx departs from ANSI, interrexx follows ooRexx, so a
two-way comparison against ANSI would report deliberate choices as defects.

## The differential corpus

`rust/corpus/` holds deterministic programs: `lang/` (language features), `errors/` (parse errors
through `rexxc`), `expr/`, `num/`, `model/` and `gate-tables/`. `differential.txt` names
the programs `tests/corpus.rs` runs under both interpreters. stdout and exit status compare byte for
byte; stderr does too, apart from a narrow trace-indent normalisation that a program can opt out of.
A program here prints nothing that varies between runs of one interpreter.

A program that needs files runs in a directory of its own at the same absolute path on both sides,
emptied between runs. A `<name>.env` sidecar sets `PATH`, `REXX_PATH`, `CWD` or `LD_LIBRARY_PATH`
for both sides, and a `<name>.d/` directory holds fixtures.

Each `rust/corpus/lang/*.rex` has a recorded copy of its source lines under
`rust/crates/interrexx-parse/tests/sourceline_oracle/`, and `SOURCELINE` is checked against it; editing
a corpus program means regenerating that file as the test's module comment describes.

Other tables in `rust/corpus/` are derived from ooRexx, ooTest or `oodocs` and committed with the
revision they were derived at: builtin and keyword exemptions, method bodies, collection and
introspection argument tables, refusal sites and their dispositions.

## The ooTest suite gate

`tests/suite.rs`, under `REXX_SUITE_GATE=1`, runs every group `testOORexx.rex` finds under
`ootest/ooRexx`, each through its single-group form, on interrexx and on ooRexx, each side from a
fresh copy with its own API service.

* A group passes when every test in its expected set started and passed, every test that ran is
  expected or listed, and the run ended normally. The expected set is the group's test methods,
  enumerated from source, less its rows in `rust/corpus/suite-licences.txt` (Deviation,
  environment and known-flaky rows, each with a checked reason) and
  `rust/corpus/suite-not-run.txt` (tests the framework does not run on this host).
* A crash, refusal, timeout or memory kill on interrexx renames the last started test out and runs
  the group again, so one failure does not hide the rest.
* Runs are bounded: ooRexx 300 s, interrexx four times ooRexx's time and at least 60 s, under a
  memory cap. The suite revision is pinned in `rust/corpus/suite-revision.txt` and asserted.
* ooRexx's results are reported beside interrexx's and never gate.

Older, narrower instruments still run: extracted ooTest assertions run standalone
(`assertions.rs`, `bif_assertions.rs`, `keyword_assertions.rs`), and gate tables of directive,
concept and class rows (`gate_tables/`).

## Simulation

The simulation mode (see `architecture.md`) has its own gate in `tests/group_runs/whole_groups.rs`:
the concurrency test groups and corpus programs, run under seeds derived by rule and a fixed mix of
policies (`rust/corpus/sim-gate.tsv`). It fails on a panic, a scheduler invariant breach, a
determinism breach, a hang, a design-limit refusal where ooRexx does not deadlock, or a failed
check of the program's own that no recorded ooRexx outcome shares. Known exceptions are rows in
`rust/corpus/sim-exempt.tsv`. ooRexx outcomes per group part are committed under
`rust/corpus/sim-oracle/`, and differences from them are reported, never gating. A failing run
prints a replay line naming the group, part, seed and policy.

## The specification checker

`tests/spec.rs` loads every page under `spec/src` through `interrexx-spec` and checks that each rule
(`r[id]`) has an example under it or a test citing it, that every citation names a rule, that
every example's `rexx` block prints exactly its `output` block with the stated exit code on
interrexx, and that the Deviations a page lists match its rules and the Deviation register. A test
cites a rule with `spec:` and the rule ID inside a comment in a tracked `.rs` or `.rex` file. With
`REXX_SPEC_ORACLE=1` the examples also run on ooRexx, except those under a Deviation rule.

## Environment switches

Unset, the gate switches below report and exit 0; set to `1`, a mismatch fails the run.

| variable | effect |
|---|---|
| `REXX_CORPUS_GATE` | the corpus differential and other oracle comparisons fail on a mismatch |
| `REXX_SUITE_GATE` | runs the ooTest suite gate |
| `REXX_SUITE_GROUPS` | restricts the suite to these comma-separated `dir/group` names |
| `REXX_SUITE_CRATE_FLOOR` | the least time bound, in seconds, for an interrexx group run |
| `REXX_ASSERTIONS_GATE`, `REXX_BIF_GATE`, `REXX_KEYWORD_GATE` | the extracted-assertion tables fail on a mismatch |
| `REXX_SPEC_ORACLE` | runs the specification's examples on ooRexx too |
| `REXX_SIM_SEEDS`, `REXX_SIM_POLICY`, `REXX_SIM_ONLY`, `REXX_SIM_JOBS`, `REXX_SIM_REPORT` | the simulation gate's seed count, policy, single replayed run, parallelism and report path |
| `REXX_SIM_ORACLE_REFRESH`, `REXX_SIM_CALIBRATION` | grow the committed ooRexx outcome sets; write the calibration rows |
| `REXX_WHOLE_GROUPS_ONLY` | runs only the named group files |
| `REXX_METHOD_BODIES_REFRESH`, `REXX_REFUSAL_SITES_REFRESH` | regenerate those committed tables |
| `REXX_TEST_THREAD_SANITIZER` | adapts tests to a ThreadSanitizer build |
| `REXX_SWITCH_MODE` | read by `rexx`: the switch mode, including `sim:...` |
| `REXX_API_DIR`, `XDG_RUNTIME_DIR` | set by `rust/.cargo/config.toml` so tests reach private API services, interrexx's and ooRexx's, never the user's |
| `REXX_UPSTREAM`, `REXX_ORACLE_ROOT` | set by `rust/.cargo/config.toml`: the ooRexx source tree and its build |
