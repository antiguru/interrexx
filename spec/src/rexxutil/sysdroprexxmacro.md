---
element: "SysDropRexxMacro"
kind: function
syntax: "SysDropRexxMacro(name)"
rexxref: "Rexx Utilities (RexxUtil) > SysDropRexxMacro"
---

# SysDropRexxMacro

SysDropRexxMacro takes a macro out of the [macrospace](macrospace.md). It
answers `0` when the macro was there and is gone, and `2` when there was no
macro of that name.

## Arguments

r[util.sysdroprexxmacro.args]
`name` is a required string. The count and the omission of the argument, and
an object in its place, follow [the argument rule](r:util.index.args): no
argument raises `SYNTAX 88.901`, a second argument raises `SYNTAX 88.922`, and
`.nil` raises `SYNTAX 88.909`. An object that is not a string but has a string
value is read by that value. The rules for what a name is, and for how it is
matched, are on [the page of the macrospace](r:util.macrospace.names): the
null string is a name, and it is the name of a macro only when one was added
under it.

```rexx
call try "SysDropRexxMacro()"
call try "SysDropRexxMacro(, )"
call try "SysDropRexxMacro('C4ZQD', 'x')"
call try "SysDropRexxMacro(.nil)"
call try "SysDropRexxMacro('')"
call try "SysDropRexxMacro(.mutablebuffer~new('c4zqdnone'))"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
88.901
88.901
88.922
88.909
2
2
```

## The result

r[util.sysdroprexxmacro.answer]
When a macro of the name is there, in either order and under any spelling of
the letters, the call removes it and answers `0`. A call of the name after
that finds no macro: it raises `SYNTAX 43.1` when nothing else has the name,
and a second drop of the same name answers `2`. A drop removes the one macro
and leaves the rest of the macrospace as it was. A macro that is running when
it is dropped, even by its own program, runs to its end. The file that the
macro was made from is not touched.

```rexx
call lineout 'c4zqdrop.rex', 'call SysDropRexxMacro "C4ZQD1"; return "ended: [" || SysQueryRexxMacro("C4ZQD1") || "]"'
call lineout 'c4zqdrop.rex'
call SysAddRexxMacro 'C4ZQD1', 'c4zqdrop.rex', 'B'
call SysAddRexxMacro 'C4ZQD2', 'c4zqdrop.rex', 'A'
call SysAddRexxMacro 'C4ZQD3', 'c4zqdrop.rex', 'A'
say SysDropRexxMacro('c4zqd2') SysDropRexxMacro('C4ZQD2')
say '[' || SysQueryRexxMacro('C4ZQD2') || '] [' || SysQueryRexxMacro('C4ZQD1') || '] [' || SysQueryRexxMacro('C4ZQD3') || ']'
say C4ZQD1()
say '[' || SysQueryRexxMacro('C4ZQD1') || ']' SysDropRexxMacro('C4ZQD1')
signal on syntax name e
say C4ZQD1()
e: say condition('o')~code
signal off syntax
say SysFileExists('c4zqdrop.rex') SysDropRexxMacro('C4ZQD3')
call SysFileDelete 'c4zqdrop.rex'
```

```output
0 2
[] [B] [A]
ended: []
[] 2
43.1
1 0
```
