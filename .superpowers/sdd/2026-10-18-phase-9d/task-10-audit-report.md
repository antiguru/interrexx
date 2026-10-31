# Task 10 evidence audit

This read-only audit inspected `82e10bb92c9f3dc2b2e76a8c956361bfcfabcaeb` and wrote only this report and the acceptance matrix. No builds, interpreter runs, Git mutations or disposable profile cleanup ran. Tasks 1-9 have retained reports and reviews, and Task 9 has a zero-exit complete suite. Phase closure still requires Task 10 evidence and one concrete unresolved queued disposition.

## Concrete gaps

The committed inventory has 281 rows, while the queued directory has 245 nonempty files. Six queued filenames have no inventory row, including one whose first line explicitly states that no ruling exists. The remaining five files already carry dispositions and need metadata reconciliation, preserving their existing decisions. The only inventory row ruled exactly `9d` is `gate:phase-9a-gate.md:466`, the Task 10 close callgrind obligation.

| Missing filename | Existing disposition and evidence | Proposed inventory treatment |
|---|---|---|
| `2026-10-19-perf-round-and-whole-review.md` | RULED later by Moritz 2026-10-19, plan Ruled 14 | `item=filename`, `kind=perf`, `probe=-`, `proposed=later`, `ruled=later`, reason: targeted performance round and manual whole-implementation review deferred by Moritz 2026-10-19 |
| `2026-10-18-fifo-buffered-write-seek.md` | CLOSED Deviation 48 ruled Moritz 2026-10-18 | `kind=divergence`, `probe=-`, `proposed=closed:Deviation 48`, `ruled=closed:Deviation 48`, reason: transient writes never seek, oracle defect not copied |
| `2026-10-19-security-manager-protected-primitive.md` | CLOSED Task 6.12, witness `corpus/lang/security_manager_set_protected.rex` | `kind=divergence`, `probe=corpus/lang/security_manager_set_protected.rex`, `proposed=closed:Phase 9d Task 6.12`, `ruled=closed:Phase 9d Task 6.12`, reason: protected primitive method identity and copied/defined protection delivered |
| `2026-10-19-traceobject-cost.md` | CLOSED Moritz 2026-10-19, no traced performance targets | `kind=perf`, `probe=-`, `proposed=closed:ruled 2026-10-19`, `ruled=closed:ruled 2026-10-19`, reason: traced-program performance has no target per explicit ruling |
| `2026-10-19-upstream-refused-insert-mutates.md` | NOT COPIED controller 2026-10-19 under standing oracle-bug rule | `kind=divergence`, `probe=-`, `proposed=not-a-divergence`, `ruled=not-a-divergence`, reason: upstream-only report, no suite dependency, existing ruling preserved |
| `2026-10-20-definemethods-supplier.md` | OPEN, not yet ruled. Task 6.12 re-review queue candidate and re-review r2 explicitly retain w2 as known difference | Add an explicitly pending proposal row only until root finds or obtains a ruling. Do not fill a ruled owner, claim closed, or create a Deviation from this audit |

The proposed columns use the existing inventory schema `item, kind, probe, proposed, reason, ruled`. The first five entries above are metadata-only reconciliation of written dispositions, not new policy. The sixth is a real evidence gap: the complete queued file has only the original divergence description, and progress mentions it only as queued. The r2 review explicitly exempts `w2` from its otherwise matching 32-probe result, so that review does not close it.

## Static suite audit

Both the 9c close and Task 9 tables contain 24,523 unique `(group,test)` keys with no additions or removals. At Task 9, the exact suite-9d-set header filter selects zero rows. Comparing all fields yields 640 changed rows, with oracle results unchanged: 154 crate failed-to-passed rows, two driver-blocked-to-passed rows, 451 passed rows with metadata changes, 23 failed rows with metadata changes and ten hostemu rows changing from failed to an explicit Phase 10 Extension refusal. Those ten are an existing owner disposition and must be listed as such, not silently described as a new passing assertion.

The existing `R/t9/suite-delta.py` verifies schemas, uniqueness, group/table coverage and exact raw fields, but hardcodes Task 8.8/Task 9 paths and permits changed rows. Reuse its loader and denominator checks for the 9c/close join, preserve all old/new row, oracle and crate fields, and assert no unapproved regression separately. `R/t9/suite/delta-summary.json` proves zero Task 8.8-to-9 changes, additions or removals. Its retained full suite is useful prior evidence but cannot establish the new frozen close's results.

Task 9 pure-extension rows all pass: `extensions/json/json` 33, `extensions/json/json_02` 227, `extensions/yaml/yaml` 50, `extensions/dateparser/DateParser` 112 and `extensions/dateparser/DateFormatter` 31. Binary groups show API METHOD 320 passed, Routine 60 passed, Method 56 passed, rexxc 11 passed and rexx_command 29 passed plus one existing failed assertion. The latter is `TEST_PASSINGARGS7`, licensed Deviation 40 per the 9c gate record and retained row, so parent D1 does not mean every raw assertion passes. Close documents must distinguish expected-set green from raw verdicts.

## Reusable derivations

Run these from the repository root after a complete close suite exists. `R` below abbreviates `docs/superpowers/records/2026-10-18-phase-9d`, and each derived output should be retained beside the source table. These are proposed commands, not commands already executed against a close. The two shell variables below are task-specific and preserve the original checked-in inputs.

```sh
phase9d_records=docs/superpowers/records/2026-10-18-phase-9d
phase9d_tests="$phase9d_records/close/suite/tests.tsv"
awk -F'\t' '$3=="-" && $4=="passed" && $5=="failed" {print $1"\t"$2}' "$phase9d_tests"
awk -F'\t' '$1 ~ /^extensions\/(json|yaml|dateparser)\// {print}' "$phase9d_tests"
python3 - <<'PYCODE'
from pathlib import Path
for path in sorted(Path('.superpowers/sdd/queued').iterdir()):
    if path.is_file():
        print(str(path) + '\t' + path.read_text().splitlines()[0])
PYCODE
```

For the exact baseline join, use `docs/superpowers/records/2026-10-15-phase-9c/close/suite/tests.tsv` as the old table and the close table as the new table. Use `csv.DictReader` after filtering `#` lines, assert fields `group,test,row,oracle,crate`, assert unique group/test keys and compare the union of keys. Retain columns `group,test,change,old row,old oracle,old crate,new row,new oracle,new crate`, as in `R/t9/suite-delta.py`, and include every changed row rather than only failed-to-passed rows. The committed `suite/oracle-differences.tsv` is generated by the harness itself and must be retained with its unfiltered table.

The refusal proof is split deliberately between the historical dispatch-pair enumeration and the current source scanner. `R/refusals.tsv` has 15 rows: seven `done (Task 7.1)`, seven `guard` and one `oracle-crash`, with no blank or `implement` disposition. `R/refusals-probes/sweep.py` and `sweep2.py` preserve the original probe derivations, and Task 3 report records exact commands and receiver enumeration. Current `cargo test -p rexx-exec --test refusal_sites --test refusal_dispositions --test representation_refusals` rederives constructor sites and checks their current policies, while `REXX_REFUSAL_SITES_REFRESH` is a writing mode and must not be set during frozen validation.

Parent binary coverage is already implemented in `rust/crates/rexx-exec/tests/rexxc.rs` and `compiled.rs`, and parser coverage in `rust/crates/rexx-parse/tests/errors.rs`. The close workspace run includes `rexxc_agrees_with_every_recorded_parse_error_row`, `registered_is_refused_with_the_rxapi_reason`, `each_route_runs_a_restored_program_as_the_oracles_does`, `a_foreign_or_damaged_image_is_refused`, `every_message_the_parser_names_is_raised_by_a_corpus_row_or_recorded_as_unreached` and `every_unreached_reason_still_holds`. These preserve the REGISTERED program predicate, Rxapi reason, both sides' own compilation and the oracle-written foreign image refusal. Cite their new close log results and the specific close suite binary-group rows instead of re-probing 9c manually.

## Parked items and accepted rulings

The stderr investigation `2026-10-29-suite-fixture-stderr-refusal-load-flake.md` remains an open `follow-up`, approved by Moritz's 2026-10-31 lgtm. Passing final execution does not identify its cause or create a Flaky licence. The ten original Task 8.8 own-instruction overruns, worst alloc +0.73785570%, and original-e2 parse residual are likewise accepted historical results. Plan Ruled 14 makes +0.5% a guard against obvious regressions and explicitly defers broader optimization and whole-implementation review.

Existing later placements remain valid: trace chunk swap and drive.rs macros are explicitly ruled later, the oracle bump is queued after 9d before Phase 10 gates, in-flight native interpreter lifetime and uncharged native Buffer storage are user-approved after-9d investigations, and the external-review performance agenda and unary integer specialization belong to the later performance round. The 9b1 mixed minors row retains only performance m5, with m1 Deviation 71 and m4 fixed. The residual inventory audit also preserves the guarded-trace conditional deferral and its original approval, rather than treating it as an unfinished close fix. Deviations 82, 83 and 84 already have user rulings and must not be reopened by Task 10.

Historical `9d-cluster:*` inventory rows must not be mistaken for unfinished implementation. Six such queued files already have CLOSED first lines for Tasks 6.4, 6.6, 6.9 and 6.10, with retained witness references. The carried sample gate row points into the redirect baton cluster, and the carried stream row duplicates the closed stream queued file. Reconcile these metadata pointers against final suite evidence while preserving their historical proposal and reason text.

## Review coverage and final review boundary

The correct implementation review base for the complete Phase 9d plan is `328fc0e288a9168a53c6a5b9b99f20fbe07cf7bd`, immediately before Task 1. The broader documentation history begins with plan commit `b44ffe45fb981f3aa19a996c8ff9f379dfdf11c7`, whose parent is `858e3fa0ab15356083338c63b633cc9f6fb2dfdf`. A final review should compare the implementation base to the eventual close candidate, not to the Task 8.8 base alone. Parent phase proof additionally uses the existing 9a/9b/9b1/9c gate records and the 9c whole-branch review plus scoped approval.

Do not repeat the completed whole Task 8.8 source review. The ledger identifies its range as `0c2ff21ae..9ace68a51`, with the initial report under `R/t8.8/final-review/review.md`, scoped corrections under `rereview.md`, and final R1 approval under `r1-rereview.md`. The last report covers `9d7da3281..f598eb1c6` and verifies runtime source `9d919b7c3832c55edc3a0a07974309f092102201`. Separate test-only registration and final test-repair reviews, passing `R/t8.8/final-gates/final/audit-driver-output.json`, the retained manifest and the clean Task 9 review complete the available later coverage.

A final whole-plan review can audit requirement coverage and cross-task integration using these prior approvals, inspect changes not already reviewed, and check close evidence against this matrix. It should retain the known approved boundaries and report only concrete new correctness or invariant findings. The independently queued future manual whole-implementation review is broader than this completion review and remains deferred under its existing ruling. Neither that deferred investigation nor a redundant repeat of Task 8.8 should be presented as a prerequisite newly invented by Task 10.

## Remaining close evidence

No Task 10 close run, full close callgrind or new parent/subphase gate document was verified by this audit. The runner owner must finish preparation, root must reconcile inventory and the unruled supplier item, and the final source must be committed and frozen before gates. Retain failures and every component status until all required checks are accounted for, then write the two gate documents using actual outputs. The acceptance matrix lists the exact required commands and deliverables without making advance success claims.
