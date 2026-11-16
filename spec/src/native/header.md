---
element: "The header files"
kind: concept
rexxref: "Rexx C++ Application Programming Interfaces"
---

# The header files

A native library, and a program that embeds the interpreter, is compiled
against the header files that come with the interpreter, in the directory
`include/` of this repository. This page states what those files declare
beyond the functions the other pages of this chapter describe: the files and
their platform part, the types, the families of constants, the flag words of
the exit parameter blocks, the C++ methods of the contexts, and the macros
that declare a library's routines, methods and tables. What each function of
the interface does is stated where a program can observe it, on [the loading
page](loading.md), [the calls page](calls.md), [the threads
page](threads.md) and [the command handlers page](handlers.md). The rules of
this page are checked by compiling programs against the files, not by
running Rexx examples.

## Files and languages

r[native.header.files]
The header set is `rexx.h`, the classic interface; `rexxapidefs.h`, its
return, exit and queue codes; `oorexxapi.h`, the object-oriented interface;
`oorexxerrors.h`, the error codes; and three files of a platform's own,
`rexxapitypes.h`, `rexxplatformdefs.h` and `rexxplatformapis.h`, in the
directory `platform/<platform>/`. A library includes `oorexxapi.h`, which
includes all the others; a program that uses only the classic interface may
include `rexx.h` alone, which includes the other classic and platform files.
The files include a platform file by its bare name, so both `include/` and
its platform directory belong on the compiler's search path. Every file may
be included more than once, and compiles as C99 or later and as C++11 or
later.

r[native.header.platform]
The names whose definition is a platform's are the types `REXXPFN`,
`process_id_t` and `thread_id_t`, the struct `REXXDATETIME`, the type of the
third parameter of `RexxPullQueue`, and the macros `REXXENTRY`, `REXXEXPORT`,
`__REXX64__` and `SSIZE_MIN`. A library may rely on each name existing on
every platform, and on this much of its meaning:
* `REXXENTRY`, and its other name `RexxEntry`, is the calling convention of
  every function the interpreter calls and every function it exports; a
  library writes it in the declaration of each of its own such functions.
* `REXXEXPORT` marks a function the library exports by name.
* `REXXPFN` is the type of an entry point that the classic registration
  functions take.
* `process_id_t` and `thread_id_t` identify a process and a thread, as
  `RexxSetHalt`, `RexxSetTrace` and `RexxResetTrace` take them.
* `__REXX64__` is defined exactly where a pointer has 64 bits.
* `SSIZE_MIN` is the least value of `ssize_t`.

r[native.header.platform.unix]
On Unix, `REXXPFN` is `void *`, `process_id_t` is `pid_t`, `thread_id_t` is
`pthread_t`, `REXXENTRY` and `REXXEXPORT` expand to nothing, and
`REXXDATETIME` is a struct of its own, the third parameter of `RexxPullQueue`
pointing at one. The header set is written for Unix only; another platform
gets a directory of its own with the same three files.

r[native.header.languages]
Compiled as C++, each context name (`RexxInstance`, `RexxThreadContext`,
`RexxMethodContext`, `RexxCallContext`, `RexxExitContext` and
`RexxIORedirectorContext`) names its context struct, and the struct has the
methods of [the context methods rule](r:native.header.context-methods).
Compiled as C, each names a pointer to its constant struct, and the structs
have no methods. A function of the interface that takes `RexxThreadContext *`
therefore takes a pointer to a pointer in C, while the interpreter passes a
pointer to the struct, so C code has no supported way to use a context. A
library written in C may use the types, the constants, the classic interface,
the tables, and the routine and method declarations of [the declaration
rule](r:native.header.declaration) whose code does not use its context; a
library that calls the interpreter is written in C++.

r[native.header.handles]
Each object type (`RexxObjectPtr`, `RexxStringObject`,
`RexxBufferStringObject`, `RexxArrayObject`, `RexxBufferObject`,
`RexxPointerObject`, `RexxMethodObject`, `RexxRoutineObject`,
`RexxPackageObject`, `RexxClassObject`, `RexxDirectoryObject`,
`RexxStringTableObject`, `RexxSupplierObject`, `RexxStemObject`,
`RexxMutableBufferObject` and `RexxVariableReferenceObject`) is a pointer to
an incomplete struct of its own. In C++, every one of them converts to
`RexxObjectPtr` without a cast, and `RexxBufferStringObject` converts to
`RexxStringObject`; no other conversion between them exists. In C none of
them converts to another without a cast.

## Types

r[native.header.scalars]
`wholenumber_t` is a signed integer of the width of a pointer, the type of
`ssize_t`. `stringsize_t` and `logical_t` are both the type of `size_t`; a
`logical_t` is 0 for false and anything else for true. `CSTRING` is
`const char *`, `POINTER` is `void *`, `RexxReturnCode` is `int`, and `PEXIT`,
the parameter block an exit handler receives, is `char *`.

r[native.header.handler-types]
The types of the functions a library or an embedding program gives the
interpreter to call are function types, not pointers, except the two hooks
of a package entry:
* `RexxRoutineHandler`, and its other name `RexxFunctionHandler`, is a
  classic function: it takes the name it was called by, the number of its
  arguments, the array of its arguments as `CONSTRXSTRING`, the name of the
  current queue and the `RXSTRING` for its result, and answers a `size_t`
  that is 0 when it accepts the call, as [the classic call
  rule](r:native.loading.classic-call) describes.
* `RexxExitHandler` is a classic exit handler: it takes the exit's function
  code, its subfunction code and its parameter block, and answers an `int`.
  `RexxContextExitHandler` is the same with the exit context before them.
* `RexxContextCommandHandler` is a command handler: it takes the exit context,
  the name of the environment and the command, both as `RexxStringObject`,
  and answers the result object. `RexxRedirectingCommandHandler` takes the
  redirector context after them.
* `RexxPackageLoader` and `RexxPackageUnloader` are pointers to a function
  that takes the thread context and answers nothing.

r[native.header.special-types]
Seven names are types for a declaration's special parameters: `OSELF` and
`SCOPE` are `RexxObjectPtr`, `SUPER` is `RexxClassObject`, `ARGLIST` is
`RexxArrayObject`, `NAME` is `CSTRING`, and `CSELF` and `BUFFER` are
`POINTER`. As a parameter of a routine or method declaration, `NAME` is the
name the routine or method was called by, and `ARGLIST` is the Array of the
arguments, which [the argument list rule](r:native.calls.arglist) describes.
As a parameter of a method, `OSELF` is the receiver, `SUPER` the class the
method's scope inherits from, `SCOPE` the scope, and `CSELF` the address the
receiver's variable `CSELF` holds as a Pointer or a Buffer. None of them is
valid as a result, and none of them consumes an argument of the call.

## Constants

r[native.header.value-codes]
A value code names the type of a declaration's parameter or result.
`REXX_VALUE_<name>` is the code of each name below, and `ARGUMENT_TYPE_<name>`
is the C type of a parameter declared with it. The optional bit,
`REXX_OPTIONAL_ARGUMENT`, is `0x8000`. For a name with an optional form,
`REXX_VALUE_OPTIONAL_<name>` is its code with the optional bit set, and
`ARGUMENT_TYPE_OPTIONAL_<name>` is the same type as `ARGUMENT_TYPE_<name>`.
The names that begin with two underscores are second names of a code, each
of the same type. `ARGUMENT_TYPE_OPTIONAL_RexxVariableReferenceObject` is
defined, but a declaration cannot use it: the optional variable reference has
no code. The names, each with its code and the type of its parameter, are:
* `ARGLIST`: code 2, type `RexxArrayObject`
* `NAME`: code 3, type `CSTRING`
* `SCOPE`: code 4, type `RexxObjectPtr`
* `CSELF`: code 5, type `POINTER`
* `OSELF`: code 6, type `RexxObjectPtr`
* `SUPER`: code 7, type `RexxClassObject`
* `RexxObjectPtr`: code 11, type `RexxObjectPtr`, with an optional form
* `int`: code 12, type `int`, with an optional form
* `wholenumber_t`: code 13, type `wholenumber_t`, with an optional form
* `double`: code 14, type `double`, with an optional form
* `CSTRING`: code 15, type `CSTRING`, with an optional form
* `POINTER`: code 16, type `POINTER`, with an optional form
* `RexxStringObject`: code 17, type `RexxStringObject`, with an optional form
* `stringsize_t`: code 18, type `stringsize_t`, with an optional form
* `float`: code 19, type `float`, with an optional form
* `int8_t`: code 20, type `int8_t`, with an optional form
* `int16_t`: code 21, type `int16_t`, with an optional form
* `int32_t`: code 22, type `int32_t`, with an optional form
* `int64_t`: code 23, type `int64_t`, with an optional form
* `uint8_t`: code 24, type `uint8_t`, with an optional form
* `uint16_t`: code 25, type `uint16_t`, with an optional form
* `uint32_t`: code 26, type `uint32_t`, with an optional form
* `uint64_t`: code 27, type `uint64_t`, with an optional form
* `intptr_t`: code 28, type `intptr_t`, with an optional form
* `uintptr_t`: code 29, type `uintptr_t`, with an optional form
* `logical_t`: code 30, type `logical_t`, with an optional form
* `RexxArrayObject`: code 31, type `RexxArrayObject`, with an optional form
* `RexxStemObject`: code 32, type `RexxStemObject`, with an optional form
* `size_t`: code 33, type `size_t`, with an optional form
* `ssize_t`: code 34, type `ssize_t`, with an optional form
* `POINTERSTRING`: code 35, type `POINTER`, with an optional form
* `RexxClassObject`: code 36, type `RexxClassObject`, with an optional form
* `RexxMutableBufferObject`: code 37, type `RexxMutableBufferObject`, with an optional form
* `positive_wholenumber_t`: code 38, type `wholenumber_t`, with an optional form
* `nonnegative_wholenumber_t`: code 39, type `wholenumber_t`, with an optional form
* `RexxVariableReferenceObject`: code 40, type `RexxVariableReferenceObject`
* `__int32_t`: code 22, type `int32_t`
* `__int64_t`: code 23, type `int64_t`
* `__uint32_t`: code 26, type `uint32_t`
* `__uint64_t`: code 27, type `uint64_t`
* `__uintptr_t`: code 29, type `uintptr_t`

r[native.header.versions]
Each interface table begins with its member `interfaceVersion`, the
version of the table. The unsuffixed
`<TABLE>_INTERFACE_VERSION` names are the numbers this interpreter's tables
carry, and a name ending `_<x>_<y>_<z>` is the number the table had in the
release of that number. An interpreter version is a number with two
hexadecimal digits each for the release's three parts, `0x00xxyyzz`:
`REXX_INTERPRETER_<x>_<y>_<z>` names a release, and
`REXX_CURRENT_INTERPRETER_VERSION`, the value the instance's
`InterpreterVersion` answers, is this interpreter's. A package entry's
required version is compared with it, as [the version
rule](r:native.loading.version) says. A language level has the same form,
`0x0000mmnn` for level `m.nn`: `REXX_LANGUAGE_6_<nn>` names one, and
`REXX_CURRENT_LANGUAGE_LEVEL` is the one the instance's `LanguageLevel`
answers. The values are:
* `INSTANCE_INTERFACE_VERSION_4_0_0`: `100`
* `INSTANCE_INTERFACE_VERSION_5_0_0`: `101`
* `INSTANCE_INTERFACE_VERSION_5_3_0`: `102`
* `INSTANCE_INTERFACE_VERSION`: `102`
* `THREAD_INTERFACE_VERSION_4_0_0`: `100`
* `THREAD_INTERFACE_VERSION_4_1_1`: `101`
* `THREAD_INTERFACE_VERSION_4_2_0`: `102`
* `THREAD_INTERFACE_VERSION_5_0_0`: `103`
* `THREAD_INTERFACE_VERSION_5_3_0`: `104`
* `THREAD_INTERFACE_VERSION`: `104`
* `METHOD_INTERFACE_VERSION_4_0_0`: `100`
* `METHOD_INTERFACE_VERSION_4_2_0`: `101`
* `METHOD_INTERFACE_VERSION_5_0_0`: `102`
* `METHOD_INTERFACE_VERSION`: `102`
* `CALL_INTERFACE_VERSION_4_0_0`: `100`
* `CALL_INTERFACE_VERSION_5_0_0`: `101`
* `CALL_INTERFACE_VERSION`: `101`
* `EXIT_INTERFACE_VERSION_4_0_0`: `100`
* `EXIT_INTERFACE_VERSION_5_0_0`: `101`
* `EXIT_INTERFACE_VERSION`: `101`
* `REDIRECT_INTERFACE_VERSION_5_0_0`: `100`
* `REDIRECT_INTERFACE_VERSION`: `100`
* `REXX_INTERPRETER_4_0_0`: `0x00040000`
* `REXX_INTERPRETER_4_0_1`: `0x00040001`
* `REXX_INTERPRETER_4_1_0`: `0x00040100`
* `REXX_INTERPRETER_4_1_1`: `0x00040101`
* `REXX_INTERPRETER_4_1_2`: `0x00040102`
* `REXX_INTERPRETER_4_1_3`: `0x00040103`
* `REXX_INTERPRETER_4_2_0`: `0x00040200`
* `REXX_INTERPRETER_4_3_0`: `0x00040300`
* `REXX_INTERPRETER_5_0_0`: `0x00050000`
* `REXX_INTERPRETER_5_1_0`: `0x00050100`
* `REXX_INTERPRETER_5_2_0`: `0x00050200`
* `REXX_INTERPRETER_5_3_0`: `0x00050300`
* `REXX_CURRENT_INTERPRETER_VERSION`: `0x00050300`
* `REXX_LANGUAGE_6_03`: `0x00000603`
* `REXX_LANGUAGE_6_04`: `0x00000604`
* `REXX_LANGUAGE_6_05`: `0x00000605`
* `REXX_LANGUAGE_6_06`: `0x00000606`
* `REXX_CURRENT_LANGUAGE_LEVEL`: `0x00000606`

r[native.header.entry-constants]
The constants of the entry tables and of an argument descriptor are:
* `REXX_ARGUMENT_TERMINATOR`: `0`, the word that ends a signature.
* `ARGUMENT_EXISTS`: `0x01`, set in the flags of a descriptor whose argument
  was given.
* `SPECIAL_ARGUMENT`: `0x02`, set in the flags of a special parameter's
  descriptor.
* `ROUTINE_TYPED_STYLE`: `1`, the style of a routine row whose entry [the
  declaration rule](r:native.header.declaration) defines.
* `ROUTINE_CLASSIC_STYLE`: `2`, the style of a routine row whose entry is a
  classic function.
* `METHOD_TYPED_STYLE`: `1`, the style of a method row.
* `REXX_PACKAGE_API_NO`: `20081030`, the `apiVersion` that [the package
  rule](r:native.header.package) writes.

r[native.header.error-codes]
`oorexxerrors.h` defines a name for every error number `m.n` whose major
number `m` is below 100: `Rexx_` followed by the error's symbolic name, the
`symbol` column of the table of error numbers in this repository
(`rust/crates/interrexx-inventory/data/errors.tsv`), with the value
`m * 1000 + n`. A minor number 0 stands for the major error itself.

## Exit flag words

r[native.header.exit-flags]
Seven exit parameter blocks begin with a word of one-bit flags. Each flag is
an `unsigned` bit field, and the flags of a word take its bits from bit 0
upward in the order given here:
* `RXFNC_FLAGS`, `RXOFNC_FLAGS` and `RXEXF_FLAGS`: `rxfferr` (bit 0), set
  when the function was found but refused the call; `rxffnfnd` (bit 1), set
  when it was not found; `rxffsub` (bit 2), set when the call is a CALL, which
  needs no result.
* `RXCMD_FLAGS`: `rxfcfail` (bit 0), set when the command failed, which
  raises FAILURE; `rxfcerr` (bit 1), set when it ended in error, which raises
  ERROR.
* `RXMSQ_FLAGS`: `rxfmlifo` (bit 0), set when the line goes on the queue last
  in, first out.
* `RXHLT_FLAGS`: `rxfhhalt` (bit 0), set when a halt is to be raised.
* `RXTRC_FLAGS`: `rxftrace` (bit 0), set when interactive tracing is to be
  on.

## Context methods

r[native.header.context-methods]
Compiled as C++, each context struct has a method for every function member
of its own table, of the member's name, which takes the member's parameters
after the context and calls the member with the context itself; on the thread
context, `StringSizeToObject` and `ObjectToStringSize` spell `stringsize_t` as
`size_t`, the same type, and `NewBuffer` and `NewMutableBuffer` take their
size as a `wholenumber_t`. The method, call and exit contexts also have every
method the thread context has but `DetachThread`, `HaltThread` and
`SetThreadTrace`, each calling the thread context's member through their
`threadContext`; their `DisplayCondition` answers a `logical_t`. The thread
context, and so the three others, has the methods `InterpreterVersion`,
`LanguageLevel`, `AddCommandEnvironment` and `StackCommandEnvironment`, which
call the instance's member of that name; `GetApplicationData`, which answers
the instance's `applicationData`; and `Nil`, `True`, `False` and
`NullString`, which answer the table's `RexxNil`, `RexxTrue`, `RexxFalse` and
`RexxNullString`.

r[native.header.convenience]
The thread context, and so the method, call and exit contexts, also has
methods of shorter names. Each calls a member named below it, with that
member's parameters and result, except that `StringSize` spells
`stringsize_t` as `size_t`; a name that calls several members is overloaded
by their parameters. The names and the members each calls are:
* `ObjectToCSelf`: `ObjectToCSelfScoped`
* `WholeNumber`: `WholeNumberToObject`, `ObjectToWholeNumber`
* `Uintptr`: `UintptrToObject`, `ObjectToUintptr`
* `Intptr`: `IntptrToObject`, `ObjectToIntptr`
* `StringSize`: `StringSizeToObject`, `ObjectToStringSize`
* `Int64`: `Int64ToObject`, `ObjectToInt64`
* `UnsignedInt64`: `UnsignedInt64ToObject`, `ObjectToUnsignedInt64`
* `Int32`: `Int32ToObject`, `ObjectToInt32`
* `UnsignedInt32`: `UnsignedInt32ToObject`, `ObjectToUnsignedInt32`
* `Logical`: `ObjectToLogical`, `LogicalToObject`
* `Double`: `DoubleToObject`, `ObjectToDouble`
* `CString`: `ObjectToStringValue`, `NewStringFromAsciiz`
* `String`: `NewString`, `NewStringFromAsciiz`
* `Array`: `ArrayOfOne`, `ArrayOfTwo`, `ArrayOfThree`, `ArrayOfFour`

## Declaring routines and methods

r[native.header.declaration]
`RexxRoutine<N>(returnType, name, t1, n1, ..., tN, nN)`, for `N` from 0 to
10, begins the definition of a routine of `N` parameters, and the function
body written after it is the routine's code. Each `tK` is the name of a value
code, as [the value code rule](r:native.header.value-codes) lists them, with
`OPTIONAL_` before it for an optional parameter, and the parameter `nK` has
the type `ARGUMENT_TYPE_<tK>`. `returnType` names the code of the result in
the same way, and the body returns a value of its argument type. The body
also sees a parameter `context`, the routine's `RexxCallContext *`.
`RexxMethod<N>` declares a method in the same way, whose `context` is a
`RexxMethodContext *`. The declaration defines two functions:
`<name>_impl`, whose parameters are `context` and the declared ones and whose
body is the code; and `name`, the entry a table row gives, which has C
linkage in C++. Called with a null descriptor array, the entry answers the
signature: the result's code, each parameter's code in order, then
`REXX_ARGUMENT_TERMINATOR`. Called with an array, it calls `<name>_impl` with
the values of elements 1 to `N`, each read as its parameter's code, stores
the result in element 0 as the result's code, and answers a null pointer.
`REXX_TYPED_ROUTINE_PROTOTYPE(name)` and `REXX_METHOD_PROTOTYPE(name)`
declare such an entry, for a table in another source file than the
declaration. Both expand to a declaration that ends in a semicolon, so a use
compiles with or without a semicolon of its own.

r[native.header.argument-exists]
In the code of a routine or method, `argumentExists(i)` is whether the
argument of descriptor `i` was given, and `argumentOmitted(i)` whether it was
omitted. `i` counts the declaration's parameters from 1, special ones
included, not the arguments of the call. Both read the context, so only C++
code can use them. An omitted optional parameter of a numeric code is 0, and
one of a pointer or object code is a null pointer.

r[native.header.descriptors]
`ARGUMENT_TYPE(t)` is the signature word `t` without the optional bit, and
`IS_OPTIONAL_ARGUMENT(t)` is whether the bit is set. `GET_REXX_VALUE(v, t)`
is the value the descriptor `v` holds as the code named `t`, and
`SET_REXX_VALUE(v, t, value)` stores `value` in `v` as that code and sets
its type to the code, leaving its flags. Compiled as C++, a
`ValueDescriptor` may be assigned a `RexxObjectPtr`, `RexxStringObject`,
`RexxArrayObject`, `RexxStemObject`, `RexxMutableBufferObject`, `CSTRING`,
`POINTER`, `wholenumber_t`, `float` or `double`: the assignment stores the
value as the code of that type's own name and sets the type in the same way.
It also converts implicitly to each of those types and to `stringsize_t`,
answering the value stored as the code of that type's own name, whatever its
type says.

r[native.header.tables]
A routine table is an array of `RexxRoutineEntry` and a method table one of
`RexxMethodEntry`. Each row holds a style, the name the row is found by and
an entry; the macros spell the name as the row's first argument is written.
`REXX_TYPED_ROUTINE(name, entry)` is a row of style `ROUTINE_TYPED_STYLE` for
an entry that [the declaration rule](r:native.header.declaration) defines,
`REXX_CLASSIC_ROUTINE(name, entry)` one of style `ROUTINE_CLASSIC_STYLE` for
a classic function, and `REXX_ROUTINE(style, name, entry)` one of the style
given. `REXX_METHOD(name, entry)` and `REXX_METHOD_ENTRY(name, entry)` are
rows of style `METHOD_TYPED_STYLE`. A table ends with `REXX_LAST_ROUTINE()`
or `REXX_LAST_METHOD()`, a row whose fields are all zero.
`REXX_CLASSIC_ROUTINE_PROTOTYPE(name)` declares a classic function, of type
`RexxRoutineHandler`, with the linkage of the language compiled.

r[native.header.package]
`STANDARD_PACKAGE_HEADER` is the first two fields of a package entry, each
with the comma after it: the size of `RexxPackageEntry` and
`REXX_PACKAGE_API_NO`. `NO_VERSION_YET` is a null package version.
`OOREXX_GET_PACKAGE(name)` defines `RexxGetPackage`, with C linkage and
marked `REXXEXPORT`, which answers the address of the package entry
`<name>_package_entry`; the interpreter looks that function up when it loads
the library.

## Strings and names

r[native.header.rxstring]
`MAKERXSTRING(r, p, l)` sets the `RXSTRING` or `CONSTRXSTRING` `r` to the
`l` characters at `p`. `RXSTRPTR(r)` is its pointer, and `RXSTRLEN(r)` its
length, or 0 where the pointer is null. `RXNULLSTRING(r)` is whether the
pointer is null, `RXZEROLENSTRING(r)` whether it is not null and the length
is 0, and `RXVALIDSTRING(r)` whether it is not null and the length is not 0.
Each is an expression.

r[native.header.linkage]
In C++, `BEGIN_EXTERN_C()` opens a block of declarations with C linkage and
`END_EXTERN_C()` closes it; in C both expand to nothing. `NULLOBJECT` is the
null object reference and `NO_CSTRING` the null `CSTRING`, both `NULL`;
`CONSTANT_STRING` is another name of `CSTRING` and `CONSTANT_RXSTRING` of
`CONSTRXSTRING`.
