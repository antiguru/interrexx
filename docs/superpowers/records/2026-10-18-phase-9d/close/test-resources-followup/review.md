# Scoped test-resource correction review

## Verdict and scope

Spec verdict: Approved for the two original test-resource findings. Quality verdict: Approved within `1bf06ebf4..0e1c10551e3e68fb482d9eaf11b44deefbe14c26`, with no new Critical or Important defect established in that diff. This approves the marker-specific recursive-rendering stack and long-chain wait changes on their source and focused evidence. Task 10 acceptance remains blocked because the unchanged complete sanitizer helper still fails its library stage. The retained full-helper failure is not waived, deferred or converted into published completion by this review.

The review used the exact two-commit package, the prior exact-case assessment, the implementation report and its actual covering logs. Commit `a5e56c800` moves the two test functions into `tests/resources.rs`, and `0e1c10551` adds their test-only resource selection through the shared explicit-marker predicate. The reviewer independently compared both complete moved blocks, including their comments, against the baseline and confirmed byte identity at the move commit. No production source, helper, suppression or gate policy change was included, and no build, test, debugger or live-process inspection was performed by this reviewer.

## Original findings

**Recursive join instrumentation: ADDRESSED in the scoped correction.** `rust/crates/rexx-exec/src/tests/resources.rs` selects the existing `run_program_on_stack` at 33 MiB only when `test_process::thread_sanitizer()` sees the exact value `1`. The absent-marker branch still calls the original `run_program`, preserving the ordinary 512 MiB stack and unchanged recursive Rexx program, invocation, status-245 assertion, empty-stdout assertion and error-11.1 suffix assertion. Before recursion in instrumented mode, the same closure and chosen stack execute Array MAKESTRING through a custom DEFAULTNAME returning a literal, requiring status zero, exact `rendered\nafter\n` output and empty stderr. This positive control is meaningful because it must enter rendering and continue execution before the original recursive assertion can pass, and both the focused instrumented run and complete helper record the moved join test as passed.

**Long-chain instrumented wait: ADDRESSED in focused execution.** The full 50,000-link program, invocation, successful status, exact output and TRACE assertions remain unchanged in the moved chain test. Only the explicit marker value `1` selects a 180-second channel wait, while an absent marker or value `0` retains the ordinary 20-second complexity guard. The focused instrumented run executed and passed this test after more than sixty seconds, so its success did not result from shortening the chain or skipping the assertions. The complete helper subsequently crashed without recording this test's completion, so successful focused execution must not be described as a successful complete-run chain result.

## Preservation and evidence

The shared predicate extracts the existing exact marker comparison without changing the isolated launcher's other behavior. Its containing module remains test-only, and the runtime stack size, 32 MiB reserve, production implementation, current sanitizer helper and suppression bytes remain unchanged from `1bf06ebf4`. The reviewer read `preservation.json`, checked the actual diff, independently verified the pure moves and unchanged helper/runtime files, and inspected command metadata for absent, zero and one marker values. The renamed identities are `tests::resources::<name>`, and the unchanged full helper still selects both rather than silently losing them through an exact filter.

Retained evidence lives under `docs/superpowers/records/2026-10-18-phase-9d/close/test-resources-followup/`, with the implementation report committed at `d162099f3`. `debug-focused`, `debug-other-marker` and `release-focused` each pass both cases, and `ordinary-library` passes all 1,241 tests with the marker absent. Formatting and touched-crate Clippy also return zero. `tsan-focused` executes both moved tests successfully in 64.13 seconds of harness time with marker one, actual command exit zero and no report files. Its separately retained instrumented artifact identity is SHA-256 `babbcab2275012eb3eb159520771371e600b0e47b143bc7d2b00085714bbed4d`.

## Full sanitizer result remains open

The complete helper is failing evidence, notwithstanding its mechanical shell exit zero. `helper-validation.json` and the raw helper outputs record API 0, library 66 and integration 0, with one minimal WRITE-SEGV report in `tsan-helper/raw/tsan.280212`. The library explicitly reports `scheduler::tests::a_deadline_ends_the_programs_wait_on_a_started_sleeper ... FAILED`, while the moved join test reports success. The notifier-depth case and moved long-chain case lack completion before the process abort, so neither their success nor their responsibility for the crash is established by this run. The crash attribution and deadline-test failure require further controller action and remain close blockers rather than findings waived by this scoped source approval.

## Considered behaviors declined to judge

* Attribution of the remaining complete-helper WRITE-SEGV and the scheduler deadline failure is outside this two-finding source review and remains unresolved.
* Full Phase 9d acceptance, fresh complete gate evidence and remaining performance work remain controller obligations.

No additional fix-induced correctness finding was established in the inspected diff. The correction preserves normal resource contracts and keeps both tests executing with their original assertions under instrumentation. The original failures, focused successes and new complete-helper failure are retained as distinct evidence. This report closes only the scoped review and does not close Task 10.
