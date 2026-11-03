# Phase 10 PR 5: corpus record, 2026-11-03

`scripts/fetch-corpus.sh` fetched the pinned archives, and `scripts/run-corpus.sh` ran them:

* **Crate:** the debug `rexx` built at `4d1f48309`.
* **Oracle:** `third_party/oracle` at r13346.
* **Output:** `corpus.tsv`. Its columns are the program, then rc and result for the oracle, then rc
  and result for the crate. CI runs neither script, so this record is the evidence.

## PPWizard 22.148, patched

* Both sides end rc 0.
* The page each writes for `tryme.it` has the same sha256. The hash in `corpus.tsv` belongs to that
  run's work directory, because the page names its input's path. Run from another directory, it
  differs.
* The 2026-08-12 hash in `ppwizard-gate.md` was taken from yet another path, so it is not comparable.

## The Rexx Parser at 33cf252d

All 31 suites end rc 0 on both sides, with the same "Tests ran" line. That is the spec's bar.

The five suites that ended rc 213 on the oracle in the 2026-10-17 probe now pass on it. Those were
AboutTimer, Assets, Captions, MarginBoxes and YAMLFrontMatter.

Below the bar, the assertion, failure and error counts agree in 29 suites, including the four where
the oracle has failures of its own. They differ in two:

| Suite | Oracle | Crate | Cause |
|---|---|---|---|
| BIFs | 549 assertions, 0 failures | 543, 2 | `TEST_MAX`, `TEST_MIN`: `max(11, , 13)` is 40.5 here and 93.903 "argument 0 is required" on the oracle (ruled, `phase-4-exclusions.txt` MAX/MIN; `defects.json`) |
| SyntaxErrors | 409, 0 | 383, 1 | `TEST_ERROR_99_DIRECTIVES`: `::Constant c (1+2)` outside a class raises 99.906 on both sides, but the crate's condition has no ADDITIONAL where the oracle's is `("C")`. A crate defect, queued as `2026-11-03-constant-no-class-additional.md` |

## Found and fixed by this PR

* **The defect:** a WHEN whose consequence is an IF with no ELSE and a false condition left its
  SELECT branch open. A debug build tripped the frame check in `ir/drive.rs`, and a release build
  ran on into the OTHERWISE.
* **Where it showed:** ten suites, through `Expressions.cls:382`.
* **The fix:** `4d1f48309`, with three corpus programs and a unit test. The same commit fixes a
  second mis-landing: a WHEN that did not hold landed on the next instruction's ops instead of
  EnterOtherwise, so a queued condition's SIGL named the wrong line.

## Environment

* Each side runs from one short `mktemp -d /tmp/corpus.XXXXXX` directory. PPWizard builds variable
  names from the hex of its input's path, and a long path exceeds the oracle's symbol length.
* The crate side finds the oracle's Rexx-only packages (`yaml.cls`, `logicalwrapper.cls` and the
  rest) on `PATH`, as the suite harness lays them. The five packages this crate rewrites are left
  out.
