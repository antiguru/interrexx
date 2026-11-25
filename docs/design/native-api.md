# The native API

interrexx implements ooRexx's C API through the headers in `include/` of this repository, which
declare the names of ooRexx's frozen public headers: an extension or embedder written for ooRexx
recompiles without source changes. Extensions link nothing from the interpreter, since everything crosses through function
tables handed in at call time, so an ooRexx-built extension `.so` also loads unchanged.

## The headers

`include/` holds the header set an extension or embedder compiles against (`spec/src/native/header.md`
states what it declares). Each file is hand-written text around sections between `BEGIN GENERATED`
and `END GENERATED` lines, which `interrexx-api/src/header.rs` renders from the `#[repr(C)]`
definitions of `interrexx-api/src/layout.rs`, the constants and the exports, so a change to a Rust
definition changes the header. The headers of the platform sit under `include/platform/<platform>/`;
only `unix` exists. The `headers_are_current` test of `interrexx-api` checks every file, and
`REXX_HEADERS_REFRESH=1` writes the generated sections. The ABI dump (`tests/abi_dump.rs`) compiles
a program against the headers and measures each offset and size against the Rust layout. The tests
that compile this repository's own native code build against `include/`, not against ooRexx's
headers.

## Where it lives

* `interrexx-api/src/ffi.rs`: the inbound boundary. The context interface tables and the conversion of
  a caller's pointer into a validated handle.
* `interrexx-api/src/load.rs`: the outbound boundary. Opening a library (`libloading`), finding symbols,
  calling through them.
* `interrexx-capi/src/exports.rs`, `interrexx-capi/src/rexxapi_exports.rs`, `interrexx-capi-rxapi/src/exports.rs`: the
  exported `extern "C"` names, each a shim over safe code in `interrexx`.

These modules, with the others listed in `interrexx-core/tests/unsafe_sites.rs`, are the only ones
allowed `unsafe`. Everything past them operates on handles in safe Rust.

## Contexts

The public structs (`RexxInstance`, `RexxThreadContext`, `RexxMethodContext`, `RexxCallContext`,
the exit context) are each the first field of a private `#[repr(C)]` struct whose second field
points back to the owner, as in ooRexx; a cast recovers the owner. The interface tables are
statics of `unsafe extern "C" fn` slots.

* Each activity has its own thread context and native-handle stack; each native activation has its
  own method or call context.
* An interpreter has one `RexxInstance`, which every activity's thread context links: the main
  program's, one that `REPLY` or `start` makes, and an attached thread's. It sits in a block with
  its owner record and is never freed, so an extension may keep the pointer past `Terminate`. Its
  members act on the interpreter from any thread: `Halt` and `SetTrace` post a request for the whole
  interpreter through its inbox, and `GetApplicationData` reads the shared block.
* The thread context of the activity that loaded a library outlives that activity, because
  libraries keep it.
* An interpreter keeps some memory for the rest of the process, because an extension may call
  through its instance or a thread context after it has ended. Every interpreter keeps its
  `RexxInstance` and the owner beside it. Once the main activity has made a native call or run a
  package hook, the interpreter also keeps that activity's thread context and what the context
  holds: its copy of the thread interface table, the requester (a handle on the baton and on the
  inbox) and anything left in the inbox when the interpreter ended. Nothing is posted to the inbox
  after the end. Each thread context `AttachThread` builds is kept too, because an extension may
  still call through it after its thread detached or the interpreter ended. A detached context goes
  on the instance's free list for the next `AttachThread`, so an instance keeps no more of them
  than the most threads it had attached at once. Classic `RexxStart` makes an interpreter for each
  run, so a process that calls it again and again keeps an instance for each run, and a thread
  context with what it holds for each run whose program loads a native library and calls into it.
  That memory grows without bound in such a process.
* A callback from a thread other than the one running that activity's native call raises
  `Error_Execution_invalid_thread`.
* A callback made while the interpreter's baton is held elsewhere takes the baton, or is run on
  its behalf by a pinned holder (see `architecture.md`, Concurrency).

## Handles and lifetimes

* `RexxObjectPtr` is an opaque handle registered in the calling native activation's local
  reference table, not an address. The table is released when the activation ends.
* A local handle that outlives its activation is a lookup miss. Heap slots carry a generation
  that moves on when a slot is reused. The interpreter-owned directories returned by
  `GetLocalEnvironment` and `GetGlobalEnvironment` remain valid in later callbacks on the same
  interpreter. A receiver given to a native method remains usable from later callbacks while
  Rexx still holds it. That handle alone does not keep the receiver alive.
* `True()` and `False()` are the handles of `.true` and `.false`. A result the API converts from
  `logical_t`, and the answer of a built-in method that the specification says answers true or
  false, is one of those two handles, so an extension may compare it to them by handle. A built-in
  that answers `1` or `0` as numbers (`RegularExpression~match`, say) and Rexx code that does
  `return 1` answer numbers, as on ooRexx.
* A handle is derived from the object reference, so the same object has the same handle in every
  table of one interpreter. On a 64-bit target the handle's address is the reference's bits. On
  a 32-bit target a reference that fits in 31 bits is carried in the address, and every other is
  given a number by the interpreter's handle space. A number is held while any table of the
  interpreter holds the reference, released handles included, or the interpreter keeps the
  handle for later callbacks, and is given back when the last holder lets it go. Numbers are
  handed out in turn, wrapping at 2^31 and skipping held ones, so a number given back names
  nothing else until about two billion others have been handed out since: a handle kept past its
  activation misses rather than naming another object, short of that. The space grows with the
  handles in use, not with the handles ever made. Two handles to one object agree while either
  resolves; one that no longer resolves may differ from a fresh handle to the object.
* `RequestGlobalReference` holds an object in a separate table until `ReleaseGlobalReference`.
* Memory lent to a native call (a `CSTRING` argument, `BufferData`, `MutableBufferData`) stays
  valid until that call completes. A buffer that grows during the call gets new storage, and the
  old stays alive until the call returns.
* Objects never cross interpreter instances through the API; such a call is refused. Each instance
  has its own `.environment` and `.local`.

## Calling a native method or routine

The `RexxMethodN` and `RexxRoutineN` macros generate a stub that interrexx calls twice: first with
no arguments, and the stub returns its signature (return type, then parameter types); then with an
argument array converted from Rexx values per that signature, the result written into element 0.
The signature call touches no argument. An absent required argument raises 88.901; one that does
not convert raises the conversion's 88.9xx error. A signature with a type code the table does not
know raises 93.968 (40.918 for a routine).

Raising and throwing differ, as in ooRexx:

* `RaiseException*` and `RaiseCondition` record the condition and return to the extension, which
  runs on; the condition is raised when the native call returns, and the call's result is still
  delivered.
* `ThrowException*` and `ThrowCondition` unwind out of the extension's frames (running its
  destructors). These slots and the call into an extension are `extern "C-unwind"`, and the call
  boundary catches only that unwind. An extension linked with a static `libgcc`, or one that
  swallows the unwind in `catch (...)`, aborts where ooRexx's would continue (a Deviation).
* Every other slot is `extern "C"`, so a Rust panic there aborts.

## Loading libraries

* A library name becomes `lib<name>` with the host's suffix (`.so`, `.dylib` on macOS), tried
  with `dlopen` and then under `/usr/lib`.
* The package entry is the exported `RexxGetPackage`. A `requiredVersion` above the interpreter's
  raises 98.982. Routines and methods are registered from its tables; its `loader` hook runs after
  registration, and `unloader` hooks run at termination in ooRexx's table order.
* A library is loaded for the process: the instances live at one time share one load of each
  file, its `loader` runs at the first, and the last instance to be terminated runs each
  library's `unloader` and closes it. A library whose `loader` another thread still runs when
  that walk reaches it is left to the instance it loads for, and that termination neither
  unloads nor closes it. An instance created after that
  runs the `loader` again (`r[native.loading.shared]`, `r[native.loading.unload]`,
  `r[native.loading.again]`).
* `libraries.rs` keeps that round process-wide. A load of a library whose `loader` another thread
  is running waits for it, with the interpreter's baton vacated as for an external call, so that
  loader may call into the waiting instance. A load of another library does not wait. An instance
  is created only once the last round's unloaders have returned, and the last instance counts as
  live until then, so an instance an unloader creates shares its libraries. Libraries an unloader
  loads are walked in the same termination. A thread never waits for a hook running on itself or
  on a thread that created its instance (`RexxCreateInterpreter` hands the new instance's thread
  that lineage), so a hook that creates an instance or loads a library does not wait for itself.
  Each wait is recorded with the thread's lineage, and a wait that would close a cycle of such
  waits is not made: the load finds the library loaded, or the instance joins the round being
  unloaded, as ooRexx, which makes neither wait, behaves.
* A `loader` or `unloader` runs with its instance's baton vacated, as an external call does
  (`r[native.loading.reentrant]`). A thread the hook waits for may then call into that instance,
  and so may a loader that another of the hook's loads waits for, so a wait between instances
  never runs through a baton a hook holds. A load through the instance whose `loader` another
  thread runs waits for it as a load through another instance does. The loader's guard is
  released before the baton is taken back, because such a load may hold a lend above the
  vacancy.
* A closed library is never unmapped: an attached thread, or a thread the library started, may still
  be returning into its code when the last instance ends (`r[native.loading.static]`). A later load
  by name finds the copy already mapped. A thread inside a native call is counted in its library
  until the call returns.
* A `::REQUIRES ... LIBRARY` naming a missing library is 98.903 before the first clause; an
  `EXTERNAL` naming a missing entry is 90.998 for a method and 90.999 for a routine;
  `Package~loadLibrary` answers 1 or 0; a missing library through `.Object~package~loadLibrary`
  is 98.984.
* `rxmath`, `rxregexp`, `rxunixsys` and `rxsock` are answered by Rust internal packages before any
  `dlopen` (see `extensions.md`).
* The classic registration API (`RexxRegisterFunctionDll` and friends) goes through the API
  service (see `rxapi.md`); classic handlers are resolved through it and kept once found.

## Embedding

* `librexx.so.4` exports the interpreter's entry points (`RexxCreateInterpreter`, `RexxStart`,
  `RexxTranslateProgram`, `RexxVariablePool`, `RexxSetHalt`, `RexxSetTrace` and the rest of
  `rexx.h`'s) and also the classic API's names. It owns all API client state.
* `librexxapi.so.4` exports the classic queue, registration, macrospace and memory names and
  forwards each to `librexx.so.4`, which it names as `NEEDED` and finds through a `$ORIGIN` rpath.
* On macOS the two are `librexx.4.dylib` and `librexxapi.4.dylib`, with install names under
  `@rpath`; `librexxapi` records `@loader_path/librexx.4.dylib` as a load command and opens it by
  that path. `interrexx-api/src/library_names.rs` holds the names of each host.
* The `rexx` executable compiles the same export module, so an extension it loads reaches the
  interpreter it is running in. That holds where an extension's imports resolve in the process's
  global scope, as on Linux, where the executable's exports come first. On macOS an import binds
  to the library the extension was linked against (two-level namespace), so an extension linked
  against `librexxapi` reaches the interpreter of `librexx.4.dylib`, a second one, not the one
  `rexx` runs. This follows from the binding rules; no test shows it yet. #761 proposes making
  `rexx` a launcher over `librexx`, which would leave one interpreter on every host.
* `RexxStart` runs a program with the classic system exits, on the calling thread or as a classic
  run other threads can halt or trace. `RexxCreateInterpreter` makes an instance; several may
  exist in one process, each isolated. `AttachThread` joins a foreign thread to an instance (see
  Attached threads), and `AddCommandEnvironment` registers a command handler for it.
* A program run through `RexxStart` or `RexxCreateInterpreter` is marked as run by an outside host,
  which turns off the refusal of waits nothing can end (see `architecture.md`, Concurrency). The
  `rexx` executable and the in-process test harnesses do not set the mark.
* `hostemu` and `orxncurses` are built unmodified against these two libraries by
  `scripts/build-extensions.sh` and run without ooRexx's libraries on the search path.

## Attached threads

`AttachThread` answers a thread that runs a native call of the interpreter with that call's context,
and a thread already attached with its context again, counted. Any other thread gets an attached
activity of its own. Its context is built without the baton and put on the instance's pending list,
and the interpreter files the activity, a scheduler slot with a base native frame for local
references, at the first callback through the context. So an attach answers even while the only
activity runs a native call that waits for the attaching thread.

Each callback through an attached context opens an entry on the attached thread's stack for the
length of the member, and the Rexx code it reaches runs there as the running activity, with its own
`.context~thread` number. The last `DetachThread` from the attaching thread ends the activity: its
references, strings and buffers are released, its number is freed, and its context goes on the
instance's free list, which the next `AttachThread` takes from before it builds one. A detach made
inside a callback through the context ends the activity once that callback returns. `DetachThread`,
`HaltThread` and `SetThreadTrace` through the context from another thread return at once and do
nothing (Deviation 177).

An idle attached activity does not hold the program's end, and one running a message does. After
`Terminate`, a callback through an attached context is refused with the member's failure value
(Deviation 176), as is one after the thread detached (Deviation 19). How a callback shares the
interpreter with the other activities is in `architecture.md`, Concurrency.
