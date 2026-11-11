---
element: "Macrospace"
kind: concept
rexxref: "Rexx Utilities (RexxUtil) > SysAddRexxMacro; Rexx Utilities (RexxUtil) > SysDropRexxMacro; Rexx Utilities (RexxUtil) > SysQueryRexxMacro; Rexx Utilities (RexxUtil) > SysReorderRexxMacro"
---

# The macrospace

The macrospace is a table of named routines that the interpreter's service
keeps for every program that uses it. A program puts a routine into it with
[SysAddRexxMacro](sysaddrexxmacro.md), and a program calls the routine by its
name, as it calls any other routine that it does not define itself. A routine
of the table is a macro. This page says what a macro is and how it runs; the
pages of the functions that work on the table say how each one is called and
what it answers:

* [SysAddRexxMacro](sysaddrexxmacro.md),
  [SysDropRexxMacro](sysdroprexxmacro.md),
  [SysQueryRexxMacro](sysqueryrexxmacro.md) and
  [SysReorderRexxMacro](sysreorderrexxmacro.md): putting a macro in, taking it
  out, asking whether it is there, and moving it before or after the files.
* [SysClearRexxMacroSpace](sysclearrexxmacrospace.md),
  [SysSaveRexxMacroSpace](syssaverexxmacrospace.md) and
  [SysLoadRexxMacroSpace](sysloadrexxmacrospace.md): emptying the table, and
  keeping it in a file or bringing it back.

> **Note.** The examples add only macros of their own, whose names start with
> `C4ZQ`, and drop each one again, so that a macro that an example leaves does
> not change what a later one finds.

## What a macro is

r[util.macrospace.shared]
A macro belongs to the service and not to the program that added it. It is
found by every program that uses the service, whether that program started
before the macro was added or starts later. It stays until it is dropped,
until the macrospace is cleared, or until the service ends; how long the
service lives is not specified. In the example a program starts another with a
command: the child finds the macro of its parent, adds a macro and drops the
macro of the parent, and the parent, after the child has ended, finds the one
and not the other.

```rexx
call lineout 'c4zqshared.rex', 'return "from the parent" arg(1)'
call lineout 'c4zqshared.rex'
call lineout 'c4zqchild.rex', 'say "child:" SysQueryRexxMacro("C4ZQONE") C4ZQONE("x")'
call lineout 'c4zqchild.rex', 'call SysAddRexxMacro "C4ZQTWO", "c4zqshared.rex", "A"'
call lineout 'c4zqchild.rex', 'call SysDropRexxMacro "C4ZQONE"'
call lineout 'c4zqchild.rex'
say SysAddRexxMacro('C4ZQONE', 'c4zqshared.rex')
address system 'rexx c4zqchild.rex'
say 'parent: [' || SysQueryRexxMacro('C4ZQONE') || '] [' || SysQueryRexxMacro('C4ZQTWO') || ']'
call SysDropRexxMacro 'C4ZQTWO'
call SysFileDelete 'c4zqshared.rex'
call SysFileDelete 'c4zqchild.rex'
```

```output
0
child: B from the parent x
parent: [] [A]
```

r[util.macrospace.names]
The name of a macro is a string, and it is matched without regard to case: a
macro added as `c4zqmn` is found, queried and dropped under `C4ZQMN` and under
any other spelling of the letters, and an add with a name that differs from
the name of a macro only in case replaces that macro. The blanks in a name
count, so that `c4zq b` and `c4zq  b` are two names. The null string is a name
like any other: a macro can be added, queried and dropped under it.

```rexx
call lineout 'c4zqnames.rex', 'return "m"'
call lineout 'c4zqnames.rex'
say SysAddRexxMacro('c4zqmn', 'c4zqnames.rex', 'A')
say SysQueryRexxMacro('C4ZQMN') SysQueryRexxMacro('C4zqMn') '[' || SysQueryRexxMacro(' c4zqmn') || ']'
say SysAddRexxMacro('C4ZQMN', 'c4zqnames.rex', 'B') SysQueryRexxMacro('c4zqmn')
say SysAddRexxMacro('c4zq b', 'c4zqnames.rex')
say SysQueryRexxMacro('C4ZQ B') '[' || SysQueryRexxMacro('c4zq  b') || ']'
say SysDropRexxMacro('c4zq  b') SysDropRexxMacro('C4ZQ B')
say '[' || SysQueryRexxMacro('') || ']' SysAddRexxMacro('', 'c4zqnames.rex', 'A') SysQueryRexxMacro('')
say SysDropRexxMacro('') '[' || SysQueryRexxMacro('') || ']'
say SysDropRexxMacro('C4zqmn') '[' || SysQueryRexxMacro('c4zqmn') || ']'
call SysFileDelete 'c4zqnames.rex'
```

```output
0
A A []
0 B
0
B []
2 0
[] 0 A
0 []
0 []
```


r[util.macrospace.run]
A call of the name of a macro, whether as a function or by CALL, runs the
program that the macro was made from, as a routine of the code that makes the
call. The call passes its arguments the way a call of any routine does:
`arg()` holds their number and an omitted argument is empty. Each call starts
a fresh run, so the variables of the caller are not there and the variables of
an earlier run are gone. A `RETURN` with a value ends the macro with that
value as the answer; a `RETURN` without one, or the end of the program, leaves
the call without an answer, so that a function call raises `SYNTAX 44.1` and a
CALL sets no `RESULT`. An `EXIT` ends only the macro, and the caller goes on:
the value of the `EXIT` is the answer, and an `EXIT` without a value leaves
the call without one. A `SYNTAX` condition that the macro raises is raised in
the caller, with the same code. For `PARSE SOURCE`, the second word is
`FUNCTION` or `SUBROUTINE` as for any routine, and the third is the name of
the macro in capitals, whatever case it was added in. A macro is run the same
way from a routine, from a method and from another macro, and it can call
itself.

```rexx
call lineout 'c4zqrun.rex', 'parse arg a, b'
call lineout 'c4zqrun.rex', 'if var("count") then count = count + 1; else count = 1'
call lineout 'c4zqrun.rex', 'return arg() "[" || a || "][" || b || "] count=" || count || " x=" || symbol("x")'
call lineout 'c4zqrun.rex'
call lineout 'c4zqsrc.rex', 'parse source . kind name .; return kind name'
call lineout 'c4zqsrc.rex'
call lineout 'c4zqnone.rex', 'say "ran"'
call lineout 'c4zqnone.rex'
call lineout 'c4zqexit.rex', 'exit arg(1)'
call lineout 'c4zqexit.rex'
call lineout 'c4zqexit0.rex', 'exit'
call lineout 'c4zqexit0.rex'
call lineout 'c4zqdiv.rex', 'return 1 / 0'
call lineout 'c4zqdiv.rex'
call lineout 'c4zqrec.rex', 'if arg(1) = 0 then return 0; return arg(1) + C4ZQREC(arg(1) - 1)'
call lineout 'c4zqrec.rex'
names = .array~of('C4ZQRUN', 'c4zqsrc', 'C4ZQNONE', 'C4ZQEXIT', 'C4ZQEXIT0', 'C4ZQDIV', 'C4ZQREC')
files = .array~of('run', 'src', 'none', 'exit', 'exit0', 'div', 'rec')
do i = 1 to names~items
  call SysAddRexxMacro names[i], 'c4zq' || files[i] || '.rex'
end
x = 5
say C4ZQRUN('p', 'q')
say C4ZQRUN()
say C4ZQRUN(, 'z')
call C4ZQRUN 1, 2, 3
say result
say C4ZQSRC()
call C4ZQSRC
say result
signal on syntax name none
say C4ZQNONE()
none: say condition('o')~code
signal off syntax
drop result
call C4ZQNONE
say symbol('result')
call C4ZQEXIT 7
say 'went on' result
drop result
call C4ZQEXIT0
say 'went on' symbol('result')
signal on syntax name div
say C4ZQDIV()
div: say condition('o')~code
signal off syntax
say C4ZQREC(4)
say fromroutine() .c4zqc~new~go
do n over names
  call SysDropRexxMacro n
end
do f over files
  call SysFileDelete 'c4zq' || f || '.rex'
end
exit
fromroutine: return C4ZQRUN('r')
::class c4zqc
::method go
return C4ZQRUN('m')
```

```output
2 [p][q] count=1 x=LIT
0 [][] count=1 x=LIT
2 [][z] count=1 x=LIT
3 [1][2] count=1 x=LIT
FUNCTION C4ZQSRC
SUBROUTINE C4ZQSRC
ran
44.1
ran
LIT
went on 7
went on LIT
42.3
10
1 [r][] count=1 x=LIT 1 [m][] count=1 x=LIT
```

r[util.macrospace.source]
The program of a macro is read once, when the macro is added, and what is kept
is that program as it was then: a later change to the file, or its removal,
does not change the macro. The directives of the program work as they do in
any program, but only for the macro: a routine that its `::ROUTINE` directive
defines is called by the code of the macro and by nothing else, and the caller
of the macro does not find it. A `::REQUIRES` directive is not looked at until
the macro runs, so a macro whose directive names a file that is not there is
added without a fault, and a call of it raises `SYNTAX 43.901`.

```rexx
call lineout 'c4zqsrc1.rex', 'return "first"'
call lineout 'c4zqsrc1.rex'
call lineout 'c4zqsrc2.rex', 'return inner(arg(1))'
call lineout 'c4zqsrc2.rex', '::routine inner'
call lineout 'c4zqsrc2.rex', 'return "inner of macro:" arg(1)'
call lineout 'c4zqsrc2.rex'
call lineout 'c4zqsrc3.rex', 'return "never"'
call lineout 'c4zqsrc3.rex', '::requires "c4zqnofile.rex"'
call lineout 'c4zqsrc3.rex'
call SysAddRexxMacro 'C4ZQS1', 'c4zqsrc1.rex'
call SysFileDelete 'c4zqsrc1.rex'
say C4ZQS1()
call lineout 'c4zqsrc1.rex', 'return "second"'
call lineout 'c4zqsrc1.rex'
say C4ZQS1()
call SysFileDelete 'c4zqsrc1.rex'
say SysAddRexxMacro('C4ZQS2', 'c4zqsrc2.rex') C4ZQS2('x')
say SysAddRexxMacro('C4ZQS3', 'c4zqsrc3.rex')
signal on syntax name e1
say inner('y')
e1: say condition('o')~code
signal on syntax name e2
say C4ZQS3()
e2: say condition('o')~code
signal off syntax
call SysDropRexxMacro 'C4ZQS1'
call SysDropRexxMacro 'C4ZQS2'
call SysDropRexxMacro 'C4ZQS3'
call SysFileDelete 'c4zqsrc2.rex'
call SysFileDelete 'c4zqsrc3.rex'
```

```output
first
first
0 inner of macro: x
0
43.1
43.901
```

r[util.macrospace.search]
A call finds a macro only after the labels of the code that makes the call and
the built-in functions: a macro whose name is that of a built-in function
never runs in its place. Where the macro stands among the routines of the
program, the packages it requires, the registered functions and the files is
the subject of [the search order rule](r:env.search.order), and the order that
decides it is [the order rule](r:util.sysaddrexxmacro.order) of the add
function. [The macro rule](r:env.search.macro) of the environment chapter
gives the rest.

```rexx
call lineout 'c4zqleft.rex', 'return "macro"'
call lineout 'c4zqleft.rex'
say SysAddRexxMacro('LEFT', 'c4zqleft.rex', 'B') SysAddRexxMacro('C4ZQLEFT', 'c4zqleft.rex', 'B')
say left('abcdef', 3) 'LEFT'('abcdef', 3) C4ZQLEFT()
call SysDropRexxMacro 'LEFT'
call SysDropRexxMacro 'C4ZQLEFT'
call SysFileDelete 'c4zqleft.rex'
```

```output
0 0
abc abc macro
```
