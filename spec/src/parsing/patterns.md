---
element: "Patterns"
kind: concept
ansi: "8.3.17"
rexxref: "Parsing > Templates Containing String Patterns"
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
A string in quotes finds the first place, at or after the point where the
piece begins, where its characters occur in the source, comparing each
character exactly as it is, so case matters. The break is the span of that
occurrence: the piece ends just before it and the next piece begins just
after it, and the matched characters belong to no piece. An occurrence at the
very beginning of the piece leaves an empty piece before it, and an empty
piece gives its targets the null string.

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
receives the null string, as does each target of a source that is itself
null. A later string cannot match either.

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
An expression in parentheses is evaluated when the template reaches it,
which is after the assignment of the targets before it, and its value is a
string trigger as [the string rule](r:parse.patterns.string) describes. The
value is used as the string that it is, so a value that looks like a number
is still the characters to find, and a value that is the null string is the
null string, which [never matches](r:parse.patterns.string.missing). The
expression may be as long as any expression, with operators and function
calls.

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
```

```output
ab|cd,ef
abc|efg
[abcdefg][]
[ab,cd,ef][]
ab|d,ef
```

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

r[parse.patterns.number]
The number of a position is a constant written in the template, or the value
of an expression in parentheses after a sign or `=`. Blanks may separate the
sign or `=` from the number or the parenthesis. A constant may be any number
whose value is whole, such as `03`, `4.0` or `1e1`, and one that is not, such
as `3.5`, raises `SYNTAX 26.4` when the clause runs. An expression's value is
taken with the current [NUMERIC DIGITS](r:num.settings.digits): it must be
a whole number that is not negative and has no more digits than that
setting, or the parse raises the same error. A fraction, text, the null
string and a negative value all raise it; the direction of a relative position
is the sign in the template.

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
exit
try:
  signal on syntax name e
  p = arg(1)
  parse value arg(2) with u =(p) v
  say p 'ok' u'|'v
  return
e: say p condition('o')~code
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
```

## Errors in the template

r[parse.patterns.syntax]
A template is checked when the program is read. A token that cannot be part
of a template, such as an operator character, raises `SYNTAX 38.1`, and a
sign or `=` that is followed by neither a number nor an opening parenthesis
raises `SYNTAX 38.2`. A program file that holds either does not run at all,
and its exit code is 218. The text of INTERPRET raises the same error
[when it is read](r:instr.interpret.whole).

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
call try 'u * v'
call try 'u || v'
call try 'u =n v'
call try 'u + v'
call try 'u 3 = v'
call try 'u + (1) v'
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
38.2 <= u =n v
38.2 <= u + v
38.2 <= u 3 = v
ok u + (1) v
```
