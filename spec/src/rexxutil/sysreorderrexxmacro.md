---
element: "SysReorderRexxMacro"
kind: function
syntax: "SysReorderRexxMacro(name, order)"
rexxref: "Rexx Utilities (RexxUtil) > SysReorderRexxMacro"
---

# SysReorderRexxMacro

SysReorderRexxMacro changes the order of a macro that is in the
[macrospace](macrospace.md), which moves it before or after the files in the
search for a name. It answers `0` when the macro has the order that was asked
for, and `2` when there is no macro of that name.

## Arguments

r[util.sysreorderrexxmacro.args]
`name` and `order` are both required strings. The count and the omission of an
argument, and an object in the place of one, follow [the argument
rule](r:util.index.args): a call with either one missing or omitted raises
`SYNTAX 88.901`, a third argument raises `SYNTAX 88.922`, and `.nil` raises
`SYNTAX 88.909`. An object that is not a string but has a string value is read
by that value. `order` is read as [the order rule of
SysAddRexxMacro](r:util.sysaddrexxmacro.order) reads it, with this difference:
it has no default, so that omitting it raises `SYNTAX 88.901`. The order is
checked before the name is looked for, so a bad `order` raises `SYNTAX 40.920`
for a name that no macro has.

```rexx
call lineout 'c4zqreo.rex', 'return "macro"'
call lineout 'c4zqreo.rex'
call SysAddRexxMacro 'C4ZQR', 'c4zqreo.rex', 'B'
call try "SysReorderRexxMacro()"
call try "SysReorderRexxMacro('C4ZQR')"
call try "SysReorderRexxMacro('C4ZQR', )"
call try "SysReorderRexxMacro(, 'A')"
call try "SysReorderRexxMacro('C4ZQR', 'A', 'x')"
call try "SysReorderRexxMacro(.nil, 'A')"
call try "SysReorderRexxMacro('C4ZQR', .nil)"
call try "SysReorderRexxMacro('C4ZQR', '')"
call try "SysReorderRexxMacro('C4ZQR', 'x')"
call try "SysReorderRexxMacro('C4ZQRNONE', 'x')"
call try "SysReorderRexxMacro('C4ZQRNONE', 'A')"
call try "SysReorderRexxMacro(.mutablebuffer~new('c4zqr'), .mutablebuffer~new('after'))"
say SysQueryRexxMacro('C4ZQR')
call SysDropRexxMacro 'C4ZQR'
call SysFileDelete 'c4zqreo.rex'
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
88.901
88.901
88.922
88.909
88.909
40.920
40.920
40.920
2
0
A
```

## The result

r[util.sysreorderrexxmacro.answer]
When a macro of the name is there, in any case, the call gives it the order
and answers `0`, also when it had that order already; the program of the macro
is not changed, and no other macro is touched. The change shows in the search
for a name: of a macro and a file that have the name, the macro with the order
`B` runs, and the one with the order `A` does not, as [the search
rules](r:util.macrospace.search) of the macrospace say. When no macro has the
name, the call answers `2` and changes nothing.

```rexx
call lineout 'c4zqreof.rex', 'return "file"'
call lineout 'c4zqreof.rex'
call lineout 'c4zqreom.rex', 'return "macro"'
call lineout 'c4zqreom.rex'
call lineout 'c4zqreoo.rex', 'return "other"'
call lineout 'c4zqreoo.rex'
call SysAddRexxMacro 'C4ZQREOF', 'c4zqreom.rex', 'B'
call SysAddRexxMacro 'C4ZQREOO', 'c4zqreoo.rex', 'B'
say SysQueryRexxMacro('C4ZQREOF') C4ZQREOF()
say SysReorderRexxMacro('c4zqreof', 'A') SysQueryRexxMacro('C4ZQREOF') C4ZQREOF() SysQueryRexxMacro('C4ZQREOO')
say SysReorderRexxMacro('C4ZQREOF', 'after') SysQueryRexxMacro('C4ZQREOF') C4ZQREOF()
say SysReorderRexxMacro('C4ZQREOF', 'B') SysQueryRexxMacro('C4ZQREOF') C4ZQREOF()
say SysReorderRexxMacro('C4ZQREOF', 'B') SysQueryRexxMacro('C4ZQREOF') C4ZQREOF()
say SysReorderRexxMacro('C4ZQREOX', 'A') '[' || SysQueryRexxMacro('C4ZQREOX') || ']'
call SysDropRexxMacro 'C4ZQREOF'
call SysDropRexxMacro 'C4ZQREOO'
call SysFileDelete 'c4zqreof.rex'
call SysFileDelete 'c4zqreom.rex'
call SysFileDelete 'c4zqreoo.rex'
```

```output
B macro
0 A file B
0 A file
0 B macro
0 B macro
2 []
```
