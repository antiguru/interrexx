---
element: "SysGetErrorText"
kind: function
syntax: "SysGetErrorText(errornumber)"
rexxref: "Rexx Utilities (RexxUtil) > SysGetErrorText; Rexx Utilities (RexxUtil) > A Note on Error Codes"
---

# SysGetErrorText

SysGetErrorText answers the description that the system has for an error
number, such as one of the numbers that SysMkDir or SysFileDelete answers
([the answer rule](r:util.index.codes)). The words of a description are the
system's and differ from one system to the next, so the examples compare
descriptions with each other and with the null string and never print one.

## Arguments

r[util.sysgeterrortext.args]
`errornumber` is a required argument. A call with no argument, or with the
argument omitted, raises `SYNTAX 88.901`, and a second argument raises
`SYNTAX 88.922`; an omitted place at the end of the call adds nothing to the
count ([the argument rule](r:util.index.args), and for the order of the checks
[Deviation 101](r:util.index.deviation.101)). An object other than a string is
replaced by the string that its `request` with the argument `STRING` answers,
and then read as a number, so that a mutable buffer that holds `2` is read as
`2`; an object whose string is not a number raises `SYNTAX 88.907`, `.nil`
included.

```rexx
call try "SysGetErrorText()"
call try "SysGetErrorText(,)"
call try "SysGetErrorText(2, 3)"
call try "SysGetErrorText(.nil)"
call try "SysGetErrorText(.object~new)"
call try "SysGetErrorText(.mutablebuffer~new('2')) == SysGetErrorText(2)"
call try "SysGetErrorText(2,) == SysGetErrorText(2)"
call try "SysGetErrorText(unsetvariable)"
call try "SysGetErrorText('')"
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
88.907
88.907
1
1
88.907
88.907
```

r[util.sysgeterrortext.number]
The error number is read as a number in the sense of Rexx: a sign, leading
zeros, blanks around it, a decimal point and an exponent are allowed, as long
as its value is a whole number. A string that is not a number, or whose value
has a fraction, raises `SYNTAX 88.907`, and so does a whole number outside the
range from -2147483648 to 2147483647. The value is not rounded to the
`NUMERIC DIGITS` in force: a number with more digits than that is read in
full, so `2.0001` still has a fraction under `NUMERIC DIGITS 3`, and `1234`
is the number `1234`.

```rexx
ref = SysGetErrorText(2)
do n over '2','02','+2','2.0','2e0','20E-1',' 2 ','0.2e1','1.5','2.5e0','two','1 2','0x2','','2147483647','-2147483648','2147483648','-2147483649','99999999999'
  call try n
end
numeric digits 3
call try '1234'
call try '2.0001'
call try '1E3'
exit
try:
  signal on syntax name e
  text = SysGetErrorText(arg(1))
  say '['arg(1)']' (text == ref) (text \== '')
  return
e: say '['arg(1)']' condition('o')~code
  return
```

```output
[2] 1 1
[02] 1 1
[+2] 1 1
[2.0] 1 1
[2e0] 1 1
[20E-1] 1 1
[ 2 ] 1 1
[0.2e1] 1 1
[1.5] 88.907
[2.5e0] 88.907
[two] 88.907
[1 2] 88.907
[0x2] 88.907
[] 88.907
[2147483647] 0 1
[-2147483648] 0 1
[2147483648] 88.907
[-2147483649] 88.907
[99999999999] 88.907
[1234] 0 1
[2.0001] 88.907
[1E3] 0 1
```

## The answer

r[util.sysgeterrortext.answer]
The answer is the description that the system gives for the number: the same
number has the same description each time, and a call whose number comes from
a function that answers a failure, such as the `17` that SysMkDir answers for
a directory that exists, answers what a call with the literal number does.
The numbers `0`, `2` and `-1` each have a description that is not the null
string. `CALL` leaves the description in `RESULT`.

```rexx
say (SysGetErrorText(0) \== '') (SysGetErrorText(2) \== '') (SysGetErrorText(-1) \== '')
say (SysGetErrorText(2) == SysGetErrorText(2)) (SysGetErrorText(2) \== SysGetErrorText(13))
call SysMkDir 'c6zqerr'
say SysMkDir('c6zqerr') (SysGetErrorText(SysMkDir('c6zqerr')) == SysGetErrorText(17))
say SysRmDir('c6zqerrnone') (SysGetErrorText(SysRmDir('c6zqerrnone')) == SysGetErrorText(2))
call SysRmDir 'c6zqerr'
call SysGetErrorText 2
say (result == SysGetErrorText(2))
```

```output
1 1 1
1 1
17 1
2 1
1
```
