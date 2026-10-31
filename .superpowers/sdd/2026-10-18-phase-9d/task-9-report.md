# Task 9 report: Phase 9 leaves the labels

Status: DONE. Source is stable at `561bcc3ffe9614da8108aa58ba44bcef8d885b3f`, based on `f2dcbd70f` in the existing linked worktree on `plan/rust-rewrite`. Task 9 removes closed refusal ownership and updates the scanners and fixtures that police it. Task 10 has not run, and the instruction inventory ZIP remains untouched.

## Implementation and red/green proof

* `bcac34dd4`: commits the `Phase 9.1` literal regression before execution. `red-retry.status` is 101, with the expected failing assertion in `red-retry.stdout`. The earlier `red.status` is 1 because sandboxed memcap could not access its cgroup. Escalated memcap resolved launcher access without changing host setup.
* `f5f2c95b4`: removes `Owner::Phase9`, closes the Phase 9 vocabularies, updates refusal fixtures, and exempts the pure-Rexx extension directories from the concurrency harness's blanket extension exclusion using `suite_rows::PHASE_9_EXTENSION_DIRS`.
* `3f423c0ee`: commits the adjacent sentence-punctuation regression before execution. `red-punctuation.status` is 101, with the expected failure for `owned by Phase 9.`.
* `0076c084f`: restricts the dotted-phase exclusion to a dot followed by a digit. This preserves sentence punctuation while distinguishing `Phase 9.1`.
* `561bcc3ff`: clarifies an open embedding attribution whose historical spec citation named Phase 9 within the OWNER sentence. The Phase 10 Embedding owner remains the same.

The final policy run passes both scanner cases and the closed-phase source and exclusion scans. The initial policy run found the embedding citation described above and exited 101 at `f5f2c95b4`. Its original output files were accidentally overwritten by the successful retry, so `policy-attempt-1-reconstructed.txt` explicitly retains only the command, revision, status and verbatim failure excerpt recovered from the tool transcript. The driver now refuses existing gate names instead of overwriting output.

## Traceback disposition and roadmap

The composite INTERPRET exclusion closes with the dispositions already recorded by the responsible instances. The propagated report is delivered by Task 8.2 with `raise_propagate_traceback.rex`, fragment indentation by Task 8.7b with `interpret_frame_invocation_indent.rex`, and the RAISE RETURN sender by Task 6.9 with `raise_result_and_sender.rex`. Task 8.3's `routine_call_frame_names.rex` witnesses Routine~call's METHOD CALL frame and name, while the PROGRAM-frame name is Deviation 66, ruled by Moritz on 2026-10-21 and pinned by `a_missing_signal_label_inside_interpret_keeps_the_programs_name`. The roadmap's row 9 receives `CLOSED 2026-10-31`, and historical OWNER rows already marked delivered remain historical records.

## Changed files

* `rust/crates/rexx-core/src/owner.rs`
* `rust/crates/rexx-exec/src/dispatch/native/tests.rs`
* `rust/crates/rexx-exec/tests/support/literals.rs`
* `rust/crates/rexx-exec/tests/closed_phases.rs`
* `rust/crates/rexx-exec/tests/refusal_dispositions.rs`
* `rust/crates/rexx-exec/tests/internal_routines.rs`
* `rust/crates/rexx-exec/tests/concurrency_tests.rs`
* `rust/crates/rexx-exec/tests/group_runs/whole_groups.rs`
* `docs/superpowers/plans/phase-4-exclusions.txt`
* `docs/superpowers/plans/2026-07-27-rust-rewrite.md`

## Gates

Evidence is under `docs/superpowers/records/2026-10-18-phase-9d/t9/`. `gate.py` records each Cargo command, source revision, separate stdout/stderr, status and elapsed time, using `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target` and four build jobs. Its foreground launcher is `memcap 8G timeout -k 5 540 python3 /tmp/codex-p9d-t9-gate.py NAME`, with a 525-second inner deadline and process-group cleanup. The controller ruled that the historical aggregate ten-minute estimate must not omit required gates, while each command retains its individual bound.

| Gate | Command from `rust/` | Status |
|---|---|---|
| preflight | `python3 scripts/test_preflight.py` | 0 |
| fmt | `cargo fmt --all --check` | 0 |
| clippy | `cargo clippy -j4 -p rexx-core -p rexx-exec --all-targets -- -D warnings` | 0 |
| core | `cargo test -j4 -p rexx-core --test unsafe_sites --lib` | 0, 24 passed |
| policy | `cargo test -j4 -p rexx-exec --test closed_phases --test phase_ten_reasons --test refusal_dispositions --test refusal_sites --test representation_refusals --test owners --test internal_routines --test concurrency_tests` | 0, 98 passed |
| native | `cargo test -j4 -p rexx-exec --lib dispatch::native::tests` | 0, 6 passed |
| release build | `cargo build --release -j4 -p rexx-exec --bins` | 0 |
| corpus | `REXX_CORPUS_GATE=1 cargo test --release -j4 -p rexx-exec --test corpus` | 0, 34 passed, 1 ignored, 1242/1242 corpus programs match |
| suite | `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=<t9>/suite/table.tsv REXX_SUITE_DIAGNOSTICS=<t9>/suite-diagnostics cargo test --release -j4 -p rexx-exec --test suite l3_core_suite -- --exact --nocapture` | 0, 402 groups, 0 red |

The two red scanner executions used `cargo test -j4 -p rexx-exec --test closed_phases the_literal_scan_finds_a_phase_inside_a_longer_text` under `memcap 8G` and a 540-second outer timeout. The first also ran through `scripts/test_preflight.py`, with the same Cargo environment as the gates. Both reached their intended assertions and exited 101 before the corresponding fixes were committed. The successful policy gate reran the scanner cases at the final source revision.

## Suite delta and licences

The whole suite command exited 0, with 402 groups and zero red verdicts. `python3 /tmp/codex-p9d-t9-delta.py` exited 0 and compared `suite/tests.tsv` against `t8.8/final-gates/final/suite/tests.tsv`, using the retained `suite-delta.py`. Both contain 24,523 test rows with identical keys and values: zero changed, added or removed rows, retained in `suite/delta-vs-t8.8.tsv` and `suite/delta-summary.json`. This comparison includes the raw oracle and crate verdicts and is independent of licence policy.

No licence rows, refusal-disposition rows, refusal-site rows or representation allowlist entries changed, as recorded by `source-identity.json`. The concurrency derivation test passes without changes to the committed pinning list. The historical OWNER rows at the rexxc and ReleaseLocalReference entries already have delivered resolutions and remain historical records. The newly closed composite INTERPRET row and the clarified live embedding attribution both pass the exclusion scan.


## Performance and remaining concerns

No performance run is required for these changes because they do not change an interpreter execution path. The production change removes the obsolete owner variant and its refusal-label lookup arms, and no production site still constructed that variant at the baseline. Other Rust changes are policy scanners, test fixtures and group selection, while delivered runtime semantics and the existing refusal messages stay unchanged. The retained named gates total 703.794 seconds (11.73 minutes), excluding red runs and the overwritten initial policy attempt, and every individual gate stayed below its bound. No command remains live and no implementation concern remains open for this task.
