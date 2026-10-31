The original closing run is archived under `../attempt-2/`. References below to `close/tsan/`, `lib.txt`, `int.txt` and the original sanitizer reports describe their pre-archive locations. The retained files now live under `../attempt-2/tsan/`, with their original paths and bytes recorded by the attempt manifest. Focused diagnostic probes remain in this directory.

# Task 10 TSan thread leak diagnosis

The six actual thread reports identify an unnecessary finished reader inside an intentionally unfinished command collector. Each report names `command::collect` at `command.rs:893`, the scoped stderr reader, and says the thread is finished. That reader starts even when `running.stderr` is `None`, while the collector joins the stdout reader before joining stderr. The process can therefore terminate with a finished joinable stderr thread while an abandoned command continues reading stdout.

The source lifecycle supports this explanation independently of the report stack. An output-only CLI redirection uses inherited stderr because `Sinks::own_descriptors` is true, so the stderr reader immediately returns an empty vector. A HALT abandons the command wait and decrements `activities.in_flight` at `scheduler.rs:2641`, allowing interpreter completion without waiting for the child's remaining output. `Pool::drop` joins idle workers only, so the busy worker and its active scope survive until process termination.

The diagnosis is distinct from the queued native-call interpreter retention issue. The abandoned command is removed from the in-flight count and tracked separately in `abandoned_blocks`. The native-call retention guard at `lib.rs:4405` instead forgets the interpreter when an in-flight count remains. These reports require neither a native call nor that retention branch, as the isolated Rexx command probe demonstrates.

## Existing binary probes

All probes used existing frozen binaries without builds or source changes. The foreground command was `timeout --kill-after=10 120 memcap 8G python3 docs/superpowers/records/2026-10-18-phase-9d/close/tsan-leaks-diagnosis/probe.py`, which exited zero. Each case recorded separate stdout, stderr, process status, binary hash, argv, and sanitizer options in its own directory. Sanitizer settings preserved `suppressions=rust/tsan.supp`, `allocator_may_return_null=1`, and `second_deadlock_stack=1`, using absolute paths and a new per-case `log_path` without disabling any check.

| Case | Binary | Status | Process seconds | Reports |
| --- | --- | ---: | ---: | ---: |
| Basic ordinary command | TSan | 0 | 0.215 | 0 |
| Output redirection completing normally | TSan | 0 | 0.315 | 0 |
| Output redirection abandoned by SIGINT | TSan | 66 | 2.520 | 1 |
| Same SIGINT case, six-second wait before program exit | TSan | 0 | 6.724 | 0 |
| Same SIGINT case, stderr also redirected | TSan | 0 | 1.715 | 0 |
| Original SIGINT case | Release | 0 | 0.557 | 0 |

The positive sanitizer reproduction emitted the same `collect:893` finished-thread report as all six original reports. The three immediate HALT variants and the drained HALT variant produced identical intended stdout, `a\nb -4 1 first\nhalted 5\n`, with empty stderr. Waiting before program exit permits normal collection and joining, while redirecting stderr creates a real reader that remains blocked alongside stdout until process termination. These controls isolate the finished no-op reader and process-exit timing rather than a generic failure of scoped thread joining.

## Findings and recommendation

The strongest supported cause is real unreaped pthread state at process termination arising from the no-op reader and intentional command abandonment. There is no evidence that a completed `collect` forgets its explicit reader joins, and both normal collection controls passed TSan. Calling the report a sanitizer false positive or extending the native-call deferral would therefore overstate the evidence. The six original reports and negative gate status remain valid until a reviewed fix and fresh verification exist.

The narrow candidate fix is to spawn each scoped reader only when an actual stream exists. The stderr path can return an empty vector directly when no pipe exists, and the stdout path should handle absent stdout and merged pipes symmetrically. Existing concurrent draining, return-code collection, routing, merged ordering, and prompt HALT abandonment must remain intact. Joining all busy workers at shutdown would wait for arbitrary command or native-call completion and would change the behavior proved by the normal release signal tests.

The candidate has not been implemented or verified because the source is frozen. Its regression proof should retain the exact signal test expectations and demonstrate that the minimal immediate-HALT reproduction stops reporting a finished no-op reader. Both actual streams must still drain concurrently when present, including input and large output/error cases already covered by normal command tests. Naturally completed real streams can also precede a still-running stream, so the candidate fixes the demonstrated unnecessary-reader case without proving every possible shutdown interleaving clean.

## Separate integration failure

The concurrency arity test failed on a binary layout assertion, independently of these thread reports. `tests/support/arity.rs:699` compares the selected CLI's parent against `current_exe().parent().parent()`, assuming the test binary resides under `debug/deps`. The instrumented integration executable ran under `debug/build/rexx-exec/<hash>/out`, so the assumed profile directory became `debug/build/rexx-exec/<hash>` while the CLI correctly remained under `debug`. `int.txt` explicitly records these unequal paths after the selected CLI equals Cargo's selected CLI, so the failure does not establish a CLI child leak.

## General reader lifetime follow-up

A narrower general lifecycle fix can cover real stderr EOF before stdout completion as well as the demonstrated no-op reader. Keep both existing scoped reader threads, but immediately drop their scoped handles and place their results in separate `Vec` variables declared outside `thread::scope`. Each reader can borrow its own output vector exclusively, and the scope waits for every borrower before either result is read. This eliminates retaining either reader's joinable pthread handle while waiting for the other stream, without moving stdout reading or adding a channel.

The installed nightly standard library establishes the required behavior directly. `std/src/sys/thread/unix.rs:149` drops its native thread handle with `pthread_detach`, while `std/src/thread/scoped.rs:159` waits for the scope's running-thread count to reach zero before returning. The scope counter therefore retains borrowed storage even though the pthread no longer retains joinable-thread resources after completion. The existing input writer already drops its scoped handle, providing a local example of this separation between native detachment and Rust scope lifetime.

Preserving the current panic behavior requires explicit catches inside the two detached reader closures. The current explicit `join().unwrap_or_default()` calls convert a reader panic into an empty vector, while a panic escaping a detached scoped reader causes `thread::scope` itself to panic through its unhandled-panic flag. An internal `catch_unwind(AssertUnwindSafe(...)).unwrap_or_default()` around each reader's current vector-producing body preserves the original fallback, including discarding partial bytes when that reader panics. The input writer should retain its current uncaught panic behavior because it is already handled by the automatic scope wait.

The suggested alternative of a detached scoped stderr worker sending its result through `mpsc`, with stdout read directly on the collector's thread, is also compatible with the inspected contracts. `read_all` has no interpreter-local TLS or baton dependency, and the only routed callback posts copied bytes into the mutex-protected scheduler inbox. The collector already executes with halting signals unblocked, and existing child reader threads inherit that same mask, so moving stdout reading does not change the signal mask used by its blocking I/O. Merged output still needs one reader for ordering, and both the stderr helper and direct stdout body need the explicit panic catches described above.

The borrowed-result variant is preferable for a focused repair because it keeps the existing concurrency arrangement and avoids a new result channel. Both alternatives retain the scope until all I/O borrowers finish, so normal collection remains synchronous while HALT can still abandon the busy pool worker through the current pool lifecycle. Neither alternative requires waiting for all workers at interpreter shutdown or changing native-call ownership. This design follow-up is based on source inspection only, with no implementation, build, or additional probe claim.

## Scope and artifacts

The frozen revision remains `9922e56f3bf6c72442d796f8a43f631a86621abd` with no tracked file, index, or HEAD changes. New artifacts are confined to this ignored SDD report and `docs/superpowers/records/2026-10-18-phase-9d/close/tsan-leaks-diagnosis/`. The latter contains the executable probe description, six isolated case directories, `results.json`, and an original-evidence hash manifest. No builds, complete suites, suppressions, new skips, source services, or original-report edits were performed.
