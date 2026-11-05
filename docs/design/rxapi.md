# The API service

External data queues, the macrospace, and the function, subcommand and exit registries live in a
per-user service process, `rxapi`, written in Rust (`rust/crates/interrexx-rxapi`). It does not speak
ooRexx's protocol, and an interrexx client never talks to ooRexx's daemon or the reverse.

Return codes below are the `rexxapidefs.h` values. Names of queues, macros, registrations and
libraries compare caselessly (ASCII folding) and keep the spelling first supplied.

## Queues

* Named queues live until deleted or until the service ends. Items are byte strings of any length,
  NUL bytes and empty items included. FIFO adds at the tail, LIFO at the head, pulls take the head.
* A valid name is 1 to 250 bytes of ASCII letters, digits and `.?!_`; anything else is 5.
  `SESSION`, in any case, names the session queue.
* Create with a name in use makes a queue under a generated name (`S...`, letters and digits, at
  least 10 characters) and reports the duplicate; a generated name too long for the buffer is
  truncated and the call returns 1.
* Flags are checked before names: a bad add flag is 6, a bad wait flag is 7.
* A waiting pull has no timeout. Each add satisfies one waiter, with no fairness promise. Deleting
  a queue with a waiter returns 10. A blocked pull ends when its connection closes, and the queue
  stays usable.
* Pull results go into the caller's buffer only when the item is strictly shorter than it;
  otherwise a NUL-terminated block is allocated for `RexxFreeMemory`. Timestamps use `struct tm`
  conventions: month 0 to 11, year since 1900.

### The session queue

Each client has one, reached as `SESSION`. Its handle is inherited through `RXQUEUESESSION`
(hex, with or without `0x`); a client without one uses its own process id and sets the variable,
so children and `cmd | rxqueue` share the parent's queue. The interpreter connects and attaches to
its session queue before its first command or child process and keeps that connection for its life.
A session queue is removed when no connection uses it, whatever it holds, and session queues do
not keep the service alive. When the service cannot be reached and the session queue is empty, a
command still runs with the queue kept in process; if it holds lines, the command raises 48.1 with
`RXAPI_NORXAPI`.

### Rexx surface

`RXQUEUE()` dispatches on the option's first letter (`C D E G O S`). `.RexxQueue` keeps the
uppercased name; `pull` does not wait and `lineIn` does, both answering `.nil` on failure. `push`,
`queue`, `lineOut` and `say` raise 48.1 with `SYSTEM QUEUE (<code name>)` on failure. `PULL` on an
empty queue reads standard input.

## Registrations

Functions, subcommand handlers and exits each have an independent registry.

* A **library registration** (name, library, procedure, optional 16 bytes of user data, drop
  authority) lives in the service, is visible to every client of the user, and outlives the
  process that made it until dropped. The pair (name, library) identifies it; registering a pair
  again returns 10 and adds a reference.
* An **entry-point registration** (`...Exe`) is a function pointer private to the registering
  process. It never reaches the service.
* Names are not validated and are truncated to 255 bytes.
* Drop authority: owner-only registrations can be dropped only by the registering process (40
  otherwise); each registration adds a reference and each drop removes one; the last drop removes
  the registration. A drop with a library that leaves references returns 10, a name-only one 0.
  When several libraries hold a name, a name-only operation acts on the most recently touched.
* Function registrations are always owner-only and have no library qualifier.
* Libraries load in the client, never in the service. A failed load or symbol lookup returns 30.
* `RxFuncAdd` registers before it resolves and does not undo a failed resolve. `RxFuncQuery`
  answers 0 for a registered or loaded name. `RxFuncDrop` from a process other than the
  registrant answers 1.

## Macrospace

A per-user table of compiled routines, each with a name and a search position: 1 (before builtins
and files) or 2 (after). Any other stored position is never found by a call. `RexxAddMacro`
translates the file in the calling process and returns 7 for anything it cannot read or compile.
Save and load use the caller's file system and current directory. The saved file layout is
interrexx's own (a 16-byte text, a signature, a count, then per macro its name, position and image,
length-prefixed and little-endian); a file written by ooRexx is refused as invalid. A listed save
past 256 MiB of images returns 1. Load with a name list loads only those names.

## The service process

* **Start.** The first client that needs the service starts it detached, in its own session, with
  inherited descriptors closed. A lock file decides who owns the socket, so concurrent starters
  converge on one service and a crash leaves nothing that blocks the next start. A client gives up
  after 5 seconds, and every call then returns 1004.
* **Directory.** The first of `REXX_API_DIR`, `$XDG_RUNTIME_DIR/interrexx`, and a `0700` directory
  under the temp directory whose socket path fits. A set `REXX_API_DIR` that is passed over is
  reported on standard error.
* **Versions.** Each protocol version has its own socket (`rxapi.<version>.sock`) and lock file, so
  an upgraded client starts a new service and the old one keeps serving old clients.
* **Exit.** The service exits when no client is connected and it holds no named queue,
  registration or macro. `rxapi --stop` is the explicit stop, used by tests; with nothing running
  it exits 0 silently. `RexxShutDownAPI` does nothing and returns 1.
* **Death.** A closed connection is the only end signal, for a clean exit and a crash alike. If the
  service dies, its state is lost, connected clients get 1004, and the next new process starts a
  fresh service.
* **Identity.** Registrations and blocked pulls belong to a connection, so process id reuse cannot
  reach them.
* `rxapi` prints the lock and service lines `rxapi.testGroup` expects, and exits 13 when another
  instance holds the lock.

## Wire protocol

A Unix stream socket. A connection opens with `hello` and `welcome`, carrying the protocol version.
After that each direction carries frames: an 8-byte length, an 8-byte call number, and a body
encoding one request or one reply, little-endian and length-prefixed. The vocabulary is the
`Request` and `Reply` enums in `interrexx-rxapi/src/message.rs`.

The core is a state machine (`interrexx-rxapi/src/core`) that takes one request and answers one reply,
with no sockets, threads or clocks. A request that must wait answers "parked", and its reply is sent
when a later request satisfies it. The daemon is a thread per client around one owner of the core.
A direct in-process transport serves tests, so each test owns a fresh core.

## rxqueue and rxsubcom

`rxqueue [queue] [/FIFO | /LIFO | /CLEAR]` copies standard input to an existing queue, a line per
item. `-` is a synonym for `/`; one option at most. Lines end at `\n`, `\r` or `\r\n`; a line past
65472 bytes is truncated; byte 0x1A ends input. The queue defaults to `RXQUEUE`, then `SESSION`.
Messages go to standard output and the exit status is the API code modulo 256; a syntax error
exits 0. Without `RXQUEUESESSION`, a session-queue target is refused with exit 9 (a Deviation:
ooRexx accepts lines into a queue nothing can reach).

`rxsubcom REGISTER|QUERY|DROP|LOAD envname [library [procname]]` maps to the subcommand registry,
always registering as droppable. The exit status is the API code; usage errors exit 255.
