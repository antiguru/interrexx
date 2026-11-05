# The native API

interrexx implements ooRexx's C API against its frozen public headers (`api/` in
`third_party/ooRexx`): an extension or embedder written for ooRexx recompiles without source
changes. Extensions link nothing from the interpreter, since everything crosses through function
tables handed in at call time, so an ooRexx-built extension `.so` also loads unchanged.

## Where it lives

* `rexx-api/src/ffi.rs`: the inbound boundary. The context interface tables and the conversion of
  a caller's pointer into a validated handle.
* `rexx-api/src/load.rs`: the outbound boundary. Opening a library (`libloading`), finding symbols,
  calling through them.
* `librexx/src/exports.rs`, `librexx/src/rexxapi_exports.rs`, `librexxapi/src/exports.rs`: the
  exported `extern "C"` names, each a shim over safe code in `rexx-exec`.

These modules, with the others listed in `rexx-core/tests/unsafe_sites.rs`, are the only ones
allowed `unsafe`. Everything past them operates on handles in safe Rust.

## Contexts

The public structs (`RexxInstance`, `RexxThreadContext`, `RexxMethodContext`, `RexxCallContext`,
the exit context) are each the first field of a private `#[repr(C)]` struct whose second field
points back to the owner, as in ooRexx; a cast recovers the owner. The interface tables are
statics of `unsafe extern "C" fn` slots.

* Each activity has its own thread context and native-handle stack; each native activation has its
  own method or call context.
* The interpreter's `RexxInstance`, and the thread context of the activity that loaded a library,
  outlive that activity, because libraries keep them.
* A callback from a thread other than the one running that activity's native call raises
  `Error_Execution_invalid_thread`.
* A callback made while the interpreter's baton is held elsewhere takes the baton, or is run on
  its behalf by a pinned holder (see `architecture.md`, Concurrency).

## Handles and lifetimes

* `RexxObjectPtr` is an opaque handle registered in the calling native activation's local
  reference table, not an address. The table is released when the activation ends.
* A handle that outlives its activation is a lookup miss, never another object: heap slots carry a
  generation that moves on when a slot is reused.
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

* A library name becomes `lib<name>.so`, tried with `dlopen` and then under `/usr/lib`.
* The package entry is the exported `RexxGetPackage`. A `requiredVersion` above the interpreter's
  raises 98.982. Routines and methods are registered from its tables; its `loader` hook runs after
  registration, and `unloader` hooks run at termination in ooRexx's table order.
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
* The `rexx` executable compiles the same export module, so an extension it loads reaches the
  interpreter it is running in.
* `RexxStart` runs a program with the classic system exits, on the calling thread or as a classic
  run other threads can halt or trace. `RexxCreateInterpreter` makes an instance; several may
  exist in one process, each isolated. `AttachThread` joins a foreign thread to an instance as a
  new activity, and `AddCommandEnvironment` registers a command handler for it.
* `hostemu` and `orxncurses` are built unmodified against these two libraries by
  `scripts/build-extensions.sh` and run without ooRexx's libraries on the search path.
