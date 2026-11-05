---
element: VERIFY
kind: builtin
syntax: "VERIFY(string, reference [, option] [, start] [, length])"
ansi: "9.3.24"
rexxref: "Functions > Built-in Functions > VERIFY"
deviations: [4]
---

# VERIFY

VERIFY scans a string, or a run of its characters, for the first character
that is not in a set of reference characters, or, with the Match option,
for the first one that is, and answers its position.

## Arguments

r[bif.verify.method]
VERIFY is a builtin that runs as a String method, and `string`, its first
argument, is the receiver; the other four become the message's arguments,
in their order. `string` and `reference` are
[strings](r:conv.string.null), `option` is an
[option](r:bif.verify.option), `start` is a
[position](r:conv.position.check), and `length` is a
[length](r:conv.length.check). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.call.count): two are required and five is
the most.

```rexx
say verify('a1', 'a') 'a1'~verify('1', 'M')
signal on syntax name a
say verify('abc', 'a', , 0)
a: say condition('o')~code
signal on syntax name b
say verify('abc', 'a', , 1, -1)
b: say condition('o')~code
signal on syntax name c
say verify('abc', 'a', , 'first')
c: say condition('o')~code
signal on syntax name d
say verify('abc', 'a', , 1, 1, 1)
d: say condition('o')~code
signal on syntax name f
say verify('abc', , 'M')
f: say condition('o')~code
```

```output
2 2
93.924
93.923
40.12
40.4
40.5
```

r[bif.verify.option]
`option` selects what VERIFY scans for, by its first character alone, in
either case: `N` or `n` selects Nomatch, and `M` or `m` selects Match.
Only that character decides; `option` may go on with any characters, and
they change nothing. When `option` is omitted, it is Nomatch.

```rexx
say verify('xy', 'x') verify('xy', 'x', 'n') verify('xy', 'x', 'Nope'),
    verify('xy', 'y', 'm') verify('xy', 'y', 'Many') verify('xy', 'y', 'M 1')
```

```output
2 2 2 2 2 2
```

r[bif.verify.option.check]
An `option` whose first character is not one of those four, and an `option`
that is the null string, raise `SYNTAX 93.915`. This is a range check: it
runs after every argument has passed its form check, and before the range
checks of `start` and `length`. It is made whatever `string` and
`reference` are.

```rexx
signal on syntax name a
say verify('abc', 'a', 'Y')
a: say condition('o')~code
signal on syntax name b
say verify('abc', 'a', '')
b: say condition('o')~code
signal on syntax name c
say verify('abc', 'a', ' M')
c: say condition('o')~code
signal on syntax name d
say verify('', '', 'Z')
d: say condition('o')~code
signal on syntax name f
say verify('abc', 'a', 'Y', 'z')
f: say condition('o')~code
signal on syntax name g
say verify('abc', 'a', 'Y', 0)
g: say condition('o')~code
```

```output
93.915
93.915
93.915
93.915
40.12
93.915
```

The reference manual names error 93.915 for a method option that is not
one of those allowed. The ANSI draft (9.3.24) checks the option with its
other arguments and raises 40.28.

r[bif.verify.start.default]
When `start` is omitted, it is 1: the scan starts at the first character of
`string`.

```rexx
say verify('aab', 'a', , ) verify('aab', 'a', , 1)
```

```output
3 3
```

r[bif.verify.length.default]
When `length` is omitted, the scan runs from position `start` to the end of
`string`.

```rexx
say verify('x--y', '-', , 2) verify('x--y', '-', , 2, 2)
```

```output
4 0
```

## The result

r[bif.verify.nomatch]
With Nomatch, when `start` is at most `LENGTH(string)` and `reference` is
not the null string, VERIFY returns the smallest position `i` from `start`
to `start + length - 1`, and at most `LENGTH(string)`, such that the
character of `string` at `i` does not occur in `reference`, and `0` when
every character in that run occurs in it.

```rexx
say verify('cafe', 'efac') verify('ca-fe', 'efac'),
    verify('ab-cd', 'abcd', 'N', 4) verify('ab-cd', 'abcd', 'N', 2, 1)
```

```output
0 3 0 0
```

r[bif.verify.match]
With Match, when `start` is at most `LENGTH(string)` and `reference` is not
the null string, VERIFY returns the smallest position `i` in the same run
such that the character of `string` at `i` occurs in `reference`, and `0`
when no character in that run occurs in it.

```rexx
say verify('to be', ' ', 'M') verify('to be', 'xyz', 'M'),
    verify('x.y.z', '.', 'M', 3) verify('x.y.z', '.', 'M', 3, 1)
```

```output
3 0 4 0
```

r[bif.verify.exact]
A character occurs in `reference` only when `reference` holds that same
[character](r:conv.string.chars), byte for byte: case matters, and
whitespace, control characters and `'00'x` are characters like any other.
The order of `reference` and characters repeated in it make no
difference.

```rexx
say verify('ABC', 'abc') verify('a' || '00'x || 'b', 'ba') verify('a b', 'ba  ab'),
    verify('tab' || '09'x, 'bat', 'M') verify('tab' || '09'x, 'bat')
```

```output
1 2 0 1 4
```

r[bif.verify.past-end]
When `start` is greater than `LENGTH(string)`, the result is `0`, whatever
`reference` and `option` are, for every `start` that passes its check.

```rexx
say verify('abc', 'x', 'N', 4) verify('abc', 'b', 'M', 999999999999999999),
    verify('abc', '', 'N', 4)
```

```output
0 0 0
```

r[bif.verify.empty-string]
When `string` is the null string, every `start` is past its end, so the
result is `0`, whatever `reference` is and whichever option is selected.

```rexx
say verify('', 'a') verify('', '') verify('', 'a', 'M') verify('', '', 'N', 3)
```

```output
0 0 0 0
```

r[bif.verify.empty-reference]
When `start` is at most `LENGTH(string)` and `reference` is the null
string, the result is `start` with Nomatch and `0` with Match. This holds
whatever `length` is, 0 included: no character of `string` is examined.

```rexx
say verify('abc', '') verify('abc', '', 'N', 3) verify('abc', '', 'N', 2, 0),
    verify('abc', '', 'M') verify('abc', '', 'M', 2)
```

```output
1 3 2 0 0
```

The Nomatch answer for a `length` of 0 follows the reference manual, which
gives `start` for a null `reference` without a condition on `length`; the
ANSI draft has no `length` argument.

r[bif.verify.length.zero]
When `length` is 0 and `reference` is not the null string, the run has no
characters and the result is `0`, with either option.

```rexx
say verify('abc', 'x', 'N', 1, 0) verify('abc', 'a', 'M', 1, 0)
```

```output
0 0
```

r[bif.verify.answer]
VERIFY writes its answer as a whole number in decimal digits only: no sign,
no leading zero, no period and no exponent, and `0` for zero. The caller's
NUMERIC DIGITS setting plays no part, so the answer is exact however many
digits it has.

```rexx
numeric digits 3
say verify(copies('a', 1200) || 'b', 'a') verify(copies('a', 1300), '', 'N', 1234),
    verify('ab', 'ab')
```

```output
1201 1234 0
```

The reference manual does not say how the answer is written or whether
NUMERIC DIGITS applies to it; this follows the observed behaviour.

## Deviations

r[bif.verify.deviation.4]
interrexx does not specify whether the answer of VERIFY is the same object
as another string of equal value: as an equal number the program already
holds, such as the `start` argument of a call that answers it, or a literal
such as `'1201'`, or the equal answer of another call. Through
`identityHash`, or as a key of an `IdentityTable` or `IdentitySet`, the
answer may be either, and a program can rely only on the value, which is
equal under `==`. The oracle differs: it often answers a new object where
interrexx answers the existing one.

```rexx
n = 1200 + 1
w = copies('a', 1300)
say (verify(w, '', 'N', n) == n) (verify(copies('a', 1200) || 'b', 'a') == '1201')
```

```output
1 1
```
