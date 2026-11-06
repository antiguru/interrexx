---
element: "Conventions"
kind: concept
ansi: "9.2.1"
rexxref: "Functions > Built-in Functions"
deviations: [30]
---

# Conventions

The rules on this page hold on every page that links to them. A builtin
page states what its arguments are (a string, a position, a length, a pad)
and links here for what that means, instead of restating it.

## Strings and characters

r[conv.string.chars]
A string is a finite sequence of characters, and a character is one byte:
any of the 256 values from `'00'x` to `'FF'x`. The length of a string is
the number of bytes in it. No rule in this book decodes a string in UTF-8
or any other encoding, so a character that such an encoding writes as two
bytes is two characters here.

```rexx
say length('c3a9'x)
say c2x(substr('c3a9'x, 2))
```

```output
2
A9
```

r[conv.string.null]
The null string is the string of length 0, written `''`. A builtin makes no
check of an argument its page calls a string: no string there raises an
error, so the null string and a string of any bytes are as valid as any
other.

```rexx
say length(upper(''))
say c2x(upper('61007a'x))
```

```output
0
41005A
```

r[conv.string.object]
An argument that a page calls a string and that is an object other than a
string is replaced by a string, and the builtin works on that string. When
the object has a `MAKESTRING` method that answers a string, the replacement
is that answer, as for an `Array` or a `MutableBuffer`. When the object has
no `MAKESTRING` method, the replacement is the answer of its `STRING`
method; when that answer is not a string either, see
[Deviation 30](r:conv.string.deviation.30). This book does not specify the
replacement when `MAKESTRING` answers an object other than a string, or for
an object whose class defines its own `REQUEST` method.

```rexx
say upper(.object~new) upper(.both~new) upper(.mutablebuffer~new('mb'))
say c2x(.array~of(1, 2)) length(.array~of(1, 2)~string)
::class both
::method makestring
  return 'made'
::method string
  return 'told'
```

```output
AN OBJECT MADE MB
310A32 8
```

r[conv.string.whitespace]
Whitespace is the blank, `'20'x`, and the horizontal tab, `'09'x`, and no
other character. Line feed, carriage return, form feed and vertical tab are
not whitespace.

```rexx
signal on syntax name e
say substr('abc', '09'x || '2 ')
say substr('abc', '2' || '0a'x)
e: say condition('o')~code
```

```output
bc
40.12
```

## Positions

r[conv.position.origin]
Positions count from 1. Position 1 is the first character of a string, or
its first word where a page counts words, position 2 the one after it, and
so on. No character or word is at position 0.

```rexx
say substr('abc', 1, 1) substr('abc', 3, 1)
say word('x y z', 2)
```

```output
a c
y
```

## Whole-number arguments

> **Note.** Position and length arguments are whole numbers. The two rules below say
> which strings a whole-number argument accepts and what value it then has.

r[conv.whole.form]
A whole-number argument must be a string of this form: optional whitespace;
an optional sign, `+` or `-`, which may be followed by whitespace; one or
more digits, with at most one period among or around them; optionally an
exponent, which is `E` or `e` directly after the last digit or the period,
an optional sign, and one or more digits; optional whitespace. Any other
string raises `SYNTAX 40.12`, the null string and a string of only
whitespace included.

```rexx
say substr('abcdef', ' + 2 ', 1) substr('abcdef', '2.', 1),
    substr('abcdef', '.2E1', 1) substr('abcdef', '20e-1', 1),
    substr('abcdef', '2.E0', 1)
signal on syntax name a
say substr('abc', '')
a: say condition('o')~code
signal on syntax name b
say substr('abc', '1 E1')
b: say condition('o')~code
signal on syntax name c
say substr('abc', '+-2')
c: say condition('o')~code
signal on syntax name d
say substr('abc', '0x2')
d: say condition('o')~code
```

```output
b b b b b
40.12
40.12
40.12
40.12
```

r[conv.whole.digits]
On a 64-bit host, which is what interrexx targets, the value of a
whole-number argument is its number rounded to 18 significant digits; when
the first digit dropped is 5 or more, the magnitude rounds up, otherwise
down. The caller's NUMERIC DIGITS setting plays no part. If the rounded
value has a fractional part other than zero, or its magnitude is 10\*\*18 or
more, the argument raises `SYNTAX 40.12`; otherwise its value is that
integer. (The reference manual gives 9 digits for 32-bit hosts; this book
does not specify those.)

```rexx
say substr('abcdef', '2.000', 1) substr('abcdef', '1.0000000000000000001', 1)
say '['word('a b', 999999999999999999)']'
numeric digits 3
signal on syntax name a
say substr('abcdef', 2, 1.0001)
a: say condition('o')~code
signal on syntax name b
say substr('abc', '2.5')
b: say condition('o')~code
signal on syntax name c
say word('a b', 1000000000000000000)
c: say condition('o')~code
signal on syntax name d
say word('a b', '999999999999999999.5')
d: say condition('o')~code
```

```output
b a
[]
40.12
40.12
40.12
40.12
```

## Builtins that run as String methods

r[conv.method]
A builtin that runs as a String method does its work by sending the
`String` message of the same name. One of its arguments, usually the first
and for a few builtins the second, is the receiver; the others become the
message's arguments, in their order. The builtin and the message then give
the same result for the same arguments. A builtin page says when its
builtin is one of these. The range checks below, and the
[order of the checks](r:conv.check.order), are rules for these builtins
only; any other builtin's page states its own range checks.

```rexx
say substr('abcd', 2, 2) 'abcd'~substr(2, 2) upper('ab', 2) 'ab'~upper(2)
```

```output
bc bc aB aB
```

> **Note.** The ooRexx reference manual (Functions > Built-in Functions)
> names the builtins that run this way; the ANSI standard, which has no
> String methods, does not.

r[conv.position.check]
In a builtin that runs as a String method, a position argument is a
whole-number argument whose value must be positive. A value of 0 or less
raises `SYNTAX 93.924`; `-0` is 0.

```rexx
signal on syntax name a
say substr('abc', 0)
a: say condition('o')~code
signal on syntax name b
say word('a b', '-0')
b: say condition('o')~code
signal on syntax name c
say substr('abc', -1)
c: say condition('o')~code
```

```output
93.924
93.924
93.924
```

r[conv.length.check]
In a builtin that runs as a String method, a length argument is a
whole-number argument whose value must not be negative. A negative value
raises `SYNTAX 93.923`. A length of 0 is allowed, and so is `-0`, which is
0.

```rexx
say '['substr('abc', 2, 0)']' '['substr('abc', 2, '-0')']'
signal on syntax name a
say substr('abc', 1, -1)
a: say condition('o')~code
```

```output
[] []
93.923
```

> **Note.** These two codes follow from the reference manual: a builtin of this kind
> sends a String message, the range of a position or length is checked by
> that method, and an incorrect call to a method is error 93, whose subcodes
> 924 and 923 are the position and length cases. The ANSI draft (9.2.1),
> which has no such methods, raises 40.14 for a position and 40.13 for a
> length.

## Pad arguments

r[conv.pad]
A pad argument must be a string of exactly one character. Any character may
be the pad, the blank and `'00'x` included. A pad of any other length raises
`SYNTAX 40.23`; so does the null string, and so does a character that an
encoding writes as two bytes ([two characters](r:conv.string.chars)).

```rexx
say substr('ab', 1, 4, '*') c2x(substr('ab', 1, 3, '00'x))
signal on syntax name a
say substr('abc', 1, 2, '**')
a: say condition('o')~code
signal on syntax name b
say substr('abc', 1, 2, '')
b: say condition('o')~code
signal on syntax name c
say substr('abc', 1, 2, 'c3a9'x)
c: say condition('o')~code
```

```output
ab** 616200
40.23
40.23
40.23
```

## Omitted and empty arguments

r[conv.arg.omitted]
An argument is omitted when its place in the argument list holds no
expression: nothing before the first comma, nothing between two commas, or
nothing after the last one. An omitted optional argument takes the default
that its page states.

```rexx
say substr('abc', 1, , '.') substr('abc', 2, )
```

```output
abc bc
```

r[conv.arg.empty]
An argument whose expression yields the null string is not omitted and never
takes the default. It is checked like any other value: as a whole number it
raises `SYNTAX 40.12` ([whole-number form](r:conv.whole.form)), as a pad
`SYNTAX 40.23` ([pad](r:conv.pad)), and as a string it is the null string.

```rexx
signal on syntax name a
say substr('abc', 1, '')
a: say condition('o')~code
signal on syntax name b
say substr('abc', 1, 5, '')
b: say condition('o')~code
```

```output
40.12
40.23
```

## The order of the checks

r[conv.check.order]
In a builtin that runs as a String method, when more than one argument
fails, the form checks run first: the
whole-number checks of [form](r:conv.whole.form) and
[value](r:conv.whole.digits), and the [pad](r:conv.pad) check, applied to
the arguments from left to right, and the first failure is raised. Only when
every argument passes them do the range checks of
[positions](r:conv.position.check) and [lengths](r:conv.length.check) run,
again from left to right. A builtin page that has checks of its own states
where they fall in this order.

```rexx
signal on syntax name a
say substr('abc', 0, 'x')
a: say condition('o')~code
signal on syntax name b
say substr('abc', 0, 1, '**')
b: say condition('o')~code
signal on syntax name c
say substr('abc', 'x', 1, '**')
c: say condition('o')~code
signal on syntax name d
say translate('abc', , , '**', 'x')
d: say condition('o')~code
signal on syntax name f
say substr('abc', 0, -1)
f: say condition('o')~code
```

```output
40.12
40.23
40.12
40.23
93.924
```

> **Note.** The range checks come last because they belong to the String method, which
> runs only once the call has been made. The reference manual does not say
> that the function call checks the form of its arguments, under error 40,
> before it sends the message; that part matches ooRexx. The
> ANSI draft (9.2.1) checks each argument's form and range together, one
> argument at a time.

## Errors and results

r[conv.error.name]
A rule that says something "raises `SYNTAX m.n`" means that the builtin
returns no value and the SYNTAX condition is raised with error number `m`
and subcode `n`. A trap for it sees `condition('C')` as `SYNTAX`,
`condition('O')~code` as the string `m.n`, and the variable `RC` set to
`m`. This book does not state the error message.

```rexx
signal on syntax name e
x = substr('abc', 0)
say 'not reached'
e: say condition('c') condition('o')~code rc
```

```output
SYNTAX 93.924 93
```

r[conv.returns]
A rule that says a builtin "returns" a string means that the function
call's value is exactly that string: the same characters in the same order,
with no blanks added or removed.

```rexx
say '['substr('abc', 2)']' '['substr(' a ', 1)']'
```

```output
[bc] [ a ]
```

## Deviations

r[conv.string.deviation.30]
interrexx does not convert an argument whose `STRING` method answers an
object that is not a string, such as a `Directory`. When a builtin receives
such an argument where its page calls for a string, interrexx ends the
program with exit code 120 and a message that this is not implemented,
after raising `NOSTRING` when a trap for it is set. ooRexx reads the
answer as a string, and what it does then depends on the builtin: `UPPER`
ends the program with Error 5, `LENGTH` answers an unrelated number, and `STRIP`
crashes the interpreter. A program can rely on none of these answers.

```rexx
o = .weird~new
say 'before'
say upper(o)
say 'after'
::class weird
::method string
  return .directory~new
```

```output rc=120
before
```
