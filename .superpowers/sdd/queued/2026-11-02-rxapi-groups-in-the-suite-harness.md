**CLOSED 2026-11-03 by Phase 10 PR 6 Task 1 (d04cbdaac).** Each suite run has a private service per side; the rows, `R1_GROUPS` and `Deferral::Rxapi` are gone and the two licence rows are in `rust/corpus/suite-licences.txt`. `rexx-bench`'s shared oracle runtime is not changed.

**OPEN, queued 2026-11-02 by Phase 10 PR 2. Owner: Phase 10 PR 6 (suite gate).**

# The rxapi groups do not yet run inside the suite harness

PR 2 delivered the service, `.RexxQueue`, `RXQUEUE`, registrations, macrospace, `rxqueue` and
`rxsubcom`, and the groups were run whole by hand against the oracle (`RXQUEUE`, `RexxQueue` identical;
`rxsubcom` 13/13; `rxQueue` 8/9; `rxapi` 0/1). The suite harness still excludes them:
`reaching_rxapi` and `R1_GROUPS` in `rust/crates/rexx-exec/tests/support/suite_rows/mod.rs`, the
Phase 10 `Rxapi` rows in `docs/superpowers/plans/phase-9-exclusions.txt`, `FUNCTION.TEST_REXXQUEUE` in
`RECORDED` (`api_group_tests.rs`), and so `Deferral::Rxapi` stays.

To do: run the crate side with `rexx_exec::use_api_service` on a private directory, and the oracle side
with a private `XDG_RUNTIME_DIR` (the oracle's socket follows it at r13346); remove the rows and
`R1_GROUPS`; then `Deferral::Rxapi`; close `2026-09-28-rexxqueue-in-function-group.md`.

Licence rows ruled by Moritz 2026-11-02, held out because `the_row_files_hold_only_checked_rows`
rejects them while the Phase 10 rows exist:

    utilities/rxapi	TEST_RXAPI_LOCKED	Deviation 86	the test expects rxapi already running in every Rexx process and the oracle's two lock-file lines from a second rxapi
    utilities/rxqueue/rxQueue	TEST_STDERR_TO_RXQUEUE	Deviation 40	a child rexx's stdout and stderr merged into one pipe interleave by hand-over

Also: `rexx-bench`'s oracle runs use the shared `rust/target/oracle-runtime` with no watcher.
