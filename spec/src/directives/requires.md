---
element: "::REQUIRES"
kind: directive
syntax: "::REQUIRES name [LIBRARY | NAMESPACE ns]"
rexxref: "Directives > ::REQUIRES"
---

# ::REQUIRES

::REQUIRES makes the public classes and routines of another package known to
this one. The other package is a Rexx program, which the interpreter installs
and whose prolog it runs before the requiring package goes on, or a native
library that it loads. A NAMESPACE option gives the required package a name
that can qualify the classes and routines it supplies. This page specifies the
directive, what a required package makes visible, the faults that requiring
can raise and the two options. How a name is searched for as a file or as a
library is the subject of the environment chapter, and the examples here name
files that the program writes into the current directory.

## Requiring a package

r[dir.requires.create]
The operand of ::REQUIRES is a symbol or a string that names the package, and
[the names rule](r:dir.index.names) gives the spelling of each. Which of the
two finds a file is a property of the file system, so that a name that has
lower-case letters is best written as a string. Which directories are
searched for the file, and the endings and case that are tried, are [the
required-file rule](r:env.search.required). The interpreter installs the
required package as [installation](r:dir.index.install) describes, which
includes running its prolog, before the requiring package creates a class or
runs its own prolog. An `exit` or `return` in the required prolog ends only
that prolog, and the requiring package goes on. A directive with no operand,
or with an operand that is neither a symbol nor a string, raises `SYNTAX
19.904`, found when the program is read. A name that finds no file raises
`43.901` when the package is installed. The directive takes no code, and any
clause after it that is not a null clause raises `SYNTAX 99.916` when the
program is read.

```rexx
f = 'SPEC_DIR_REQUIRES_SYM.TMP'
call sysfiledelete f
call lineout f, 'say "lib prolog"'
call lineout f, 'exit'
call lineout f, '::routine from_lib public'
call lineout f, '  return "from lib"'
call lineout f
src = .array~of('say "prolog"', 'say from_lib()', -
  '::requires spec_dir_requires_sym.tmp')
p = .package~new('spec-dir-requires-sym', src)
src = .array~of('say "prolog 2"', '::requires "'f'"')
p = .package~new('spec-dir-requires-str', src)
call try '::requires'
call try '::requires ("'f'")'
call try '::requires 1'
call try '::requires "'f'" -- a comment', '', '/* more */', '::requires "'f'"'
call try '::requires "'f'"', 'say 1'
call sysfiledelete f
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-requires-op', src)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
lib prolog
prolog
from lib
prolog 2
::requires => 19.904
::requires ("SPEC_DIR_REQUIRES_SYM.TMP") => 19.904
::requires 1 => 43.901
::requires "SPEC_DIR_REQUIRES_SYM.TMP" -- a comment => ok
::requires "SPEC_DIR_REQUIRES_SYM.TMP" => 99.916
```

r[dir.requires.once]
A package that is required again is not installed again and its prolog does
not run again. The requirement may come from the same package, from another
package that requires it, or from a package that a required package requires,
and the name may differ as long as it finds the same file. Every requirer
sees the one package, so that the order in which a package is first required
fixes the order of the prologs.

```rexx
fa = 'spec-dir-requires-a.tmp'
fb = 'spec-dir-requires-b.tmp'
call sysfiledelete fa
call sysfiledelete fb
call lineout fa, 'say "a prolog"'
call lineout fa
call lineout fb, 'say "b prolog"'
call lineout fb, '::requires "'fa'"'
call lineout fb
src = .array~of('say "prolog"', -
  '::requires "'fb'"', -
  '::requires "'fa'"', -
  '::requires "./'fa'"', -
  '::requires "'fb'"')
p = .package~new('spec-dir-requires-once', src)
call sysfiledelete fa
call sysfiledelete fb
```

```output
a prolog
b prolog
prolog
```

## What becomes visible

r[dir.requires.visible]
A required package makes visible to the requiring package its public classes,
which [the class rule](r:dir.class.visibility) describes, and its public
routines, which [the routine rule](r:dir.routine.public) describes. Its
private classes and routines, and the variables that its prolog set, are not
visible. Those two rules also give the order in which names are looked up
among the package's own and the required ones.

```rexx
f = 'spec-dir-requires-vis.tmp'
call sysfiledelete f
call lineout f, 'secret = "set in the lib prolog"'
call lineout f, '::class shown public'
call lineout f, '::class hidden'
call lineout f, '::routine open public'
call lineout f, '  return "open"'
call lineout f, '::routine closed'
call lineout f, '  return "closed"'
call lineout f
src = .array~of('say .shown~id', -
  'say .hidden', -
  'say open()', -
  'say symbol("secret")', -
  'signal on syntax name e', -
  'say closed()', -
  'e: say condition("o")~code', -
  '::requires "'f'"')
p = .package~new('spec-dir-requires-vis', src)
call sysfiledelete f
```

```output
SHOWN
.HIDDEN
open
LIT
43.1
```

## Faults

r[dir.requires.failure]
A fault in requiring raises a `SYNTAX` condition in the requiring package, and
the installation stops as [the failure rule](r:dir.index.failure) says. A
required program with a fault in its text raises that fault's error, a fault
that its prolog raises is raised again in the requiring package with the same
code, and a required package that requires an absent file raises `43.901`. A
package that requires itself, or that is required by a package that it
requires, directly or through others, raises `SYNTAX 98.952`. The cycle is
found before the prolog of any package in it runs.

```rexx
f = 'spec-dir-requires-bad.tmp'
g = 'spec-dir-requires-deep.tmp'
tx = 'spec-dir-requires-text.tmp'
c1 = 'spec-dir-requires-cyc1.tmp'
c2 = 'spec-dir-requires-cyc2.tmp'
call sysfiledelete f
call sysfiledelete g
call sysfiledelete tx
call sysfiledelete c1
call sysfiledelete c2
call lineout f, 'say "bad prolog"'
call lineout f, 'x = 1 / 0'
call lineout f
call lineout tx, '::frob'
call lineout tx
call lineout g, '::requires "spec-dir-requires-nosuchfile-xyz.rex"'
call lineout g
call lineout c1, 'say "cyc1 prolog"'
call lineout c1, '::requires "'c2'"'
call lineout c1
call lineout c2, 'say "cyc2 prolog"'
call lineout c2, '::requires "'c1'"'
call lineout c2
call try '::requires "'f'"'
call try '::requires "'g'"'
call try '::requires "'tx'"'
call try 'say "main prolog"', '::requires "'c1'"'
call try '::class k', '::frob'
call sysfiledelete f
call sysfiledelete g
call sysfiledelete tx
call sysfiledelete c1
call sysfiledelete c2
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-requires-fail', src)
  say 'ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
bad prolog
::requires "spec-dir-requires-bad.tmp" => 42.3
::requires "spec-dir-requires-deep.tmp" => 43.901
::requires "spec-dir-requires-text.tmp" => 99.916
say "main prolog" / ::requires "spec-dir-requires-cyc1.tmp" => 98.952
::class k / ::frob => 99.916
```

## Libraries

r[dir.requires.library]
With LIBRARY the operand names a native library, which the interpreter loads
instead of a Rexx program, and a library that cannot be loaded raises `SYNTAX
98.903`. A package processes its LIBRARY directives first, wherever they stand
in its text, and only then its other ::REQUIRES directives, so a library that
cannot be loaded raises before any required program has run its prolog. A
library can be required more than once. Whether a library name is case
sensitive, and where libraries are searched for, are [the library
rule](r:env.search.library). The LIBRARY option does not combine with
NAMESPACE: the two together raise `SYNTAX 25.904`.

```rexx
f = 'rq-p.tmp'
call sysfiledelete f
call lineout f, 'say "required prolog"'
call lineout f
call try '::requires "rexxutil" library'
call try '::requires "rexxutil" library', '::requires "rexxutil" LIBRARY'
call try '::requires "'f'"', '::requires "spec_no_such_lib_xyz" library'
call try '::requires "spec_no_such_lib_xyz" library', '::requires "'f'"'
call try '::requires "rexxutil" library namespace a'
call try '::requires "rexxutil" namespace a library'
call sysfiledelete f
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-requires-lib', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::requires "rexxutil" library => ok
::requires "rexxutil" library / ::requires "rexxutil" LIBRARY => ok
::requires "rq-p.tmp" / ::requires "spec_no_such_lib_xyz" library => 98.903
::requires "spec_no_such_lib_xyz" library / ::requires "rq-p.tmp" => 98.903
::requires "rexxutil" library namespace a => 25.904
::requires "rexxutil" namespace a library => 25.904
```

## Namespaces

r[dir.requires.namespace]
NAMESPACE takes a symbol, which becomes a name of the required package. A
class or a routine that the package makes visible is then found by the name
qualified with the namespace and a colon: `ns:name(...)` calls the routine and
`ns:name` is the class, as in `ns:name~new`. Qualified names are symbols, so
the namespace is matched without regard to case. The unqualified name is
looked up as [the routine rule](r:dir.routine.public) and [the class
rule](r:dir.class.visibility) say. When two directives give one namespace
name, the later one names its package. A string, or no symbol after NAMESPACE,
raises `SYNTAX 20.920`, and a keyword other than LIBRARY or NAMESPACE, a
repeated keyword or any further word raises `25.904`, found when the program
is read. The namespace name `REXX` belongs to the interpreter and raises
`99.944`.

```rexx
fa = 'spec-dir-requires-nsa.tmp'
fb = 'spec-dir-requires-nsb.tmp'
call sysfiledelete fa
call sysfiledelete fb
call lineout fa, '::routine hello public'
call lineout fa, '  return "hello A"'
call lineout fa, '::class widget public'
call lineout fa, '::method who'
call lineout fa, '  return "widget A"'
call lineout fa
call lineout fb, '::routine hello public'
call lineout fb, '  return "hello B"'
call lineout fb, '::class widget public'
call lineout fb, '::method who'
call lineout fb, '  return "widget B"'
call lineout fb
src = .array~of('say hello()', -
  'say a:hello() b:hello() A:Hello()', -
  'say .widget~new~who a:widget~new~who b:widget~new~who', -
  '::requires "'fa'" namespace a', -
  '::requires "'fb'" namespace b')
p = .package~new('spec-dir-requires-ns', src)
src = .array~of('say hello() a:hello()', '::requires "'fb'"', -
  '::requires "'fa'" namespace a')
p = .package~new('spec-dir-requires-ns2', src)
src = .array~of('say a:hello()', '::requires "'fa'" namespace a', -
  '::requires "'fb'" namespace a')
p = .package~new('spec-dir-requires-ns3', src)
call try '::requires "'fa'" namespace'
call try '::requires "'fa'" namespace "a"'
call try '::requires "'fa'" namespace a b'
call try '::requires "'fa'" namespace a namespace b'
call try '::requires "'fa'" frob'
call try '::requires "'fa'" namespace rexx'
call try '::requires "'fa'" Namespace a'
call sysfiledelete fa
call sysfiledelete fb
exit
try:
  signal on syntax name e
  p = .package~new('spec-dir-requires-nsop', .array~of(arg(1)))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
hello A
hello A hello B hello A
widget A widget A widget B
hello B hello A
hello B
::requires "spec-dir-requires-nsa.tmp" namespace => 20.920
::requires "spec-dir-requires-nsa.tmp" namespace "a" => 20.920
::requires "spec-dir-requires-nsa.tmp" namespace a b => 25.904
::requires "spec-dir-requires-nsa.tmp" namespace a namespace b => 25.904
::requires "spec-dir-requires-nsa.tmp" frob => 25.904
::requires "spec-dir-requires-nsa.tmp" namespace rexx => 99.944
::requires "spec-dir-requires-nsa.tmp" Namespace a => ok
```

r[dir.requires.namespace.lookup]
A qualified name whose namespace no ::REQUIRES gave raises `SYNTAX 98.987`,
and so does a qualified call of it. A qualified class that the namespace's
package does not make visible raises `98.988`, and a qualified routine that it
does not make visible raises `43.902`. What a package makes visible is what
[the visibility rule](r:dir.requires.visible) says, including the public
classes and routines that it found through its own ::REQUIRES. A qualified
name written as a function call needs the parenthesis to follow the name at
once, and without it the name is read as a class. The CALL instruction does
not accept the qualified form, as [its error rule](r:instr.call.error) says.

```rexx
fa = 'spec-dir-requires-lk.tmp'
call sysfiledelete fa
call lineout fa, '::routine only_a public'
call lineout fa, '  return "only a"'
call lineout fa, '::routine closed'
call lineout fa, '  return "closed"'
call lineout fa, '::class hid'
call lineout fa
fc = 'spec-dir-requires-lkc.tmp'
call sysfiledelete fc
call lineout fc, '::requires "'fa'"'
call lineout fc, '::routine own_c public'
call lineout fc, '  return "own c"'
call lineout fc
src = .array~of('call try "say c:hello()"', -
  'call try "say c:widget"', -
  'call try "say a:nowidget"', -
  'call try "say a:hello"', -
  'call try "say a:hello()"', -
  'call try "say a:only_a()"', -
  'call try "say a:closed()"', -
  'call try "say a:hid"', -
  'call try "say n:own_c()"', -
  'call try "say n:only_a()"', -
  'exit', -
  'try:', -
  '  signal on syntax name e', -
  '  interpret arg(1)', -
  '  return', -
  'e: say arg(1) "=>" condition("o")~code', -
  '  return', -
  '::requires "'fa'" namespace a', -
  '::requires "'fc'" namespace n')
p = .package~new('spec-dir-requires-lookup', src)
call sysfiledelete fa
call sysfiledelete fc
```

```output
say c:hello() => 98.987
say c:widget => 98.987
say a:nowidget => 98.988
say a:hello => 98.988
say a:hello() => 43.902
only a
say a:closed() => 43.902
say a:hid => 98.988
own c
only a
```
