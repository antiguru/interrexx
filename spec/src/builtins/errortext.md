---
element: ERRORTEXT
kind: builtin
syntax: "ERRORTEXT(n)"
ansi: "9.5.5"
rexxref: "Functions > Built-in Functions > ERRORTEXT"
---

# ERRORTEXT

ERRORTEXT answers the message that goes with an error number: the sentence
that names the kind of error, without the number, the program or the line.

## Arguments

r[bif.errortext.args]
ERRORTEXT takes one argument, `n`, and nothing else. The argument count is
checked by the [builtin call rules](r:bif.index.call.count): one argument is
required and one is the most, so a call with two raises `SYNTAX 40.4`,
whatever the second is. ERRORTEXT does not run as a String method: sending the
message `ERRORTEXT` to a string raises `SYNTAX 97.1`.

```rexx
signal on syntax name a
say errortext()
a: say condition('o')~code
signal on syntax name b
say errortext(26, 'S')
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
`n` is a whole number that passes the
[whole-number rules](r:conv.whole.form), so a number such as `26.0`, `2.6e1`,
`' 26 '` or `026` is 26. Its value must be in the range 0 to 99: a whole number
outside it raises `SYNTAX 40.903`, `-0` is 0, and a string that is not a whole
number, the null string included, raises `SYNTAX 40.12`, as does a number
whose magnitude is [too large](r:conv.whole.digits). The caller's NUMERIC
DIGITS plays no part. The form is checked before the range, so a number that
is both out of range and not whole, such as 100.5, raises `SYNTAX 40.12`.

```rexx
numeric digits 1
say errortext(26) '/' errortext(26.0) '/' errortext(' 26 ') '/' errortext('026'),
    '/' errortext(2.6e1) '/' errortext(99)
say '['errortext('-0')']'
signal on syntax name a
say errortext(100)
a: say condition('o')~code
signal on syntax name b
say errortext(-1)
b: say condition('o')~code
signal on syntax name c
say errortext(1.5)
c: say condition('o')~code
signal on syntax name d
say errortext('')
d: say condition('o')~code
signal on syntax name e
say errortext('1e2')
e: say condition('o')~code
signal on syntax name f
say errortext(1e30)
f: say condition('o')~code
signal on syntax name g
say errortext(100.5)
g: say condition('o')~code
```

```output
Invalid whole number. / Invalid whole number. / Invalid whole number. / Invalid whole number. / Invalid whole number. / Translation error.
[]
40.903
40.903
40.12
40.12
40.903
40.12
40.12
```

## The result

r[bif.errortext.result]
For an error number `n` that has a message, ERRORTEXT returns the text of the
message: a sentence that ends in a period, with no error number, no subcode
and no values filled in. Error numbers are the major numbers of the
`SYNTAX` conditions, and a subcode is not an argument: `ERRORTEXT(40)` is the
text for every error `40.n`. The numbers 3 to 11, 13 to 31, 33 to 49, 88 to 93
and 97 to 99 have a message, and the following lists them all.

```rexx
do i = 0 to 99
  if errortext(i) \== '' then say i errortext(i)
end
```

```output
3 Failure during initialization.
4 Program interrupted.
5 System resources exhausted.
6 Unmatched "/*" or quote.
7 WHEN or OTHERWISE expected.
8 Unexpected THEN or ELSE.
9 Unexpected WHEN or OTHERWISE.
10 Unexpected or unmatched END.
11 Control stack full.
13 Invalid character in program.
14 Incomplete DO/LOOP/SELECT/IF.
15 Invalid hexadecimal or binary string.
16 Label not found.
17 Unexpected PROCEDURE.
18 THEN expected.
19 String or symbol expected.
20 Symbol expected.
21 Invalid data on end of clause.
22 Invalid character string.
23 Invalid data string.
24 Invalid TRACE request.
25 Invalid subkeyword found.
26 Invalid whole number.
27 Invalid DO or LOOP syntax.
28 Invalid LEAVE or ITERATE.
29 Environment name too long.
30 Name or symbol too long.
31 Name starts with number or ".".
33 Invalid expression result.
34 Logical value not 0 or 1.
35 Invalid expression.
36 Unmatched "(" or "[" in expression.
37 Unexpected ",", ")", or "]".
38 Invalid template or pattern.
39 Evaluation stack overflow.
40 Incorrect call to routine.
41 Bad arithmetic conversion.
42 Arithmetic overflow/underflow.
43 Routine not found.
44 Function or message did not return data.
45 No data specified on function RETURN.
46 Invalid variable reference.
47 Unexpected label.
48 Failure in system service.
49 Interpretation error.
88 Invalid argument.
89 Variable or message term expected.
90 External name not found.
91 No result object.
92 OLE error.
93 Incorrect call to method.
97 Object method not found.
98 Execution error.
99 Translation error.
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
The text of a number is the text that a `SYNTAX` condition of that major
number carries as its `ERRORTEXT` entry, so `ERRORTEXT(n)` for the number
`rc` of a trapped syntax error is the same string as the condition's own.

```rexx
signal on syntax name h
x = 1 + 'a'
h:
c = condition('o')
say c~rc errortext(c~rc) (errortext(c~rc) == c~errortext)
```

```output
41 Bad arithmetic conversion. 1
```

> **Note.** The ANSI draft (9.5.5) gives ERRORTEXT a second argument, `S`, that selects the
> standard English text. ooRexx takes one argument only, and so does
> interrexx: a second argument raises `SYNTAX 40.4`. The texts are those of
> ooRexx's message catalogue, in English.
