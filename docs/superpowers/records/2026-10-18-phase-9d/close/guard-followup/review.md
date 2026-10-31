# Task 10 guard and seam follow-up review

## Verdict and scope

Spec verdict: Approved for the authorized guard and seam correction. Quality verdict: Approved, with no remaining Important or Critical finding in this follow-up diff. The reviewed source range is `4bd2e0d70..988026178ba00de5f2fa7c737114e68743333b55`, containing changes to the method-table helper and dispatch-seam test only. The prior scoped review missed these two fix-induced failures, which the fresh release G4 run exposed and retained under `close/attempt-1/`. This review addresses those failures and permits fresh closing gates, without treating the aborted attempt or focused verification as whole-phase acceptance.

## Corrections

**Collection guard: ADDRESSED.** `rust/crates/rexx-exec/src/dispatch/class_protocol/method_table.rs:73` restores the original string-keyed-table check and `unbuilt_collection_owner` refusal before either the native compatibility walk or normal SUPPLIER dispatch. The unchanged helper in `environment.rs:1497` first resolves the actual `.local` and `.environment` identities through `env_seam::which`, so ordinary Directory and StringTable subclasses do not acquire this refusal. The original guard comment and exact `.local` refusal assertion remain intact, and the unchanged seven regression fixtures still cover overridden suppliers, copied protected suppliers, reentrancy and atomic installation. The guard therefore restores the previous whole-collection Phase 10 Rxapi boundary while preserving the accepted SUPPLIER correction for complete collections.

**Dispatch seam: ADDRESSED.** `rust/crates/rexx-exec/tests/dispatch_seam.rs` adds the specific moved file `src/dispatch/class_protocol/method_table.rs` to `CLEARANCE_CONSUMERS`. The change adds no directory-prefix allowance and leaves the existing scanner and other allowed paths unchanged. A synthetic `src/dispatch/class_protocol/unlisted.rs` case verifies that a sibling consumer remains rejected. The six seam tests pass, including the clearance-consumer assertion that failed in the first fresh release attempt.

## Evidence and preservation

The final implementation report appends the follow-up to `.superpowers/sdd/2026-10-18-phase-9d/task-10-corrections-report.md`, with evidence committed at `e4e486286`. The reviewer read the source diff, exact identity guard, existing refusal assertion, preservation record, invocation metadata and raw verification output under `docs/superpowers/records/2026-10-18-phase-9d/close/guard-followup/`. The recorded command at source `988026178` passes all 1,239 library tests and 37 integration tests across corrections, semantic boundaries, dispatch seam, refusal sites, refusal dispositions and closed labels. Formatting and touched-crate Clippy both return 0, and the regression helper still exercises ordinary execution and collection on every allocation. The F1 constant evaluator, F2 RAISE implementation, seven corpus witnesses and original refusal-assertion file are unchanged from the follow-up baseline.

## Considered behaviors declined to judge

* Fresh complete closing gates remain the controller's next step and are not established by this focused approval.
* Previously ruled deviations and performance residuals remain outside this narrow correction review.

No additional fix-induced Important or Critical finding was established in the two-file diff. The existing whole-collection refusal and bounded dispatch-consumer list are restored without changing the accepted supplier behavior. This reviewer ran no tests, builds or probes and made no source, index or HEAD mutation. The original whole-phase and scoped correction reviews remain historical records alongside this follow-up verdict.
