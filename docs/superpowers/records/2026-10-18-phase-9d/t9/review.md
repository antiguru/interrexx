# Task 9 independent review

## Spec compliance

Verdict: Source compliant for Task 9 at `561bcc3ffe9614da8108aa58ba44bcef8d885b3f`, compared with `f2dcbd70f`. Final evidence verdict is pending the corpus and suite results. This review covers Task 9 only, and the phase gate documents belong to unstarted Task 10. No missing, extra, or misunderstood source requirement was found.

* `rust/crates/rexx-core/src/owner.rs:56` removes `Owner::Phase9`, its label and parsing arms, and obsolete assertions. `rust/crates/rexx-exec/src/dispatch/native/tests.rs:150` and `rust/crates/rexx-exec/tests/internal_routines.rs:107` remove the corresponding open phase and exhaustive-match arms.
* `rust/crates/rexx-exec/tests/closed_phases.rs:37` closes Phase 9. Its regression at `:322` and `rust/crates/rexx-exec/tests/support/literals.rs:164` distinguish dotted numeric phases while retaining sentence punctuation.
* `rust/crates/rexx-exec/tests/refusal_dispositions.rs:60` removes the closed phase from `OPEN_PHASES`. Its owner parser at `:478` and fixtures in the hunks beginning at `:770`, `:846` and `:949` use supported Phase 10 ownership and retain ownerless, optional, mixed and mismatched-disposition coverage. `rust/crates/rexx-exec/tests/group_runs/whole_groups.rs:2294` updates the synthetic refusal labels and matching suffix together.
* `rust/crates/rexx-exec/tests/concurrency_tests.rs:390` reuses `suite_rows::PHASE_9_EXTENSION_DIRS`. The named risk of accidentally exempting sibling extension directories was checked against `rust/crates/rexx-exec/tests/support/suite_rows/mod.rs:279`: each of the three exact prefixes ends with `/`, and no C extension prefix is included.
* `docs/superpowers/plans/phase-4-exclusions.txt:7153` closes the composite traceback row with separate delivery witnesses and existing Deviation 66. The named risk of inventing a traceback disposition was checked against prior task reports: Task 8.2 `:51`, Task 8.3 `:94`, Task 8.7b `:94`, and Task 6.9 `:120`. The narrow canonical Deviation 66 entry at `phase-4-exclusions.txt:2761` records Moritz's 2026-10-21 ruling and the exact PROGRAM-name witness.
* `docs/superpowers/plans/2026-07-27-rust-rewrite.md:661` receives the required dated closure. `docs/superpowers/plans/phase-4-exclusions.txt:6350` preserves Phase 10 Embedding ownership while removing a historical Phase 9 citation from the owner sentence.

## Code quality

Verdict: Approved for the reviewed source. No Critical, Important, or Minor correctness or maintainability findings. The scanner change is local, preserves punctuation behavior, and adds regression coverage for the distinguishing boundary. The extension change reuses the existing canonical directory set, while the only production change removes obsolete refusal ownership without adding dependencies or unsafe code.

## Evidence and limitations

The supplied diff was read once, with the output-truncated central portion reread. No changed source file was recrawled except the explicitly named canonical Deviation 66 entry outside the diff. No Git mutation, build, or test rerun was performed. The only writes are the two requested review artifacts.

Retained gate metadata for the reviewed revision records status 0 for preflight, formatting, clippy, core, policy, native tests and release build. Their retained stdout and stderr contain no warning, error or FAILED lines. The two regression failures retain status 101, while the first sandboxed memcap attempt records launcher failure separately. The original failed policy output was overwritten, and `policy-attempt-1-reconstructed.txt` is explicitly a transcript reconstruction rather than raw evidence.

Corpus and suite gates, exact suite-row delta, and final aggregate elapsed time remain pending in the implementer report read for this review. Final evidence-complete approval is withheld until those records are updated. No per-task performance run is required by the run-path condition because the diff removes only obsolete owner metadata from production. This task review makes no claim that Task 10 or the phase-wide close gates have completed.
