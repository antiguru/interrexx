**CLOSED 2026-11-03 by Phase 10 PR 3 (08695d1e2, #12).** PR 3 plan item 4: a blocking wait re-blocks the halting signals, so a group Ctrl-C halts each interpreter.

**DEFERRED to Phase 10, ruled by Moritz 2026-10-18.** the second interpreter instance is row 10's; no probe.

# Two interpreters in one process reopen the Ctrl-C command race

Found by the Phase 6 S2-S5 Task 21 re-review 3 (F-3). For the interpreter API phase.

P67 (amended) closes the race where a group SIGINT's command end beats the halt only for one
interpreter per process: the handler runs on the interpreter thread or the waiting pool thread. With
two interpreters in one embedder process a group Ctrl-C halted the next command 11/120 under load
(0/60 with one). Direction: consult `sigpending` before filing a command's end, or route the signal to
the waiting thread. Also: `install_signal_handlers` blocks the halting signals in its caller's thread,
which an embedder must know.
