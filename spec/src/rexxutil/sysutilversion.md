---
element: "SysUtilVersion"
kind: function
syntax: "SysUtilVersion()"
rexxref: "Rexx Utilities (RexxUtil) > SysUtilVersion"
---

# SysUtilVersion

SysUtilVersion answers the version of the RexxUtil package as three numbers
joined by periods. Its value is a host value; this page gives only the form.

## Arguments

r[util.sysutilversion.args]
SysUtilVersion takes no argument. An argument raises `SYNTAX 88.922`, and an
omitted place at the end of the call adds nothing to the count
([the argument rule](r:util.index.args)).

```rexx
call try "SysUtilVersion('x')"
call try "SysUtilVersion(,'x')"
call try "SysUtilVersion(.nil)"
say SysUtilVersion(,) == SysUtilVersion()
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
88.922
88.922
88.922
1
```

## The answer

r[util.sysutilversion.answer]
The answer has the form `n.m.l`: three whole numbers with one period between
each two. `NUMERIC DIGITS` does not change it, and a `CALL` leaves it in
`RESULT`.

```rexx
v = SysUtilVersion()
parse var v major '.' minor '.' patch
say datatype(major, 'W') datatype(minor, 'W') datatype(patch, 'W')
say (major'.'minor'.'patch == v) v~countStr('.')
numeric digits 3
say SysUtilVersion() == v
call SysUtilVersion
say (result == v)
```

```output
1 1 1
1 2
1
1
```
