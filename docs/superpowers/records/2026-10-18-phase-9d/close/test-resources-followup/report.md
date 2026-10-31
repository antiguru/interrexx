# Test resource correction follow-up

The test-only correction is implemented, but acceptance is blocked by the unchanged full TSan helper's library failure. The pure-move commit is `a5e56c800`, and the behavior commit is `0e1c10551`. Both were committed before any build or runtime verification, starting from `1bf06ebf4` on the existing `plan/rust-rewrite` worktree. Focused normal and instrumented checks passed, while the complete helper returned independent API 0, library 66 and integration 0 statuses.

## Scope and preservation

The extraction moved both complete test blocks and comments into `rust/crates/rexx-exec/src/tests/resources.rs`, using plain `mod resources;`. Their identities changed from `tests::<name>` to `tests::resources::<name>`, with byte counts and SHA-256 proof in [move-proof.json](move-proof.json). [Preservation evidence](preservation.json) verifies both original Rexx programs, invocations and assertion blocks byte-for-byte, and confirms that only three test-source files changed. Production source, the 512 MiB ordinary interpreter stack, 32 MiB reserve, suppression bytes, current helper and its exclusions are unchanged.

Only `REXX_TEST_THREAD_SANITIZER=1` selects the test resource adaptations, through the shared test-only predicate in `test_process.rs`. The join uses the existing bounded-stack runner at 33 MiB and first runs a nonrecursive Array MAKESTRING through a custom DEFAULTNAME returning `rendered`. Its preflight requires status zero, exact `rendered\nafter\n` stdout and empty stderr before the original recursive body checks 245, empty stdout and error 11.1. The chain retains all 50,000 links, output and TRACE assertions, selecting a 180-second instrumented wait while absent and other marker values retain the ordinary 20-second guard.

## Verification

All Cargo commands used `CARGO_INCREMENTAL=0`, four jobs and the prescribed cached normal or TSan target. Builds and Clippy used `memcap 16G` with 600-second bounds, focused test processes used `memcap 8G` with 300-second bounds, and the ordinary library used 8 GiB with a 2400-second outer bound. The unchanged full helper used 8 GiB and 3000 seconds, and no running bound was extended. Each command directory retains argv, environment, source hashes, actual status, elapsed time and separate stdout and stderr, while the unchanged helper itself retains its customary merged per-stage logs.

| Command | Actual exit | Seconds |
| --- | ---: | ---: |
| `debug-build-sandbox-launch-failure` | 1 | 0.007 |
| `debug-build` | 0 | 71.235 |
| `debug-focused` | 0 | 3.633 |
| `debug-other-marker` | 0 | 3.578 |
| `release-build` | 0 | 161.177 |
| `release-focused` | 0 | 2.013 |
| `ordinary-library` | 0 | 10.701 |
| `format` | 0 | 2.414 |
| `clippy` | 0 | 11.718 |
| `tsan-build` | 0 | 48.560 |
| `tsan-focused` | 0 | 64.703 |
| `tsan-helper` | 0 | 88.532 |

The first sandbox launcher exited 1 before Cargo started because `memcap` could not access a writable delegated cgroup. Its raw evidence remains separate, and the established approved host execution mechanism then ran all verification under the same required caps. Debug focused checks passed both tests with the marker absent and with marker `0`, and release focused checks passed both with the marker absent. The complete ordinary library passed 1,241 tests with no failures, while formatting and rexx-exec all-targets Clippy also exited zero.

The actual instrumented focused run passed both tests in 64.13 seconds of harness time and produced no sanitizer report files. Successful completion proves the positive rendering preflight and original recursive assertions passed on the same 33 MiB stack, followed by the complete chain within its 180-second wait. [Focused artifacts](tsan-focused-artifacts.json) records the selected TSan binary SHA-256 `babbcab2275012eb3eb159520771371e600b0e47b143bc7d2b00085714bbed4d`, captured after focused execution and before the helper. The initial hash collector covered normal `deps` paths, so these supplemental artifacts explicitly cover nightly's `build/rexx-exec/.../out` executable location rather than leaving its empty initial binary map unexplained.

## Full-helper failure and limits

The helper's shell exit zero does not represent acceptance, because [independent validation](helper-validation.json) records API 0, library 66, integration 0 and one sanitizer report. API passed 70 tests, and the four integration result blocks passed 63, 8, 41 and 1 tests respectively. The library printed `scheduler::tests::a_deadline_ends_the_programs_wait_on_a_started_sleeper ... FAILED` before crashing, without the normal final failure diagnostics. It also printed the moved join test as passed, but did not report completion of the moved chain test, so focused chain success does not establish success in this complete run.

The retained [sanitizer report](tsan-helper/raw/tsan.280212) records a WRITE-SEGV at an unknown address, without a usable stack, and the library log records repeated DEADLYSIGNAL and a nested sanitizer abort. This does not prove identity with the prior whole-library WRITE fault or the archived standalone recursive READ fault. The archived red evidence remains attempt-3 group-3's recursive exit 66 and group-4's 20-second wait failure exit 101, distinct from this candidate's focused successes and new full-helper failure. No semantic failure was retried, no timeout was reclassified as a sanitizer exit, and no further diagnosis or final closing gates were started.

All foreground subprocess calls and tool sessions finished, and the enclosing memcap wrappers completed without a leftover-cgroup diagnostic. No child, service or background watcher remains from this work. The user ZIP remains untracked and untouched, and no external runtime, oracle, native input, C++, sample or suite files were edited. The controller must decide the next bounded action for the full-helper failure before accepting this correction or beginning another close.
