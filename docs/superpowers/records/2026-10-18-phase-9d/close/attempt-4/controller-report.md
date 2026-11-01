# Fourth close: Negative at G4, host overloaded

Freeze `abf0fe679`. Preflight, G1, G2 passed. G3 raw 124, identical bounded continuation 0. G4 raw 124 at
its unchanged 2400-second bound, stopped at `introspection_arity`. The suite and every later action did not
run, so this attempt proves nothing about them.

`concurrency_tests` took 1847.53 s against 441.08 s in attempt 3. With none of our processes running, host
CPU was 78% busy and load average 33, from outside this session's PID namespace.

Two tests failed:

* `the_seeded_gate`: 10 reds, all `STREAM.testGroup`, `TEST_QUERYDIR_EXISTS` and
  `TEST_QUERYFILE_EXISTS_NOTOPEN`, which compare file timestamps with the wall clock.
* `each_group_of_the_derived_list_in_one_run_in_both_modes`: `TRACE_TraceObject` derived-every outcome
  differs at `TEST_TRACEOBJECT_COLLECTOR`, which relies on `syssleep 0.1` for threads to end.

Control (`load-control/`, seed `241864837125493658:pre:2,k=1`, same binary): quiet host 0 of 8 runs red; with
96 spinning processes 3 of 8 red, the same two STREAM tests each time. The TraceObject failure was not
replayed; its attribution to load is an inference. Neither failure is adopted or waived; the next close must
pass them.
