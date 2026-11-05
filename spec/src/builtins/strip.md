---
element: STRIP
kind: builtin
syntax: "STRIP(string [, option] [, chars])"
ansi: "9.3.20"
rexxref: "Functions > Built-in Functions > STRIP"
deviations: [4]
---

# STRIP

STRIP removes characters from the start of a string, from its end, or from
both, for as long as they belong to a set of unwanted characters. By
default that set is the whitespace.

## Arguments

r[bif.strip.method]
STRIP is a builtin that runs as a String method, and `string`, its first
argument, is the receiver; `option` and `chars` become the message's
arguments, in that order. `string` and `chars` are
[strings](r:conv.string.null), and `option` is an
[option](r:bif.strip.option). The argument count is checked by the
[builtin call rules](r:bif.index.call.count): one is required and three is
the most.

```rexx
x = ' pq '~strip('t')
say '['strip('   mn   ')']' '[' || x || ']'
signal on syntax name a
say strip('ab', 'Q')
a: say condition('o')~code
signal on syntax name b
say strip('ab', , 'a', 1)
b: say condition('o')~code
signal on syntax name c
say strip(, 'L')
c: say condition('o')~code
signal on syntax name d
say strip()
d: say condition('o')~code
```

```output
[mn] [ pq]
93.915
40.4
40.5
40.3
```

r[bif.strip.option]
`option` selects the end or ends of `string` that are stripped, by its
first character alone, in either case: `L` or `l` selects the leading end,
`T` or `t` the trailing end, and `B` or `b` both. Only that character
decides; `option` may go on with any characters, and they change nothing.
When `option` is omitted, it is `B`.

```rexx
say '['strip(' k  l ', 'l')']' '['strip(' k  l ', 'T')']' '['strip(' k  l ')']'
say '['strip('  a  ', 'Leading')']' '['strip('  a  ', 'trailing')']',
    '['strip('  a  ', 'Both')']'
say '['strip('  a  ', 'lx')']' '['strip('  a  ', , )']'
```

```output
[k  l ] [ k  l] [k  l]
[a  ] [  a] [a]
[a  ] [a]
```

r[bif.strip.option.check]
An `option` whose first character is not one of those six, and an `option`
that is the null string, raise `SYNTAX 93.915`. No other argument has a
check, so this is the only check STRIP makes of its own after the argument
count, and it is made whatever `string` and `chars` are.

```rexx
signal on syntax name a
say strip('ab', 'X')
a: say condition('o')~code
signal on syntax name b
say strip('ab', '')
b: say condition('o')~code
signal on syntax name c
say strip('ab', ' L')
c: say condition('o')~code
signal on syntax name d
say strip('', 'Y')
d: say condition('o')~code
signal on syntax name f
say strip('ab', 1, 'ab')
f: say condition('o')~code
signal on syntax name g
say strip(, 'Y')
g: say condition('o')~code
```

```output
93.915
93.915
93.915
93.915
93.915
40.5
```

> **Note.** The reference manual names error 93.915 for a method option that is not
> one of those allowed. The ANSI draft (9.3.20) reads `option` the same way,
> by its first character in either case, but raises 40.21 for a null option
> and 40.28 for any other bad one, where ooRexx raises 93.915; and it makes
> `chars` one character, where ooRexx takes the characters of `chars` as a
> set, as [the rule below](r:bif.strip.chars) says.

r[bif.strip.chars]
`chars` is the set of unwanted characters: the characters it holds, in any
order and whether or not one repeats. When `chars` is omitted, the set is
the two [whitespace](r:conv.string.whitespace) characters, the blank and
the tab. Case matters: `a` is not `A`. Any character may be in the set,
`'00'x` and those from `'80'x` to `'FF'x` included.

```rexx
say '['strip('xyxaxyb', , 'xy')']' '['strip('A a A', , 'a')']'
say c2x(strip('00'x || 'a' || '00'x, , '00'x)) c2x(strip('80'x || 'a' || '81'x, , '81'x))
say c2x(strip('09'x || ' a' || '09'x)) c2x(strip('0a'x || 'a' || '0d'x))
say c2x(strip('0b'x || 'a' || '0c'x)) c2x(strip('09'x || ' a ' || '09'x, , ' '))
```

```output
[axyb] [A a A]
61 8061
61 0A610D
0B610C 0920612009
```

## The result

r[bif.strip.result]
STRIP returns `string` with the run that `option` selects removed: the run
at its start, the run at its end, or both. The leading characters removed are
the longest run at the start of `string` made of characters in the
[set](r:bif.strip.chars), and the trailing ones the longest such run at the
end. Characters between them are not touched, whether or not they are in
the set. A character of the set that is not in such a run, because a
character outside the set stands between it and the end, stays.

```rexx
say '['strip('xxaxbxx', 'L', 'x')']' '['strip('xxaxbxx', 'T', 'x')']',
    '['strip('xxaxbxx', 'B', 'x')']'
say '['strip('abcba', 'L', 'ba')']' '['strip('abcba', , 'ba')']' '['strip('  a b  ')']'
say c2x(strip(' ' || '09'x || 'a', , ' '))
```

```output
[axbxx] [xxaxb] [axb]
[cba] [c] [a b]
0961
```

r[bif.strip.all]
When every character of `string` is in the set, the result is the null
string, whichever `option` is. The null string strips to the null string.

```rexx
say '['strip('xxx', 'L', 'x')']' '['strip('xxx', 'T', 'x')']' '['strip('xxx', , 'x')']'
say '['strip('')']' '['strip('  ' || '09'x)']' '['strip('', , 'x')']'
```

```output
[] [] []
[] [] []
```

r[bif.strip.chars.null]
A null `chars` is an empty set, so STRIP removes nothing and returns
`string`, for every `option`. A null `chars` is not omitted, so it does not
give the default set.

```rexx
say '['strip('  a  ', , '')']' '['strip('  a  ', 'L', '')']',
    '['strip('  a  ', 'T', '')']'
```

```output
[  a  ] [  a  ] [  a  ]
```

r[bif.strip.omitted-chars]
An omitted `chars` takes the default set, an omitted `option` takes `B`, and
the two are independent of each other.

```rexx
say '['strip('  a  ', 'L', )']' '['strip('  a  ', , )']' '['strip('xxaxx', , 'x')']'
```

```output
[a  ] [a] [a]
```

## Deviations

> **Note.** An argument whose `STRING` method answers an object that is not a string
> falls under [Deviation 30](r:conv.string.deviation.30), as it does for
> every builtin.

r[bif.strip.deviation.4]
interrexx does not specify whether the result of STRIP is the same object
as another string of equal value: as `string` itself, when nothing is
stripped, as in `strip(s)` for an `s` with no whitespace at either end, or
as an equal string produced some other way, such as the literal `'ab'`
beside `strip('  ab')`. Through `identityHash`, or as a key of an
`IdentityTable` or `IdentitySet`, the answer may be either, and a program
can rely only on the value, which is equal under `==`. ooRexx differs: it
often answers a new object where interrexx answers the existing one.

```rexx
s = 'p' || 'q'
say (strip(s) == s) (strip('  ab') == 'ab')
```

```output
1 1
```
