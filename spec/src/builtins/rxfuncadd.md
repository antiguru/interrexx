---
element: RXFUNCADD
kind: builtin
syntax: "RXFUNCADD(name, module [, procedure])"
rexxref: "Functions > Built-in Functions > RXFUNCADD"
---

# RXFUNCADD

RXFUNCADD asks for a function of a library to be made callable by `name`
like any other function. It answers `0` if the function is callable afterwards
and `1` if it is not. [RXFUNCQUERY](rxfuncquery.md) asks whether a name is
available, and [RXFUNCDROP](rxfuncdrop.md) removes a registration again.

A library is named by `module`. The library the examples use is `rxmath`,
which ships with both interpreters on a Unix-like system and provides the
mathematical functions `RxCalcSqrt`, `RxCalcExp`, `RxCalcLog` and their
relatives. The names of libraries and the way a name is turned into a file
are those of a Unix-like system; those of other platforms are not specified.
Which libraries a program can name beyond `rxmath` is not specified.

> **Note.** A registration can outlive the program that made it, and a
> registration that a program makes can be removed only by that program. The
> examples that leave one behind drop it before they end, so they leave
> nothing behind and do not depend on registrations of other names. Making a
> function of a library such as `rxmath` available is not a
> registration: it lasts until the program ends and nothing removes it, so
> each example picks a function of that library that no other example on
> this page uses. A registration is shared with programs that are not part of
> the examples, and what they register or drop is not specified.

## Arguments

r[bif.rxfuncadd.args]
`name` and `module` are required [strings](r:conv.string.null), and
`procedure` is an optional string. RXFUNCADD does not run as a String method:
sending the message `RXFUNCADD` to a string raises `SYNTAX 97.1`. The
[call rules](r:bif.index.call.count) for the number of arguments hold: fewer
than two raise `SYNTAX 40.3`, more than three raise `SYNTAX 40.4`, and an
omitted `name` or `module` raises `SYNTAX 40.5`. The count is checked before
the omissions, so a call with a fourth argument and no `name` raises
`SYNTAX 40.4`.

```rexx
signal on syntax name a
say rxfuncadd('spec_rxfuncadd_args')
a: say condition('o')~code
signal on syntax name b
say rxfuncadd('spec_rxfuncadd_args', , 'x')
b: say condition('o')~code
signal on syntax name c
say rxfuncadd(, 'rxmath')
c: say condition('o')~code
signal on syntax name d
say rxfuncadd('spec_rxfuncadd_args', 'rxmath', 'x', 'y')
d: say condition('o')~code
signal on syntax name e
say 'spec_rxfuncadd_args'~rxfuncadd('rxmath')
e: say condition('o')~code
signal on syntax name f
say rxfuncadd(, 'rxmath', 'x', 'y')
f: say condition('o')~code
```

```output
40.3
40.5
40.5
40.4
97.1
40.4
```

## The result

r[bif.rxfuncadd.library]
A function that the library provides is made available by giving its name
as `name`, in any mixture of cases, and `module` as the name of the library.
The call answers `0`. The function can be called from then on, and
[RXFUNCQUERY](rxfuncquery.md) answers `0` for it. The library is loaded as a
whole: every function it provides is available afterwards, not only the one
that `name` names. Before the library is loaded, a call of any of its
functions raises `SYNTAX 43.1`. The answer is written as the digit `0` or
`1`, with no sign, no blanks and no decimal point.

```rexx
signal on syntax name a
say rxcalcsqrt(16)
a: say condition('o')~code
signal on syntax name b
say rxcalcexp(0)
b: say condition('o')~code
signal off syntax
say rxfuncquery('RxCalcSqrt') rxfuncquery('RxCalcExp')
say rxfuncadd('rxCALCsqrt', 'rxmath')
say rxfuncquery('RxCalcSqrt') rxfuncquery('RxCalcExp')
say rxcalcsqrt(16) rxcalcsqrt(2, 5)
say length(rxfuncadd('RxCalcSqrt', 'rxmath'))
```

```output
43.1
43.1
1 1
0
0 0
4 1.4142
1
```

r[bif.rxfuncadd.module]
`module` is the bare name of the library, as `rxmath` is. A file name such as
`librxmath.so`, the same name without its extension, `rxmath.so`, and a path,
including the absolute path of the library's file, each name no library: each
is an [unknown library](r:bif.rxfuncadd.unknown-library).

```rexx
say rxfuncadd('RxCalcTan', 'librxmath.so')
say rxfuncadd('RxCalcTan', 'librxmath')
say rxfuncadd('RxCalcTan', 'rxmath.so')
say rxfuncquery('RxCalcTan')
say rxfuncdrop('RxCalcTan') rxfuncdrop('RxCalcTan') rxfuncdrop('RxCalcTan')
say rxfuncquery('RxCalcTan')
say rxfuncadd('RxCalcTan', 'rxmath')
say rxcalctan(0)
```

```output
1
1
1
0
0 0 0
1
0
0
```

r[bif.rxfuncadd.procedure]
When `module` names a library that
[can be found](r:bif.rxfuncadd.unknown-library)
and provides its functions as a package, as `rxmath` does, `procedure` is not
consulted: it may be omitted, may be the null string, or may name anything,
and the answer depends only on `name`. A `name` that the library does not
provide, which is the case of an unknown entry point, answers `1`, and
registers nothing, so the name is not available afterwards. Whether a library
that can be found but does not provide its functions as a package can be
loaded, and what `procedure` means for it, is not specified.

```rexx
say rxfuncadd('RxCalcSinH', 'rxmath', '')
say rxfuncquery('RxCalcSinH')
say rxfuncadd('spec_rxfuncadd_proc', 'rxmath', 'RxCalcLog10')
say rxfuncquery('spec_rxfuncadd_proc')
say rxfuncadd('spec_rxfuncadd_proc', 'rxmath')
say rxfuncquery('spec_rxfuncadd_proc')
```

```output
0
0
1
1
1
1
```

```rexx
say rxfuncadd('RxCalcCosH', 'rxmath', 'no such entry point')
say rxfuncquery('RxCalcCosH')
```

```output
0
0
```

r[bif.rxfuncadd.provided]
A `name` that is available without a registration of its own answers `0`
whatever `module` and `procedure` are, even when they name no library, and the
call changes nothing. Such a name is a function the interpreter provides from
the start, such as `SysSleep` and the loader function `SysLoadFuncs`, or a
function of a library that is already loaded, whether an earlier call loaded
it or a `::requires` directive with the `LIBRARY` keyword did. A name that
only an earlier call registered for an
[unknown library](r:bif.rxfuncadd.unknown-library) is not such a name.

```rexx
say rxfuncquery('SysSleep')
say rxfuncadd('SysSleep', 'no_such_library_spec', 'no_such_entry')
say rxfuncquery('SysSleep')
say rxfuncadd('SysLoadFuncs', 'no_such_library_spec')
call SysLoadFuncs
call rxfuncadd 'RxCalcPower', 'rxmath'
say rxfuncadd('RxCalcPower', 'no_such_library_spec')
say rxcalcpower(2, 10)
```

```output
0
0
0
0
0
1024
```

```rexx
say rxfuncquery('RxCalcLog')
say rxfuncadd('RxCalcLog', 'no_such_library_spec')
say rxfuncdrop('RxCalcLog') rxfuncquery('RxCalcLog')
say rxcalclog(1)
::requires 'rxmath' LIBRARY
```

```output
0
0
1 0
0
```

r[bif.rxfuncadd.unknown-library]
A `module` that names no library the interpreter can load, because there is
none of that name or because the name is spelled with another case, makes the
call answer `1`, and `name`, unless it is already available, is
nonetheless registered: [RXFUNCQUERY](rxfuncquery.md) answers `0` for it
afterwards. The function cannot be called, and the attempt raises `SYNTAX
43.1`. The registration is the kind that
[RXFUNCDROP](rxfuncdrop.md) removes. When another program holds a
registration of the name, [the repeat rule](r:bif.rxfuncadd.repeat) says what
the call does instead. What it records of `procedure` is not observable and
not specified.

```rexx
say rxfuncquery('spec_rxfuncadd_gone')
say rxfuncadd('spec_rxfuncadd_gone', 'no_such_library_spec', 'entry')
say rxfuncquery('spec_rxfuncadd_gone')
signal on syntax name a
say spec_rxfuncadd_gone(1)
a: say condition('o')~code
signal off syntax
say rxfuncadd('spec_rxfuncadd_case', 'RXMATH', 'RxCalcSqrt')
say rxfuncquery('SPEC_RXFUNCADD_CASE')
say rxfuncdrop('spec_rxfuncadd_gone') rxfuncdrop('spec_rxfuncadd_case')
say rxfuncquery('spec_rxfuncadd_gone') rxfuncquery('spec_rxfuncadd_case')
```

```output
1
1
0
43.1
1
0
0 0
1 1
```

r[bif.rxfuncadd.repeat]
Within one program, every call that answers `1` for an unknown library adds a
registration of its own, even when an earlier call registered the same name for
the same library: the registrations pile up, and
[RXFUNCDROP](rxfuncdrop.md) removes them one call at a time. A name already
registered does not change the answer of a later call: it is `1` for an
unknown library, and `0` for a library that provides the name, which makes the
name available through the library and leaves the earlier registrations to be
dropped. When another program holds a registration of the name, a call for an
unknown library answers `1` as well, and a call of RXFUNCDROP by this program
answers `1` and leaves the name available.

```rexx
say rxfuncadd('spec_rxfuncadd_twice', 'no_such_library_spec', 'p')
say rxfuncadd('spec_rxfuncadd_twice', 'no_such_library_spec', 'p')
say rxfuncdrop('spec_rxfuncadd_twice') rxfuncquery('spec_rxfuncadd_twice')
say rxfuncdrop('spec_rxfuncadd_twice') rxfuncquery('spec_rxfuncadd_twice')
say rxfuncdrop('spec_rxfuncadd_twice') rxfuncquery('spec_rxfuncadd_twice')
say rxfuncadd('RxCalcArcCos', 'no_such_library_spec', 'p')
say rxfuncadd('RxCalcArcCos', 'rxmath')
say rxfuncdrop('RxCalcArcCos') rxfuncdrop('RxCalcArcCos') rxfuncquery('RxCalcArcCos')
```

```output
1
1
0 0
0 1
1 1
1
0
0 1 0
```
