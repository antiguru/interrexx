# Task 10: Instrumented test resources

## Objective and authority

Implement two test-only resource adaptations after the third failed close, starting at `1bf06ebf4` on `plan/rust-rewrite` in `/home/moritz/dev/repos/ooRexx-rust-rewrite`. The negative freeze is released and all recorded children have stopped. The user authorized finishing Phase 9d, and the controller recorded correction round 4/5 after the earlier three-round scope rule. A fresh implementer and scoped review are required by the current development procedure. Production behavior, sanitizer selection and suppressions must remain unchanged.

Read the current Task 10 section in `docs/superpowers/plans/2026-10-18-phase-9d.md`, the archived `close/attempt-3/controller-report.md`, and `.superpowers/sdd/2026-10-18-phase-9d/task-10-tsan-exact-case-assessment.md`. Here `close` means `docs/superpowers/records/2026-10-18-phase-9d/close`. The original write fault is not proved identical to the standalone read fault, so do not claim that identity. The exact standalone failures below are sufficient red evidence for this test-resource correction. No new language deviation, skip or suppression is authorized.

## Red evidence and contracts

* `close/attempt-3/tsan-candidate-selection/group-3/`: only `tests::a_join_rendering_itself_through_defaultname_runs_out_of_stack`, actual 66 in 10.738 seconds, full symbolized StackDepot/CurrentStackId/native recursion report. It passes in both current normal closing runs.
* `close/attempt-3/tsan-candidate-selection/group-4/`: only `tests::a_long_chain_of_stem_subclass_instances_is_walked_once`, actual 101 in 21.664 seconds, `the chain did not end` at its 20-second timeout, no sanitizer report. Both current normal closing runs pass the full 50,000-link case.
* The recursive join contract is 11.1, rc 245 and empty stdout after the original program reaches the native control-stack guard. The ordinary runtime has a 512 MiB interpreter stack, and stack measurement reserves 32 MiB.
* The long-chain contract retains all 50,000 links, literal output, TRACE assertion and normal 20-second complexity guard. Instrumentation may receive a bounded 180-second wait, with no workload reduction.

## Scope and implementation

* Start by verifying branch, HEAD and tracked cleanliness. Preserve the untracked user file `rexx-instruction-cost-inventory.zip`.
* `src/tests.rs` is unwieldy. Make an early pure-move commit extracting the two resource-sensitive tests, with all their comments and bodies byte-for-byte, into a normal child module such as `src/tests/resources.rs`, declared with plain `mod resources;`. Do not use `#[path]`. Record the old/new test identities and mechanical preservation proof.
* Adapt only explicit `REXX_TEST_THREAD_SANITIZER=1`. Reuse the existing test-only marker convention in `src/test_process.rs`, sharing its predicate if that avoids duplication without introducing future abstractions.
* For the instrumented recursive join, use existing `crate::run_program_on_stack` with an initial 33 MiB candidate, which exceeds the 32 MiB reserve and leaves about 1 MiB active headroom. Keep ordinary execution through the existing `run_program` path and its default 512 MiB stack.
* At the SAME instrumented stack, first prove a nonrecursive Array MAKESTRING calls a custom DEFAULTNAME returning a literal, reaches an `after` marker, and exits zero with exact expected stdout and empty stderr. Then run the original recursive program with the unchanged 245, empty-stdout and 11.1 assertions. This prevents an immediate entry-stack failure from masquerading as the intended recursive guard.
* For the instrumented long-chain case, retain the original 50,000-link program and every output/trace/status assertion, selecting 180 seconds only for the instrumented receive timeout. Ordinary absent/other marker modes retain 20 seconds.
* Preserve every existing comment. No new unsafe, production resource changes, test skips, suppression changes or helper exclusion changes. Avoid em dashes and structuring semicolons in comments.
* Commit the behavior patch before any build or runtime verification. Keep every command foreground, with no services or background watchers. If a process must remain running, return BLOCKED with its session ID rather than ending DONE.

## Verification and bounds

Use `CARGO_INCREMENTAL=0`, four Cargo jobs, existing cached targets and exact relevant flags. Builds use `memcap 16G`, bounded to 600 seconds. Test processes use `memcap 8G`, with an explicit bound sufficient for the instrumented 180-second internal wait. Never extend a running bound or silently retry a semantic failure. If compilation times out, return the actual result and ask the controller for the next bounded continuation ruling.

* Existing normal target: `.superpowers/sdd/2026-10-18-phase-9d/task10-targets/ee52ea5e62daeb6bc35ca4475de4d1e7a96a4bfe/workspace`.
* Existing TSan target: same prefix with `/tsan`; exact helper is `close/tsan.sh`, with `RUSTFLAGS=-Zsanitizer=thread`, current suppression file, allocator/deadlock settings and explicit test marker. Do not use a debugger or live process inspection.
* First run the two changed tests under normal debug and release, retaining original normal resource values and all assertions. Run the complete ordinary rexx-exec library, formatting and relevant Clippy checks, recording all failures by name.
* Run the two changed tests with actual TSan instrumentation and marker one. Require the successful rendering preflight, intended 11.1/245 result, full-chain success and no sanitizer report files.
* Once focused proof is green, run the unchanged current-plan TSan helper, under 8 GiB and 3000 seconds, into a fresh follow-up log directory. Record every API/library/integration status independently and require zero report files, rather than trusting the helper's aggregate shell status.
* Do not run another whole workspace, whole ooTest suite, Loom, pinning or callgrind. The controller owns the next fresh complete close after review.

## Evidence and return

Write `.superpowers/sdd/2026-10-18-phase-9d/task-10-test-resources-report.md` and durable `close/test-resources-followup/report.md`. Retain raw argv, actual statuses, separate stdout/stderr, source/binary hashes, marker modes, resource bounds, native stack choice and positive preflight result. Clearly distinguish archived red evidence from the new candidate and never reinterpret a timeout as a sanitizer exit. The normal programs, assertions, ordinary resource choices, suppression bytes, current helper and production source must have preservation evidence. Report final commits, covering test paths and outcomes, any uncertainty or failure, and confirmation that no child remains.

## Workspace rules

Work only on this existing branch worktree and double-check the expected revision. No `sudo`, `git reset --hard`, `git add -A`, checkout-based reverts, force pushes, push, merge or pull request. Use explicit paths for staging, and use the existing approved git escalation because the common git directory is outside this writable root. If a filesystem operation actually fails for insufficient permissions, report it instead of changing permissions. Keep C++, build inputs, samples, ooTest, ooDocs, oracle and native API inputs read-only.
