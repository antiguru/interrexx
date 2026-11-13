---
element: "Native libraries"
kind: chapter
rexxref: "Directives > ::REQUIRES; Directives > ::ROUTINE"
---

# Native libraries

This chapter specifies what a Rexx program sees of code that is not written in
Rexx: the routines and methods of a native library, the classic functions that
RXFUNCADD registers, and the threads such code starts. The library is written
against the interpreter's public C interface; this chapter states only what a
program can observe of it, so every rule has a Rexx side.

The pages of this chapter are:

* [Loading](loading.md): how a library provides its routines and methods, what
  loading it does and how often, how a declaration finds an entry in it, and
  how the classic functions of a library without a package are registered and
  called.
* [Native routines and methods](calls.md): how arguments are converted and
  results written, how native code raises errors and conditions, and what it
  sees of its caller and, for a method, of its object.
* [Threads and halting](threads.md): threads that native code starts and
  attaches, what program end does with them, the one interpreter instance
  every activity is given, and halting activities from native code.

What a library's name means, and where the file is searched for, is [the
library rule](r:env.search.library). The directives that name a library are
[`::REQUIRES ... LIBRARY`](r:dir.requires.library),
[`::ROUTINE ... EXTERNAL`](r:dir.routine.external) and
[`::METHOD ... EXTERNAL`](r:dir.method.external), and the functions that
register one are [RXFUNCADD](../builtins/rxfuncadd.md),
[RXFUNCQUERY](../builtins/rxfuncquery.md) and
[RXFUNCDROP](../builtins/rxfuncdrop.md). Those pages state what they say
without a library of the chapter's own; this chapter adds what needs one.

## The fixture libraries

The examples of this chapter call three small libraries that come with this
book, `specfixture`, `specclassic` and `specnewer`. Their source is the file
`spec/fixture/specfixture.cpp`, written against the public interface only, and
the runner of the examples compiles it and puts the directory of the libraries
on the search path of the loader, so a program finds them by name, as it would
find any library installed on the host. No entry keeps anything from one call
to the next.

`specfixture` has a package entry, which provides these routines:

| Routine | Arguments | Answers |
| --- | --- | --- |
| `FxHello` | none | `hello from specfixture` |
| `FxEcho` | any object | the object itself |
| `FxString` | a string | the same string |
| `FxAdd` | two 32-bit integers | their sum |
| `FxPositive` | a positive whole number | the number |
| `FxNonNegative` | a nonnegative whole number | the number |
| `FxDouble` | a binary floating-point number | the number |
| `FxLogical` | a logical value | the value |
| `FxItems` | an Array | the number of its items |
| `FxOptional` | an optional 32-bit integer | the number, or `omitted` |
| `FxArgs` | any arguments | the size of the argument list and the number of arguments given |
| `FxName` | none | the name it was called by |
| `FxNothing` | none | no result |
| `FxRaise` | a message | raises `SYNTAX 40.900` with it, sets the caller's variable `AFTER` to `ran on`, and answers `returned` |
| `FxCondition` | a name, a description and an object | raises the condition of that name with them, and answers `returned` |
| `FxSend` | an object and a message name | sends the message and answers its result, `no result`, or `condition` and the code of a condition the message raised, which it clears |
| `FxSetVar`, `FxGetVar`, `FxDropVar` | a variable name, and for `FxSetVar` a value | set, get (or `(unset)`) and drop a variable of the caller |
| `FxNumeric` | none | the caller's `DIGITS`, `FUZZ` and `FORM` |
| `FxOnThread` | an object and a message name | starts a thread that attaches to the interpreter, sends the message to the object and detaches; answers `1` at once |
| `FxOnThreadIdle` | an object, a message name and a number of seconds | as `FxOnThread`, but after the message the thread stays attached, sending nothing, for that many seconds before it detaches |
| `FxInstance` | none | the interpreter instance the calling activity is given, written as an address |
| `FxHaltAll` | none | asks the interpreter to halt every activity |
| `FxHaltThis` | none | asks the interpreter to halt the calling activity |
| `FxTwin`, `FXTWIN` | none | `first row` and `second row`: two rows whose names differ only in case |
| `FxClassicJoin`, `FxClassicFail`, `FxClassicNone` | strings | the classic functions below, as rows of the package |

and these methods:

| Method | Arguments | Answers |
| --- | --- | --- |
| `FxSelf` | none | the receiver |
| `FxMessage` | none | the name of the message |
| `FxScope` | none | the class that defines the method |
| `FxObjSet`, `FxObjGet` | a variable name, and for `FxObjSet` a value | set and get (or `(unset)`) an object variable of the method's scope |
| `FxMethodRaise` | a message | raises `SYNTAX 93.900` with it, and answers `returned` |

A loader runs when the library is loaded, and adds 1 to the entry
`SPECFIXTURE.LOADS` of `.local`, which it creates with the value 1.

`specclassic` has no package entry. It exports classic functions only:
`FxClassicJoin` answers the name it was called by, a colon, the number of its
arguments, a colon, and its arguments separated by commas, with an omitted
argument written `()`; `FXCLASSICUPPER` is the same function under a name in
capitals; `FxClassicFail` refuses every call; and `FxClassicNone` answers no
result.

`specnewer` has a package entry that asks for an interpreter newer than any
there is, with the one routine `FxNewer`.
