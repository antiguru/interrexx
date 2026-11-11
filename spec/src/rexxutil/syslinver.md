---
element: "SysLinVer"
kind: function
syntax: "SysLinVer()"
rexxref: "Rexx Utilities (RexxUtil) > SysLinVer"
---

# SysLinVer

SysLinVer answers a string that identifies the version of a Linux system. This
page describes it on a Linux host. Like [SysVersion](sysversion.md), its
answer is a host value, and a program reads the value from the system.

## Arguments

r[util.syslinver.args]
SysLinVer takes no argument. An argument raises `SYNTAX 88.922`, and an
omitted place at the end of the call adds nothing to the count
([the argument rule](r:util.index.args)).

```rexx
call try "SysLinVer('x')"
call try "SysLinVer(,'x')"
call try "SysLinVer(.nil)"
say SysLinVer(,) == SysLinVer()
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

r[util.syslinver.answer]
On a Linux host the answer is the word `Linux`, a blank and the release of
the running kernel, written as `uname -r` writes it. It is the same string
that [SysVersion](r:util.sysversion.answer) answers.

```rexx
say word(SysLinVer(), 1)
say SysLinVer() == SysVersion()
address system 'uname -r' with output stem uname.
say substr(SysLinVer(), 7) == uname.1
call SysLinVer
say (result == SysVersion())
```

```output
Linux
1
1
1
```
