# Phase 10 design: rxapi, embedding, extensions, corpus

Decides D7 and scopes roadmap row 10 of `docs/superpowers/plans/2026-07-27-rust-rewrite.md`. Row
10 stays the authority on what L3-full must cover; this document says how Phase 10 gets there.

## Intent

A real ooRexx program that uses external queues, `RxFuncAdd` or a C extension runs on interrexx, a
C program can embed it, and real third-party programs run on it. This ends with L3-full: the only
ooTest exclusions left are `rust/corpus/suite-licences.txt` and `rust/corpus/suite-not-run.txt`.

Moritz does not trust the C++ daemon's memory safety, and reusing it sits badly with the licence
of a `MIT OR Apache-2.0` project. Interrexx therefore ships no C++ of its own at run time.

## Goals

1. A Rust daemon replacing `rxapi`, serving named and session queues, macrospace, and the function,
   subcom and exit registries.
2. `RxFuncAdd`, `RxFuncQuery`, `RxFuncDrop`, `.RexxQueue` and the macrospace functions working
   through it, and `rxqueue` and `rxsubcom` as Rust binaries.
3. The embedding API: `RexxCreateInterpreter`, `RexxStart`, `librexx.so` and `librexxapi.so` with
   the SONAMEs the extensions `NEED`, the second interpreter instance, and `AttachThread` and
   `AddCommandEnvironment`.
4. `rxregexp`, `rxmath`, `rxunixsys` and `rxsock` as Rust builtins, so `LIBRARY rxregexp` and
   `RxFuncAdd(..., 'rxmath', ...)` bind to Rust with no `.so`. `hostemu` and `orxncurses` stay C
   add-ons, built unmodified from `third_party/ooRexx` by script and passing against these libraries,
   resolved without the oracle's `lib` directory on the search path. `cargo build` alone gives a
   pure-Rust interpreter.
5. A real-program corpus, PPWizard and the Rexx Parser, fetched by script.
6. L3-full, and every queued item the inventory rules `phase:10` closed or re-ruled.

## Non-goals

* Wire compatibility with C++ `rxapi`. An oracle and interrexx never share a daemon.
* Proving Windows, macOS or other Unix-likes. Linux is built and tested. Other platforms stay
  possible through the transport trait and nothing more.
* Perf gates, callgrind and new test tooling. One perf round follows Phase 10.1.
* Phase 10.1. Nothing here may block it, and nothing here builds it.
* Durable queues. Daemon death loses queue contents and registrations, as `rxapi` death does.

## Anti-goals

* Any C++ from `rexxapi/` compiled into, translated into or paraphrased into interrexx.
* A daemon that needs installing, configuring or stopping by hand.
* Correctness that depends on noticing a process died.
* An in-process mode reachable by users.
* Tests that touch the user's real daemon or the oracle's `rxapi`.

## Decisions by Moritz

* **D7 is decided, replacing the Phase 0 recommendation:** a Rust daemon with its own protocol.
  Compared against daemonless shared files, which need per-OS locking and wakeups in the core
  logic, and against peer-hosted session queues, which cannot share named queues.
* **Clean-room implementation:** one agent studies the C++ and writes a description, and a
  different agent implements from the description.
* **Registrations outlive the registering program,** as on the oracle, so `RxFuncQuery` answers 0
  on a later run and no Deviation is needed.
* **An in-process service for test isolation,** as the same code, not a second implementation.
* **Stacked PRs,** squash merged in order.
* **Linux is the tested platform.**
* **The clean room covers `librexxapi` too,** and the oracle is run, never disassembled or traced.
* **The guarded trace destination hang is deferred** out of Phase 10 to the queue.
* **`.environment` is per interpreter, recorded as Deviation 85** (2026-11-02). Objects crossing
  instances through the native API are refused.
* **`rxregexp`, `rxmath`, `rxunixsys` and `rxsock` become Rust builtins,** with `hostemu` and
  `orxncurses` staying C add-ons (2026-11-02). The `.cls` wrappers of `rxregexp` and `rxsock` are
  rewritten under the clean-room rules.
* **No perf gates** in 10 or 10.1, and testing with as few tools as possible.
* **D-U1 extends to the C exports** (2026-11-02): one export module per new cdylib crate (`librexx`,
  `librexxapi`) may use `unsafe`, holding only `extern "C"` shims over safe code, pinned in
  `unsafe_sites.rs`.
* **The started daemon closes inherited descriptors through one `unsafe` function** (2026-11-02): at
  daemon start-up it closes every descriptor above 2 except its own lock file and listener, pinned in
  `unsafe_sites.rs`. Chosen over adding `nix` and over dropping the requirement.

## The clean room

The description is `docs/superpowers/specs/rxapi-behaviour.md`, committed. It states observable
behaviour: operations, arguments, return codes, ordering, lifetimes, error conditions. It holds no
C++ identifiers beyond the public API and no structure of the implementation. Its author reads
`rexxapi/`, and may run the oracle.

The implementer never opens `third_party/ooRexx/rexxapi/`. A dispatch instruction is not a control,
so the implementer works in a tree where that directory is absent, with the oracle available only
as a built binary. Public interface comes from `rexx.h`, `rexxapi` headers' public declarations and
`oodocs`. The public headers are `api/`. The oracle may be run, and may not be disassembled or
traced. `docs/superpowers/plans/rxapi-protocol.md`, the D7 block and the `rexxapi/` citations in test
support are absent from the implementer's tree and stay in the repo and its history. The author
reviews nothing of the implementation; behaviour disagreements return as
questions the author answers in the description.

The same split applies to `rxqueue` and `rxsubcom`, whose C++ sources sit under `utilities/`. The
`librexxapi` half of PR 3 follows the same rules, because `rexxapi/client/` is the same code. The
rest of PR 3 implements a documented API the crate already lays out.

## The service

The core is a library crate: a state machine taking one request and answering one reply, with no
sockets, threads or clocks. A request that must wait, such as a pull on an empty queue, answers
"parked" and the reply arrives when another request satisfies it.

A `Transport` trait carries requests. It has two implementations:

* a local stream, a Unix socket on Linux, used by the daemon;
* a direct call, used only by tests and absent from release binaries' reachable paths.

The daemon binary is a thin loop around the core. Messages are one versioned Rust-defined format,
and a version mismatch reached by any other route is a clean refusal.

### Daemon lifetime

* The first client that needs it starts it, detached.
* A lock file decides who owns the socket, so concurrent starters converge on one daemon and a crash
  leaves nothing that blocks the next start.
* It exits when no client is connected and it holds no queue, registration or macro. A daemon
  holding registrations stays up, as `rxapi` does.
* One daemon per user. The socket directory is private to the user: `$XDG_RUNTIME_DIR` when set,
  otherwise a `0700` directory under the temp directory.
* `REXX_API_DIR`, a name the builder may change, overrides the directory.
* Each protocol version has its own socket, so an upgraded client starts a new daemon instead of
  being refused by an old one. The old daemon keeps serving old clients. A cross-version refusal
  therefore cannot arise in normal use.
* `rxapi` with a stop option is the explicit stop. End-to-end tests use it for teardown, and users
  never need it. `RexxShutDownAPI` stays the oracle's no-op.

### Process death

A closed connection is the only end signal, for a clean exit and a crash alike; there is no shutdown
message (Moritz, 2026-11-02). A process attaches its session queue through its connection before
its first command, and the daemon removes a session queue once no connection uses it, whatever it
holds (`rxapi-behaviour.md` Resolution 4). Blocked pulls end when their connection closes.

## Testing

Two tiers, and no new tooling:

* **In-process:** each test owns a fresh core. These run in parallel, and blocked pulls run under
  the simulation mode so queue races are deterministic.
* **End to end:** a real daemon with a test-private `REXX_API_DIR`, for what crosses a process
  boundary, such as `ls | rxqueue` and the `utilities/rxapi`, `rxQueue` and `rxsubcom` groups.

ooTest's suite gate stays the conformance measure, and the oracle diffs stay reported, not gated.

## Corpus

Fetched by a script with pinned versions and sha256 hashes, never committed. PPWizard's licence
forbids distributing `ppwizard.rex` alone. PPWizard's bar is the two-line patch in
`docs/superpowers/plans/ppwizard-gate.md`, with output matching the patched oracle; re-derive both
before relying on them. The Rexx Parser runs its own suites at the commit in
`docs/superpowers/records/2026-10-17-corpus-probe/README.md`, each suite standalone, and its bar is
per-suite parity with the oracle: same rc and same "Tests ran" line. Results are committed as a
record, since CI does not run the corpus. Rosetta Code and Regina's `trip` are
optional, and CI does not fetch the corpus.

## PR stack

Each PR leaves `main` green. Squash merging rewrites the lower commits, so each upper PR is rebased
onto `main` when the one below lands.

0. Oracle bump: the submodule pin and ooTest move together to one fixed upstream revision, suite
   tables refreshed. Everything after is judged against that revision.
1. Description, core, transports, daemon, in-process instance.
2. `RxFuncAdd` family, `.RexxQueue`, macrospace, `rxqueue`, `rxsubcom`.
3. Embedding API, libraries, second interpreter instance, the four `phase:10` native-API and
   embedding queue items.
4. Four Rust builtins, two C add-ons, with the `Row10NativeApi` groups they unlock.
5. Corpus.
6. `gate:phase-9a-gate.md:476`, L3-full close.

## Left to the builder

Crate names, message encoding, module layout, the environment variable's name, how the direct
transport is kept out of release paths, and how PRs split further when one grows too large.

## Calls made by the author, for Moritz to check

* The oracle bump goes first, as PR 0, so the behaviour description and the extension builds are
  not written against a tree that then moves.
* One socket per protocol version, and a stop option on `rxapi` as the only stop.
* `rxunixsys` is a sixth extension, because L3-full's clause has no row excluding it.
* A test that cannot pass on this host stops for Moritz's ruling; the plan proposes the row, and no
  exclusion row is added without him.
* The Rexx Parser's bar is parity with the oracle per suite, since the oracle itself fails some.
* The core's "parked" reply, so the simulation mode can drive it.
* `rxqueue` and `rxsubcom` are inside the clean room.
* CI does not run the corpus, since its sources are not ours to redistribute and the fetch would
  add a network dependency to required checks.
