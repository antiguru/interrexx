# Selected library diagnostic replay

The authorized diagnostic replay terminated with sanitizer status 66 after 22.868 seconds. It selected the same 1227 tests using the same existing binary and 15 exact skip arguments, with default test concurrency, default output capture, the helper's sanitizer settings, an 8 GiB cap, and a 600-second diagnostic bound. This result remains separate from the failed closing gate and cannot replace it. The direct invocation also used the wrong working directory, as described below, so it does not reproduce the complete Cargo test environment faithfully.

## Captured fault

The new report `tsan.231661` describes a read SEGV at `0x7f4be636d000`, absolute PC `0x55b2f22e1380`, with reported sanitizer thread `T234421`. Saved executable mappings give load bias `0x55b2f1e7a000`, making the instruction ELF address `0x467380`. `addr2line` resolves that address to `__sanitizer::StackDepotBase<__sanitizer::StackDepotNode, 1, 20>::Put`, and disassembly shows `mov (%rsi,%r8,8),%r10`, an indexed read of stack trace words. The original failed gate instead reported a write SEGV with no usable mapping or traceback, so the two failures must remain distinct.

The current binary's `CurrentStackId` takes the sanitizer shadow-stack base and the difference between its top and base, divided by eight, and passes that trace extent to `StackDepotPut`. The retained disassembly shows no clamp in that path, while `ThreadState` allocates the same 0x80000-byte shadow stack identified in the earlier diagnosis. This makes an invalid or excessive trace extent a plausible explanation for the captured read fault, but no fault registers or late thread mappings were available to prove the actual base, index, or extent. It also does not identify the owning test or prove the original write fault has the same cause.

The live stack from the separately authorized join diagnostic passed through the same stack depot while processing allocation stacks. That observation supports a connection between deep recursive execution and expensive sanitizer trace processing, and the join interpreter consumed nearly one CPU second per elapsed second. The join diagnostic did not itself emit a sanitizer fault, and the captured full-library read fault had no usable application traceback. These observations therefore support an evidence-ranked hypothesis rather than a source fix or additional exclusion.

## Invocation fidelity limitation

The replay script launched the binary from `rust/`, while Cargo unit tests normally run from `rust/crates/rexx-exec/`. Four datadriven tests failed before the crash, each reading a relative fixture directory under `tests/`: dropped stems, untrapped stem-sort errors, unload order, and delayed condition traps. Those directories exist beneath the crate root and are absent beneath `rust/`, establishing a concrete invocation error in this diagnostic. No corrective replay was launched, and these failures are retained rather than attributed to production code.

The mistake changes the workload and prevents treating this replay as an equivalent rerun of the original gate. The saved maps, actual fault PC, and exact disassembly still identify where this diagnostic process faulted. Any further attempt to attribute the original gate must retain that distinction and use the correct crate working directory. The original gate remains failed independently of this diagnostic's interpretation.

## Test-name attribution limits

`unfinished-tests.json` compares printed completion names with the normal debug library block and records both the original and diagnostic sets. The normal harness reports 1241 tests, but only 1225 distinct completion names were parsed from the retained block, so the candidate lists are explicitly incomplete rather than an exhaustive active-test enumeration. Candidates lacking completion in both runs include cyclic-reference conversion, join default-name recursion, the long iterative stem chain, collection tests, and several scheduler or simulation tests. None of these names is an owning-test attribution for sanitizer thread `T234421`, and the crash can prevent completion reporting for unrelated tests.

## Provenance and cleanup

The replay maps were captured at `2026-10-31T16:49:30.260567Z`, shortly after process start at `16:49:30.130699Z`. The timeout process and binary PIDs were 231660 and 231661, and both were absent in an elevated read-only process check at `16:55:11Z`. The outer runner exited zero after recording actual inner status 66, with both values retained in `outer-runner.json`. No debugger was attached to this library replay, no source or skip change was made, and all original closing evidence remains untouched.
