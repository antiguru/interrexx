# Architecture

interrexx runs ooRexx programs. It keeps ooRexx's object model and replaces the execution model:
a register IR run by a stackless driver, activities as continuations under a per-interpreter
scheduler, and value representations a program cannot observe.

## Crates

All under `rust/crates/`.

| crate | role |
|---|---|
| `interrexx-num` | decimal arithmetic under `NUMERIC DIGITS`, `FUZZ` and `FORM` |
| `interrexx-core` | object handles, the arena heap and collector, behaviours, register frame arenas |
| `interrexx-parse` | source retention, scanner, clause splitter, grammar; produces the AST |
| `interrexx-classes` | the class and method-dictionary model, and the native class table |
| `interrexx-lib` | the `.orx` class library sources from `third_party/ooRexx`, embedded at build time |
| `interrexx-inventory` | tables derived mechanically from the ooRexx tree (builtins, error messages) |
| `interrexx` | the interpreter: IR compiler and driver, dispatch, builtins, streams, commands, scheduler; the `rexx`, `rexxc` and `rexx-ir` binaries |
| `interrexx-api` | the C ABI boundary: context interface tables, value conversion, library loading |
| `interrexx-rxapi` | the API service: queues, registries, macrospace; the `rxapi`, `rxqueue` and `rxsubcom` binaries |
| `interrexx-capi` | `librexx.so.4`: the C entry points, and owner of the API's client state |
| `interrexx-capi-rxapi` | `librexxapi.so.4`: the classic API names, forwarding to `librexx.so.4` |
| `interrexx-oracle` | runs a program under an interpreter and captures stdout, stderr and exit status separately |
| `interrexx-extract` | lifts ooTest test methods into standalone programs |
| `interrexx-bench` | the benchmark harness and suite tools |
| `interrexx-spec` | parser and checker for the language specification under `spec/` |

## Parsing

A hand-written scanner turns retained source bytes into tokens, a clause splitter cuts them at
`;`, line ends and continuations, and a hand-written recursive-descent parser builds the AST.
Keyword-ness is decided by position, never by the scanner, because Rexx has no reserved words.
Blanks are significant (`f(x)` is a call, `f (x)` is abuttal), so the scanner reports them. The AST
is plain owned Rust data, not heap objects: nothing in the language or the C API exposes code
below Method, Routine and Package, and source is exposed only as text. The parser has its own
nesting limit, past which it raises 11.1.

## The IR and executor

There is one engine. A code body compiles lazily, on first entry, into a `Chunk`: an immutable
stream of 16-byte `Op`s over a register file. Chunks are cached keyed by body and by the trace
setting they were compiled under; a change of trace setting recompiles. Every instruction compiles;
the only compile failure is `ChunkTooLarge`, when a body needs more registers, slots, call sites or
call arguments than a `u16` indexes, or more ops or constants than a `u32` does. That body is
refused loudly.

* **Variables.** A per-body plan maps each upcased name to a slot, built by one walk of the body
  and cached on the interpreter. Names a body acquires at run time (`INTERPRET`, `DROP (v)`,
  exposed names) are resolved after the plan.
* **Registers** live in a frame carved from the activity's frame arena (`interrexx-core/src/frame.rs`),
  read without bound checks under the invariants in that module's doc.
* **Clauses.** The driver runs clause by clause; a clause op opens each clause, and the clause
  boundary is where line numbers are set, queued `CALL ON` traps are delivered, the slice countdown
  is checked and requests are applied.
* **Trace is explicit ops.** Trace echoes and intermediate values are ops in the stream, so an
  untraced chunk carries none of them and `TRACE I` output survives promotion.
* **Quickening.** A side table of hints lets arithmetic and call sites remember a fast path. A hint
  never removes a precondition check; the fast path re-validates and falls through.
* **Calls are stackless.** A Rexx-to-Rexx call or send pushes the callee's frame and continues in
  the same driver loop; `RETURN` delivers into the caller's register. A depth cap raises Error 11.
* **INTERPRET** compiles its fragment to a chunk of its own, resolved against the enclosing frame.

## Values and the object model

A handle (`ObjRef`) is 64 bits with a 2-bit tag: a heap slot with a 30-bit generation, a small
integer, `.nil`, or a string of up to 7 bytes held inline. The heap is a vector of slots; a freed
slot's generation moves on, so a stale handle misses instead of naming a new object. Collection is
mark-and-sweep, triggered at allocation by slot count and by bytes allocated; every allocation is a
potential collection point, and values in flight are held in rooted registers or temps. Tracing is
one exhaustive match over the `Body` enum.

**Representation is never observable.** Every value behaves as an object of its class: identity,
`objectName`, object variables, methods, `copy`, collection. Inline strings, small integers and any
fast path that assumes a plain kind are optimisations, and each must be able to become a full
object the moment a program asks something only an object can answer. A difference only the
representation explains is a defect unless a Deviation licenses it. The licensed rule: plain
String values (numbers, `.true` and `.false` included) are value-based. Whether two equal ones are
one object is unspecified, they carry no per-object state (`objectName=` refuses), and the
`identityHash` of a held value is stable. Subclass and enhanced strings are full objects.

* **Numbers.** A string is bytes, never UTF-8 text; every index and length is a byte offset. A
  number's rendering is fixed when it is created, under the `DIGITS` and `FORM` then in force. Text
  values cache their parse; the cache is exact and rounding belongs to each operation.
* **Stems** are objects with a default and a tail map; a dropped tail is a tombstone, not an absent
  key. `stem. = x` and `DROP stem.` replace the object; a tail assignment mutates it. Iteration
  order over a stem's tails is unspecified (a Deviation).
* **Dispatch** follows ooRexx's documented search order, including `UNKNOWN` and NOMETHOD, the
  `PRIVATE`, `PACKAGE` and `PROTECTED` checks, and the restricted check on `run`, `setMethod` and
  `unsetMethod`. Strings answer messages through the required-string protocol where the language
  asks for a string.
* **Instances** dispatch against the behaviour their class held at construction. `~define`,
  `~defineMethods` and `~delete` give the class a new behaviour, so existing instances keep the
  old one; `~inherit` and `~uninherit` change the behaviour in place, so existing instances see it.
* **Class lifetime.** Built-in classes are immortal. Every other class, and every package, method,
  routine and compiled program, is an ordinary heap object and is collected when unreachable. Rows
  keyed by a program id are pruned when the program is freed, and an id is never reused.
* **UNINIT** runs when a collection finds the object unreachable, when an activity ends, and at
  termination for every live object that defines it. The order over several instances at
  termination is not reproducible on ooRexx and nothing may depend on it.
* **Startup** has no saved image: the embedded `.orx` library is parsed and run at every start.

## Collections

One contents protocol (iterate index and item pairs, look up, put, remove, count) serves the
collection classes; ordered collections are backed by an array store, mapped ones by a hash store.
Each store holds handles in both positions and is traced. `makeArray` stays per class: hash
collections answer indexes, `Array` and `List` answer items, `Stem` its tails. A base-class object
hashes internally; any other object, including a String subclass, is sent `HASHCODE`. Hash
iteration order is unspecified (a Deviation). Argument arity per entry point is taken from the
ooRexx method table, not guessed.

## Concurrency

* **Isolation.** An interpreter owns its heap, collector, classes, `.environment`, `.local`,
  scheduler and guard table, and shares nothing with another interpreter. `.environment` is per
  interpreter (a Deviation; ooRexx's is per process).
* **Activities are continuations**: a chain of activations with register frames, in an arena of
  their own. Switching between them is a pointer swap.
* **The baton.** Exactly one thread touches an interpreter's state at a time: the baton holder.
  The baton is released only at a driver exit, around native calls and blocking operations, where
  the releasing thread holds no interpreter value. The off-baton operation posts a typed completion
  to the interpreter's inbox, and the activity finishes it when it next runs. A C callback from off
  the baton requests it, or is delegated to the holder.
* **Pinning.** An activity with Rust frames between the scheduler and its driver (a native
  re-entry, a sort comparator, a trap handler, `INTERPRET`) is pinned and waits on a nested
  scheduler on its own stack. A wait that inverts that nesting is refused loudly.
* **Scheduling** is round-robin: a ready queue in arrival order, timed sleepers in a deadline heap,
  FIFO guard waiters. A timer thread sets the slice bit every 24 ms; the switch happens at the next
  clause boundary. HALT, trace toggles and raises from elsewhere set request bits that the target
  applies at its next clause boundary; nobody touches another activity's frames.
* **Signals.** SIGINT, SIGTERM and SIGHUP handlers are installed where none is set. A handler sets
  an atomic and writes a byte to a self-pipe; the timer thread turns that into HALT in every live
  interpreter.
* **Rexx semantics.** Guard locks per object and scope, FIFO, with deadlock detection raising
  98.905; `GUARD WHEN` re-evaluates on a set or drop of a variable it names; `REPLY` splits the
  continuation into a new activity, moving its frames; `.context~thread` numbers reuse freed ones
  oldest first; program end waits for every activity, then collects and runs uninits.
* **Per-activation state.** Settings, the elapsed clock, the `RANDOM` seed and `SETLOCAL` follow
  ooRexx's activation rules: internal calls copy the caller's settings in, `INTERPRET` shares them,
  methods, routines and started activities start fresh.

### Simulation mode

`REXX_SWITCH_MODE=sim:SEED[,policy][,knobs]` makes a run deterministic under one seed. One clock
seam serves every clock read; virtual time starts at a seeded epoch, advances per clause and jumps
to the next deadline when every activity waits. Each kind of decision (schedule, order, clock,
collection, children) draws from its own stream, so a knob does not re-roll the schedule. Every
native call, command and stdin read runs on the baton. Policies: `pre:d` (d preemptions among the
first k contended steps), `uniform:p`, and `pct:d`; each keeps a fairness floor. Knobs include
`gc=q`, `halt@K` and `fail=wait:K`. The scheduler checks its own invariants at every switch, and a
run records a decision trace that `sim:trace=FILE` replays. Anything that cannot be simulated
(named semaphores, rxapi) is refused in sim. ooRexx differences under a seed are reported, never
gating.

## Streams and platform

* Output is buffered in the interpreter and written at exit or before a blocking stdin read;
  nothing writes to descriptors 1 and 2 directly. `.STDOUT` and `.STDERR` write into those buffers.
* The interpreter mutates no process-global state. It holds a shadow environment and a shadow
  current directory, initialised from the process unless the caller supplies them. Every relative
  path, `VALUE(..., 'ENVIRONMENT')`, `DIRECTORY()` and every child process uses the shadows.
* A stream's state lives in its object. Files are write-through, reads have a 4 KiB read-ahead,
  and the stream builtins are message sends through a per-activation stream table keyed by
  qualified name. `.File` qualification is lexical and works for paths that do not exist.
* `SAY`, trace output, error reports and input reads route through the `.local` monitors as
  ooRexx's do, with a direct write only while the default route is untouched.
* Commands run through `std::process::Command` with the shadows; stdout and stderr are drained and
  appended where ooRexx's own writes would fall. `cd`, `set`, `unset` and `export` are handled in
  the interpreter. The security manager is checked per executable for commands, streams,
  environment lookups, protected methods, calls and requires.
* OS access goes through `std`, then `rustix`, then a safe crate; raw `libc` only in granted
  modules.

## Standing decisions

| | decision | reason |
|---|---|---|
| D1 | arena of slots with tagged, generation-checked handles; one traced `Body` enum | memory safety without hand-written mark methods; full-GC pause measured within 1.5x of ooRexx |
| D2 | no saved image; bootstrap from the embedded `.orx` sources at start | an image is an optimisation and startup is fast enough without one |
| D3 | isolated interpreters, a per-interpreter baton, no process-wide lock; cross-activity requests by flag or channel only | no invariant may rest on one activity running at a time |
| D4 | the decimal arithmetic is a port of ooRexx's, not a decimal crate | ANSI rounding and rendering differ from IEEE decimal in tested details |
| D5 | the C API is source-compatible with ooRexx's frozen headers; `RexxObjectPtr` is a handle in the activation's reference table | extensions recompile unchanged; a stale handle misses instead of aliasing |
| D6 | platform calls through `std`, then `rustix`, then raw `libc` only with a reason | safety by construction |
| D7 | a Rust API daemon with its own protocol; no wire compatibility with ooRexx's `rxapi` | no C++ at run time, and no shared daemon with ooRexx |
| D8 | conformance ladder: differential corpus (L0), extracted assertions (L1), the ooTest framework boots (L2), the suite passes (L3) | the suite needs a nearly complete interpreter to run at all |
| D9 | performance regressions judged by instruction counts against a pinned binary | instruction counts are deterministic; wall clock is not on this host |
| D10 | hand-written recursive-descent parser over a hand-written scanner | measured faster, smaller and dependency-free against `chumsky` |
| D11 | RexxUtil routines are internal packages of the interpreter, not a loaded library | ooRexx resolves them as internal packages, after `::ROUTINE`s and before files |
| D12 | the security manager has one check mechanism with hooks at each interception site | one mechanism, call sites added as subsystems land |
| D13 | the AST is plain owned Rust data | nothing exposes code below Method, Routine and Package |
| D14 | strings are byte strings; character semantics, if added, arrive as operations | `'FFFE'x` is a legal value, so a UTF-8 type cannot hold Rexx strings |
| D15 | a value's identity is its bytes or its number, and a number's rendering is fixed under the `NUMERIC DIGITS` and `FORM` in force when it is created | ooRexx makes the difference observable: `x = 1e10 + 0` renders `1E+10` under `DIGITS 9` and still does under `DIGITS 20` |
| D-U1 | `unsafe` only in granted modules, listed in `interrexx-core/tests/unsafe_sites.rs`; `libloading` for `dlopen` | the workspace denies `unsafe`; each grant is one module behind a safe interface |
| D-U2 | the interpreter island is `Send` only inside `island.rs`'s baton wrapper | moving a driver between threads; `Rc` and `Cell` are touched by the baton holder only |
| D-U3 | signal handlers in `signal.rs` only set an atomic and write to a self-pipe | async-signal safety |
| D-U4 | one frame arena per activity | frames stay LIFO within an activity while activities interleave |
