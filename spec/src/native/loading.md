---
element: "Loading"
kind: concept
rexxref: "Directives > ::ROUTINE; Functions > Built-in Functions > RXFUNCADD"
---

# Loading

A native library provides code in one of two forms. A library with a *package
entry* lists its routines and its methods in two tables, and the interpreter
loads it as a whole. A library without one exports *classic functions*, each
of which is registered by name, one at a time, with
[RXFUNCADD](../builtins/rxfuncadd.md) or a `REGISTERED` declaration. The
examples use [the fixture libraries](index.md#the-fixture-libraries):
`specfixture` has a package entry and `specclassic` has not.

## Libraries with a package entry

r[native.loading.package]
A library with a package entry is loaded by a [`::REQUIRES ...
LIBRARY`](r:dir.requires.library) directive, by the `loadLibrary` method of a
package, by an `EXTERNAL "LIBRARY ..."` declaration of a routine or a method,
or by [RXFUNCADD](r:bif.rxfuncadd.library) of any name it provides. From then
on until the program ends, every routine of its routine table can be called by
its name from every package of the program, by CALL or as a function, in any
case and as a quoted name, as [the search order](r:env.search.order) places
it, and [RXFUNCQUERY](r:bif.rxfuncquery.answer) answers `0` for each. Before
the library is loaded, a call of one of its routines raises `SYNTAX 43.1`.

```rexx
signal on syntax name a
say fxhello()
a: say condition('o')~code
signal off syntax
say rxfuncquery('FxHello') rxfuncquery('FxAdd')
say rxfuncadd('FxHello', 'specfixture')
say rxfuncquery('FxHello') rxfuncquery('FxAdd')
say fxhello()
say FXADD(1, 2) 'FxAdd'(3, 4) 'fxadd'(5, 6)
call fxadd 7, 8
say result
```

```output
43.1
1 1
0
0 0
hello from specfixture
3 7 11
15
```

```rexx
say .context~package~loadLibrary('specfixture')
say fxecho('loaded by loadLibrary')
```

```output
1
loaded by loadLibrary
```

r[native.loading.once]
A library is loaded once in a program. Its package entry's loader, if it has
one, runs when the library is first loaded, before any of its routines can be
called and, for a [`::REQUIRES`](r:dir.requires.library) directive, before
the program's first clause. A later directive, `loadLibrary` or RXFUNCADD for
the same library loads nothing and runs nothing: `loadLibrary` answers `1`, and
RXFUNCADD answers `0` for a name the library provides. The fixture's loader
counts its runs in `.local`.

```rexx
say .local~specfixture.loads
say .context~package~loadLibrary('specfixture') .local~specfixture.loads
say rxfuncadd('FxEcho', 'specfixture') .local~specfixture.loads
p = .package~new('n1zqonce', .array~of('::requires "specfixture" LIBRARY'))
say .local~specfixture.loads
::requires 'specfixture' LIBRARY
::requires 'specfixture' LIBRARY
```

```output
1
1 1
0 1
1
```

```rexx
say .local~hasIndex('SPECFIXTURE.LOADS')
say rxfuncadd('FxHello', 'specfixture')
say .local~specfixture.loads
```

```output
0
0
1
```

r[native.loading.version]
A package entry names the oldest interpreter version it works with. A library
whose entry asks for a newer interpreter than the running one is not loaded:
the first attempt to load it raises `SYNTAX 98.982`, whichever of the ways of
[the package rule](r:native.loading.package) makes it. For a `::REQUIRES`
directive of the main program that is before the first clause. After that
refusal the library counts as loaded and provides nothing: a later
`::REQUIRES` directive raises nothing, `loadLibrary` answers `1`, RXFUNCADD
answers `1`, and none of its routines can be called.

```rexx
say 'not reached'
::requires 'specnewer' LIBRARY
```

```output rc=158
```

```rexx
call try '::routine r external "LIBRARY specnewer FxNewer"'
call try '::requires "specnewer" LIBRARY'
say rxfuncadd('FxNewer', 'specnewer') rxfuncquery('FxNewer')
say .context~package~loadLibrary('specnewer')
exit
try:
  signal on syntax name e
  p = .package~new('n1zqnewer', .array~of(arg(1)))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
::routine r external "LIBRARY specnewer FxNewer" => 98.982
::requires "specnewer" LIBRARY => ok
1 1
1
```

```rexx
signal on syntax name b
say .context~package~loadLibrary('specnewer')
b: say condition('o')~code
say rxfuncadd('FxNewer', 'specnewer')
```

```output
98.982
1
```

```rexx
signal on syntax name a
say rxfuncadd('FxNewer', 'specnewer') rxfuncquery('FxNewer')
a: say condition('o')~code
call rxfuncdrop 'FxNewer'
```

```output defect=#614
98.982
```

r[native.loading.table]
The names in a package's routine table are compared without regard to case.
When two rows have names that differ only in case, a call of that name runs
the later row, and an `EXTERNAL` declaration whose entry is that name binds
the earlier one, as [the entry rule](r:native.loading.entry) finds it. The
fixture's table has `FxTwin` before `FXTWIN`.

```rexx
say fxtwin() 'FxTwin'() twin()
::requires 'specfixture' LIBRARY
::routine twin external "LIBRARY specfixture fxtwin"
```

```output
second row second row first row
```

r[native.loading.entry]
The entry that an `EXTERNAL "LIBRARY library entry"` declaration names is
looked for in the routine table for a [`::ROUTINE`](r:dir.routine.external)
directive and in the method table for a [`::METHOD`](r:dir.method.external)
directive, and is matched without regard to case. A declaration without an
entry names the routine's or the method's own name. The two tables are
separate: a routine is not found as a method, which raises `SYNTAX 90.998`,
and a method is not found as a routine, which raises `90.999`, as an entry
that neither table has does. The routine that a declaration binds can be
called by the name that the directive gives it, and the method is the method
of that name of the class. The class methods `loadExternalRoutine` of
[`Routine`](../classes/routine.md) and `loadExternalMethod` of
[`Method`](../classes/method.md) take the same operand.

```rexx
o = .n1zqk~new
say hi() fxecho('own name') (o~me == o) o~fxmessage
say .routine~loadExternalRoutine('zz', 'LIBRARY specfixture FxAdd')~call(1, 2)
say .method~loadExternalMethod('m', 'LIBRARY specfixture FXSELF')~class
call try '::routine r external "LIBRARY specfixture FxSelf"'
call try '::routine r external "LIBRARY specfixture FxNoSuch"'
call try '::class k', '::method m external "LIBRARY specfixture FxHello"'
call try '::class k', '::method fxecho external "LIBRARY specfixture"'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('n1zqentry', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
::routine hi external "LIBRARY specfixture FXHELLO"
::routine fxecho external "LIBRARY specfixture"
::class n1zqk
::method me external "LIBRARY specfixture fxself"
::method fxmessage external "LIBRARY specfixture"
```

```output
hello from specfixture own name 1 FXMESSAGE
3
The Method class
::routine r external "LIBRARY specfixture FxSelf" => 90.999
::routine r external "LIBRARY specfixture FxNoSuch" => 90.999
::class k / ::method m external "LIBRARY specfixture FxHello" => 90.998
::class k / ::method fxecho external "LIBRARY specfixture" => 90.998
```

r[native.loading.classic-row]
A row of a package's routine table may be a classic function, which is called
as [the classic call rule](r:native.loading.classic-call) says for a classic
function that RXFUNCADD registered.

```rexx
say fxclassicjoin('a', , 'c')
::requires 'specfixture' LIBRARY
```

```output defect=#581
FXCLASSICJOIN:3:a,(),c
```

## Classic functions

r[native.loading.classic]
For a library without a package entry, `RXFUNCADD(name, module, procedure)`
registers `name` for the function that the library exports under the name
`procedure`, looked up as it is spelled and, when that finds nothing, in
capitals. An omitted `procedure` is `name`. When the library exports such a
function, the call answers `0` and `name` can be called from then on. When it
does not, the call answers `1`, and `name` is registered nonetheless, as for
[an unknown library](r:bif.rxfuncadd.unknown-library): RXFUNCQUERY answers
`0`, a call raises `SYNTAX 43.1`, and RXFUNCDROP removes it. A registration of
a classic function is the kind that RXFUNCDROP removes, after which a call
raises `43.1`.

```rexx
say rxfuncadd('MyJoin', 'specclassic', 'FxClassicJoin')
say myjoin('x')
say rxfuncadd('MyUpper', 'specclassic', 'FxClassicUpper') myupper('y')
say rxfuncadd('MyLower', 'specclassic', 'fxclassicjoin') rxfuncquery('MyLower')
signal on syntax name a
say mylower('z')
a: say condition('o')~code
say rxfuncdrop('MyJoin') rxfuncdrop('MyUpper') rxfuncdrop('MyLower')
signal on syntax name b
say myjoin('x')
b: say condition('o')~code
```

```output
0
MYJOIN:1:x
0 MYUPPER:1:y
1 0
43.1
0 0 0
43.1
```

```rexx
say rxfuncadd('FxClassicJoin', 'specclassic')
say fxclassicjoin('x')
say rxfuncdrop('FxClassicJoin')
```

```output defect=#579
0
FXCLASSICJOIN:1:x
0
```

r[native.loading.classic-call]
A classic function receives its arguments as strings, in order, and an omitted
argument as no string; the number of arguments does not count omitted ones at
the end. It receives the name of the function in capitals, however the call
spelled it. Its answer is the result of the call. A function that leaves no
result makes a function call raise `SYNTAX 44.1` and a CALL drop the variable
`RESULT`, and a function that refuses the call raises `SYNTAX 40.1`, whose
message names the function as it was registered, in capitals.

```rexx
call rxfuncadd 'MyJoin', 'specclassic', 'FxClassicJoin'
call rxfuncadd 'MyNone', 'specclassic', 'FxClassicNone'
call rxfuncadd 'MyFail', 'specclassic', 'FxClassicFail'
say myjoin(1, , 3) myjoin(1, , ) myjoin(, 2)
call myjoin 'a', 'b'
say result
signal on syntax name a
say mynone()
a: say condition('o')~code
signal off syntax
result = 'set'
call mynone
say symbol('RESULT')
signal on syntax name b
say myfail()
b: say condition('o')~code condition('o')~message
call rxfuncdrop 'MyJoin'
call rxfuncdrop 'MyNone'
call rxfuncdrop 'MyFail'
```

```output
MYJOIN:3:1,(),3 MYJOIN:1:1 MYJOIN:2:(),2
MYJOIN:2:a,b
44.1
LIT
40.1 External routine "MYFAIL" failed.
```

```rexx
call rxfuncadd 'MyJoin', 'specclassic', 'FxClassicJoin'
say 'MyJoin'('a') 'myjoin'()
call rxfuncdrop 'MyJoin'
```

```output defect=#582
MYJOIN:1:a MYJOIN:0:
```

r[native.loading.registered]
A [`::ROUTINE`](r:dir.routine.external) directive with `EXTERNAL "REGISTERED
library entry"` declares a classic function: when the package is installed,
the routine's name is registered for the function `entry` of the library, as
RXFUNCADD of the routine's name, the library and the entry registers it, and
the routine is that function. An omitted entry is the routine's name, which is
in capitals. The routine stays bound after RXFUNCDROP removes the
registration. When the library exports no such function, the directive raises
`SYNTAX 90.999` and leaves the registration behind.

```rexx
say rxfuncquery('n1zqrr')
say n1zqrr(1, 2)
say rxfuncdrop('n1zqrr') rxfuncquery('n1zqrr')
say n1zqrr(3)
::routine n1zqrr external "REGISTERED specclassic FxClassicJoin"
```

```output defect=#580
0
N1ZQRR:2:1,2
0 1
N1ZQRR:1:3
```

```rexx
signal on syntax name e
p = .package~new('n1zqreg', -
  .array~of('::routine n1zqqq external "REGISTERED specclassic FxNoSuch"'))
say 'installed'
e: say condition('o')~code
say rxfuncquery('n1zqqq') rxfuncdrop('n1zqqq') rxfuncquery('n1zqqq')
```

```output
90.999
0 0 1
```
