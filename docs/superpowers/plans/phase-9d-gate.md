# Phase 9d gate record

Spec: `docs/superpowers/specs/2026-10-17-phase-9d-divergences-and-close-design.md`. Plan: `docs/superpowers/plans/2026-10-18-phase-9d.md`. Ledger: `.superpowers/sdd/2026-10-18-phase-9d/progress.md`. `C` below is `docs/superpowers/records/2026-10-18-phase-9d/close/`.

## Close run

Frozen at `d675de751`, Rust tree `c25eeeeca`, on fresh target directories, by `C/run.sh`. Status per action is in `C/finished.json`, and each action's `.json`, `.stdout` and `.stderr`.

| Action | Raw status | Result |
| --- | --- | --- |
| preflight, G1 fmt, G2 clippy | 0 | |
| G3 release no-run | 124 | 600 s bound hit while compiling. Identical bounded continuation exit 0 before any test, `C/G3-build-release-continuation-1.json`, `C/build-continuation-ruling.json` |
| G4 full release | 0 | 3831 passed across 178 result blocks, which include child `cargo test` re-runs; 157 test binaries plus 11 doc-test runs |
| whole suite | 0 | 402 groups, 24,523 rows, no row changed from Task 9 |
| G5, G6 full debug | 0 | 3836 passed across 178 result blocks, same shape |
| G7 pinning clippy, G8 pinning | 0 | 16 tests |
| G9 Loom | 0 | 15 models |
| TSan | 0, validated 0 | api, lib, int 0; 70, 1234, 113 tests; no `tsan.*` file, `C/tsan-status.json`, `C/tsan/` |
| performance build, copy | 0 | head SHA-256 `4f56bcfe...`, `/home/moritz/dev/repos/ooRexx-p9-pin/rexx-9d-d675de751bf465140ec834482c9c7fb9f0832177` |
| callgrind | 1 | budget exit, see Performance |

## Suite and inventory

`C/derived-final/summary.json`, from `C/profile-verifier/evidence-audit.py`: 402 groups, 24,523 rows, 640 rows changed from the 9c close, none from Task 9, 453 extension rows, 102 oracle differences, 15 refusal rows disposed, 287 inventory rows and 245 queued files ruled. The set header derivation prints nothing (`C/derived/set-header.stdout`). Extension rows: `C/derived/extensions.tsv`. Refusals: `C/derived/refusal-dispositions.tsv`. Queued first lines: `C/derived/queued-first-lines.tsv`. The one binary exception is Deviation 40, `rexx_command` `TEST_PASSINGARGS7`. Gate item `phase-9a-gate.md:466`, the close callgrind, was executed; `C/derived-final/` is the strict audit with no pending item. Its acceptance is the Performance ruling below.

## Performance

Every program against the pin, SHA-256 `c4b4e8c4...`, three rounds, whole process: `C/callgrind.txt`. `C/derived-profiles/validation.json` validates 198 profiles over 33 programs. The verifier's first run failed on the rexxcps path spelling `bench-programs/../bench-rexxcps`; its normalising copy is `C/profile-verifier/verify-profiles.py`, and the failing output stays in `C/profile-verifier-first-negative/`. Raw profiles: `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/close/`.

Whole-process rows over +0.5%, retained as measured:

| Program | Whole | libc subtracted |
| --- | --- | --- |
| `define` | +3.5657% | +8.5576% |
| `sendloop` | +2.8467% | +0.2531% |
| `dispatch` | +1.9278% | +0.1692% |
| `compound` | +1.4212% | +1.2446% |
| `pingsem` | +0.5231% | +0.4933% |

Spec section 8's budget is +0.5% on every program, and ruling 14 accepted one +0.53% figure. Moritz accepted these five on 2026-11-01 ("accept."), for the perf round. Causes known so far, and the follow-up, are in `.superpowers/sdd/queued/2026-09-20-performance-todo.md`, "Phase 9 close residuals".

## Negative history

Four earlier closes are retained under `C/attempt-1` to `C/attempt-4`, each with its own record. Attempt 1 failed G4; attempts 2 and 3 failed TSan; attempt 4 hit G4's bound under external host load and is the only one where a replay control showed the failure load-dependent, for the STREAM tests only (`C/attempt-4/controller-report.md`). Its `TRACE_TraceObject` failure was not replayed; it passed here.

## TSan limits

The WRITE-SEGV of attempts 2 to 4's diagnostics was the long stem chain overrunning TSan's unchecked 65,536-entry shadow stack, `C/tsan-sequence-controls/root-cause.md`. Under the sanitizer marker that test runs 5,000 links (`7feb9ba1a`); ordinary gates run 50,000. A passing TSan run cannot show that no other test overruns silently. `C/tsan.sh` adds the 100,000-depth PARSE skip to the historical list, and its `TIMING` skip name matches no test.
