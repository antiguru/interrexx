# Task 10 runner preparation

The runner is prepared and no close gate has executed. `close/run.sh` delegates to the adjacent `runner.py`, which exposes `plan`, `start`, `step` and `finish` modes. Each `step` executes the next action in the foreground and records separate stdout, stderr, command, timestamps and child status without retrying or overwriting an action. Task 10 remains incomplete until the controller executes and reviews the evidence and writes the gate records.

## Invocation and freeze

From the repository root, run `memcap 16G bash docs/superpowers/records/2026-10-18-phase-9d/close/run.sh start --head <full-close-commit>`, then the same launcher with `step` until all actions have records, then `finish`. The controller must commit the runner and any remaining dispositions before `start`, and preserve the committed tree until `status.txt` says `finished`. `plan --head <full-close-commit>` prints the concrete commands without executing gates or creating run state. The outer `memcap` requires the controller's approved escalated execution because the default sandbox cannot create its cgroup.

The freeze records HEAD, the Rust tree object and hashes of the runner, legacy TSan helper, original pin, checkout-local native libraries, oracle libraries and binary, ooTest files and reference sources. Each action verifies HEAD, tracked working tree, index, untracked Rust inputs and the frozen external hashes before execution, then verifies them again before accepting its record. Start records installed toolchains, nightly components, Cargo and rustc versions, SVN provenance and the selected environment. An exclusive runner lock prevents concurrent invocations, and `finish` checks every recorded process group has stopped before releasing the freeze.

## Commands and bounds

The commands derive from Task 10, the 9c close runner and the Phase 6.1 `bggates.sh` and `tsan.sh` helpers. Every Cargo action sets `CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=4` and an explicit target directory, with `-j 4` on directly invoked build, clippy and test commands. Each action executes through `timeout --kill-after=15 <seconds> memcap <limit>`, with a controller watchdog and process group cleanup if the launcher does not exit. Raw nonzero exits remain nonzero in the records and final status.

* Preflight: `python3 scripts/test_preflight.py`, under 8G and 600 seconds.
* G1: `cargo fmt --all --check`, under 8G and 600 seconds.
* G2: `cargo clippy -j 4 --workspace --all-targets -- -D warnings`, under 16G and 600 seconds, using a fresh clippy target.
* G3: `REXX_CORPUS_GATE=1 cargo test -j 4 --workspace --release --no-run`, under 16G and 600 seconds.
* G4: `REXX_CORPUS_GATE=1 cargo test -j 4 --workspace --release --no-fail-fast`, under 8G and 2400 seconds.
* Whole suite: `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=<close>/suite/table.tsv cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite -- --nocapture`, under 8G and 600 seconds, retaining suite diagnostics.
* G5: `REXX_CORPUS_GATE=1 cargo test -j 4 --workspace --no-fail-fast --no-run`, under 16G and 600 seconds.
* G6: `REXX_CORPUS_GATE=1 cargo test -j 4 --workspace --no-fail-fast`, under 8G and 3000 seconds.
* G7: `cargo clippy -j 4 -p rexx-exec --all-targets --features pinning -- -D warnings`, under 16G and 600 seconds, using a separate pinning target.
* G8: build `cargo test -j 4 -p rexx-exec --features pinning --test concurrency_tests --no-run` under 16G, then run the same test selection with `-- measured::a_ measured::an_` under 8G, each bounded to 600 seconds.
* G9: build `RUSTFLAGS="--cfg loom" cargo test -j 4 -p rexx-exec --test loom --no-run` under 16G, then run without `--no-run` under 8G, each bounded to 600 seconds and using a separate loom target.
* TSan: `CARGO_BUILD_JOBS=4 memcap 8G bash .superpowers/sdd/2026-10-07-phase-6-1/p61-gates/tsan.sh <target> <close>/tsan` from `rust/`, bounded to 3000 seconds. The helper uses nightly, its existing exact exclusions and its existing diagnostics paths. The runner retains its raw shell status and separately validates the printed `api`, `lib` and `int` statuses and absence of `tsan.*` reports in `tsan-status.json`, because the helper's final shell status does not represent all children.
* Performance build: `cargo build -j 4 --release -p rexx-exec --bin rexx`, under 16G and 600 seconds, then an exclusive immutable copy to `/home/moritz/dev/repos/ooRexx-p9-pin/rexx-9d-<full-close-commit>`.
* Performance: `bench-programs/callgrind.sh -B 0.5 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/close pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=<immutable-copy>`, under 8G and 43200 seconds. This preserves the helper's all-program selection, default rounds and separate whole-process and libc-subtracted columns, with stdout copied verbatim to `close/callgrind.txt`.

G4 and G6 retain the Phase 6.1 timer and criterion table environment variables in durable close paths. Whole suite success requires its actual child exit 0, with no exclusions or test skips added by this runner. Performance output retains every `OVER` and nonzero exit, without adjudicating the accepted residual under the October 19 ruling. The controller reviews that evidence separately from functional gate success.

## Storage and verification

Read-only inspection found approximately 712G available on the repository filesystem and 16G in `/tmp`. Build targets therefore use fresh per-purpose directories under the already ignored `.superpowers/sdd/2026-10-18-phase-9d/task10-targets/<full-close-commit>/`, and start requires 200 GiB free before creating them. The existing `rust/target` is a real directory and this runner does not alter it, previous caches, profiles, raw outputs or the user's ZIP. Inspection of Rust source found the benchmark suite's binary resolver uses `CARGO_TARGET_DIR`, while the separate benchmark-band executable retains a hardcoded target path, so any gate exposing a remaining hardcoded dependency must be reported rather than silently redirected.

Shell syntax, Python syntax and command-construction checks passed without launching builds, services or gates. Those checks confirmed the repository root calculation, unique action names, the 2400/3000 workspace bounds, fresh clippy target separation, Cargo environment and 16G prebuild caps. Read-only inspection found stable, nightly and 1.98 toolchains, as well as `cargo`, `memcap`, Python and Valgrind, and confirmed the pin SHA-256 `c4b4e8c4ce7f1f12e4a73fa37cb15eca262717092c2c1aed727febf437523c67`. Prerequisite availability, nightly rust-src, actual native loading and suite revision remain runtime evidence collected by start and preflight, rather than claimed gate results.

The controller reported pending queued-item dispositions and inventory reconciliation during preparation, so freeze and gate execution remain blocked on that work. The performance destination requires the controller's approved escalation for the exact plan path, and existing files there cause refusal instead of overwrite. A terminated runner or a failed freeze guard retains incomplete evidence and does not declare completion, and no automatic retry or cleanup removes those records. The controller must review any such interruption before deciding how to proceed.
