---
element: "SysVersion"
kind: function
syntax: "SysVersion()"
rexxref: "Rexx Utilities (RexxUtil) > SysVersion"
---

# SysVersion

SysVersion answers a string that names the operating system and its release.
The string is a host value: this page gives its form, and a program that
needs the value reads it, as the examples do, from what the system itself
reports.

## Arguments

r[util.sysversion.args]
SysVersion takes no argument. An argument raises `SYNTAX 88.922`, whatever it
is, and an omitted place at the end of the call adds nothing to the count
([the argument rule](r:util.index.args)).

```rexx
call try "SysVersion('x')"
call try "SysVersion(,'x')"
call try "SysVersion(.nil)"
call try "SysVersion(.object~new)"
say SysVersion(,) == SysVersion()
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
88.922
1
```

## The answer

r[util.sysversion.answer]
The answer is the name of the operating system, a blank and the release of the
running system, written as `uname -sr` writes them. Its first word is the name
of the system; on Linux it is `Linux`. `NUMERIC DIGITS` does not change it,
and the call may be made by `CALL`, which leaves the string in `RESULT`.

```rexx
address system 'uname -sr' with output stem uname.
say SysVersion() == uname.1
say (word(SysVersion(), 1) == word(uname.1, 1)) (words(SysVersion()) == words(uname.1))
numeric digits 3
say SysVersion() == uname.1
call SysVersion
say (result == uname.1)
say ('SYSVERSION'() == uname.1) (sysversion() == uname.1)
```

```output
1
1 1
1
1
1 1
```
