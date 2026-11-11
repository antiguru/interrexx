---
element: "::ROUTINE"
kind: directive
syntax: "::ROUTINE name [PUBLIC | PRIVATE] [EXTERNAL spec]"
rexxref: "Directives > ::ROUTINE"
deviations: [143]
---

# ::ROUTINE

::ROUTINE declares a routine that the code of its package calls by name, as a
function or with CALL. A routine is private to its package unless PUBLIC
makes it visible to the packages that require it. This page specifies the
routine and its name, the options, the place of a routine in the search for a
called name, what a required package makes visible, and the routines that a
native library supplies. What a routine object does once it exists belongs to
the classes chapter.

## The routine

r[dir.routine.create]
The section of a ::ROUTINE, the clauses up to the next directive, is the body
of a routine, and the directive clause itself is not part of it. A call runs
the body as a unit of its own, as [the unit rule](r:dir.index.unit) says:
its variables and labels are its own, and the arguments are read with `ARG`
and [`USE ARG`](r:instr.use.arg). A routine returns a value with RETURN. A
routine whose body is empty returns none, so a function call of it raises
`SYNTAX 44.1`, and a CALL of it leaves `RESULT` dropped. The routine is
installed before the prolog runs, as [the installation
rule](r:dir.index.install) says, so the prolog and the ACTIVATE method of
a class can call it.

```rexx
say r1(2, 3)
call R1 4, 5
say result
say 'Mixed'(1) mixed(7)
x = 'prolog'
say sees()
call nothing
say symbol('result')
signal on syntax name e
say nothing()
e: say condition('o')~code
exit
::routine r1
  use arg a, b
  return a * b
::routine "Mixed"
  return 'm' arg(1)
::routine sees
  return symbol('x') x
::routine nothing
```

```output
6
20
m 1 m 7
LIT X
LIT
44.1
```

r[dir.routine.name]
The name of a routine is a symbol or a string. [The names
rule](r:dir.index.names) gives its spelling, and the lookup of a routine by
name ignores case, so `::routine "Mixed"` is called by `mixed` and by `MIXED`.
A package declares one routine of a name: a second ::ROUTINE of the name, in
any case, raises `SYNTAX 99.903` when the program is read. A directive whose
name is missing, or is neither a symbol nor a string, raises `19.903`. A
routine and a method of the same name are unrelated.

```rexx
call try '::routine r', '::routine R'
call try '::routine r', '::routine "r"'
call try '::routine r', '::routine "Rr"'
call try '::routine "a b"'
call try '::routine'
call try '::routine +'
call try '::class k', '::routine r', '::class l', '::routine r'
call try '::class k', '::method r', '::routine r'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-routine-name', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::routine r / ::routine R => 99.903
::routine r / ::routine "r" => 99.903
::routine r / ::routine "Rr" => ok
::routine "a b" => ok
::routine => 19.903
::routine + => 19.903
::class k / ::routine r / ::class l / ::routine r => 99.903
::class k / ::method r / ::routine r => ok
```

## Options

r[dir.routine.options]
The keywords after the name are PUBLIC, PRIVATE and EXTERNAL, in any case.
PRIVATE is the default, and writing it is the same as writing no access
keyword. A routine names at most one of PUBLIC and PRIVATE, and a keyword
written twice, a word that is not one of the three, and so a keyword of
[::METHOD](method.md) such as PACKAGE or GUARDED, raise `SYNTAX 25.903`.
EXTERNAL takes an operand and the others do not, as
[the rule for external routines](r:dir.routine.external) says. The errors of
the directive are found when the program is read.

```rexx
call try '::routine r public'
call try '::routine r Private'
call try '::routine r PUBLIC private'
call try '::routine r public public'
call try '::routine r private private'
call try '::routine r package'
call try '::routine r guarded'
call try '::routine r class'
call try '::routine r frob'
exit
try:
  signal on syntax name e
  p = .package~new('spec-dir-routine-opt', .array~of(arg(1)))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
::routine r public => ok
::routine r Private => ok
::routine r PUBLIC private => 25.903
::routine r public public => 25.903
::routine r private private => 25.903
::routine r package => 25.903
::routine r guarded => 25.903
::routine r class => 25.903
::routine r frob => 25.903
```

## Which routine a name finds

r[dir.routine.search]
A routine of the package is found by a function call or a CALL after the
labels of the calling code and the built-in functions, and before the public
routines of the required packages and the external programs. It makes no
difference whether the routine is public. The search that
[CALL](r:instr.call.search) describes therefore has the routines of the
package between the built-in functions and the external routines: a label of
the same name hides a routine when the name is written as a symbol, a name
written as a string skips the labels and finds the routine, and a routine
named like a built-in function is reached only by a string name that is not in
capitals, because the symbol and the capitalised string find the built-in
function first. A program file of the same name is an external routine and is
found only when no routine of the package or of a required package has the
name. Where such a file is looked for is [the places rule](r:env.search.places).

```rexx
f = 'spec_dir_routine_extprog.rex'
call sysfiledelete f
call lineout f, "return 'external program'"
call lineout f
src = .array~of('say length("abcd")', -
  'say "LENGTH"("abcd")', -
  'say "length"("abcd")', -
  'say shadow()', -
  'say "shadow"()', -
  'call shadow', -
  'say result', -
  'say "SHADOW"()', -
  'say spec_dir_routine_extprog()', -
  'exit', -
  'shadow: return "label"', -
  '::routine length', -
  '  return "routine"', -
  '::routine shadow', -
  '  return "routine shadow"', -
  '::routine spec_dir_routine_extprog', -
  '  return "routine"')
p = .package~new('spec-dir-routine-search', src)
src = .array~of('say spec_dir_routine_extprog()')
p = .package~new('spec-dir-routine-search2', src)
call sysfiledelete f
```

```output
4
4
routine
label
routine shadow
label
routine shadow
routine
external program
```

## Public routines

r[dir.routine.public]
A routine with PUBLIC is also found by the code of every package that requires
its package, and a private one only by the code of its own package, that is by
its prolog, its routines and its methods. A public routine runs in its own
package, so it calls the private routines of that package and does not see the
routines of the package that called it. A package finds a name first among its
own routines, then among the public routines of the packages it requires,
taking the first of these in the order of the [::REQUIRES](requires.md)
directives. The public routines that a required package itself found by its
own ::REQUIRES are public routines of that package too, and come after its
own. A routine that is declared in the requiring package, public or private,
hides a public routine of the same name.

```rexx
fa = 'spec-dir-routine-liba.tmp'
fb = 'spec-dir-routine-libb.tmp'
call sysfiledelete fa
call sysfiledelete fb
call lineout fa, '::routine pub public'
call lineout fa, '  return "pubA"'
call lineout fa, '::routine priv'
call lineout fa, '  return "privA"'
call lineout fa, '::routine both public'
call lineout fa, '  return "bothA"'
call lineout fa, '::routine reach public'
call lineout fa, '  return priv() "via" pub()'
call lineout fa, '::routine blind public'
call lineout fa, '  signal on syntax name e'
call lineout fa, '  return mainonly()'
call lineout fa, 'e: return "blind" condition("o")~code'
call lineout fa
call lineout fb, '::requires "'fa'"'
call lineout fb, '::routine pubb public'
call lineout fb, '  return "pubB sees" pub()'
call lineout fb, '::routine both public'
call lineout fb, '  return "bothB"'
call lineout fb
src = .array~of('say pub() both()', -
  'say reach()', -
  'say pubb()', -
  'say blind()', -
  'signal on syntax name e', -
  'say priv()', -
  'e: say condition("o")~code', -
  '::requires "'fa'"', -
  '::requires "'fb'"', -
  '::routine mainonly public', -
  '  return "main"')
p = .package~new('spec-dir-routine-pub', src)
src = .array~of('say both()', '::requires "'fb'"', '::requires "'fa'"')
p = .package~new('spec-dir-routine-pub2', src)
src = .array~of('say pub() both()', '::requires "'fb'"')
p = .package~new('spec-dir-routine-pub3', src)
src = .array~of('say pub() both()', '::requires "'fb'"', -
  '::routine both', '  return "local both"', -
  '::routine pub private', '  return "local pub"')
p = .package~new('spec-dir-routine-pub4', src)
call sysfiledelete fa
call sysfiledelete fb
```

```output
pubA bothA
privA via pubA
pubB sees pubA
blind 43.1
43.1
bothB
pubA bothB
local pub local both
```

## The table of routines

r[dir.routine.table]
Unless an entry or a class of the same name is found first ([the
order](r:env.objects.search.symbols)), the environment symbol `.ROUTINES`
holds a table of the routines that the package declares, public and private,
whose indexes are the routine names in upper case and whose items are routine
objects. It does not hold the routines that the package found through its
::REQUIRES. The symbol exists only in a package that declares a routine; for
any other package it is the symbol's own text. Each package has its own table.
The example runs a routine through the table with the [CALL
method](r:class.routine.call) of the routine object, which the classes chapter
owns.

```rexx
say .routines~items
say .routines~allindexes~sort~makearray~makestring('l', ' ')
say .routines['PRIV']~call(4)
say .routines['MIXED']~call
p = .package~new('spec-dir-routine-table', .array~of('say .routines'))
::routine priv
  return 'priv' arg(1)
::routine "Mixed" public
  return 'm'
::routine pub2 public
  return 2
```

```output
3
MIXED PRIV PUB2
priv 4
m
.ROUTINES
```

## Routines of a native library

r[dir.routine.external]
EXTERNAL declares a routine that a native library supplies, and the directive
has no section. Its operand is a string whose first word is LIBRARY or
REGISTERED, in any case, followed by the name of the library and optionally
one more word, the entry in it. A missing operand, or one that is not a
string, raises `SYNTAX 19.905`, and any other operand, such as a first word
that is neither, a missing library name or more words than the entry, raises
`99.917`. A clause after the directive that is not a null clause raises
`99.939`. A library that cannot be loaded raises `98.903` for LIBRARY and
`90.999` for REGISTERED, and a library that loads but has no routine of the
entry name raises `90.999`. These are raised when the section of the directive
ends, so a clause in the section is refused first; [the rule for external
methods](r:dir.method.external) says how that order goes for the faults of
other directives. The operand of the LIBRARY form is the one that
[::METHOD](r:dir.method.external) takes. How a library is searched for is
[the library rule](r:env.search.library). Which routine of a library the
declaration binds is not specified here, except for the REGISTERED entry that
[the rule of Deviation 143](r:dir.routine.deviation.143) refuses.

```rexx
call try '::routine r external "library spec_no_such_lib_xyz"'
call try '::routine r external "Library spec_no_such_lib_xyz entry"'
call try '::routine r external "registered spec_no_such_lib_xyz"'
call try '::routine r external "registered spec_no_such_lib_xyz entry"'
call try '::routine r external "frob spec_no_such_lib_xyz"'
call try '::routine r external "library"'
call try '::routine r external "library rexxutil nosuchentry_xyz"'
call try '::routine r external library'
call try '::routine r external "library a b c"'
call try '::routine r external ""'
call try '::routine r external'
call try '::routine r external "library spec_no_such_lib_xyz"', 'say 1'
call try '::routine r external "library spec_no_such_lib_xyz"', '::frob'
call try '::frob', '::routine r external "library spec_no_such_lib_xyz"'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-routine-ext', src)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
::routine r external "library spec_no_such_lib_xyz" => 98.903
::routine r external "Library spec_no_such_lib_xyz entry" => 98.903
::routine r external "registered spec_no_such_lib_xyz" => 90.999
::routine r external "registered spec_no_such_lib_xyz entry" => 90.999
::routine r external "frob spec_no_such_lib_xyz" => 99.917
::routine r external "library" => 99.917
::routine r external "library rexxutil nosuchentry_xyz" => 90.999
::routine r external library => 19.905
::routine r external "library a b c" => 99.917
::routine r external "" => 99.917
::routine r external => 19.905
::routine r external "library spec_no_such_lib_xyz" => 99.939
::routine r external "library spec_no_such_lib_xyz" => 98.903
::frob => 99.916
```

## Deviations

r[dir.routine.deviation.143]
A REGISTERED declaration whose library loads, and whose entry is not a
function that the library registers through the classic interface, raises
`SYNTAX 90.999` when the section of the directive ends, as every REGISTERED
declaration that finds no routine does. The entry `RxCalcTan` of the library
`rxmath` is such an entry. ooRexx accepts the directive for that entry and
raises nothing when the package is loaded; the process ends when the routine
is called. For other such entries, `SysSleep` of `rexxutil` among them,
ooRexx raises the same error.
interrexx raises the error where the directive is, so a package with the
declaration never loads. The example is not run against ooRexx, whose process
would end.

```rexx
signal on syntax name e
src = .array~of('call tt 0', -
  '::routine tt external "registered rxmath RxCalcTan"')
p = .package~new('a3zqregdev', src)
say 'loaded'
e: say condition('o')~code
```

```output
90.999
```
