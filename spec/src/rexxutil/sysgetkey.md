---
element: "SysGetKey"
kind: function
syntax: "SysGetKey([option])"
rexxref: "Rexx Utilities (RexxUtil) > SysGetKey"
---

# SysGetKey

SysGetKey is meant to read one key from a terminal without waiting for the
Enter key. This page states what the call does when the standard input of
the program is not a terminal, which is the case that its examples can run: a
file, a pipe or the null device. The examples that read from a file or a pipe
start a second interpreter with `rexx` on a program of their own and give it
the input.

## Arguments

r[util.sysgetkey.args]
`option` is an optional string. Read in any case, it is `ECHO` or `NOECHO`;
any other string, the null string and a string with a blank around the word
included, raises `SYNTAX 40.920`. An omitted `option` is the same as `ECHO`.
A second argument raises `SYNTAX 88.922` before `option` is read at all, so
before a `.nil` or a wrong option is reported
([Deviation 101](r:util.index.deviation.101)); `.nil` and
an object with no string value raise `SYNTAX 88.909`; any other object is
replaced by its string ([the argument rule](r:util.index.args)). An omitted
place at the end of the call adds nothing to the count.

```rexx
call try "SysGetKey('bad')"
call try "SysGetKey('')"
call try "SysGetKey('E')"
call try "SysGetKey('N')"
call try "SysGetKey(' echo')"
call try "SysGetKey('echo ')"
call try "SysGetKey('echoes')"
call try "SysGetKey('echo', 'x')"
call try "SysGetKey('bad', 'x')"
call try "SysGetKey(.object~new)"
call try "SysGetKey(.nil)"
call try "'['SysGetKey('echo')']'"
call try "'['SysGetKey('ECHO')']'"
call try "'['SysGetKey('NoEcho')']'"
call try "'['SysGetKey('noecho')']'"
call try "'['SysGetKey()']'"
call try "'['SysGetKey(,)']'"
call try "'['SysGetKey(.mutablebuffer~new('NOECHO'))']'"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
40.920
40.920
40.920
40.920
40.920
40.920
40.920
88.922
88.922
88.909
88.909
[]
[]
[]
[]
[]
[]
[]
```

## Without a terminal

r[util.sysgetkey.noterminal]
When the standard input is not a terminal, SysGetKey answers the null string
at once, whatever the input holds. It does not read the input, so the next
reading function of the program gets the input from its start, and it writes
nothing, for `ECHO` as well as for `NOECHO`.

```rexx
call lineout 'c6zqkey.rex', "say '['SysGetKey()']' length(SysGetKey('noecho'))"
call lineout 'c6zqkey.rex', "say '['linein()']'"
call lineout 'c6zqkey.rex', "say '['linein()']'"
call lineout 'c6zqkey.rex'
address system 'printf "ab\ncd\n" | rexx c6zqkey.rex' with output stem out.
do i = 1 to out.0
  say out.i
end
address system 'rexx c6zqkey.rex < /dev/null' with output stem out.
do i = 1 to out.0
  say out.i
end
call SysFileDelete 'c6zqkey.rex'
```

```output
[] 0
[ab]
[cd]
[] 0
[]
[]
```
