---
element: "SysQueryRexxMacro"
kind: function
syntax: "SysQueryRexxMacro(name)"
rexxref: "Rexx Utilities (RexxUtil) > SysQueryRexxMacro"
---

# SysQueryRexxMacro

SysQueryRexxMacro asks whether the [macrospace](macrospace.md) holds a macro
of a name. It answers `A` or `B`, the order that the macro has, or the null
string when there is no such macro. It changes nothing.

## Arguments

r[util.sysqueryrexxmacro.args]
`name` is a required string. The count and the omission of the argument, and
an object in its place, follow [the argument rule](r:util.index.args): no
argument raises `SYNTAX 88.901`, a second argument raises `SYNTAX 88.922`, and
`.nil` raises `SYNTAX 88.909`. An object that is not a string but has a string
value is read by that value. The name is matched as [the macrospace
page](r:util.macrospace.names) says: in any case, with its blanks counting,
and the null string is a name.

```rexx
call try "SysQueryRexxMacro()"
call try "SysQueryRexxMacro(,)"
call try "SysQueryRexxMacro('C4ZQQ', 'x')"
call try "SysQueryRexxMacro(.nil)"
call try "SysQueryRexxMacro('')"
call try "SysQueryRexxMacro(.mutablebuffer~new('c4zqqnone'))"
exit
try:
  signal on syntax name e
  interpret 'say "[" || ' arg(1) '|| "]"'
  return
e: say condition('o')~code
  return
```

```output
88.901
88.901
88.922
88.909
[]
[]
```

## The result

r[util.sysqueryrexxmacro.answer]
The answer is the one letter, in capitals, of the order that the macro was
added with or last reordered to
([SysAddRexxMacro](r:util.sysaddrexxmacro.order),
[SysReorderRexxMacro](r:util.sysreorderrexxmacro.answer)), whatever letters
were written for it: `after` and `a` are both `A`, and an add without an order
is `B`. When there is no macro of the name it is the null string, which a
macro that was dropped or cleared away does not leave behind. The answer does
not say where the program of the macro came from, and a file of the name does
not make it a macro.

```rexx
call lineout 'c4zqqry.rex', 'return "x"'
call lineout 'c4zqqry.rex'
call lineout 'c4zqqryfile.rex', 'return "file"'
call lineout 'c4zqqryfile.rex'
call SysAddRexxMacro 'C4ZQQ1', 'c4zqqry.rex'
call SysAddRexxMacro 'C4ZQQ2', 'c4zqqry.rex', 'after'
call SysAddRexxMacro 'C4ZQQ3', 'c4zqqry.rex', 'a'
call SysAddRexxMacro 'C4ZQQ4', 'c4zqqry.rex', 'bx'
say SysQueryRexxMacro('C4ZQQ1') SysQueryRexxMacro('c4zqq2') SysQueryRexxMacro('C4ZQQ3') SysQueryRexxMacro('C4ZQQ4')
say '[' || SysQueryRexxMacro('C4ZQQ5') || ']' '[' || SysQueryRexxMacro('c4zqqryfile') || ']' '[' || SysQueryRexxMacro('c4zqqry.rex') || ']'
call SysReorderRexxMacro 'C4ZQQ1', 'a'
say SysQueryRexxMacro('C4ZQQ1')
call SysDropRexxMacro 'C4ZQQ1'
call SysDropRexxMacro 'C4ZQQ2'
call SysDropRexxMacro 'C4ZQQ3'
call SysDropRexxMacro 'C4ZQQ4'
say '[' || SysQueryRexxMacro('C4ZQQ1') || ']'
call SysFileDelete 'c4zqqry.rex'
call SysFileDelete 'c4zqqryfile.rex'
```

```output
B A A B
[] [] []
A
[]
```
