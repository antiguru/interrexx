# Phase 10 gate record

Spec: `docs/superpowers/specs/2026-11-02-phase-10-design.md`. Master plan:
`docs/superpowers/plans/2026-11-02-phase-10.md`. Phase 10 was delivered as a stack of PRs, each
squash merged in order. It had no performance gate: Moritz ruled on 2026-11-01 that Phases 10 and
10.1 skip it, with one performance round at the end.

| PR | Merge | Content |
| --- | --- | --- |
| 0 | `82c1ca9cc` (#9) | Design, plan, and the oracle and ooTest at r13346 |
| 1 | `531334817` (#10) | rxapi behaviour description, core and daemon |
| 2 | `f093eae5a` (#11) | Queues, registrations, macrospace, `rxqueue`, `rxsubcom` |
| 3 | `08695d1e2` (#12) | C API, `librexx` and `librexxapi` |
| 4 | `005055985` (#13) | `rxregexp`, `rxmath`, `rxunixsys`, `rxsock` in Rust; `hostemu` and `orxncurses` as C add-ons |
| 5 | `a42a017e3` (#14) | Corpus: PPWizard and the Rexx Parser |
| 6 | this PR | The rxapi groups in the suite harness, the r13347 tests, the last rows, the close |

## Goals (spec "Goals")

| Goal | Evidence |
| --- | --- |
| 1. Rust daemon | PR 1. The daemon has its own protocol and is not wire compatible (D7). Deviation 86 covers its start on demand |
| 2. `RxFuncAdd` family, `.RexxQueue`, macrospace, `rxqueue`, `rxsubcom` | PR 2. The `utilities/rxapi`, `rxQueue` and `rxsubcom` groups are green in the suite gate below |
| 3. Embedding API | PR 3. `layout::REFUSING_MEMBERS` is empty (`rexx-api/src/layout.rs`). `AttachThread` and `AddCommandEnvironment` are served, not refused. No production code constructs an `Owner::Phase10` refusal |
| 4. Extensions | PR 4. Deviations 87 to 92 |
| 5. Corpus | PR 5, record `docs/superpowers/records/2026-11-03-phase-10/pr5/` |
| 6. L3-full and the `phase:10` queue | Below |

## L3-full

Run at `5b66173c4` on 2026-11-04 in the main checkout, ootest r13347, oracle r13346:

* **Suite gate** (`REXX_SUITE_GATE=1 ... --test suite l3_core_suite`): `suite: 405 groups, 0 red`.
  Every group's verdict in `verdicts.tsv` is `equal`.
* **Exclusions:** `phase-9-exclusions.txt` is deleted, along with the held row form and the Phase 10
  row owner. The rows left are in `rust/corpus/suite-licences.txt` (Deviation, Environment and Flaky
  rows) and `rust/corpus/suite-not-run.txt` (NotRun rows, all derived: `restrictOS line N`,
  `test type NAME`, `platform windows`). Each Environment row cites an oracle-run record in
  `rust/corpus/suite-oracle-runs/` and Moritz's ruling.
* **Corpus gate** (`REXX_CORPUS_GATE=1 ... --test corpus`): `1329 of 1329 matching`.
* **Workspace:** `cargo test --workspace --no-fail-fast` passes. Its one failure, a sidecar the
  control proved inert, was fixed in `5b66173c4`; the corpus tests were rerun green after it.
  `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` are clean.

Moritz's PR 6 rulings are in `docs/superpowers/plans/2026-11-03-phase-10-pr6.md`, "Rulings".

## Queued items ruled `phase:10`

The inventory is `docs/superpowers/records/2026-10-18-phase-9d/queued-inventory.tsv`.

| Item | Disposition |
| --- | --- |
| `2026-09-28-rexxqueue-in-function-group.md` | Closed by PR 6 Task 1 (`d04cbdaac`) |
| `2026-09-29-release-local-reference-memory.md` | Closed by PR 3 (`08695d1e2`) |
| `2026-10-02-kept-context-held-98983-overwritable.md` | Closed by PR 3 (`08695d1e2`) |
| `2026-10-04-aborted-run-leaks-interp.md` | Closed by PR 3 (`08695d1e2`) |
| `2026-10-05-two-interpreters-signal-race.md` | Closed by PR 3 (`08695d1e2`) |
| `2026-10-17-phase-10-corpus-and-rxfunc.md` | Closed by PR 2 (`f093eae5a`) and PR 5 (`a42a017e3`) |
| `gate:phase-9a-gate.md:476` | Probed with `rxmath`, and the oracle and the crate agree. A library routine's frame is a `ROUTINE` named for the routine. Its executable is a `Routine`, the same one on every call and a different one per routine, with no source and no package. Kept as `rust/corpus/lang/library_routine_frame_executable.rex` |
| `2026-10-19-guarded-trace-destination-spin.md` | Re-ruled out of Phase 10 by Moritz on 2026-11-02 |

## Left open, queued

* `2026-11-03-constant-no-class-additional.md`: 99.906 has no ADDITIONAL on the crate.
* `2026-11-03-syswordexp-without-commands.md`: SysWordexp's native expansion. Deviation 90 refuses
  the function.
