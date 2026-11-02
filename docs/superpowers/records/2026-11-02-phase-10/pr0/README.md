# Phase 10 PR 0: oracle bump

Pins: `third_party/ooRexx` 90267a193 (r13346), ooTest and docs r13347, built by
`scripts/setup-third-party.sh` as `Release`. `rexx -v` answers `5.3.0 r0`, `Build date: Jul 30 2026`
(the epoch in the script): the binary does not carry its revision.

The old binary was built from cb9563364 with a local NumberString patch, so the effective step is
that tree to r13346, wider than the 39 commits between the two submodule pins.

## First run, at a0977a4d8

`cargo test --locked --workspace --no-fail-fast -j4` under `memcap 8G`: 3803 passed, 33 failed, 5
ignored (`first-run-failures.txt`, one name per test, two names shared by two targets). The run
before it stopped at build: `rexx-lib` pins `CoreClasses.orx`, and the image gained
`CommandClasses.orx`, so every program refused until the file and its entry points were bootstrapped.

| Class | Tests | Cause |
|---|---|---|
| Upstream API moved | 8 | `rexx-api` layout 6 (new members, versions); `api_group_partition` 2 |
| Image moved | 8 | `CommandClasses.orx`: native entries 1, `.environment` 2, `eval` count 1; line 1457 to 1459: `signals` 3, `trace_delivery_memory` 1 |
| New ooTest or docs revision | 13 | `rexx-extract` 10; derived concurrency list 1; sim table 1; `LogicalWrapper` rows 1 |
| Oracle fixed, crate followed | 2 | `bif_assertions` SYMBOL (#2080, #2118); `method_bodies` native raise reported at the clause |
| Crate defect, new ooTest row | 1 | `keyword_assertions` ADDRESS in INTERPRET, queued and exempted |
| PR 0's own edit | 1 | `refusal_sites` line shift |

Exposed later, by the fixes above or by the crash re-run: POS overrun, DATE day 0, 98.975 for a
spread hole, `RexxCallBack` in row 10, Stem in the sim gate, `added53` in the doc extractor.

## Crate changes

POS overrun removed. DATE day 0 is 40.19. `at((,2))` is 98.975. SYMBOL and the scanner follow #2080
and #2118. A native boundary raise is reported at the sending clause. `CommandClasses.orx` bootstraps
with its entry points deferred as `Phase 10 Embedding`.

Queued, not fixed: `.superpowers/sdd/queued/2026-11-02-command-handler-classes.md`,
`2026-11-02-interpret-address-with-stem.md`, `2026-11-02-oracle-r13346-corpus-diffs.md`.

## Oracle answers

`crash-sweep-old.txt` and `crash-sweep-new.txt`: every program in `oracle-crashes.txt` once on each
build, `crash-repeats.txt` the repeats. Entries 2, 3, 6, 13, 17, 18, 20, 37, 39, 49, 56, 63 no longer
fail and keep a one-line entry for the citations to them. 23 (9 of 20), 26 (0 of 20, was 1 in 10 to
30) and 57 (9 of 12) are intermittent and stay. 16 stays, rebuilt against the new headers, rc 139.
13, 49 and 63 name no upstream fix, and the old build was `RelWithDebInfo`.

`defects-changed.txt`: `defects.json` repros on both builds, 102 run, 17 differ, of which
`debuginput-linein-recursion`, `sysfiletree-stat-failure-undefined` and `syswait-no-child-undefined`
are pointer noise and `do-count-wider-than-digits` is the 8 s deadline.
`deviation-probes.tsv`: 59 oracle-bug Deviations, 3 and 30 changed. A running `rxapi` from the old
build served the new binary's clients during these runs.

## Final run, at d84034c4d

`cargo test --locked --workspace --no-fail-fast -j4` under `memcap 8G`: exit 0, 3836 passed, 0 failed,
5 ignored, 178 test result lines. `cargo fmt --all --check` and
`cargo clippy --locked --workspace --all-targets -- -D warnings` exit 0.

`REXX_CORPUS_GATE=1` over `corpus.rs`, at 8ab654dfd: 10 of 1249 disagree, queued. The sim oracle sets
were refreshed under `REXX_SIM_ORACLE_REFRESH=1`. The ooTest suite gate (`REXX_SUITE_GATE`) was not run.
