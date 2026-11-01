# TSan WRITE-SEGV: shadow-stack overrun by the long-chain test

Unchanged instrumented ELF `babbcab2...` at `b8bb53daa`. Diagnostic only, not gate acceptance.

## Observed

`gdb-first-fault/` (later-sleeper deadline test, then long chain): first SIGSEGV on `rexx-interp`, at
`mov %rcx,(%rax)` in `__tsan::ScopedInterceptor::ScopedInterceptor+82`, entered from `__interceptor_bcmp`.
`rax` is `thr->shadow_stack_pos` (read back from `0x10(%rsi)` one instruction earlier) and equals the fault
address `0x7ffff6100000`, the exact end of a mapping followed by an unmapped gap. The Rust frames beneath are
the repeating `begin_send` / `unknown_or_nomethod` / `stem_unknown` / `forward_to_value` chain; no Rust
pointer is involved in the faulting write.

`gdb-solo-overrun/` (long chain alone): shadow stack base `0x7ffff6772000`, end `0x7ffff67f2000`, 0x80000
bytes or 65536 entries as in `tsan-limits-diagnosis/report.md`. A hardware watchpoint on the first slot past
the end fired in `__tsan_func_entry` on `rexx-interp` with `rax` equal to the end. That slot already held
code addresses, so it is mapped memory owned by something else. The test then reported ok.

## Conclusion

Each chain link costs five real frames of forwarding recursion, so 50000 links exceed the 65536-entry shadow
stack. The runtime does not check the end. Alone, the overrun writes silently into a mapped neighbour and the
test passes; a solo pass therefore never showed the test fits. After a predecessor test the neighbouring page
is unmapped, which gives the WRITE-SEGV. Native stack use at the overrun was well inside the 512 MiB stack.

This explains the pair reproductions. It is consistent with, but does not prove, the other WRITE-SEGV reports
(the full helper's, and the four-thread run's fault in `TraceSwitchPartImpl`, which could be neighbour
corruption). The next full helper run decides those.

`simple-then-chain/` is not a result: Cargo began recompiling the sanitizer `-Zbuild-std` standard library and
the runner killed it at 0.3 s (`unexpected_compilation`), before any test. The ELF hash stayed unchanged.
