# Task 10 close runner readiness review

Verdict: Changes requested before freeze. This review inspected the prepared runner committed at `d18219237`, its report and the historical helpers without executing builds, gates or profiling. Root remains coordinator, and no source or index change was made. The supplier correction and separate Important F1/F2 source findings remain outside this runner review.

## Readiness findings

1. **Important: The external manifest omits inputs actually executed by the required harness.** `close/runner.py:62-70` hashes only `ooRexx/build/bin/rexx` from the oracle bin directory and omits the oracle samples tree. `tests/support/group_runner.rs:426` places the oracle bin directory on REXX_PATH, where the harness loads the actual `yaml.cls` and other pure-Rexx packages, while `:719` links oracle samples into both interpreter layouts. `tests/rexxc.rs:215` explicitly runs oracle `rexxc`, which is also omitted. Changing those files during a freeze would pass the before/after guard and could change required results. Hash the consumed oracle `bin` and `samples` trees, including file membership and actual symlink targets, and retain the exact chosen paths before start. Preserve the existing local and oracle native-library manifests and clarify any remaining environmental limits.

2. **Important: The required copied TSan status artifact is written before its validated verdict exists.** `close/runner.py:209` writes `tsan-status.json` before `:211` computes `validated_status`, and before `:215-219` can attach a post-run freeze failure. The legacy helper can exit zero even when an api/lib/int child failed, so the named TSan status file can present `status: 0` with no validated result in precisely the case the runner intends to reject. `TSan.json` and the step/final exit correctly carry the later verdict, but the Task 10-specific status artifact and runner report are inconsistent. Write the copied TSan status only after child validation and the post-run guard, retaining both raw shell status and validated status in that artifact.

These findings concern evidence integrity and do not establish a runtime defect. Correct them in the original runner owner's scope and retain a narrow re-review of the changed manifest and TSan serialization paths. Neither requires repeating completed Task 8.8 or Task 9 validation. Do not start the source freeze while the separately authorized production corrections remain incomplete.

## Command and execution audit

The runner defines 17 ordered actions covering preflight, G1-G9, the complete suite, legacy TSan and close performance. G2 uses a fresh per-close clippy target, and separate pinning and loom targets preserve their configuration. Direct Cargo commands set four jobs and `CARGO_INCREMENTAL=0`, release/debug compilation and pinning/loom prebuilds receive 16G, and executions receive 8G. G4 and G6 preserve their required 2400/3000-second bounds and corpus gate, while the standalone suite enables its actual zero-exit assertion and retains table/diagnostic paths.

G8's two libtest filters match existing feature-gated `measured::a_*` and `measured::an_*` tests in `tests/concurrency_tests.rs`. G9 sets `RUSTFLAGS=--cfg loom`, matching the integration target's `#![cfg(loom)]`. The legacy TSan path is absolute in the action, so the report's shortened shell spelling does not cause a working-directory failure. Its helper invokes nightly with `-Zbuild-std`, runs api/lib/integration selections and emits all three child statuses, which the runner correctly checks in its primary action record.

The required bounds on the workspace test executions and full performance action are explicit. Fresh builds and lint have 600-second runner bounds, which are additional operational limits rather than proof they will finish in that time. The runner records timeout/interruption as failure and does not retry or increase limits. No actual fresh-target completion or nightly rust-src readiness was established by this read-only review.

The runner does not explicitly assert test population counts after a zero child exit. Current G8/G9 source selections are nonempty, so there is no demonstrated empty-selection defect in these exact commands. During completion review, require positive populations in the retained G8, G9 and TSan component logs and the full suite's discovered denominator, rather than treating exit zero alone as population proof. The complete corpus and binary/parser obligations remain part of the workspace results and close matrix.

## Freeze, status and process behavior

Start requires a full committed HEAD, clean tracked tree and index, no untracked Rust inputs and fresh target/output state. Guards compare HEAD, the Rust tree and external hashes before each action and after ordinary child execution. The driver writes evidence and target artifacts without staging, committing or editing tracked source, and exclusive creation prevents overwriting earlier action results. Its separate immutable performance copy refuses an existing destination and the pin hash is checked before profiling.

Each step is foreground under the prescribed outer memcap, obtains an exclusive runner lock and starts one bounded child process group. Interruption handling sends TERM then KILL to that group, waits for its leader and marks the attempt interrupted, while finish checks every recorded group is gone. A group found alive after an otherwise exited leader is recorded as an orphaned-child failure even after cleanup. No background watcher, automatic retry or raw-profile deletion is introduced.

Finish requires a record for every action and writes every failing status before marking the run finished. It can release a completed unsuccessful freeze, which is correct provided the controller reads `finished.json` and the nonzero finish exit instead of interpreting the word finished as passing. Source or external-input drift prevents normal finish and needs explicit controller handling. The copied TSan status inconsistency is the actionable exception to otherwise preserved child-status accounting.

## Provenance, paths and performance

The configured target root is inside the already ignored Task 10 target area and start requires 200 GiB free. The runner deliberately leaves the real existing `rust/target` directory untouched. The benchmark suite binary resolver has existing target-aware behavior, while `rexx-bench-band.rs:143` retains its historical hardcoded target path, as the runner report discloses. That standalone program is not directly selected by any close action here, so this review does not invent a gate failure from its mere existence.

Start records toolchain/component listings, compiler/Cargo versions, SVN info and selected environment, and each action records its effective explicit environment. These records are provenance, not hermetic isolation of all host libraries and executables. The concrete omitted oracle execution inputs in finding 1 are stronger than that general environmental limitation and must be included because the suite depends on them directly. Parent binary proof must retain oracle rexxc provenance as well as the oracle interpreter.

The callgrind action uses the full default program set, `-B 0.5`, the required Phase 9 pin, the immutable close executable and the required close output path. It retains stdout, stderr, hashes and raw nonzero status, and copies stdout verbatim to `callgrind.txt`. `OVER` lines are printed to stderr by the helper and remain in `callgrind.stderr`, so the gate document must link that file rather than implying that the table copy alone contains every budget warning. Plan Ruled 14 accepts the October 19 guard interpretation, and the separately accepted Task 8.8 residuals remain unchanged. A budget-only nonzero close result must be reported honestly and adjudicated under that existing ruling, not converted into a claimed universal budget pass.

## Review limits

The only additional executable check was parsing the runner's Python AST and loading its definitions without calling main or actions. No gates, helper scripts, child tests, profiles, toolchain installation, source mutation or index operation ran. Current root work and the untracked final-source review/probes were left untouched. After the two runner corrections and the separate source/scoped-review work, root can execute the committed close candidate and assess the actual results against the acceptance matrix.


## Scoped correction review at 2a758bf67

Verdict: Approved for runner readiness. This re-review covers only `d18219237..2a758bf67` for `close/runner.py` and the updated runner report. Both original findings are ADDRESSED, with no new fix-induced correctness or evidence-integrity breakage found. This approval does not claim that gates have executed or that the separate production corrections are approved.

Finding 1 is ADDRESSED by `tree_inputs` and the two consumed oracle tree entries. The manifest now includes directory and file membership, file hashes, symlink spelling and resolved targets, and follows linked directories while rejecting ancestor cycles. The selected oracle root matches `tests/support/oracle.rs`, covering the actual bin lookup, compiler and linked samples paths identified in the original finding. Existing local and oracle native-library entries remain in the manifest.

Finding 2 is ADDRESSED by moving `tsan-status.json` serialization after legacy child validation and the post-run guard, using the identical final record written to `TSan.json`. Read-only inspection of the retained selfcheck artifacts confirmed both files are identical for all four controls: failing lib child with raw shell zero, post-run freeze drift, clean pass and emitted TSan diagnostic. Their validated statuses are respectively 1, 1, 0 and 1, with raw shell status and component statuses preserved. No fixture or real gate was rerun during this scoped review.

Action commands, caps, exclusions, bounds and foreground process handling are unchanged by this correction. Existing review limits and the October 19 performance guard ruling remain in effect, including retention of raw OVER output. No source or index mutation, build, gate, profile or subagent action ran in this re-review. Root can proceed after its independently required source/scoped-review and commit prerequisites are satisfied.
