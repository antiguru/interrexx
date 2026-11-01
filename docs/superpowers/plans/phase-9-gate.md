# Phase 9 gate record

Spec: `docs/superpowers/specs/2026-10-10-phase-9-core-conformance.md`, section 10. Sub-phase records: `phase-9a-gate.md`, `phase-9b-gate.md`, `phase-9b1-gate.md`, `phase-9c-gate.md`, `phase-9d-gate.md`. `C` is `docs/superpowers/records/2026-10-18-phase-9d/close/`, at `d675de751`.

| Criterion | Evidence |
| --- | --- |
| 1. L3-core | `C/whole-suite.json` exit 0, 402 groups, 24,523 rows. Tables `C/suite/table.tsv`, `tests.tsv`, `verdicts.tsv`; oracle-difference report `C/derived-final/oracle-differences.tsv`, 102 rows |
| 2. Exclusions | `phase-9-exclusions.txt`, `rust/corpus/suite-licences.txt`, `rust/corpus/suite-not-run.txt`; owner checks pass in G4 and G6 |
| 3. Labels | `phase_ten_reasons`, `owners`, `closed_phases`, `refusal_dispositions`, `refusal_sites`, `representation_refusals` pass in G4 and G6 |
| 4. Binaries | `C/derived-final/parent-binary-rows.tsv`; exception Deviation 40 `TEST_PASSINGARGS7`. `compiled`, `rexxc` parse-error and D6 coverage tests pass in G4 and G6 |
| 5. Performance | `phase-9d-gate.md` Performance: five programs over +0.5% whole-process, worst `define` +3.57%, accepted by Moritz 2026-11-01 |
| 6. Gates | `phase-9d-gate.md` Close run: G1 to G9 and TSan; G3's raw 124 is retained with its continuation |

Phase 10 handover: `.superpowers/sdd/queued/2026-10-17-phase-10-corpus-and-rxfunc.md`.
