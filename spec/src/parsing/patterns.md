---
element: "Patterns"
kind: concept
ansi: "8.3.17"
rexxref: "Parsing > Templates Containing String Patterns, Parsing > Templates Containing Positional (Numeric) Patterns, Parsing > Parsing with Variable Patterns, Parsing > Advanced Topics in Parsing > Combining String and Positional Patterns"
---

# Patterns

A pattern is a trigger: it makes a break in the source, and the targets on
either side of it share the pieces that the break separates. This page says
where each kind of trigger puts its break and which characters the pieces
hold. What a target does with its piece is the business of [the templates
page](templates.md).

A break has a start and an end. For a string it is the span of the match, and
for a position it is a single point, so its start and its end are the same.
Unless a rule below says otherwise, a piece begins where the previous break
ends, or at the first character of the source before any break, and it ends
where the next break begins, or at the end of the source after the last
break. Positions count as [the convention](r:conv.position.origin) says, from
1, and the point at a position lies just before the character there.

## Strings

r[parse.patterns.string]
A [string literal](r:lex.literals.string), or a [hex](r:lex.literals.hex) or
[binary](r:lex.literals.bin) literal, finds the first place, at or after the
point where the piece begins, where its characters occur in the source,
comparing each character exactly as it is, so case matters. The break is the
span of that occurrence: the piece ends just before it and the next piece
begins just after it, and the matched characters belong to no piece. An
occurrence at the very beginning of the piece leaves an empty piece before it,
and an empty piece gives its targets the null string.

```rexx
parse value 'id=42;tag=blue' with key '=' val
say key'|'val
parse value 'id=42;tag=blue' with key '=' val ';' rest
say key'|'val'|'rest
parse value 'id=42;tag=blue' with 'id=' num
say num
parse value 'id=42;tag=blue' with before 'id' after
say '['before']['after']'
parse value 'id=42;Tag=blue' with before 'tag' after
say '['before']['after']'
```

```output
id|42;tag=blue
id|42|tag=blue
42;tag=blue
[][=42;tag=blue]
[id=42;Tag=blue][]
```

r[parse.patterns.string.missing]
When the string does not occur, and when it is the null string, which never
matches, the break lies at the end of the source. The piece before it runs
to the end, and the next piece begins there, so each target after the pattern
receives the null string, as [for a null source](r:parse.templates.fewer)
does each target. A later string cannot match either.

```rexx
parse value 'alpha beta' with u ',' v
say '['u']['v']'
parse value 'alpha beta' with u 'a' v "" w
say '['u']['v']['w']'
parse value 'alpha beta' with u '' v
say '['u']['v']'
parse value '' with u ',' v 3 w
say '['u']['v']['w']'
```

```output
[alpha beta][]
[][lpha beta][]
[alpha beta][]
[][][]
```

r[parse.patterns.string.sequence]
When strings follow each other with no target between them, the second
searches from the end of the first match, not from the beginning of the
source. The characters between the two matches belong to no piece.

```rexx
parse value ',,tail' with ',' ',' rest
say rest
parse value 'a<b>c' with '<' '>' rest
say rest
parse value ',x' with u ',' v ',' w
say '['u']['v']['w']'
```

```output
tail
c
[][x][]
```

## Variable patterns

r[parse.patterns.variable]
An expression in parentheses is evaluated when the template reaches the end
of the piece that it ends, which is after the targets of the earlier pieces
are assigned and [before the targets of that piece are](r:parse.index.order).
Its value is a string trigger as [the string rule](r:parse.patterns.string)
describes. A value that is an object and not a string is replaced by its
string, as [the conventions](r:conv.string.object) say. The value is used as
the string that it is, so a value that looks like a number is still the
characters to find, and a value that is the null string is the null string,
which [never matches](r:parse.patterns.string.missing). The expression may be
as long as any expression, with operators and function calls.

```rexx
sep = ','; n = 4; none = ''
parse value 'ab,cd,ef' with u (sep) v
say u'|'v
parse value 'abc4efg' with u (n) v
say u'|'v
parse value 'abcdefg' with u (n) v
say '['u']['v']'
parse value 'ab,cd,ef' with u (none) v
say '['u']['v']'
parse value 'ab,cd,ef' with u (left(sep, 1) || 'c') v
say u'|'v
sep = '-'
parse value ',ab,cd' with sep (sep) rest  /* still the old '-' */
say '['sep']['rest']'
sep = '-'
parse value ',ab,cd' with sep 2 (sep) rest  /* now the comma */
say '['sep']['rest']'
```

```output
ab|cd,ef
abc|efg
[abcdefg][]
[ab,cd,ef][]
ab|d,ef
[,ab,cd][]
[,][cd]
```

```rexx
mark = .dash~new
parse value 'ab--cd' with u (mark) v
say u'|'v
exit
::class dash
::method makestring
  return '--'
```

```output
ab|cd
```

> **Note.** The ANSI draft (6.3.2.71) allows only a variable symbol in the
> parentheses of a variable pattern, and nothing else. ooRexx, and this
> book, take any expression.

## Positions

r[parse.patterns.absolute]
A whole number written as a trigger, or `=` followed by a whole number, puts
a point at that position: the piece before it ends there and the next piece
begins there. The characters before position `n` are the characters
numbered below `n`, so `u 4 v` gives `u` the first three. A position
beyond the end of the source puts the point at the end, and position 0 is the
same as position 1, so a source of `L` characters has the points 1 to `L+1`.
The `=` form says the same as the bare number. Only the `=` form can take
the position from a variable or an expression, written in parentheses, as
the bare parentheses are a string trigger.

```rexx
parse value 'abcdefghij' with u 4 v
say u'|'v
parse value 'abcdefghij' with u 4 v =8 w =(2 + 1) y
say u'|'v'|'w'|'y
parse value 'abcdefghij' with u 11 v
say '['u']['v']'
parse value 'abcdefghij' with u 99 v
say '['u']['v']'
parse value 'abcdefghij' with 0 u
say u
n = 6
parse value 'abcdefghij' with u =(n) v
say u'|'v
parse value 'abcdefghij' with u (n) v
say '['u']['v']'
```

```output
abc|defghij
abc|defg|hij|cdefghij
[abcdefghij][]
[abcdefghij][]
abcdefghij
abcde|fghij
[abcdefghij][]
```

r[parse.patterns.backward]
A position that lies at the beginning of the current piece or before it
makes the piece run to the end of the source, not end empty or before its
start. The next piece begins at the position, so the text from there on is
shared out again. This is the way a template reads part of the source twice.

```rexx
parse value 'abcdefghij' with u 8 v 3 w
say u'|'v'|'w
parse value 'abcdefghij' with u 1 v
say u'|'v
parse value 'abcdefghij' with u 5 v 5 w
say u'|'v'|'w
parse value 'abcdefghij' with 6 u 2 v
say u'|'v
```

```output
abcdefg|hij|cdefghij
abcdefghij|abcdefghij
abcd|efghij|efghij
fghij|bcdefghij
```

r[parse.patterns.relative]
A plus or minus sign followed by a whole number puts a point that many
characters after, or before, the start of the previous break. That start is
the start of the matched characters for a string, the position itself for a
position, the beginning of the source before any break, and the end of the
source after a break that found nothing. The point is limited to the
positions from 1 to the end of the source. A relative position does not
begin the piece that precedes it where the previous break ended, as other
triggers do, but where that break began, so the characters of a matched
string are part of that piece. A point that is not after the start of that
piece makes it run to the end of the source, as [for any
position](r:parse.patterns.backward). `+0` and `-0` put the point at the
start of the previous break.

```rexx
parse value 'abcdefghij' with u +3 v +3 w +3 y
say u'|'v'|'w'|'y
parse value 'abcdefghij' with 5 u -2 v
say u'|'v
parse value 'abcdefghij' with u 'g' w +1 y
say u'|'w'|'y
parse value 'abcdefghij' with u 'g' +1 y
say u'|'y
parse value 'abcdefghij' with u 'z' -2 v
say u'|'v
parse value 'abcdefghij' with u 20 -2 v
say u'|'v
parse value 'abcdefghij' with u 3 -9 v
say u'|'v
parse value 'abcdefghij' with u +0 v
say u'|'v
```

```output
abc|def|ghi|j
efghij|cdefghij
abcdef|g|hij
abcdef|hij
abcdefghij|ij
abcdefghij|ij
ab|abcdefghij
abcdefghij|abcdefghij
```

r[parse.patterns.length]
A `>` or `<` followed by a whole number, or by an expression in parentheses,
puts a point relative to the start of the previous break, which is the same
start that [a sign](r:parse.patterns.relative) uses. The number is checked as
[for any position](r:parse.patterns.number). `>n` and `<n` are the two
length patterns, and the next two rules describe them.

```rexx
n = 2
parse value 'abcdefghij' with u > 3 v < (n) w
say u'|'v'|'w
```

```output
abc|bc|bcdefghij
```

r[parse.patterns.length.after]
`>n` puts the point `n` characters after the start of the previous break,
limited to the end of the source, and it is the same as `+n` for every `n`
above 0. At 0 they differ: the piece before `>0` ends at once, where the
previous break starts, so it is empty, and the next piece begins there, while
`+0` makes the preceding piece run to the end of the source.

```rexx
parse value 'abcdefghij' with u >3 v >3 w
say u'|'v'|'w
parse value 'abcdefghij' with u 'c' v >0 w
say '['u']['v']['w']'
parse value 'abcdefghij' with u 'c' v +0 w
say '['u']['v']['w']'
parse value 'abcdefghij' with u >20 v
say '['u']['v']'
n = 2
parse value 'abcdefghij' with u 'f' near <(n) rest >(n) last
say u'|'near'|'rest'|'last
```

```output
abc|def|ghij
[ab][][cdefghij]
[ab][cdefghij][cdefghij]
[abcdefghij][]
abcde|de|de|fghij
```

r[parse.patterns.length.before]
`<n` puts the point `n` characters before the start of the previous break,
limited to position 1, and it does not leave the targets between the previous
break and `<n` the piece that the earlier triggers gave them. Those targets
receive the `n` characters that lie just before the start of the previous
break, or the characters from position 1 when fewer lie there, and the next
piece begins at the point. So `<n` reads again the text before the break that
precedes it. It is not the same as `-n`, which leaves the preceding piece to
run to the end of the source. When no break precedes it, the start is position
1 and the targets before `<n` receive the null string.

```rexx
parse value 'abcdefghij' with u 'f' near <3 rest
say u'|'near'|'rest
parse value 'abcdefghij' with u 5 near <9 rest
say u'|'near'|'rest
parse value 'abcdefghij' with u 5 near <0 rest
say u'|'near'|'rest
parse value 'abcdefghij' with u <2 v
say '['u']['v']'
parse value 'abcdefghij' with u 5 near -3 rest
say u'|'near'|'rest
```

```output
abcde|cde|cdefghij
abcd|abcd|abcdefghij
abcd||efghij
[][abcdefghij]
abcd|efghij|bcdefghij
```

> **Note.** The ANSI draft has no `>` or `<` pattern: they are an addition
> of ooRexx, which this book follows.

r[parse.patterns.number]
The number of a position is a constant written in the template, or the value
of an expression in parentheses after a sign, `=`, `>` or `<`. Blanks may
separate the sign, `=`, `>` or `<` from the number or the parenthesis. The
number, whether constant or expression, must be whole: a constant may be
written in any form whose value is whole, such as `03`, `4.0` or `1e1`, and
`3.5` is not. It must have no more digits than the current [NUMERIC
DIGITS](r:num.settings.digits) allows, so `1e9` is accepted only when that
setting is at least 10, counted on its value (`0999` passes under 3 and
`0.1e4` does not), and it must not be negative. A number that fails any
of these checks raises `SYNTAX 26.4` when the clause runs. A fraction, text,
the null string and a negative value of an expression all raise it; the
direction of a relative position is the sign in the template. A constant
that is a [symbol](r:lex.symbols.constant) is checked the same way when the
clause runs, so `3.5` and `3a` raise `SYNTAX 26.4` then and not when the
program is read. A value of an expression that is an object and not a string
is replaced by its string, as [the conventions](r:conv.string.object) say, and
that string is then checked; this holds after `=`, `+`, `-`, `>` and `<`.

```rexx
parse value 'abcdefghij' with u 03 v 4.0 w 1e1 y
say u'|'v'|'w'|'y
n = ' 3 '
parse value 'abcdefghij' with u =(n) v
say u'|'v
parse value 'abcdefghij' with u + 3 v - (n) w
say u'|'v'|'w
call try 3.5, 'abcdefghij'
call try 'ab', 'abcdefghij'
call try '', 'abcdefghij'
call try -2, 'abcdefghij'
numeric digits 3
call try 999, 'abcdefghij'
call try 1000, 'abcdefghij'
call lit 'u 999 v'
call lit 'u 1000 v'
call lit 'u 0999 v'
call lit 'u 0.1e4 v'
call lit 'u +3a v'
call lit 'u 1e3 v'
call lit 'u +3.5 v'
call lit 'u >1000 v'
numeric digits 12
call lit 'u 1e9 v'
exit
try:
  signal on syntax name e
  p = arg(1)
  parse value arg(2) with u =(p) v
  say p 'ok' u'|'v
  return
e: say p condition('o')~code
  return
lit:
  signal on syntax name e2
  interpret 'parse value "abcdefghij" with' arg(1)
  say arg(1) 'ok'
  return
e2: say arg(1) condition('o')~code
  return
```

```output
ab|c|defghi|j
ab|cdefghij
abc|defghij|abcdefghij
3.5 26.4
ab 26.4
 26.4
-2 26.4
999 ok abcdefghij|
1000 26.4
u 999 v ok
u 1000 v 26.4
u 0999 v ok
u 0.1e4 v 26.4
u +3a v 26.4
u 1e3 v 26.4
u +3.5 v 26.4
u >1000 v 26.4
u 1e9 v ok
```

```rexx
arr = .array~of(2)
parse value 'abcdefghij' with u +(arr) v
say u'|'v
parse value 'abcdefghij' with u >(arr) v
say u'|'v
parse value 'abcdefghij' with u =(arr) v
say u'|'v
buf = .mutablebuffer~new('3')
parse value 'abcdef' with u =(buf) v
say u'|'v
```

```output
ab|cdefghij
ab|cdefghij
a|bcdefghij
ab|cdef
```

## Errors in the template

r[parse.patterns.syntax]
A template is checked when the program is read. A token that cannot be part of
a template, such as `*`, `||`, `>>` or `>=`, raises `SYNTAX 38.1`. A sign,
`=`, `>` or `<` followed by any token other than a number or an opening
parenthesis, a comma or a symbol that does not start with a digit or a period
among them, raises `SYNTAX 38.2`, and one that is the last token of the clause
raises `SYNTAX 38.901`. (A `-` that ends a line is a
[continuation](r:lex.clauses.continuation) and not a template token.) A
program file that holds any of these does not run at all, and its exit code is
218. The text of INTERPRET raises the same error [when it is
read](r:instr.interpret.whole).

```rexx
say 'never shown'
parse value 'abc' with u =n v
```

```output rc=218
```

```rexx
say 'never shown'
parse value 'abc' with u * v
```

```output rc=218
```

```rexx
say 'never shown'
parse value 'abc' with u >
```

```output rc=218
```

```rexx
call try 'u * v'
call try 'u || v'
call try 'u >> v'
call try 'u =n v'
call try 'u + v'
call try 'u > v'
call try 'u < , v'
call try 'u 3 = v'
call try 'u + (1) v'
call try 'u >(1) v'
call try 'u +'
call try 'u 3 ='
call try 'u <'
exit
try:
  signal on syntax name e
  interpret 'parse value "abc" with' arg(1)
  say 'ok' arg(1)
  return
e: say condition('o')~code '<=' arg(1)
  return
```

```output
38.1 <= u * v
38.1 <= u || v
38.1 <= u >> v
38.2 <= u =n v
38.2 <= u + v
38.2 <= u > v
38.2 <= u < , v
38.2 <= u 3 = v
ok u + (1) v
ok u >(1) v
38.901 <= u +
38.901 <= u 3 =
38.901 <= u <
```
