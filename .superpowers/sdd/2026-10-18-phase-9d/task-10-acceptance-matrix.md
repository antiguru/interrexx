# Task 10 acceptance matrix

This is a requirements audit at `82e10bb92c9f3dc2b2e76a8c956361bfcfabcaeb`, not a close verdict. Paths abbreviated `R/` refer to `docs/superpowers/records/2026-10-18-phase-9d/`. The governing requirements are the Task 10 brief, the 9d design spec and parent spec section 10. Later explicit rulings override the original budget wording, as recorded below.

## Spec goals and constraints

| Requirement | Existing evidence | Required close evidence or gap |
|---|---|---|
| 2.1 rederive and partition all unexcluded oracle-passing failures | `rust/corpus/suite-9d-set.txt` header derivation, `R/clusters.tsv`, `R/clusters.md`, Task 1 report/review | Run the exact header filter on close `suite/tests.tsv`, retain empty output, check original set membership and unique cluster assignment |
| 2.1 scclient and sfclient treated as members | Task 1 measured redirect baton cause, Task 6.13 delivery and Task 9 complete suite | Cite close sample group rows and existing numbered rules if any |
| 2.1 suite assertion exits zero | `R/t9/suite.json`, `suite.stdout`, `suite/tests.tsv`, `suite/verdicts.tsv`, 402 groups and 24,523 keys | Fresh complete close suite exit 0, full denominator and static input provenance |
| 2.2 pure-Rexx extensions belong to 9d | Task 4 report/review, `support/suite_rows/mod.rs`, Task 9 concurrency partition | Close rows for both json groups, both dateparser groups and yaml, all expected rows pass. Harness yaml source remains oracle build copy |
| 2.2 C libraries remain Phase 10 | `phase-9-exclusions.txt`, canonical extension partition, `phase_ten_reasons` and owner policy tests | Retain policy results and source selection, preserve C library owners |
| 2.3 static refusal enumeration | `R/refusals.tsv`, Task 3 report, `R/refusals-probes/sweep.py`, `sweep2.py`, `rust/corpus/refusal-sites.tsv` and `tests/refusal_sites.rs` | All 15 enumerated rows currently disposed: 7 done, 7 guard, 1 oracle-crash. Close scanner/representation/disposition tests must pass |
| 2.3 reached pairs covered and implemented or ruled | Task 7 report/review, phase-9 corpus witnesses, `refusal-dispositions.tsv` | Retain complete gated corpus and static scanner results. Do not reinterpret guards or invent new licences |
| 2.4 queued and carried inventory | `R/queued-inventory.tsv`, 281 rows, batch ruling and task closure records | Six queued filenames absent. One absent file explicitly says not yet ruled. Reconcile before claiming inventory complete |
| 2.4 every queued file has disposition first line | 245 nonempty files checked at audit snapshot | Retain all 245 path/first-line pairs, not just a count. `2026-10-20-definemethods-supplier.md` still OPEN/not yet ruled |
| 2.5 no Phase 9 refusal owner, SysFork/semaphores Deviations 42/43 | Task 5 and Task 9 reports/reviews, owner variant removed | Close `closed_phases`, `refusal_dispositions`, `owners`, `phase_ten_reasons`, `representation_refusals`, `refusal_sites` results |
| 2.6 traceback, dotted phase and roadmap closure | Task 9 report, `R/t9/review.md`, roadmap row 9 CLOSED 2026-10-31 | Cite actual delivered traceback witnesses/Deviation 66 and scanner regressions, preserve Phase 9.1 |
| 2.6 two gate records and Phase 10 handoff | Task 10 brief and `.superpowers/sdd/queued/2026-10-17-phase-10-corpus-and-rxfunc.md` | Write `docs/superpowers/plans/phase-9d-gate.md` and `phase-9-gate.md` after close evidence exists |
| Non-goals/anti-goals | Existing numbered Deviations, native API/rxapi/C owners and suite-not-run OS rules | Preserve read-only ooTest, existing approved dispositions and oracle crash evidence. No relabelling to finish |
| Budget/rulings | Plan Ruled 14: +0.5% guard against obvious regressions, later performance round. Task 8.8 final user-approved residuals | Retain every close measurement even on nonzero budget exit. Do not claim all-program budget pass or reopen accepted original Task 8.8 residuals |
| Source constraints | Existing unsafe-site, representation, specialization and corpus policy checks | Full close tests, frozen source/index until status finished, retain all raw evidence |

## Task 10 commands and deliverables

| Step | Required execution/deliverable | Acceptance evidence still required |
|---|---|---|
| 1 pre-run commit/freeze | `close/run.sh` modeled on 9c close, commit recorded before execution | Source SHA, script hash, target/build provenance, exact environment, every command/status, final `finished` marker and explicit freeze release |
| 1 formatting | `cargo fmt --all --check` from rust | close formatting exit/status/log |
| 1 fresh clippy | `CARGO_INCREMENTAL=0 cargo clippy -j4 --workspace --all-targets -- -D warnings`, fresh target, capped build | close clippy exit/status/log |
| 1 separate compile | `CARGO_INCREMENTAL=0 memcap 16G cargo test -j4 --workspace --release --no-run` | close build exit/status/log before release tests |
| 1 workspace release | `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 timeout 2400 memcap 8G cargo test -j4 --workspace --release --no-fail-fast` | workspace exit 0, every binary result, corpus count and policy results |
| 1 whole suite | `REXX_SUITE_GATE=1` and full discovery, table/diagnostics paths | suite exit 0, tests/table/verdicts/oracle-differences TSVs and diagnostics |
| 1 G5 | `REXX_CORPUS_GATE=1 cargo test --workspace --no-fail-fast --no-run` under `memcap 16G` | debug build exit/status/log |
| 1 G6 | `REXX_CORPUS_GATE=1 timeout 3000 memcap 8G cargo test -j4 --workspace --no-fail-fast` | debug test exit/status/log, corpus and all binaries |
| 1 G7 | fresh pinning target `cargo clippy -p rexx-exec --all-targets --features pinning -- -D warnings` | capped build, exit/status/log |
| 1 G8 | `memcap 8G cargo test -p rexx-exec --features pinning --test concurrency_tests -- measured::a_ measured::an_` | pinning self-tests exit/status/log |
| 1 G9 | `RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=<loom> memcap 8G cargo test -p rexx-exec --test loom` | capped build, Loom exit/status/log |
| 1 TSan | from rust: `CARGO_BUILD_JOBS=4 memcap 8G bash ../.superpowers/sdd/2026-10-07-phase-6-1/p61-gates/tsan.sh <target> <logs>` | nightly execution results and copied TSan status file in close, retain every component result |
| 2 close pin | release rexx at close commit, immutable `rexx-9d-<sha>` | executable hash and source/build proof, Phase 9 pin hash |
| 2 callgrind | `bench-programs/callgrind.sh -B 0.5 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/close pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=<copy>` | complete table copied to `close/callgrind.txt`, raw records retained, actual budget status interpreted under Ruled 14 |
| 3 9d gate | suite filter, 9c join, extension rows, refusal/inventory/labels/gates/performance evidence | all derivations retained, changed rows listed, unresolved inventory ruling addressed |
| 4 parent gate | six criteria across 9a-9d, suite table, oracle differences, Phase 10 handover | complete final evidence record, then commit |

G5-G9 source is `.superpowers/sdd/2026-10-07-phase-6-1/p61-gates/bggates.sh`. Use four jobs and `CARGO_INCREMENTAL=0`, and cap each build as Task 10 requires. The historical TSan path is repository-relative and needs `../` when invoked from `rust/`. Preserve exact historical command semantics while recording any launcher adaptation explicitly.

## Parent section 10 criteria

| Criterion | Existing proof | Close assertion |
|---|---|---|
| 1 L3-core D1 | Task 9 zero-exit suite, complete 402-group denominator, 24,523 keys | fresh zero exit, all discovered groups, table and oracle differences linked from parent gate |
| 2 exclusions | `phase-9-exclusions.txt`, `suite-licences.txt`, `suite-not-run.txt`, Task 9 passing owner policy | only Phase 10 exclusion rows, only Deviation/environment/flaky licences, permitted not-run reasons and passing owner checks |
| 3 labels D3 | Task 9 policy and independent review | all structural checks pass, closed_phases includes Phase 9, no live owner/source refusal names closed phase |
| 4 binaries D4-D6 | `phase-9c-gate.md` criterion 4, current rexxc/compiled/parser tests and Task 8.8 default results | close suite expected sets for rexx_command/rexxc/API METHOD/Method/Routine. Parse-error TSV agreement except program-selected REGISTERED with Rxapi reason, parser coverage/unreached tests, restore routes/observables and foreign oracle image refusal all pass |
| 5 performance | prior per-task profiles, Ruled 14 and user accepted Task 8.8 residuals | fresh full close callgrind table and honest guard/ruling interpretation |
| 6 gates | prior close/test records are provenance, not a replacement | fresh G1-G9 with corpus gate and repeated TSan, all results accounted for |

## Gate document evidence map

The 9d record should begin with frozen commit, executable hashes, command/status table and measured suite denominator. Follow with the set derivation, 9c raw-row delta and extension rows, then refusal and inventory dispositions and labels. Include complete whole-process performance figures with the exact budget ruling and retained negative evidence. The parent record maps these artifacts and historical 9a/9b/9b1/9c gate records to each of its six criteria, and links the Phase 10 handover without asserting unperformed checks.
