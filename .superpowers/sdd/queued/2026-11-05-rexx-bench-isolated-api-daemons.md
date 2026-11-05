**OPEN, queued 2026-11-05. Owner: the performance round after Phases 10 and 10.1, ruled by Moritz
2026-11-05 ("lgtm to have isolated api daemons").**

# rexx-bench shares one oracle runtime directory

`rust/.cargo/config.toml` forces `XDG_RUNTIME_DIR` to the shared `target/oracle-runtime` for every
process cargo runs, so `rexx-bench`'s oracle runs all share it, and nothing stops the oracle's
`rxapi` after them. A daemon a bench run starts outlives it, and its
queues, registrations and macrospace carry into the next run, on either engine's side.

To do: give each bench run a private API directory per side, as the suite harness does since Phase 10
PR 6 (the crate through `REXX_API_DIR`, the oracle through a private `XDG_RUNTIME_DIR`), and stop
each daemon when the run ends, success or failure. Reuse the tests' helper
(`rexx-exec/tests/support/oracle_runtime.rs`) rather than a second copy.

Test: after a bench run, no `rxapi` process whose environment names the run's directories is left,
and a second run sees an empty macrospace and no queues from the first.
