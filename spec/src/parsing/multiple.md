---
element: "Several strings and case"
kind: concept
ansi: "8.3.17"
rexxref: "Parsing > Advanced Topics in Parsing > Parsing Several Strings, Parsing > Using UPPER, LOWER, and CASELESS"
---

# Several strings and case

This page covers the two features that act on a whole parse and not on one
template: the comma that moves to the next template and the next source
string, and the options UPPER, LOWER and CASELESS that decide how the source
is read. The instructions chapter specifies the keywords of PARSE, ARG and
PULL that give a source and select an option. This page specifies what the
options do to a parse.

## Several templates

r[parse.multiple.comma]
A comma ends the template before it and begins the next one, and the next
template parses the next source string. Each template starts afresh on its
own string: its first piece begins at the first character, and a relative
position before any trigger counts from there. The instructions that take
their source from the arguments of a call, ARG and PARSE ARG, have one
string for each argument, in order. An argument that was omitted is the null
string.

```rexx
call show 'ab cd', 'ef=gh', 'ij kl mn'
call show 'ab cd', , 'ij kl mn'
exit
show: parse arg u v, w '=' y, z +2 q
  say u'|'v'|'w'|'y'|'z'|'q
  return
```

```output
ab|cd|ef|gh|ij| kl mn
ab|cd|||ij| kl mn
```

r[parse.multiple.single]
A source of any other kind, such as a VALUE or a VAR, is one string. Its first
template parses it. A template with no string left to parse, whether it
follows such a source or the last argument, parses the null string, so each
target of it receives the null string and a pattern in it finds nothing.

```rexx
parse value 'ab cd ef' with u v, w y, z
say '['u']['v']['w']['y']['z']'
data = 'one two'
parse var data first, second
say first'|'second'|'
```

```output
[ab][cd ef][][][]
one two||
```

r[parse.multiple.counts]
A source string with no template after the last comma is not parsed, and no
variable sees it. A template that is missing, for example the one before a
leading comma, assigns [nothing](r:parse.templates.empty) and still uses up
its string.

```rexx
call one 'ab', 'cd', 'ef'
call two 'ab'
call three 'ab', 'cd'
exit
one: parse arg u
  say u
  return
two: parse arg u, v, w
  say '['u']['v']['w']'
  return
three: parse arg , v
  say v
  return
```

```output
ab
[ab][][]
cd
```

r[parse.multiple.shared]
The templates of one list are parsed from left to right, each after the one
before it, so the variables assigned by an earlier template are in place for
the later ones. The [order rule](r:parse.index.order) applies across the
commas as well.

```rexx
call mark 'a-b', 'x-a-z'
exit
mark: parse arg first '-' second, third (first) fourth
  say first second third fourth
  return
```

```output
a b x- -z
```

## Case

r[parse.multiple.convert]
The options UPPER and LOWER change the source before any matching: UPPER
makes each of the ASCII letters `a` to `z` upper case, and LOWER makes each of
`A` to `Z` lower case, and no other character changes. Every source string is
changed, so a parse with several strings changes them all. The values
assigned are pieces of the changed string. The patterns of the template are
not changed, so a literal pattern must be written in the case that the
option produces, or it will not match.

```rexx
s = 'Alpha-BETA-gamma'
parse upper var s u '-' v '-' w
say u v w
parse lower var s u '-' v '-' w
say u v w
parse upper var s u 'beta' v
say '['u']['v']'
parse upper var s u 'BETA' v
say '['u']['v']'
parse lower value 'ABC É' with u v
say u v
call two 'Ab-Cd', 'eF-gH'
exit
two: parse upper arg p '-' q, r '-' s
  say p q r s
  return
```

```output
ALPHA BETA GAMMA
alpha beta gamma
[ALPHA-BETA-GAMMA][]
[ALPHA-][-GAMMA]
abc É
AB CD EF GH
```

r[parse.multiple.caseless]
The option CASELESS makes the matching of string triggers ignore the case
of the ASCII letters: `a` and `A` count as the same character, and so on for
each letter. It applies to string patterns, and to the value of a variable
pattern in the same way. The source is not changed, so the values assigned
keep their case, and positions, words and the whitespace rules are as they
are without it. Without CASELESS, matching is
[exact](r:parse.patterns.string). A conversion and CASELESS may be given
together, and then the conversion changes the source and the match ignores
the case of what is left.

```rexx
s = 'Alpha-BETA-gamma'
parse caseless var s u 'beta' v
say u'|'v
parse var s u 'beta' v
say u'|'v
pat = 'bEtA'
parse caseless var s u (pat) v
say u'|'v
parse caseless var s u 'GAMMA'
say u
parse caseless value 'xyz' with u 'Y' v
say u v
parse upper caseless value 'Ab-cD' with u 'cd' v
say u'|'v
```

```output
Alpha-|-gamma
Alpha-BETA-gamma|
Alpha-|-gamma
Alpha-BETA-
x z
AB-|
```

> **Note.** The ANSI draft (8.3.17) has the option UPPER and no other: it has
> neither LOWER nor CASELESS, which ooRexx adds and this book follows.
