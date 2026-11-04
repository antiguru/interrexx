**CLOSED 2026-11-03 by Phase 10 PR 6 Task 1 (d04cbdaac).** `api_group_tests.rs` runs TEST_REXXQUEUE on a private API service; it matches the oracle, and `RECORDED` is empty.

**DEFERRED to Phase 10, ruled by Moritz 2026-10-18.** not probed, it reaches rxapi; roadmap row 10 names FUNCTION TEST_REXXQUEUE.

# FUNCTION's TEST_REXXQUEUE needs RexxQueue (owner: Phase 10)

`.rexxqueue~create('ADDRESSWITH')` is rc 120, `the LIBRARY REXX entry point
"rexx_create_queue" is not implemented (Phase 10)`. Recorded in phase-4-exclusions.txt;
remove it from `RECORDED` in `api_group_tests.rs` when the queues land.
