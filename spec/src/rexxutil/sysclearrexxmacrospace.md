---
element: "SysClearRexxMacroSpace"
kind: function
syntax: "SysClearRexxMacroSpace()"
rexxref: "Rexx Utilities (RexxUtil) > SysClearRexxMacroSpace"
---

# SysClearRexxMacroSpace

SysClearRexxMacroSpace removes every macro from the
[macrospace](macrospace.md), and answers `0`.

## Arguments

r[util.sysclearrexxmacrospace.args]
The function takes no argument. An argument raises `SYNTAX 88.922`, as [the
argument rule](r:util.index.args) says; an omitted place at the end of the
call adds nothing to the count, so that a call with only omitted places is a
call with no argument.

```rexx
signal on syntax name a
say SysClearRexxMacroSpace('x')
a: say condition('o')~code
signal on syntax name b
say SysClearRexxMacroSpace(, 'x')
b: say condition('o')~code
signal off syntax
say SysClearRexxMacroSpace() SysClearRexxMacroSpace(,)
```

```output
88.922
88.922
0 0
```

## The result

r[util.sysclearrexxmacrospace.answer]
The call removes all the macros, whatever their orders and whoever added them,
and answers `0`; it answers `0` also when the macrospace was empty. After it,
a query for a name that had a macro answers the null string, and a call of the
name finds no macro. The files that the macros were made from are not touched.
Because the macrospace belongs to the service ([the sharing
rule](r:util.macrospace.shared)), the call empties it for every program that
uses the service, so that a program that is not the only one to use the
service clears away macros that it did not add.

```rexx
call lineout 'c4zqclr.rex', 'return "x"'
call lineout 'c4zqclr.rex'
call SysAddRexxMacro 'C4ZQCLR1', 'c4zqclr.rex', 'B'
call SysAddRexxMacro 'C4ZQCLR2', 'c4zqclr.rex', 'A'
say SysQueryRexxMacro('C4ZQCLR1') SysQueryRexxMacro('C4ZQCLR2') C4ZQCLR1()
say SysClearRexxMacroSpace() SysClearRexxMacroSpace()
say '[' || SysQueryRexxMacro('C4ZQCLR1') || '] [' || SysQueryRexxMacro('C4ZQCLR2') || ']'
signal on syntax name e
say C4ZQCLR1()
e: say condition('o')~code
signal off syntax
say SysFileExists('c4zqclr.rex')
call SysFileDelete 'c4zqclr.rex'
```

```output
B A x
0 0
[] []
43.1
1
```
