**OPEN, queued 2026-11-05. Owner: the performance round after Phases 10 and 10.1, ruled by Moritz
2026-11-05 ("rxsock: lgtm, defer to performance round").**

# rxsock waits by polling

A blocking rxsock call does not block. `rust/crates/rexx-exec/src/builtin/rxsock.rs` re-parks every `WAIT_POLL` (5 ms)
and retries the call non-blocking (`wait_again`). An idle server burns a wakeup per 5 ms, and a reply
arrives up to 5 ms late.

Proposal: park the activity with `ParkReason::Block`, as `command.rs` does for a command's wait, and
run the wait off the baton on a helper thread. The helper `poll(2)`s the socket together with a
cancel descriptor, with the call's deadline (or none) as the timeout. A HALT or the interpreter's
termination writes the cancel descriptor, so the wait ends at once and the halt is delivered as now.

Tests:

* A `SockRecv` blocked on a socket answers within 1 ms of the peer's send (today up to 5 ms).
* A HALT sent to a program blocked in `SockRecv` and `SockAccept` ends the call and raises HALT.
* `SockSelect` with a timeout returns after the timeout, not later than today.
* A program blocked in `SockRecv` costs no CPU while idle (measured wakeups, not a sleep count).
