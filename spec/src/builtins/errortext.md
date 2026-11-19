---
element: ERRORTEXT
kind: builtin
syntax: "ERRORTEXT(n)"
ansi: "9.5.5"
rexxref: "Functions > Built-in Functions > ERRORTEXT"
deviations: [179]
---

# ERRORTEXT

ERRORTEXT answers the message that goes with an error number or an error code:
the text that names the kind of error, or the one that says what went wrong,
without the number, the program or the line. What
the text says is not part of the book (see [the rule on error
messages](r:conv.error.name)), so the examples of this page look at whether a
message exists and at which numbers share one, never at its words.

## Arguments

r[bif.errortext.args]
ERRORTEXT takes one argument, `n`. The argument count is checked by the
[builtin call rules](r:bif.index.call.count): one argument is required, so a
call with none raises `SYNTAX 40.3`, and a call with three raises `SYNTAX
40.4`. A second argument is [Deviation 179](r:bif.errortext.deviation.179)'s. ERRORTEXT does not run as a [String
method](r:class.string.builtins): sending the message `ERRORTEXT` to a string
raises `SYNTAX 97.1`.

```rexx
signal on syntax name a
say errortext()
a: say condition('o')~code
signal on syntax name b
say errortext(26, 'S', 1)
b: say condition('o')~code
signal on syntax name c
say errortext(, )
c: say condition('o')~code
signal on syntax name d
say 16~errortext
d: say condition('o')~code
```

```output
40.3
40.4
40.3
97.1
```

r[bif.errortext.n]
`n` is an error number or an error code. A whole number that passes the
[whole-number rules](r:conv.whole.form) is an error number, so a number such as
`26.0`, `2.6e1`, `' 26 '` or `026` is 26, and `40.0` is the error number 40.
Its value must be in the range 0 to 99: a whole number outside it raises
`SYNTAX 40.903`, and `-0` is 0. Any other string that holds a period is an
error code, as [`RAISE SYNTAX`](r:instr.raise.syntax) and the `CODE` entry of a
condition object write one: the part before the first period is the error
number and the part after it the subcode, each read as a whole number by the
same rules, so blanks around a part, leading zeros and an exponent are allowed.
The error number must be in the range 0 to 99 and the subcode in the range 0
to 999; anything else raises `SYNTAX 40.926`. The subcode is read as written,
not as the fraction of a number: `40.1` and `40.001` are both subcode 1, and
`40.10` is subcode 10. A computed number can lose or keep trailing zeros, so an
error code is passed as the string it is written as. A string with no period
that is not a whole number, the null string included, raises `SYNTAX 40.12`, as
does a whole number whose magnitude is [too large](r:conv.whole.digits). The
caller's NUMERIC DIGITS plays no part.

```rexx
numeric digits 1
say (errortext(26) \== '') (errortext(26.0) == errortext(26)) (errortext(' 26 ') == errortext(26)),
    (errortext('026') == errortext(26)) (errortext(2.6e1) == errortext(26)) (errortext(99) \== '')
say '['errortext('-0')']' (errortext('40.0') == errortext(40))
say (errortext('40.1') == errortext('40.001')) (errortext('40.1') == errortext(' 40 . 1 ')),
    (errortext('40.1') \== errortext('40.10')) (errortext('4.01e1') == errortext('4.010'))
do n over .array~of(100, -1, '', '1e2', 1e30, '15e-1', '40.1000', '100.5', '.5',,
    '40.-1', 'x.y', '40.1.1')
  call try n
end
exit
try:
  signal on syntax name bad
  say errortext(arg(1))
  return
bad: say '['arg(1)']' condition('o')~code
  return
```

```output
1 1 1 1 1 1
[] 1
1 1 1 1
[100] 40.903
[-1] 40.903
[] 40.12
[1e2] 40.903
[1E30] 40.12
[15e-1] 40.12
[40.1000] 40.926
[100.5] 40.926
[.5] 40.926
[40.-1] 40.926
[x.y] 40.926
[40.1.1] 40.926
```

## The result

r[bif.errortext.result]
For an error number `n` that has a message, ERRORTEXT returns the text of the
message, which is not the null string, with no error number and no subcode.
Its wording is unspecified. Error numbers are the major numbers of the
`SYNTAX` conditions: `ERRORTEXT(40)` is the text that every error `40.n` shares.
For an error code that has a message, ERRORTEXT returns the message of that
subcode, which is not the null string. The places where a raised error puts its
values are shown in it as `&1`, `&2` and so on, as they stand in the message,
not necessarily in order; a subcode whose message has no values has none. For
an error code that has no message, it returns the null string. The numbers 3
to 11, 13 to 31, 33 to 49, 88 to 93 and 97 to 99 have a message, and the first
example lists them all.

```rexx
do i = 0 to 99
  if errortext(i) \== '' then say i
end
```

```output
3
4
5
6
7
8
9
10
11
13
14
15
16
17
18
19
20
21
22
23
24
25
26
27
28
29
30
31
33
34
35
36
37
38
39
40
41
42
43
44
45
46
47
48
49
88
89
90
91
92
93
97
98
99
```

```rexx
t = errortext('40.1')
say (t \== '') (pos('&1', t) > 0) (pos('&', errortext('10.1')) = 0),
    '['errortext('40.10')']' '['errortext('1.5')']'
```

```output
1 1 1 [] []
```

r[bif.errortext.undefined]
For an error number in the range 0 to 99 that has no message, ERRORTEXT
returns the null string: 0 to 2, 12, 32, 50 to 87, and 94 to 96.

```rexx
none = ''
do i = 0 to 99
  if errortext(i) == '' then none = none i
end
say strip(none)
say length(errortext(0)) length(errortext(77)) length(errortext(95))
```

```output
0 1 2 12 32 50 51 52 53 54 55 56 57 58 59 60 61 62 63 64 65 66 67 68 69 70 71 72 73 74 75 76 77 78 79 80 81 82 83 84 85 86 87 94 95 96
0 0 0
```

r[bif.errortext.condition]
The text of a number is the text that a [`SYNTAX`](r:cond.syntax.code)
condition of that major number carries as its `ERRORTEXT` entry, so
`ERRORTEXT(n)` for the number `rc` of a trapped syntax error is the same
string as the condition's own.

```rexx
signal on syntax name h
x = 1 + 'a'
h:
c = condition('o')
say c~rc (errortext(c~rc) \== '') (errortext(c~rc) == c~errortext)
```

```output
41 1 1
```

> **Note.** The ANSI draft (9.5.5) gives ERRORTEXT a second argument, `S`,
> that selects the standard English text. interrexx takes one argument only, as
> [Deviation 179](r:bif.errortext.deviation.179) says. The texts are interrexx's
> own and are not specified.

## Deviations

r[bif.errortext.deviation.179]
ERRORTEXT takes one argument, as at language level 6.06 ([Deviation
179](../deviations.md)), so a second argument raises `SYNTAX 40.4`, whatever it
is. ooRexx, at level 6.07, takes a second argument of values that fill the
`&1`, `&2` places of an error code's message.

```rexx
signal on syntax name b
say errortext(26, 'S')
b: say condition('o')~code
signal on syntax name c
say errortext('40.1', 'x')
c: say condition('o')~code
```

```output
40.4
40.4
```
