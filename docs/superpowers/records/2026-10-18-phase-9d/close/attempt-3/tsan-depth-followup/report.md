# Fresh TSan library failure diagnosis

The fresh library SEGV is not yet attributed to an exact test or root cause. The original report `tsan/tsan.215861` contains a write fault at absolute PC `0x557998cc36a2`, followed in `lib.txt` by nested sanitizer failure and process exit 66. No original process mapping or usable symbolic traceback accompanies that report. The two focused tests below establish distinct behavior without reproducing that SEGV.

| Exact test | Inner status | Observation |
| --- | ---: | --- |
| `tests::a_join_rendering_itself_through_defaultname_runs_out_of_stack` | 124 | One test started, remained active for its 90-second bound, no sanitizer report |
| `tests::a_cycle_made_inside_a_forward_is_insufficient_stack` | 0 | One test passed in 1.49 seconds, no sanitizer report |
| Join test, explicitly authorized stack diagnostic repeat | 124 | Same 90-second bound, one read-only live stack, no sanitizer report |

Each focused run used the already-built frozen TSan library binary under `memcap 8G`. The outer command was `timeout --kill-after=5 110 memcap 8G python3 .../probe.py EXACT_TEST`, with an inner `timeout --kill-after=5 90 BINARY --exact EXACT_TEST --nocapture`. Both outer runner commands exited zero because the probe script records the child's actual status rather than adopting it, and each case records both statuses in `outer-runner.json`. Each case also retains argv, binary hash, separate stdout and stderr, process mappings, timestamps, and the unchanged current helper options, including `REXX_TEST_THREAD_SANITIZER=1`, existing suppressions, allocator setting, and deadlock-stack setting.

## Symbol evidence

The current binary's `__tsan_func_entry` begins at ELF address `0x445620`. Its unchecked shadow-stack caller write is at `0x445656`, while the corresponding slow-path write in `TraceRestartFuncEntry` is at `0x4400c5`. The original fault PC has page offset `0x6a2`, which cannot equal either of those instruction offsets under the binary's page-aligned ELF load bias. This rules out directly identifying the fresh fault as either of those exact writes, but does not rule out a later fault caused by prior shadow-stack corruption or another sanitizer-runtime failure.

ELF address `0x4456a2` is an `int3` padding byte following `__tsan_func_exit`, rather than a write instruction. The nearby functions are `__tsan_func_entry` at `0x445620`, `__tsan_func_exit` at `0x445670`, and `DDMutexInit` at `0x4456b0`. Mapping the original absolute PC to `0x4456a2` would require assuming an unrecorded load bias and would still contradict the reported write access. Exact disassemblies and ELF load layout are retained as JSON artifacts so this failed attribution is reviewable.

## Normal coverage and decision

The same exact join-recursion and cycle-forward tests passed in the current normal release and debug closing runs. Their successful rows appear in `G4-test-release.stdout` at lines 1862 and 1766, and in `G6-test-debug.stdout` at lines 1870 and 1767, respectively. The join test deliberately checks recursive default-name rendering reaches error 11.1, while the cycle-forward test checks nine cycle shapes return the same error through a syntax handler. The focused sanitizer timeout therefore cannot justify removing either production assertion or treating either test as a proven source of the fresh SEGV.

There is no supported source patch or additional exclusion decision yet. The explicitly authorized diagnostic repeat of the join case collected the missing live stack while preserving the exact test, binary, options, 8 GiB cap, and 90-second bound. Its interpreter stack was in `StackDepotBase::Put -> CurrentStackId -> MetaMap::AllocBlock -> user_alloc -> malloc -> Rust allocation`, while the calling test thread waited for the interpreter and the timer polled normally. At 37 seconds, that interpreter was runnable and had consumed 35 seconds of CPU, supporting sanitizer allocation-stack processing overhead rather than a scheduler wait, without attributing the original SEGV.

The single debugger session ran from `2026-10-31T16:45:22.363903Z` to `2026-10-31T16:45:24.214978Z`, exited zero, and explicitly detached. Its wall duration was 1.851075 seconds, with a shorter unmeasured ptrace scheduling pause. The repeated diagnostic reached its original timeout with status 124 and no sanitizer report, so it adds a stack observation rather than acceptance evidence. Original negative gate evidence and unresolved SEGV attribution remain intact.

## Proposed next diagnostic, not executed

One diagnostic replay of the selected library invocation is now more discriminating than another guessed exact-test sweep. It should use the same existing binary, original `--exact` and 15 exact skips, inherited test-thread settings, helper instrumentation environment, existing suppressions and checks, and 8 GiB cap, with a separately approved diagnostic bound such as 600 seconds. Capture the main binary PID and executable mappings immediately after exec, retain separate stdout, stderr, raw sanitizer logs, exact argv and environment whitelist, hashes, process statuses, and a fresh diagnostic-only destination. Preserve the original negative close result regardless of whether that replay passes, fails, or times out, and do not add `--nocapture` or serialization because either changes the concurrent execution arrangement under investigation.

An absolute fault PC plus the saved mappings and exact ELF can identify the faulting module, load bias, symbol, and machine instruction without attaching a debugger to the full run. That should distinguish a sanitizer bookkeeping write from an application write if the failure repeats in the same mapped module. It cannot necessarily identify the owning test or establish the preceding corruption path, especially if the sanitizer again cannot unwind or the fault occurs in an unmapped dynamically generated region. Debugger registers or a targeted stack should be considered only after that narrower symbol evidence is reviewed, with separate authorization if needed.

## Scope and process cleanup

The frozen revision remains `ee52ea5e62daeb6bc35ca4475de4d1e7a96a4bfe`. No source, tracked document, index, HEAD, skip list, suppression, or sanitizer check was changed, and no build or full suite was run. Elevated read-only process checks at `2026-10-31T16:43:52Z` and `2026-10-31T16:47:23Z` found none of the three timeout PIDs or test PIDs still present, and the only repeated exact test was the separately authorized live-stack diagnostic. All new files are confined to `close/tsan-depth-followup/`, and original report hashes are retained in `provenance.json`.

## Subsequently authorized diagnostic results

The parent later authorized one selected-library diagnostic replay and one distinct cyclic-reference debugger diagnostic, whose raw evidence lives in adjacent `tsan-library-replay-1/` and `tsan-cyclic-reference-fault/` directories. The selected-library replay ended 66 after 22.868 seconds with a read SEGV at ELF `0x467380`, resolved from its actual saved mappings to an indexed trace read inside sanitizer `StackDepotBase::Put`. It also launched from the wrong working directory, causing four relative-fixture tests to fail, so its detailed report explicitly preserves the workload difference and does not equate that read fault with the original write fault. This is an invocation mistake in diagnostic work, and no corrective replay or acceptance adoption was performed.

The distinct `tests::a_cyclic_reference_is_insufficient_stack_in_every_conversion` diagnostic ran from the correct Cargo crate directory under batch GDB with ASLR enabled and the same binary, sanitizer checks, 8 GiB cap, and 90-second bound. It passed its one selected test in 7.91 seconds and the inferior exited normally, with no intercepted SIGSEGV or sanitizer report. The debugger and outer runner both exited zero, and an elevated process check at `16:55:11Z` found no remaining recorded timeout, inferior, or matching debugger process. No further case or replay was launched, and neither diagnostic supplies an exact reproducer or source-fix basis for the original write SEGV.
