---
element: "Search"
kind: concept
rexxref: "Functions > Search Order; Functions > Search Order > Locating External Rexx Files; Directives > ::REQUIRES"
---

# Search

A program names code that lives outside it in four ways: a call of a routine
that the program does not define, a `::REQUIRES` directive, a program name on
the command line, and a library name. This page specifies how each is found:
the places that are searched and their order, the extensions that are added to
a name, how the case of a name matters, what a repeated search finds again,
and what a program called by name leaves behind in the program that called it.

The page states the behavior of a Linux host. The names of the places depend
on the host: `/` separates the parts of a path, `:` separates the entries of
`REXX_PATH` and `PATH`, and a file name is case sensitive.

> **Note.** The test harness runs each example from one working directory.
> interrexx runs it without a program file. ooRexx runs it as a file with the
> extension `.rex` in that directory. So no example depends on the directory
> or the extension of its own program: where one matters, the example writes a
> calling file of its own. An example that needs a directory builds one, and
> uses names that no host has. The examples set `REXX_PATH` and
> `PATH` with `VALUE`, change the current directory with `DIRECTORY`, and put
> them back. The rules about [the `rexx` command](r:env.search.program) are
> tested by the recorded runs of the command in
> `rust/crates/interrexx/tests/rexx_command/cases`, which run interrexx and
> ooRexx on the same words from a directory of known programs and compare what
> they write and their status; each such rule is cited there.

## Which names are searched

r[env.search.kinds]
These situations search for what a name stands for, and `newFile` takes a file
without a search. A call of a routine that nothing in the program, no required
package and no built-in function answers to searches for a file, a registered
function and a macro, in the order of [the order rule](r:env.search.order). A
`::REQUIRES` directive without `LIBRARY`, and the methods `new` and
`loadPackage` of `Package` when they are given only a name, search for a file
of Rexx code as [the required-file rule](r:env.search.required) says.
`findProgram` of `Package` searches for a file in the way of a routine call
([its rule](r:class.package.findprogram)). The command `rexx` searches for the
program it is told to run, as [the program rule](r:env.search.program) says.
`newFile` of `Method` and of `Routine` takes the file as it is named and
searches for nothing ([its rule](r:env.search.newfile)). A library name is
searched as [the library rule](r:env.search.library) says. A search that finds
nothing is the error of the situation: `SYNTAX 43.1` for a call (the name is
shown in upper case when it was written as a symbol, [as `CALL`
says](r:instr.call.search)), `43.901` for a `::REQUIRES` and for `new` and
`loadPackage` ([the file rule](r:class.package.new.file)), and `3.901` for the
command ([its rule](r:env.invocation.notfound)).

```rexx
call try 'call a3zqkindsx'
call try 'r = a3zqkindsx()'
call try 'r = .package~new("a3zqkindsx")'
call try 'r = .package~new("a3zqk1", .array~of("::requires ""a3zqkindsx"""))'
call try 'r = .package~new("a3zqk2", "nop")~loadPackage("a3zqkindsx")'
call try 'r = .routine~newFile("a3zqkindsx")'
say .package~new('a3zqk3', 'nop')~findProgram('a3zqkindsx')~isNil
say .package~new('a3zqk4', 'nop')~loadLibrary('a3zqkindsx')
exit
try: signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
call a3zqkindsx => 43.1
r = a3zqkindsx() => 43.1
r = .package~new("a3zqkindsx") => 43.901
r = .package~new("a3zqk1", .array~of("::requires ""a3zqkindsx""")) => 43.901
r = .package~new("a3zqk2", "nop")~loadPackage("a3zqkindsx") => 43.901
r = .routine~newFile("a3zqkindsx") => 3.1
1
0
```

## The places

r[env.search.places]
A name that is not a path, as [the next rule](r:env.search.names) says, is
looked for in these places, in this order: the directory of the file that
holds the code that makes the call; the current directory; each directory that
the environment variable `REXX_PATH` lists, in its order; and each directory
that `PATH` lists, in its order. The directory of a caller is skipped when the
caller has no file: the code of a package made from source text and the string
that `rexx -e` runs have none, and the command `rexx` has no caller at all.
The code of a routine of a file has the directory of that file, wherever it is
called from, so a routine that a required package supplies looks first beside
the package's own file; [the caller rule](r:env.search.caller) gives an
example. `REXX_PATH` and `PATH` are read at the time of each search, and list
their directories separated by colons; a directory that does not exist or has
no such file adds nothing, a directory that is not absolute is read from the
current directory, and a variable that is not set lists none. In the example,
the files of the same name stand in the places one after another, and each
call finds the first place that has a file; the last lines make the entries of
`REXX_PATH` relative to the current directory.

```rexx
d = directory()
do n over .array~of('a3zqpl', 'a3zqpl/rp1', 'a3zqpl/rp2', 'a3zqpl/pp')
  call sysmkdir n
end
oldrp = value('REXX_PATH', d'/a3zqpl/nonesuch:'d'/a3zqpl/rp1:'d'/a3zqpl/rp2', -
  'ENVIRONMENT')
oldp = value('PATH', , 'ENVIRONMENT')
call value 'PATH', d'/a3zqpl/pp:'oldp, 'ENVIRONMENT'
call show 'none'
call put 'a3zqpl/pp/a3zqwhere.rex', 'PATH'
call show 'PATH'
call put 'a3zqpl/rp2/a3zqwhere.rex', 'REXX_PATH second'
call show 'REXX_PATH 2'
call put 'a3zqpl/rp1/a3zqwhere.rex', 'REXX_PATH first'
call show 'REXX_PATH 1'
call put 'a3zqwhere.rex', 'current directory'
call show 'current'
call sysfiledelete 'a3zqwhere.rex'
call directory 'a3zqpl'
call value 'REXX_PATH', 'rp2:rp1', 'ENVIRONMENT'
call show 'relative'
call directory d
call value 'REXX_PATH', oldrp, 'ENVIRONMENT'
call value 'PATH', oldp, 'ENVIRONMENT'
do n over .array~of('pp', 'rp1', 'rp2')
  call sysfiledelete 'a3zqpl/'n'/a3zqwhere.rex'
  call sysrmdir 'a3zqpl/'n
end
call sysrmdir 'a3zqpl'
exit
put: call sysfiledelete arg(1)
  call lineout arg(1), 'return "'arg(2)'"'
  call lineout arg(1)
  return
show: signal on syntax name e
  call a3zqwhere
  say arg(1)':' result
  return
e: say arg(1)':' condition('o')~code
  return
```

```output
none: 43.1
PATH: PATH
REXX_PATH 2: REXX_PATH second
REXX_PATH 1: REXX_PATH first
current: current directory
relative: REXX_PATH second
```

## Names that are paths

r[env.search.names]
A name that begins with a slash is an absolute path, and is looked for there
and nowhere else. A name that begins with `./` or `../` is a path from the
current directory, and is looked for there and nowhere else: neither the
directory of the caller nor `REXX_PATH` nor `PATH` is tried for it. Any other
name, one with slashes inside it such as `sub/name` included, is joined to
each place of [the places rule](r:env.search.places) in turn. The extensions
and the case of [the following rules](r:env.search.extension) apply to the
name of every kind.

```rexx
d = directory()
call sysmkdir 'a3zqnm'
call sysmkdir 'a3zqnm/rp'
call sysmkdir 'a3zqnm/rp/sub'
call put 'a3zqnm/rp/a3zqplain.rex', 'plain'
call put 'a3zqnm/rp/sub/a3zqdeep.rex', 'deep'
oldrp = value('REXX_PATH', d'/a3zqnm/rp', 'ENVIRONMENT')
call show 'plain', 'a3zqplain'
call show 'dot', './a3zqplain'
call show 'absolute', d'/a3zqnm/rp/a3zqplain'
call show 'sub', 'sub/a3zqdeep'
call show 'dot sub', './sub/a3zqdeep'
call directory 'a3zqnm/rp/sub'
call show 'dotdot', '../a3zqplain'
call show 'dotdot sub', '../sub/a3zqdeep'
call directory d
call value 'REXX_PATH', oldrp, 'ENVIRONMENT'
call sysfiledelete 'a3zqnm/rp/sub/a3zqdeep.rex'
call sysfiledelete 'a3zqnm/rp/a3zqplain.rex'
call sysrmdir 'a3zqnm/rp/sub'
call sysrmdir 'a3zqnm/rp'
call sysrmdir 'a3zqnm'
exit
put: call sysfiledelete arg(1)
  call lineout arg(1), 'return "'arg(2)'"'
  call lineout arg(1)
  return
show: signal on syntax name e
  call (arg(2))
  say arg(1)':' result
  return
e: say arg(1)':' condition('o')~code
  return
```

```output
plain: plain
dot: 43.1
absolute: plain
sub: deep
dot sub: 43.1
dotdot: plain
dotdot sub: deep
```

## Extensions

r[env.search.extension]
A name that has a period in its last part, the part after the last slash,
names a file, and no extension is added to it. Any other name is tried with
each of these endings in turn, and the first ending for which a file is found
in some place ends the search: the endings come before the places, so that a
file that has an earlier ending in a later place is found before a file that
has a later one in an earlier place. The endings are, in order: `.cls`, only
for the [required-file search](r:env.search.required); the extension of the
file that holds the calling code, from the last period of the last part of the
file's name, when that part has one; `.REX`; `.rex`; and no ending at all. A
call, `findProgram` and the command `rexx` leave the `.cls` out, and the
command has no calling file. In the example, a file `a3zqxc.xyz` calls a
routine, and the files of that name are taken away one by one: the routine is
found with the ending `xyz`, then `REX`, then `rex`, then with none. The file
with the ending `.cls` stays to the end and is never found by the call. A
second caller, `a3zqxd`, has no extension, and so has no ending of its own to
add: it finds `.REX` first, and the file that has the extension `xyz` is not
tried for it. A name with a period, `a3zqpd.x`, finds the file of that name
and not `a3zqpd.x.REX` or `a3zqpd.x.rex`, and finds nothing when that file is
gone.

```rexx
call put 'a3zqex.cls', 'cls'
call put 'a3zqex.xyz', 'xyz'
call put 'a3zqex.REX', 'REX'
call put 'a3zqex.rex', 'rex'
call put 'a3zqex', 'none'
call lineout 'a3zqxc.xyz', 'return a3zqex()'
call lineout 'a3zqxc.xyz'
call lineout 'a3zqxd', 'return a3zqex()'
call lineout 'a3zqxd'
say 'with extension:' a3zqxc.xyz()
say 'without:' a3zqxd()
call sysfiledelete 'a3zqex.xyz'
say 'with extension:' a3zqxc.xyz()
say 'without:' a3zqxd()
call sysfiledelete 'a3zqex.REX'
say 'with extension:' a3zqxc.xyz()
call sysfiledelete 'a3zqex.rex'
say 'with extension:' a3zqxc.xyz()
call sysfiledelete 'a3zqex'
signal on syntax name e
say 'with extension:' a3zqxc.xyz()
e: say condition('o')~code
call put 'a3zqpd.x', 'as named'
call put 'a3zqpd.x.REX', 'with REX'
call put 'a3zqpd.x.rex', 'with rex'
say 'period:' 'a3zqpd.x'()
call sysfiledelete 'a3zqpd.x'
signal on syntax name e2
say 'period:' 'a3zqpd.x'()
e2: say 'period:' condition('o')~code
call sysfiledelete 'a3zqpd.x.REX'
call sysfiledelete 'a3zqpd.x.rex'
call sysfiledelete 'a3zqex.cls'
call sysfiledelete 'a3zqxc.xyz'
call sysfiledelete 'a3zqxd'
exit
put: call sysfiledelete arg(1)
  call lineout arg(1), 'return "'arg(2)'"'
  call lineout arg(1)
  return
```

```output
with extension: xyz
without: REX
with extension: REX
without: REX
with extension: rex
with extension: none
43.1
period: as named
period: 43.1
```

## Case

r[env.search.case]
A file name is tried in two forms, the name as it was written and the whole
name in lower case, directories included; the extension that is added is never
changed. The two forms come after the extensions and before the places, so
that for each ending the written form is tried in every place before the
lower-case form is tried in the first. A call written as a symbol is written
in upper case ([the call rule](r:instr.call.search)), and so finds a file of
the name in lower case. A name that is spelled in neither form as the file is,
`a3zqmixed` for a file `A3zqMixed.rex` or `a3zqUp/A3zqIn` for a file
`a3zqUp/a3zqin.rex`, finds nothing. In the example, a caller that has no
extension of its own, so that it adds no ending of its own, calls the names.
`A3ZQCS` finds the file of its lower-case form and `A3zqMixed` the file of its
own spelling. `A3zqOrd` finds the file `a3zqord.REX`, because the ending
`.REX` is tried before `.rex`, although the file `A3zqOrd.rex` is of the
written form. A directory is lower-cased with the name. For `A3ZQCR` the
written form is found in `REXX_PATH` although the lower-case form is in the
current directory, and for `A3ZQCW` the written form in the current directory
is found before the lower-case form in `REXX_PATH`.

```rexx
d = directory()
call sysmkdir 'a3zqsd'
call sysmkdir 'a3zqUp'
call sysmkdir 'a3zqcr'
call put 'a3zqcs.rex', 'lower'
call put 'A3zqMixed.rex', 'mixed'
call put 'a3zqord.REX', 'lower REX'
call put 'A3zqOrd.rex', 'mixed rex'
call put 'a3zqsd/a3zqinsd.rex', 'in the lower directory'
call put 'a3zqUp/a3zqin.rex', 'in a mixed directory'
call put 'a3zqcr/A3ZQCR.rex', 'upper in REXX_PATH'
call put 'a3zqcr.rex', 'unused'
call put 'a3zqcr/a3zqcw.rex', 'lower in REXX_PATH'
call put 'A3ZQCW.rex', 'upper in the current directory'
call lineout 'a3zqcl', 'parse arg n'
call lineout 'a3zqcl', 'signal on syntax name e'
call lineout 'a3zqcl', 'call (n)'
call lineout 'a3zqcl', 'return result'
call lineout 'a3zqcl', 'e: return condition("o")~code'
call lineout 'a3zqcl'
old = value('REXX_PATH', d'/a3zqcr', 'ENVIRONMENT')
say a3zqcl('A3ZQCS')
say a3zqcl('A3zqMixed')
say a3zqcl('a3zqmixed')
say a3zqcl('A3zqOrd')
say a3zqcl('A3ZQSD/A3ZQINSD')
say a3zqcl('a3zqUp/A3zqIn')
say a3zqcl('A3ZQCR')
say a3zqcl('A3ZQCW')
call value 'REXX_PATH', old, 'ENVIRONMENT'
do f over .array~of('a3zqcs.rex', 'A3zqMixed.rex', 'a3zqord.REX', -
    'A3zqOrd.rex', 'a3zqsd/a3zqinsd.rex', 'a3zqUp/a3zqin.rex', -
    'a3zqcr/A3ZQCR.rex', 'a3zqcr.rex', -
    'a3zqcr/a3zqcw.rex', 'A3ZQCW.rex', 'a3zqcl')
  call sysfiledelete f
end
call sysrmdir 'a3zqsd'
call sysrmdir 'a3zqUp'
call sysrmdir 'a3zqcr'
exit
put: call sysfiledelete arg(1)
  call lineout arg(1), 'return "'arg(2)'"'
  call lineout arg(1)
  return
```

```output
lower
mixed
43.1
lower REX
in the lower directory
43.1
upper in REXX_PATH
upper in the current directory
```

## The directory of the caller

r[env.search.caller]
The directory that comes first in [the places](r:env.search.places) belongs to
the file that holds the code of the call, and not to the program that started
the run or to the code that called that file. A file that was found in a place
and called as a routine looks first beside itself, a file that it calls looks
beside that file in turn, and a directory in a name such as `sub/name` is
joined to the directory of the file that makes the call. In the example, the
file `a3zqcdm.rex` stands in a directory that is not the current directory,
and finds a file beside itself that the current directory lacks. A file
`a3zqtwo.rex` stands both beside it and in the current directory, and it finds
the one beside it, while the program of the example finds the other. It calls
a file in a directory below its own, and that file finds the files of its own
directory and not the files beside its caller. The program of the example,
which stands in the current directory, does not find the file of `a3zqsib`.

```rexx
call sysmkdir 'a3zqcd'
call sysmkdir 'a3zqcd/a3zqsub'
call put 'a3zqcd/a3zqsib.rex', 'beside the caller'
call put 'a3zqcd/a3zqtwo.rex', 'the directory of the caller'
call put 'a3zqtwo.rex', 'the current directory'
call put 'a3zqcd/a3zqsub/a3zqunder.rex', 'beside the callee'
f = 'a3zqcd/a3zqcdm.rex'
call sysfiledelete f
call lineout f, 'return a3zqsib() "/" a3zqtwo() "/" "a3zqsub/a3zqdown"()'
call lineout f
f = 'a3zqcd/a3zqsub/a3zqdown.rex'
call sysfiledelete f
call lineout f, 'return a3zqunder() "/" safe("a3zqsib")'
call lineout f, '::routine safe'
call lineout f, '  signal on syntax name e'
call lineout f, '  call (arg(1))'
call lineout f, '  return result'
call lineout f, 'e: return condition("o")~code'
call lineout f
say 'a3zqcd/a3zqcdm.rex'()
say a3zqtwo()
signal on syntax name e
say a3zqsib()
e: say condition('o')~code
call sysfiledelete 'a3zqcd/a3zqsub/a3zqdown.rex'
call sysfiledelete 'a3zqcd/a3zqsub/a3zqunder.rex'
call sysfiledelete 'a3zqcd/a3zqcdm.rex'
call sysfiledelete 'a3zqcd/a3zqsib.rex'
call sysfiledelete 'a3zqcd/a3zqtwo.rex'
call sysfiledelete 'a3zqtwo.rex'
call sysrmdir 'a3zqcd/a3zqsub'
call sysrmdir 'a3zqcd'
exit
put: call sysfiledelete arg(1)
  call lineout arg(1), 'return "'arg(2)'"'
  call lineout arg(1)
  return
```

```output
beside the caller / the directory of the caller / beside the callee / 43.1
the current directory
43.1
```

## Files of Rexx code that are required

r[env.search.required]
A `::REQUIRES` directive without `LIBRARY`, and `new` and `loadPackage` of
`Package` given only a name, search for a file by [the
places](r:env.search.places), [the names](r:env.search.names), [the
extensions](r:env.search.extension) and [the case](r:env.search.case) of the
rules above, with these differences. The first ending is `.cls`, so that a
file `name.cls` is found before every file `name.REX`, `name.rex` and `name`.
The place that comes first is the directory of the file of the requiring
package for a `::REQUIRES`, and of the file of the receiver for `loadPackage`;
`new` of `Package` has no caller, so it looks in the current directory and
then in the other places, and `findProgram`, which has the directory of its
receiver, has no `.cls` ending. A package that was made from source text has
no file, so no directory. A name with a period has no ending, so that
`name.REX` finds neither `name.REX.cls` nor `name.REX.rex`. The package that
is found is installed as [the file rule](r:class.package.new.file) says, and
is not read again when a file of that name was loaded before. In the example,
the file `a3zqrd/a3zqrreq.rex` requires the file `a3zqrsib` that stands beside
it and not in the current directory, where the file is `a3zqrsib.cls` and not
the file `a3zqrsib.rex`. A package that is made from the file finds the file
by `loadPackage` and by `findProgram` but not by `new`, and `findProgram` does
not take the `.cls` file. A source package finds no file beside it. A file
`a3zqpd.x.cls` stands in the current directory without the file `a3zqpd.x`,
and `new` does not find it for the name `a3zqpd.x`.

```rexx
call sysmkdir 'a3zqrd'
call put 'a3zqrd/a3zqrsib.cls', 'say "sibling cls ran"'
call put 'a3zqrd/a3zqrsib.rex', 'say "sibling rex ran"'
call put 'a3zqrd/a3zqrreq.rex', '::requires "a3zqrsib"'
pk = .package~new('a3zqrd/a3zqrreq.rex')
say base(pk~findProgram('a3zqrsib'))
say base(pk~loadPackage('a3zqrsib')~name)
call try '.package~new("a3zqrsib.rex")'
call put 'a3zqpd.x.cls', 'say "ran"'
call try '.package~new("a3zqpd.x")'
call try '.package~new("a3zqsrc", .array~of("::requires ""a3zqrsib.rex"""))'
call try '.package~new("a3zqsrc", "nop")~loadPackage("a3zqrsib.rex")'
call sysfiledelete 'a3zqrd/a3zqrsib.cls'
call sysfiledelete 'a3zqpd.x.cls'
call sysfiledelete 'a3zqrd/a3zqrsib.rex'
call sysfiledelete 'a3zqrd/a3zqrreq.rex'
call sysrmdir 'a3zqrd'
exit
base: return substr(arg(1), lastpos('/', arg(1)) + 1)
put: call sysfiledelete arg(1)
  call lineout arg(1), arg(2)
  call lineout arg(1)
  return
try: signal on syntax name e
  interpret 'r =' arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
sibling cls ran
a3zqrsib.rex
a3zqrsib.cls
.package~new("a3zqrsib.rex") => 43.901
.package~new("a3zqpd.x") => 43.901
.package~new("a3zqsrc", .array~of("::requires ""a3zqrsib.rex""")) => 43.901
.package~new("a3zqsrc", "nop")~loadPackage("a3zqrsib.rex") => 43.901
```

## The program of the command

r[env.search.program]
The program that the command `rexx` is told to run is looked for in [the
places](r:env.search.places) with no directory of a caller: the current
directory, then the directories of `REXX_PATH`, then those of `PATH`. The
names of [the names rule](r:env.search.names) and the case of [the case
rule](r:env.search.case) hold, and the endings are `.REX`, `.rex` and none, in
that order and with no `.cls`: an ending is tried in every place before the
next ending is tried, so that `name.REX` of `REXX_PATH` is run before
`name.rex` of the current directory, and a file with no ending is run only
when no file with an ending is found. A name that finds a directory only, or
nothing, is [the not-found error](r:env.invocation.notfound). The program is
named as [the program rule](r:env.invocation.program) says. The cases in the
test of the command show each of these with a file of the kind in each place;
the string of `rexx -e` has no file and so no directory of a caller, and a
program that the command runs searches the directory of its own file first for
the routines that it calls.

## A file that is taken as it is

r[env.search.newfile]
`newFile` of `Method` and of `Routine` ([the method
rule](r:class.method.newfile), [the routine rule](r:class.routine.newfile))
takes the file name as it was given, from the current directory when it is not
absolute. It adds no ending, it does not try the lower-case form, and it does
not look in the directory of a caller, in `REXX_PATH` or in `PATH`; a file
that is not found there raises `SYNTAX 3.1`. In the example, the file
`a3zqnfd/a3zqnf.rex` is found by `REXX_PATH` for `new` of `Package`, but
`newFile` finds it only by the name that gives its directory and its ending.

```rexx
d = directory()
call sysmkdir 'a3zqnfd'
call lineout 'a3zqnfd/a3zqnf.rex', 'nop'
call lineout 'a3zqnfd/a3zqnf.rex'
old = value('REXX_PATH', d'/a3zqnfd', 'ENVIRONMENT')
call try '.package~new("a3zqnf")'
call try '.routine~newFile("a3zqnf")'
call try '.method~newFile("a3zqnf.rex")'
call try '.routine~newFile("a3zqnfd/a3zqnf")'
call try '.routine~newFile("a3zqnfd/a3zqnf.rex")'
call try '.method~newFile("A3ZQNFD/A3ZQNF.REX")'
call try '.method~newFile("a3zqnfd/a3zqnf.rex")'
call value 'REXX_PATH', old, 'ENVIRONMENT'
call sysfiledelete 'a3zqnfd/a3zqnf.rex'
call sysrmdir 'a3zqnfd'
exit
try: signal on syntax name e
  interpret 'r =' arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
.package~new("a3zqnf") => ok
.routine~newFile("a3zqnf") => 3.1
.method~newFile("a3zqnf.rex") => 3.1
.routine~newFile("a3zqnfd/a3zqnf") => 3.1
.routine~newFile("a3zqnfd/a3zqnf.rex") => ok
.method~newFile("A3ZQNFD/A3ZQNF.REX") => 3.1
.method~newFile("a3zqnfd/a3zqnf.rex") => ok
```

## What a search finds again

r[env.search.fresh]
A routine that a call finds in a file is searched for again at every call, and
the file is read again: a file that was changed is run as it now is, a file
that was removed is not found, and a file that did not exist is found when it
appears. The file makes a package of its own for each call, so that two calls
of one file answer two packages, both with the absolute name of the file ([the
file rule](r:class.package.new.file)). A package that the file requires is
another matter: [it is installed once](r:dir.requires.once) for all the
packages that require it, so that its prolog runs at the first call only. In
the example, `a3zqfrsh.rex` is changed between the calls, `a3zqfrsp.rex`
returns its package and requires a file with a prolog, and the packages are
not the same although they have one name.

```rexx
f = 'a3zqfrsh.rex'
call put f, 'v1'
say a3zqfrsh()
call put f, 'v2'
say a3zqfrsh()
call sysfiledelete f
call try
call put f, 'v3'
say a3zqfrsh()
call sysfiledelete f
call lineout 'a3zqfrsl.rex', 'say "prolog of the required file"'
call lineout 'a3zqfrsl.rex'
call lineout 'a3zqfrsp.rex', 'return .context~package'
call lineout 'a3zqfrsp.rex', '::requires "a3zqfrsl.rex"'
call lineout 'a3zqfrsp.rex'
p1 = a3zqfrsp()
p2 = a3zqfrsp()
say (p1 == p2) (p1~name == p2~name) (p1~name == qualify('a3zqfrsp.rex'))
call sysfiledelete 'a3zqfrsl.rex'
call sysfiledelete 'a3zqfrsp.rex'
exit
put: call sysfiledelete arg(1)
  call lineout arg(1), 'return "'arg(2)'"'
  call lineout arg(1)
  return
try: signal on syntax name e
  say a3zqfrsh()
  return
e: say condition('o')~code
  return
```

```output
v1
v2
43.1
v3
prolog of the required file
0 1 1
```

## What a called program leaves behind

r[env.search.import]
A program that a call found as a file adds its public routines and public
classes to the package that made the call, as a `::REQUIRES` of that package
would: the code of that package can use them from then on, the code of other
packages cannot, and its private routines and classes are not added. Every
call that finds a file adds those of its routines and classes whose names the
package does not have yet. A name it already has is not replaced, so the
package keeps the first. In the example the file is rewritten between the
calls, and gains a public routine: the public routine that both versions have
is still the first one, and the new one is added; a package that is made from
source text does not find it, and the private routine is not found.

```rexx
f = 'a3zqimp.rex'
call put 'v1'
say a3zqimp() a3zqimppub()
call put 'v2'
say a3zqimp() a3zqimppub() a3zqimpnew()
say .context~package~importedRoutines~items .context~package~importedClasses~items
say .a3zqimpk~id
call try 'a3zqimppriv()'
call try '.package~new("a3zqimpo", "return a3zqimppub()")~name'
call sysfiledelete f
exit
put: call sysfiledelete f
  call lineout f, 'return "main-'arg(1)'"'
  call lineout f, '::routine a3zqimppub public'
  call lineout f, '  return "pub-'arg(1)'"'
  call lineout f, '::routine a3zqimppriv'
  call lineout f, '  return "priv"'
  if arg(1) = 'v2' then do
    call lineout f, '::routine a3zqimpnew public'
    call lineout f, '  return "new-v2"'
    end
  call lineout f, '::class a3zqimpk public'
  call lineout f
  return
try: signal on syntax name e
  interpret 'say' arg(1)
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
main-v1 pub-v1
main-v2 pub-v1 new-v2
2 1
A3ZQIMPK
a3zqimppriv() => 43.1
.package~new("a3zqimpo", "return a3zqimppub()")~name => 43.1
```

## Macros and registered functions

r[env.search.macro]
A macro is a routine that was added to the macrospace with a name and an
order, `B` for before and `A` for after (the RexxUtil functions
`SysAddRexxMacro` and `SysDropRexxMacro` add and remove one). Its name is
matched without regard to case. A macro that was added with the order `B` is
found after the routines of the package and of the packages it requires, and
before a registered function and a file; a macro that was added with `A` is
found after a file. In the example, a name that has a file and a macro of `B`
runs the macro, one that has a file and a macro of `A` runs the file, and the
macro of `A` runs when the file is removed; a routine of the program wins over
a macro of `B`, and the macro name is matched in any case.

```rexx
call lineout 'a3zqmacsrc.rex', 'return "macro"'
call lineout 'a3zqmacsrc.rex'
call put 'a3zqmb.rex', 'file'
call put 'a3zqma.rex', 'file'
call sysaddrexxmacro 'A3ZQMB', 'a3zqmacsrc.rex', 'B'
call sysaddrexxmacro 'A3ZQMA', 'a3zqmacsrc.rex', 'A'
call sysaddrexxmacro 'A3ZQMOWN', 'a3zqmacsrc.rex', 'B'
say a3zqmb() 'a3zqmb'() a3zqma() a3zqmown()
call sysfiledelete 'a3zqma.rex'
say a3zqma()
call sysdroprexxmacro 'A3ZQMB'
call sysdroprexxmacro 'A3ZQMA'
call sysdroprexxmacro 'A3ZQMOWN'
call sysfiledelete 'a3zqmb.rex'
call sysfiledelete 'a3zqmacsrc.rex'
exit
put: call lineout arg(1), 'return "'arg(2)'"'
  call lineout arg(1)
  return
::routine a3zqmown
  return "own routine"
```

```output
macro macro file own routine
macro
```

r[env.search.order]
A name that CALL or a function call writes, and that is not found among the
labels and the built-in functions as [the call rule](r:instr.call.search)
says, is searched for in this order: the routines of the package, public or
private; the public routines of the packages it requires, in the order of the
directives, with the routines of the libraries that `::REQUIRES ... LIBRARY`
names among them ([the routine rules](r:dir.routine.search) and [the public
rule](r:dir.routine.public)); the macros added with the order `B`; the
functions that [RXFUNCADD](r:bif.rxfuncadd.module) made callable; the files of
[the places](r:env.search.places); and the macros added with the order `A`.
The first of these that has the name is the routine that runs. In the example,
a file of the name of a function that `RxFuncAdd` registered is not run, and a
macro with `B` wins over the registered function.

```rexx
call lineout 'rxcalctanh.rex', 'return "file"'
call lineout 'rxcalctanh.rex'
call lineout 'a3zqmacsrc.rex', 'return "macro"'
call lineout 'a3zqmacsrc.rex'
say rxfuncadd('RxCalcTanh', 'rxmath')
say 'RxCalcTanh'(0)
call sysaddrexxmacro 'RXCALCTANH', 'a3zqmacsrc.rex', 'B'
say 'RxCalcTanh'(0)
call sysdroprexxmacro 'RXCALCTANH'
call sysfiledelete 'rxcalctanh.rex'
call sysfiledelete 'a3zqmacsrc.rex'
```

```output
0
0
macro
```

## Libraries

r[env.search.library]
A native library is named by a bare name: the word after `LIBRARY` in a
directive or a descriptor, the argument of `loadLibrary`, and the module of
RXFUNCADD. The library of the name `x` is the file `libx.so`, made by putting
`lib` before the name and `.so` after it, with nothing in the name changed.
The file is looked for in the search path of the loader of the host and in the
current directory. The name is case sensitive, so `rexxutil` and `rxmath` are
libraries of the host and `REXXUTIL` is not. A name that already has `lib` or
`.so` in it gets them again, so that `librexxutil` names the file
`liblibrexxutil.so` and `rexxutil.so` the file `librexxutil.so.so`, which no
host has. The empty name finds no library. The name of the entry of a library
that a descriptor gives is matched without regard to case, and a routine's own
name is the entry when the descriptor has none. A library that cannot be
loaded is the fault of [the directive](r:dir.requires.library) or the answer
of the method that loads it. The first example shows the names that the host
has and those that it has not. In the second, a file of the library of the
host is copied into the current directory under four names, in which the name
goes between `lib` and `.so` unchanged: a name with a `lib` prefix, one with
the ending `.so`, one with a period, and one in upper case. Each of those
names finds its copy, and the lower-case form of the last one does not.
interrexx does not yet look in the current directory (defect #214), so the
second example is marked with that defect.

```rexx
p = .package~new('a3zqlib', 'return 1')
do n over .array~of('rexxutil', 'rxmath', 'REXXUTIL', 'RxMath', 'librexxutil', -
    'rexxutil.so', './rexxutil', '', 'a3zqnosuchlibrary')
  say ('['n']')~left(20) p~loadLibrary(n)
end
r = .routine~loadExternalRoutine('a3zqtan', 'LIBRARY rxmath RXCALCTAN')
say r~call(0)
r = .routine~loadExternalRoutine('RxCalcCos', 'LIBRARY rxmath')
say r~call(0)
call try '::requires "RxMath" library'
call try '::requires "rxmath" library'
exit
try:
  signal on syntax name e
  p = .package~new('a3zqlibreq', .array~of(arg(1)))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
[rexxutil]           1
[rxmath]             1
[REXXUTIL]           0
[RxMath]             0
[librexxutil]        0
[rexxutil.so]        0
[./rexxutil]         0
[]                   0
[a3zqnosuchlibrary]  0
0
1
::requires "RxMath" library => 98.903
::requires "rxmath" library => ok
```

```rexx
src = ''
path = value('LD_LIBRARY_PATH',, 'ENVIRONMENT')
do d over path~changeStr(':', ' ')~makeArray(' ')
  if stream(d'/librxmath.so', 'c', 'query exists') <> '' then
    src = d'/librxmath.so'
end
if src = '' then exit
data = charin(src, 1, chars(src))
call stream src, 'c', 'close'
files = .array~of('liblibzq1.so', 'libzq2.so.so', 'libzq.c3.so', 'libZQ4.so')
do f over files
  call sysfiledelete f
  call charout f, data
  call stream f, 'c', 'close'
end
p = .package~new('a3zqlcwd', 'return 1')
do n over .array~of('libzq1', 'zq2.so', 'zq.c3', 'ZQ4', 'zq4')
  say n p~loadLibrary(n)
end
do f over files
  call sysfiledelete f
end
```

```output defect=#214
libzq1 1
zq2.so 1
zq.c3 1
ZQ4 1
zq4 0
```
