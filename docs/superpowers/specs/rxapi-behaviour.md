# rxapi behaviour

What a replacement for the ooRexx API service must do, as seen from outside. It covers the external
data queues, the macrospace, the function, subcommand and exit registries, the service process, and
the `rxqueue` and `rxsubcom` commands. It is the clean-room description named in
`docs/superpowers/specs/2026-11-02-phase-10-design.md`.

Each statement describes the oracle's observed behaviour unless marked otherwise. Results come from
running the oracle, the ooTest groups, and `oodocs`. Where the oracle and its documentation disagree,
the text gives **Observed:** and then **Documented:**. Where the Phase 10 design deliberately departs
from the oracle, the text says **Design:** and the design wins. The final section lists the questions
still open.

Public declarations are in `api/rexx.h`, `api/rexxapidefs.h` and `api/platform/unix/`. The application
programming book is not in `oodocs`, so for most of the C API the headers and this document are the
only sources.

## Terms

* **Client:** any process using the API, including `rexx`, `rxqueue`, `rxsubcom` and embedders.
* **Service:** the per-user daemon. There is one per user. All clients of a user share it.
* **Name comparison:** queue, macro, registration and library names compare caselessly, using ASCII
  case folding. Names are stored and returned in the spelling first supplied.
* **Return codes** below are the numeric values in `rexxapidefs.h`. Symbolic and numeric forms are both
  given where a test depends on the number.

## Conventions common to all operations

* Each API call completes before it returns. When `rxqueue` exits, its lines are already visible to
  the parent, and ooTest relies on this without any delay.
* Operations on one queue are atomic. No caller sees a partial add or a partial pull.
* Calls from different threads of one client are independent. A thread blocked in a waiting pull must
  not delay another thread's calls, including calls on the same queue. Both `test_delete_while_linein`
  and `test_linein_waits_pull_doesnt` depend on this.
* The first API call of any kind in a process connects to the service, starting it if necessary, and
  establishes the session queue.
* If the service cannot be reached, every operation returns `RXAPI_NORXAPI` (1004). See "Failure of
  the service".
* An allocation failure reports `RXAPI_MEMFAIL` (1002). In the macrospace family the same condition
  reports `RXMACRO_NO_STORAGE` (1).
* Name pointers are required unless a section below says otherwise. A NULL library or procedure name
  in a registration crashes the oracle's client. The replacement may return an error instead.

## External data queues

### Kinds of queue

There are named queues and session queues. A named queue exists from creation until deleted or until
the service ends. Session queues are described under "The session queue".

Queues hold byte strings. Bytes have no meaning, embedded NUL is preserved, and an empty string is a
valid item. There is no limit on item size at the API: items of 65473 bytes, 1 000 000 bytes and
50 000 000 bytes were added and pulled intact. `RXQUEUE_SIZE` is never returned by the API.

### Name validation

A valid queue name is 1 to 250 bytes (`MAX_QUEUE_NAME_LENGTH`) of ASCII letters, ASCII digits, and
the characters `.`, `?`, `!` and `_`. Anything else, including the empty string, is `RXQUEUE_BADQNAME`
(5).

`SESSION`, in any case, is reserved and means the session queue. How each operation treats it is in
the table. NULL as a name means the session queue for the operations marked below.

| Operation | `SESSION` | NULL name |
|---|---|---|
| `RexxCreateQueue` | 5 | generate a name |
| `RexxOpenQueue` | 0, `*pdup` = 1, no effect | acts on a hidden queue with an empty name |
| `RexxQueueExists` | 0 | acts on the same hidden queue |
| `RexxDeleteQueue` | 5 | acts on the same hidden queue |
| `RexxClearQueue` | clears session queue | session queue |
| `RexxQueryQueue` | session queue | session queue |
| `RexxAddQueue` | session queue | session queue |
| `RexxPullFromQueue`, `RexxPullQueue` | session queue | session queue |

The hidden empty-named queue is an accident of the oracle: `RexxOpenQueue(NULL)` reports `*pdup` = 0
the first time, `RexxQueueExists(NULL)` then answers 0, and `RexxDeleteQueue(NULL)` removes it and
answers 0. No test depends on this.

### RexxCreateQueue(name, size, requested, pdup)

* `requested` given and `strlen(requested) >= size`: returns `RXQUEUE_STORAGE` (1). This check comes
  first, so it wins over name validation. `name` and `*pdup` are untouched and nothing is created.
* `requested` invalid or `SESSION`: returns 5. `name` and `*pdup` are untouched.
* `requested` given and no such queue: the queue is created under that spelling, `name` receives
  `requested`, `*pdup` = 0, returns 0.
* `requested` given and the queue exists: a new queue is created under a generated name, `name`
  receives the generated name, `*pdup` = 1, returns 0.
* `requested` NULL: a new queue is created under a generated name, `*pdup` = 0, returns 0.
* A generated name that does not fit in `size` is truncated to `size - 1` bytes plus a NUL, and the
  call returns 1. The queue has still been created, and the caller does not learn its full name.

**Generated names.** They are valid queue names, unique among existing queues, and never equal to a
requested name that is in use. The oracle's are `S`, the session in lowercase hex, `Q`, and a unique
lowercase hex tag, for example `S1f49beQ7f7ee4000d30`. The tests require:

* the name starts with `S` and is at least 8 characters;
* it contains only letters, digits, and characters valid in both of `?!._` and `?!-_`, so letters
  and digits only is the safe choice;
* it does not fit in a 10-byte buffer, because `CLASSIC.testGroup` expects `RXQUEUE_STORAGE` from a
  generated create with `size` 10, so it must be at least 10 characters.

**Observed:** an existing name produces a generated-name queue and return code 0.
**Documented:** the concept chapter says `RXQUEUE("Create", "ANYQUE")` on an existing queue "results
in an error", while the `RXQUEUE` function section describes the generated name. The tests follow the
function section.

**Observed:** the name returned keeps the requested case, so `.RexxQueue~create("myqueue")` returns
`myqueue`. **Documented:** the example shows `MYQUEUE`, and the `RXQUEUE` section says the actual name
is the uppercase value. `test_rxqueue_create_two_args` asserts the original case.

### RexxOpenQueue(name, pdup)

Creates the queue if absent. Returns 0 with `*pdup` = 0 if created, 1 if it existed. An invalid name
returns 5 and `*pdup` is untouched.

### RexxQueueExists(name)

Returns 0 if the queue exists, `RXQUEUE_NOTREG` (9) if not, 5 if the name is invalid.

### RexxDeleteQueue(name)

Deletes the queue and any items, returning 0. Other results, checked in this order:

1. Invalid name or `SESSION`: 5.
2. No such queue: 9.
3. A pull is waiting on the queue: `RXQUEUE_ACCESS` (10). The queue is unchanged.

**Documented:** the `RXQUEUE` function also lists 12 and 1002. 12 is never returned.

### RexxClearQueue(name)

Removes all items and returns 0, whether or not there were any. A waiting pull is unaffected and
keeps waiting. Returns 5 or 9 as above for named queues.

### RexxQueryQueue(name, count)

Sets `*count` to the number of items and returns 0.

* Missing queue: returns 9 and `*count` = 0.
* Invalid name: returns 5 and `*count` is untouched.

### RexxAddQueue(name, data, flag)

The flag is checked first. A flag other than `RXQUEUE_FIFO` (0) or `RXQUEUE_LIFO` (1) returns
`RXQUEUE_PRIORITY` (6) even for an invalid or missing queue. Then invalid name gives 5 and a missing
queue gives 9.

FIFO places the item at the tail and LIFO at the head. Pulls always take the head. So adding `a b` FIFO,
then `c d` LIFO, then `e` FIFO, pulls as `d c a b e`.

The item is timestamped when it is added.

### RexxPullFromQueue(name, data, timestamp, waitflag)

The flag is checked first. A flag other than `RXQUEUE_NOWAIT` (0) or `RXQUEUE_WAIT` (1) returns
`RXQUEUE_BADWAITFLAG` (7), even for a bad name. Comments in `rexxapidefs.h` have these two swapped.
The numeric values above are the working ones. Then invalid name gives 5 and a missing queue gives 9.

With an item available, the head is removed and the call returns 0. With the queue empty:

* `RXQUEUE_NOWAIT`: returns `RXQUEUE_EMPTY` (8).
* `RXQUEUE_WAIT`: blocks until an item arrives. See "Blocking pulls".

On any error, `data` and `timestamp` are unchanged.

**Result buffer.** On entry `data` is either empty, or describes a caller buffer. Both `strptr` and
`strlength` must be initialised: a stale pointer with a non-zero length is written through.

* If `strptr` is non-NULL and the item is strictly shorter than `strlength`, the bytes are copied into
  the caller's buffer, `strptr` is unchanged, `strlength` becomes the item length, and no terminator
  is written. An item of exactly `strlength` bytes does not qualify.
* Otherwise a new block is allocated, usable with `RexxFreeMemory`, and `strptr` is replaced. The
  caller's old buffer is neither written nor freed. The block has one extra byte holding a NUL.
* With no caller buffer, an empty item still returns a non-NULL `strptr` with `strlength` 0, so
  callers can tell an empty line from no data.
* With a caller buffer, an empty item leaves the buffer unchanged and sets `strlength` to 0.

**Timestamp.** `timestamp` may be NULL. If given it holds the service's local time when the item was
added. The values follow the C `struct tm` convention rather than calendar convention:

| Field | Value | Example on 2 November 2026, a Monday |
|---|---|---|
| `year` | years since 1900 | 126 |
| `month` | 0 to 11 | 10 |
| `day` | 1 to 31 | 2 |
| `weekday` | 0 is Sunday | 1 |
| `yearday` | 0-based | 305 |
| `hours`, `minutes`, `seconds` | as usual | |
| `hundredths`, `microseconds` | always 0 | 0 |

The test binaries read the timestamp without asserting these field values. The header's comments
describe `month` as "month of the year" and `year` as "current year", which reads as calendar values.

### RexxPullQueue(name, data, datetime, waitflag)

The older form. It behaves as `RexxPullFromQueue`, then fills `datetime` (a `REXXDATETIME`) with the
same fields and sets `valid` to 1. On a failed pull the oracle still copies indeterminate fields into
`datetime`. A NULL `datetime` is allowed.

### Blocking pulls

* A pull on an empty queue with `RXQUEUE_WAIT` waits with no timeout.
* An add satisfies one waiter. Items go to waiters without any fairness guarantee: with three waiters
  started in order 1, 2, 3 and three adds, delivery was 2, 1, 3 and 2, 3, 1 across runs.
  Every item goes to exactly one waiter, and no waiter returns without an item while the queue lives.
  A LIFO add wakes a waiter just as a FIFO add does.
* An item already in the queue is returned at once even if the flag is `RXQUEUE_WAIT`.
* While any pull waits, deleting the queue returns 10. Delete succeeds again once the waiter has its
  item. `test_delete_while_linein` depends on both.
* Waiting works identically on the session queue. A child may satisfy a parent's wait.
* Signals: `SIGTERM` delivered to a `rexx` blocked in `lineIn` ends the wait, and the HALT condition
  is raised. `SIGINT` was not exercised.

**Observed, on a waiter that disappears:** when a client blocked in a pull is killed or interrupted,
the service does not notice. Delete on that queue returns 10 from then on. The next add appears to
succeed, then the service itself dies with all state lost, and the next client starts a fresh one.
**Design:** blocked pulls end when their connection closes. The waiter is simply forgotten, the
queue stays deletable, and the next item stays in the queue.

### The session queue

* Every client has a session queue, always reachable as `SESSION`. It always "exists": `RexxQueueExists`
  returns 0 and `RexxOpenQueue` returns 0 with `*pdup` = 1, and it cannot be deleted through
  `RexxDeleteQueue`.
* A session queue is identified by a handle. A client that inherits none takes its own process id as
  the handle.
* During initialisation the client sets `RXQUEUESESSION` in its own environment to the handle, written
  as lowercase hex with an `0x` prefix, for example `0x1f4a5f`. It overwrites any previous value.
* A client that finds `RXQUEUESESSION` already set uses that handle. So a child, grandchild, or any
  process given the variable shares the queue, and each sees the others' items immediately.
  `rexx` calling `rexx` calling `rxqueue` all see one queue, and this is what makes `cmd | rxqueue`
  deliver to the calling program.
* The value is parsed as hex, with or without `0x`. `1234` and `0x1234` reach the same queue. `0`,
  `0x0` and `(nil)` all reach handle 0.
* An unparseable value, such as `garbage` or the empty string, leaves the oracle with an arbitrary
  handle. Treat it as no value and take the process's own handle. No test depends on this.
* A handle the service has never seen gives an empty queue, created on demand. Every session operation
  behaves this way, so a session operation cannot return 9.
* A process's handle stays fixed for its lifetime.
* Interpreter termination closes connections but leaves the queue alone. `RexxCreateSessionQueue`
  (returns 0) and `RexxDeleteSessionQueue` (returns 0) are the internal start and stop hooks, are
  repeatable, and do not touch queue content. A later call in the same process reconnects and finds
  the same queue with its items.

**Observed:** nothing removes a session queue. It outlives its creator and every inheritor, holds its
items until the service ends, and any later process given the handle finds them. A later process
whose id happens to match an old handle would inherit the leftover queue; this was not provoked.
**Documented:** the queue is created by the top-level program, is available to children while that
process runs, and "is deleted when the process that created it ends".
**Design:** superseded by Resolution 4: a session queue goes when its last connection closes, so a
crashed process's session queue goes with its connection.

The inheritance tests need only the sharing. No test reads the value of `RXQUEUESESSION`, so the
format is free apart from these rules: it is set by the client at initialisation, a child inherits it
through the environment, and a child must not be given a different queue.

### Rexx surface

The interpreter layers these on the API.

* `RXQUEUE` function, by first letter of the option in any case:

  | Option | Effect |
  |---|---|
  | `C` | creates, returning the name; `RXQUEUE("Create","SESSION")` returns the null string |
  | `D` | returns the `RexxDeleteQueue` code |
  | `E` | returns `.true` if the queue exists |
  | `G` | returns the current queue name, `SESSION` by default |
  | `O` | returns the `RexxOpenQueue` code |
  | `S` | sets the current queue and returns the previous name, uppercased |

  Any other first letter is error 40.904, and a second argument that is not a valid symbol is 40.26.
  Set does not check existence. `Queued()` on a missing queue answers 0, and `QUEUE`, `PUSH` and
  `LINEOUT("QUEUE:", ...)` then raise 48.1 with `SYSTEM QUEUE (RXQUEUE_NOTREG)`.
* The `RXQUEUE` environment variable does not affect the interpreter. It affects only the `rxqueue`
  command.
* `.RexxQueue` keeps the uppercased name. `new` with no argument uses `SESSION`, with `.nil` it creates a
  generated-name queue, and otherwise it opens the queue. `new` with an invalid name succeeds and
  later operations fail.
* `pull` uses `RXQUEUE_NOWAIT` and returns `.nil` on any failure. `lineIn` uses `RXQUEUE_WAIT`, releases
  the object's guard while waiting, and returns `.nil` on any failure.
* `push`, `queue`, `lineOut` and `say` return 0 and on failure raise 48.1 with
  `SYSTEM QUEUE (<code name>)`, or `SYSTEM QUEUE (reason code <n>)` for codes without a name. The named
  codes are `RXAPI_NORXAPI`, `RXAPI_MEMFAIL`, `RXQUEUE_BADQNAME`, `RXQUEUE_PRIORITY`,
  `RXQUEUE_BADWAITFLAG`, `RXQUEUE_EMPTY`, `RXQUEUE_NOTREG` and `RXQUEUE_ACCESS`.
* `queued` ignores failures and returns 0 for a deleted queue. `empty` returns the
  `RexxClearQueue` code. `delete` returns the `RexxDeleteQueue` code. `create` with an invalid name or
  `SESSION` returns the null string.
* `PULL` and `PARSE PULL` on an empty queue read from standard input. `LINEIN("QUEUE:")` waits.
* After the service dies, `queued` keeps answering 0 and `QUEUE` raises 48.1 with
  `SYSTEM QUEUE (RXAPI_NORXAPI)`. `rexx` itself survives.

## Registrations

Three independent registries exist: functions, subcommand handlers and exits. The same name may
appear in each without interaction. All three follow the rules here. Subcommand and exit calls are
parallel, with `Subcom` and `Exit` in their names. Functions differ in the ways noted under
"Function registrations".

### Two kinds of registration

A **library registration** names a library, a procedure, optional user data and a drop authority.
It lives in the service and is visible to every client of the user.

An **entry-point registration** (`...Exe`) records a function pointer. It is private to the
registering process, is invisible to every other process, and disappears with that process. The
service plays no part in it.

### Lifetime

A library registration survives the end of the process that made it, and remains until dropped or
until the service ends. `RxFuncAdd` in one run followed by `RxFuncQuery` in a later run answers 0. This
is a decision of the design as well as the oracle's behaviour.

### Names

Names are not validated. Any string works, including the empty string. Library, procedure and
registration names are silently truncated to 255 bytes: a name of 300 characters registers, and its
first 255 characters then find it. Names compare caselessly, including library names, so `orxclassic1`
matches `ORXCLASSIC1`. On a case-sensitive file system loading still uses the registered spelling.

Procedure names are not compared.

### Registering a library handler

`RexxRegisterSubcomDll(name, library, procedure, userArea, dropAuthority)` and
`RexxRegisterExitDll` likewise.

* The library and procedure are not checked. Registration succeeds for a library that does not exist.
* The pair (name, library) identifies the registration.
* New pair, even with the name already registered under a different library: returns 0. Both exist.
* Pair already registered: returns `RXSUBCOM_DUP` (10), and the existing registration is unchanged
  apart from gaining a reference. Its procedure, user data, authority and owner stay as they were.

`dropAuthority` of `RXSUBCOM_NONDROP` (1) means owner only, and any other value means anyone may drop.
`userArea`, if non-NULL, supplies 16 bytes, stored and handed back to queries.

**Documented:** `rxsubcom` REGISTER says 10 means an existing handler in a different library and 30
means the same library. **Observed:** a different library gives 0 and the same library gives 10.
The test `test_register_twice` asserts 10 for the same library.

### Registering an entry point

`RexxRegisterSubcomExe(name, entryPoint, userArea)` and `RexxRegisterExitExe` likewise. Returns 0, or
10 if the process already has that name, caselessly, in which case the first entry point stays.
A library registration of the same name does not make it a duplicate. The two coexist.

### Query

`RexxQuerySubcom(name, library, flags, userWord)` and `RexxQueryExit` likewise.

* `*flags` is set to 0 first, then to 1 on success. So it is valid even when the call fails.
* `library` NULL or empty: the process's entry-point registration of that name if there is one, else a
  library registration of that name. With `library` empty the entry-point check is skipped.
* `library` given: only the (name, library) library registration.
* Returns 0 if found and `RXSUBCOM_NOTREG` (30) if not.
* `userWord`, if non-NULL, receives the stored 16 bytes (zeros if none) on success, and is untouched
  on failure.

### Drop

`RexxDeregisterSubcom(name, library)` and `RexxDeregisterExit` likewise.

With `library` given and non-empty, only the (name, library) library registration is considered. With
it NULL, the process's own entry-point registration of that name is removed if it exists, else a
library registration of that name is considered. With it empty, the entry-point step is skipped.

When several libraries hold the name, a name-only drop, query or load acts on the most recently
touched of them. A registration is touched when it is created and when a register, query or drop
names it with its library. A name-only operation does not touch it. `test_register_twice`'s sequence
is unaffected by this because only one library is involved.

Authorization and references for a library registration:

* An owner-only registration dropped by any process other than the owner returns
  `RXSUBCOM_NOCANDROP` (40) and nothing changes. The owner is the process that first registered the
  pair, identified by process id only. Once that process has ended, an owner-only registration cannot
  be dropped by anyone.
* Each registration or duplicate registration of a pair adds a reference, and one drop removes one.
  A reference made with droppable authority goes in one pool shared by everyone. A reference made
  with owner-only authority belongs to the calling process. A drop removes from the pool if the
  registration's first authority was droppable, and otherwise from the caller's own references.
* A drop that leaves references remaining keeps the registration. The return code differs by form:
  name-only drop returns 0, drop with a library returns 10.
* The drop that removes the last reference removes the registration and returns 0.
* A drop that finds no reference to remove changes nothing and still returns 0 for a name-only drop.
  Thus an owner whose reference is gone, while another process's reference remains, can repeat the
  drop forever without removing the registration. The same happens to a droppable registration that
  gained an owner-only duplicate.
* The authority stays as first registered.
* No such registration: 30.

So after register, register, the sequence "drop by name, drop with library, drop by name" gives 0, 0,
30, which is exactly `test_register_twice`.

**Documented:** return code 40 means another process registered it as `RXSUBCOM_NONDROP`, and the
documentation does not describe references.

### Load and resolve

`RexxLoadSubcom(name, library)` is internal but exported. It is the action of `rxsubcom LOAD`. The
`library` argument is ignored: `rxsubcom load one wrong-lib` succeeds if `one` loads. It finds the
registration by name alone and loads it.

`RexxResolveSubcom(name, entry)`, `RexxResolveExit(name, entry)` and `RexxResolveRoutine(name, entry)`
are internal but exported. They set `*entry` to NULL, then look up as a name-only query would, and:

* Entry-point registration: `*entry` is the pointer, returns 0.
* Library registration: load the library in the calling process and find the procedure, first as
  registered, then uppercased. Failure at either step returns 30 with `*entry` NULL.
* Registered with an empty library: returns 0 with `*entry` NULL.
* Not registered: 30.

**Documented:** `rxsubcom` LOAD lists 50 for a library that fails to load, and the header lists
`RXSUBCOM_LOADERR` (50) and `RXSUBCOM_NOPROC` (127). **Observed:** 30 for both, and
`test_register_invalid_library` and `test_register_invalid_handler` assert 30. `RXSUBCOM_MAXREG` (20) and
`RXSUBCOM_BADTYPE` (1003) and `RXSUBCOM_BADENTRY` (1001) are never returned.

Library loading happens in the client. The service never loads anything.

### Function registrations

`RexxRegisterFunctionDll(name, library, procedure)`, `RexxRegisterFunctionExe(name, entryPoint)`,
`RexxDeregisterFunction(name)`, `RexxQueryFunction(name)`.

These follow the rules above with these differences:

* A library function registration is always owner-only. `RexxDeregisterFunction` from any process
  other than the registrant returns 40.
* There is no library qualifier, no user data, and no authority argument.
* The `RXFUNC_*` codes share values with the above: 0, 10 for `RXFUNC_DEFINED`, 30 for `RXFUNC_NOTREG`.
  `RexxDeregisterFunction` can also return 40, which is `RXFUNC_MODNOTFND` by value and means not
  authorized here.

The Rexx functions:

* `RxFuncAdd(name, module, [proc])`, with `proc` defaulting to `name`: uppercases `name`. Returns 0 if
  the function is already callable or can now be resolved, and 1 otherwise. Registration happens
  first and is not undone by a failed resolve, so after a failed `RxFuncAdd` of `NOFN`, `RxFuncQuery`
  answers 0 and `RxFuncDrop` answers 0. A later `RxFuncAdd` of a name already registered is a
  duplicate, and the failure of that step is ignored.
* `RxFuncQuery(name)`: uppercases `name`. Returns 0 if the function is loaded in this process or
  registered, 1 otherwise.
* `RxFuncDrop(name)`: uppercases `name`, forgets any cached resolution in this process, and returns 0
  if deregistered, 1 for any non-zero code. Dropping from a later process than the registrant answers
  1 and leaves the registration, and `RxFuncQuery` still answers 0.
* Functions from a loaded library package, such as `SysSleep`, are not in the registry. `RxFuncQuery`
  answers 0 and `RxFuncDrop` answers 1 for them.
* **Documented:** `RxFuncAdd` returns 1 when the function could not be resolved. The fact that it stays
  registered is not documented.

`CLASSIC.testGroup` relies on these: add, query, call, drop in sequence with results 0, and the
`Exe` registration queried and called in-process, and 30 after deregistering.

### Effect on running programs

A dropped subcommand handler stays usable in a process that has already resolved it.
`test_drop_query_simple` records this as a comment and does not assert it. A function dropped by any
route other than `RxFuncDrop` in the same process likewise stays callable there, and `RxFuncQuery`
answers 0, as `test_52_externalfunction` notes. Registering `COMMAND` or
`SYSTEM` succeeds, but `ADDRESS` still reaches the built-in environment
(`test_register_builtin_environment`).

## Macrospace

The macrospace is a single, per-user table of compiled Rexx routines held by the service. Each entry
has a name, an image and a search position. Entries are visible to all clients and survive their
creators until dropped, cleared, or until the service ends.

### Names and positions

Macro names are not validated. The empty string works. They are truncated to 255 bytes and compare
caselessly.

The position is `RXMACRO_SEARCH_BEFORE` (1) or `RXMACRO_SEARCH_AFTER` (2). The API stores any value
unchanged. `RXMACRO_INVALID_POSITION` (8) is never returned, so 0, 3 and 99 are accepted and read
back. A macro whose position is neither 1 nor 2 can be queried but is never found by a call.

`RexxQueryMacro`'s position is `unsigned short`, so values above 65535 read back truncated.

At Rexx level a before-macro takes precedence over built-in external routines and files, and an
after-macro is searched after them. So a macro named `SysUtilVersion` at position 1 runs in place of
the library routine, and at position 2 it does not (`test_add_three_args_option_before` and `_after`).
A macro name is also searched for by `::REQUIRES`, before the file at position 1 and after it
otherwise.

### Operations

**RexxAddMacro(name, file, position).** Reads the file and translates it in the calling process, so a
relative path uses the caller's current directory. A file holding an already-compiled program
(`rexxc` output) is accepted. An unreadable file, a directory, an empty file, an empty path and a file with
a syntax error all return `RXMACRO_SOURCE_NOT_FOUND` (7), and no message is printed. An existing
macro of that name is replaced, including its position, and the call returns 0.

**RexxDropMacro(name).** Returns 0, or `RXMACRO_NOT_FOUND` (2).

**RexxClearMacroSpace().** Returns 0, also when empty.

**RexxQueryMacro(name, pos).** Sets `*pos` and returns 0. Returns 2 and `*pos` = 0 if absent.

**RexxReorderMacro(name, position).** Stores the position and returns 0. Returns 2 if absent.

**RexxResolveMacroFunction(name, data).** Internal but exported. Returns the image, with the same
result-buffer rules as a pull. Returns 2 if absent, with `data` unchanged.

**RexxSaveMacroSpace(count, names, file).**

* `names` NULL: saves everything. An empty macrospace returns `RXMACRO_FILE_ERROR` (5) and creates no
  file (`test_save_empty`).
* `names` given: saves those, in the order listed. `count` 0 with non-NULL `names` writes a valid
  file holding no macros and returns 0.
* A listed name that is absent returns 2, and the partial file is removed.
* A file that cannot be created (empty name, a directory, a missing parent) returns 5. An existing file
  is overwritten (`test_save`). Any failure part-way removes the partial file.
* The file goes through the caller's file system with the caller's umask, and a relative path uses
  the caller's current directory. The macrospace is unchanged.

**RexxLoadMacroSpace(count, names, file).**

* `names` NULL: loads everything. Existing macros not in the file are kept and same-named ones are
  replaced (`test_load`). Each macro keeps its saved position. `RXMACRO_ALREADY_EXISTS` (4) is never
  returned.
* A file that cannot be opened returns 5, and so does a file too short to hold a header or any
  file that ends before all it promises. A file with a valid header but the wrong signature returns
  `RXMACRO_SIGNATURE_ERROR` (6). A Rexx source file long enough to fill a header returns 6
  (`test_load_file_invalid`), while a shorter one returns 5.
* Load is not atomic. A failure reading a macro's image after earlier macros loaded leaves those loaded.
  A failure at the header or before the first image leaves nothing changed.
* `names` given: **Documented intent**, shown by `test_74_macrospace`: load only those named, compared
  caselessly. A name not in the file is silently ignored and the call returns 0.
  **Observed:** the listed macros load, but each unlisted entry that precedes the first listed entry
  in the file adds a macro with an empty name and an empty image, which then makes a later
  save-everything fail with 5. An unlisted entry after a listed one re-adds that listed macro, with
  no visible effect. The replacement should load only the listed names.

`RXMACRO_EXTENSION_REQUIRED` (3) is never returned.

### Saved file

Observable on disk, little-endian on this platform.

| Offset | Size | Content |
|---|---|---|
| 0 | 16 | the ASCII text `REXX-ooRexx 6.00`, no terminator |
| 16 | 8 | signature, 0xDDD5 |
| 24 | 8 | macro count |
| 32 | 296 × count | one descriptor per macro |
| after | sum of sizes | images, concatenated in descriptor order, with no separators |

Each descriptor holds, in order: 8 reserved bytes of zero, the NUL-terminated name in a 256-byte
field, the image size in 8 bytes, 8 bytes of zero, the image size again in 8 bytes, and the position
in 8 bytes. The oracle's name field can contain leftover bytes after the NUL, so files are not
reproducible and a reader must stop at the NUL.

A mismatch in either the text or the signature returns 6. The order of entries in a save-everything
file is unspecified, and nothing depends on it. The image is whatever `rexxc`-style compiled form
the interpreter produces, so a file is only useful to the interpreter that wrote it. See the open
questions.

### Rexx surface

| Function | Result |
|---|---|
| `SysAddRexxMacro(name, file, [order])` | the `RexxAddMacro` code; order `A` or `B` in any case by first letter, default `B`; anything else raises 40.920 |
| `SysReorderRexxMacro(name, order)` | the `RexxReorderMacro` code; order as above and required |
| `SysDropRexxMacro(name)` | the `RexxDropMacro` code |
| `SysQueryRexxMacro(name)` | `B` for position 1, `A` for any other, the null string if absent |
| `SysClearRexxMacroSpace()` | the `RexxClearMacroSpace` code |
| `SysSaveRexxMacroSpace(file)` | the `RexxSaveMacroSpace` code, saving everything |
| `SysLoadRexxMacroSpace(file)` | the `RexxLoadMacroSpace` code, loading everything |

`Macrospace.testGroup` also requires that a macro added by one interpreter is callable by another
interpreter process (`test_add_other_interpreter`) and that 100 macros save and reload.

## Memory

`RexxAllocateMemory(size)` returns a block, or NULL. `RexxFreeMemory(block)` releases one and always
returns 0. Blocks returned by pull and by macro resolve are released through `RexxFreeMemory`.

## The service

Everything here is subject to the design's daemon lifetime rules, which replace the oracle's where they
differ. This section records what clients can observe.

### Start

The first API use in a process connects, and if nothing answers, starts the service detached, with its
own session, closed descriptors, and root as the current directory. The oracle looks for `rxapi` in
`../bin` beside the API library, then on `PATH`, then in the current directory. If no service answers
within about two seconds, the first call returns 1004. Later calls in that process fail at once with
1004.

Concurrent starters converge on one service.

A newly started service holds no queues, macros or registrations. Nothing is persisted.

### Stop

**Observed:** the oracle's service never exits on its own, even with nothing held. It stops on
`SIGTERM`, exit status 1, removing its socket and leaving its empty lock file.

`RexxShutDownAPI` does nothing and returns 1, in every state. **Design:** `RexxShutDownAPI` is the
explicit stop. The return value is in the open questions.

**Design:** the service exits when no client is connected and it holds no queue, registration or macro.

### Failure of the service

Service death loses every queue, macro and library registration.

* A pull blocked at that moment returns 1004.
* Later calls in a client already connected return 1004, and the service is not restarted within
  that process. The oracle's client process is killed by `SIGPIPE` on the next call unless it ignores
  that signal, which is a defect rather than a requirement. `rexx` survives.
* A new process starts a new service.
* A stale socket left by `SIGKILL` does not obstruct the next start.

### The rxapi command

`rxapi` is the executable name in the oracle, found beside `rexx`. `rxapi.testGroup` runs it from the
directory holding `.RexxInfo~executable` while the test process is itself a client, and requires
exactly two lines on standard output:

```text
rxapi: lockfile path is <path>
rxapi: lockfile is locked by another rxapi instance; exiting
```

The test compares only the prefixes `rxapi: lockfile path is` and `rxapi: lockfile is locked`. The oracle
exits 13 in that case.

The rest of its output, in order, for a run that takes the lock:

```text
rxapi: lockfile path is <path>
rxapi: lockfile lock acquired
rxapi: service path is <path>
rxapi: service successfully started; listening
```

and at exit `rxapi: service stopped and lockfile released; exiting`. `--dryrun` first prints
`rxapi: dryrun` and stops instead of listening. Any other argument first prints `rxapi: no args allowed`
and carries on. If the service path is occupied by something other than a socket, it prints
`rxapi: service is locked by another rxapi instance; exiting` and exits 13.

### Isolation

A service serves one user. Its files are private to that user. The directory the oracle uses is
irrelevant to the replacement, as the design sets its own.

## rxqueue

```text
rxqueue [queuename] [/FIFO | /LIFO | /CLEAR]
```

A filter copying standard input to a queue.

### Arguments

* An argument starting with `/` or `-` is an option, and `-` is an exact synonym for `/`. Options are
  caseless.
* Otherwise the first such argument is the queue name, and a second is a syntax error.
* At most one option may be given. A repeated option, even the same one, is a syntax error. So is an
  unknown option, including a lone `/` or `-`.
* The default is `/FIFO`.
* Arguments are checked in order, and a syntax error stops at once, before any service access.

The queue is the name argument, else the `RXQUEUE` environment variable, else `SESSION`. An
environment variable set but empty is used as given, and fails as an invalid name.

### Flow

The named queue must already exist. `rxqueue` does not create it. It first queries the queue, so a
missing queue fails before reading any input. With `/CLEAR` it then clears the queue, reads no input,
and exits 0. Otherwise it reads lines until end of input, adding each with the chosen flag. If an add
fails, it stops with the error.

### Line splitting

Input is treated as bytes.

* `\n`, `\r` and `\r\n` each end a line.
* A last line without a terminator is added. Input ending in a terminator adds no empty line after it.
  Empty input adds nothing.
* Empty lines are added as empty items.
* NUL bytes are kept in the data.
* A line beyond 65472 bytes keeps its first 65472 and the excess is discarded, without error
  (`test_rxqueue_maximum_line_length` needs exactly-65472-byte lines intact).
* Byte 0x1A ends input. Any partial line before it is dropped, and at the start of a line it adds one
  empty item. So `a\nb\x1Ac\nd\n` adds only `a`, `ab\x1A` adds nothing, and `\x1Aab\n` adds one
  empty item.
* A read error ends input, keeping any partial line.

One oracle quirk with no test behind it: a `\r` followed by a byte other than `\n` holds that byte
over, and it is treated as the start of the next line without being tested as a terminator. So
`\r\r\n\n` adds an empty item, an item holding one `\r`, and an empty item. A NUL following `\r` is lost.

### Messages and exit codes

Messages go to standard output as one line. The exit status is the API code, taken modulo 256 by
the system.

| Condition | Message | Exit |
|---|---|---|
| success | none | 0 |
| syntax error | `The syntax of the command is incorrect.` | 0 |
| invalid name | `The name <name> is not a valid queue name.` | 5 |
| missing queue | `The queue <name> does not exist.` | 9 |
| bad access mode | `The queue access mode is not correct.` | 6 |
| size | `The size of the data is incorrect.` | 2 |
| service unavailable | `The REXX rxapi queuing system is not available.` | 236 (1004) |
| memory | `Storage for data queues is exhausted.` | 234 (1002) |
| any other API code | `The syntax of the command is incorrect.` | the code modulo 256 |

**Observed:** a syntax error exits 0. **Documented:** `Error 131 - The syntax of the command is
incorrect`. The documentation gives no exit status for it.

**Documented:** "The size of the data is incorrect" for a line above 65472 bytes in the error list,
and in the command description that the excess characters are discarded. **Observed:** the discard.
The access-mode, size and any-other rows are unreachable in normal use, since the mode is always valid
and the line is always truncated first. `System error occurred while processing the command.` has no
path that reaches it.

`rxqueue` run outside any Rexx program, with no `RXQUEUESESSION`, adds to its own session queue, which
nothing else can reach, and exits 0.

## rxsubcom

```text
rxsubcom REGISTER envname library procname
rxsubcom QUERY    envname [library]
rxsubcom DROP     envname [library]
rxsubcom LOAD     envname [library]
```

The action is caseless. `REGISTER` always registers as droppable with no user data, so a `DROP` from a
later invocation works.

| Action | Effect | Exit |
|---|---|---|
| `REGISTER` | `RexxRegisterSubcomDll` | 0, 10 |
| `QUERY` | `RexxQuerySubcom` | 0, 30 |
| `DROP` | `RexxDeregisterSubcom` | 0, 10, 30, 40 |
| `LOAD` | `RexxLoadSubcom`, library ignored | 0, 30 |

The registration rules are those above. In particular a `DROP` with a library that leaves references
returns 10, and `DROP` by name alone returns 0.

Usage errors print one line to standard output and exit 255. `test_invalid_args` asserts -1 through
`assertRc`, which treats 255 as -1 on Unix.

| Condition | Message |
|---|---|
| no action, or unknown action | `The RXSUBCOM parameters are incorrect.` |
| `REGISTER` without exactly three operands | `The RXSUBCOM REGISTER parameters are incorrect.` |
| `QUERY` with zero or more than two operands | `The RXSUBCOM QUERY parameters are incorrect.` |
| `DROP` with zero or more than two operands | `The RXSUBCOM DROP parameters are incorrect.` |
| `LOAD` with zero or more than two operands | `The RXSUBCOM LOAD parameters are incorrect.` |

An unavailable service exits 236 (1004) with no message.

**Documented:** the `rxsubcom` man page's synopsis shows `QUERY sharedLibrary [function]` and likewise
for `DROP` and `LOAD`. **Observed**, and as the reference manual says: the first operand is the
environment name, as above.

## What the ooTest groups rely on

Beyond what is called out above.

`utilities/rxqueue/rxQueue.testGroup`:

* `cmd | rxqueue` from a Rexx program delivers to that program's session queue.
* Stderr and stdout merged by the shell arrive as lines in order.
* `/clear`, `/fifo` and `/lifo` on both the default queue and a named queue, and through `address ""
  ... with input using`.
* Name precedence with `RXQUEUE` set, and an explicit `session` overriding it.
* Two 65472-byte lines through both a pipe and a redirect.

`utilities/rxsubcom/rxsubcom.testGroup` needs `orxclassic1` with entry `MyTestSubcomHandler`. It uses
`rxsubcom` through `address "path"`. An unregistered environment under `ADDRESS` raises FAILURE. It runs
in one process over many `rxsubcom` processes, so drops rely on the droppable authority.

`utilities/rxapi.testGroup` is as described under "The rxapi command", and is Unix only.

`base/bif/RXQUEUE.testGroup`:

* Names `0`, `?`, `!`, `.`, `QUEUE`, `RXQUEUE`, `QUEUE.0` and 250 underscores all open and delete.
* 100 creates of each type (no name, a fixed name, the previous result) are all unique and deletable.
* `RXQUEUE("set", ...)` state is per interpreter instance.
* Deleting `session` returns 5 and a missing queue 9.

`base/class/RexxQueue.testGroup`:

* A queue created in one interpreter process is visible in another, and `rxqueue` reaches it.
* Deleting a queue that has items succeeds.
* Pull and lineIn on an invalid or deleted queue return `.nil` at once and do not wait.
* A subclass that skips `init` raises 98.901.

`API/classic/CLASSIC.testGroup` covers create, open, exists, delete, query, add, pull and clear through
the C API, with return codes 1 and 5 for bad buffers and names, the macrospace calls including save
and load with an explicit name list holding every macro, and macro search order through
`RexxReorderMacro`.

`base/rexxutil/Macrospace.testGroup` covers the Rexx macrospace functions, as cited above. It leaves its
own macros behind and clears the macrospace as it needs, so the group assumes a macrospace it may
freely clear.

## Resolutions for interrexx

These settle where interrexx departs from, or chooses among, the behaviour above.

1. **`RexxShutDownAPI`** behaves as observed: it does nothing and returns 1. The stop is `rxapi` with
   a stop option, used by test teardown.
2. **The daemon binary is named `rxapi`** and prints what `rxapi.testGroup` relies on.
3. **The macrospace file layout is interrexx's own.** Compiled images are engine-specific. A file the
   oracle wrote is refused with the return code for an invalid file.
4. **A session queue goes when no connection uses it, whatever it holds.** A connection uses a
   session queue once it touches it or attaches to it. Before its first command or child process
   the interpreter connects and attaches to its session queue, moving any lines it holds, even when
   it holds none, and keeps that connection for its life. So `cmd | rxqueue` delivers to a running
   program after `rxqueue` exits. There is no shutdown message: the connection closing, on a clean
   exit or when the kernel closes a crashed process's socket, is the only signal, and the queue
   goes with it. `echo x | rxqueue` from a plain shell loses its lines when `rxqueue` disconnects.
   Session queues do not count for the daemon's exit rule.
5. **Registrations stay owner-only,** as observed. A daemon holding them stays up.
6. **Timestamps** keep the observed zero-based `month` and years-since-1900 `year`.
7. **A blocked pull ends when its connection closes,** and the queue stays usable. The observed
   undeletable queue and daemon death are oracle defects and are not reproduced.
8. **Process id reuse** cannot reach registrations or blocked pulls, whose identity is the
   connection. Session handles are process ids, so a later process with the same id would share an
   earlier one's queue, but by Resolution 4 only while a process using that queue is still
   connected.
9. **The start timeout is 5 seconds,** not the observed two: a client gives up on a service it
   started after that, and the first call returns 1004.
10. **`rxapi --stop` with nothing running** exits 0 and prints nothing.
11. **A drop with a library that removes no reference** returns 10 when references remain, as one
    that removes a reference does.
12. **A listed macro save past 256 MiB** of image bytes returns `RXMACRO_NO_STORAGE` (1), unless
    the macrospace itself holds more, so listing each macro once always fits.
13. **The service directory** is the first of `REXX_API_DIR`, `$XDG_RUNTIME_DIR/interrexx` and a
    private directory under the temp directory whose socket path the platform can bind. When a set
    `REXX_API_DIR` is passed over, each process choosing a directory prints one line on standard
    error naming it and the reason.
14. **A command without a service** still runs when the interpreter's session queue is empty: the
    queue stays in process and the child is not given `RXQUEUESESSION`, so its `rxqueue` reports
    the service missing itself. When the queue holds lines, the command raises 48.1 with
    `RXAPI_NORXAPI`, as other queue operations do.
