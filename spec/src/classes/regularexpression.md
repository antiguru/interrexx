---
element: "RegularExpression"
kind: class
deviations: [87, 88, 89]
rexxref: "Builtin Classes > Utility Classes > RegularExpression Class; Builtin Classes > Utility Classes > RegularExpression Class > new (Class Method); Builtin Classes > Utility Classes > RegularExpression Class > match; Builtin Classes > Utility Classes > RegularExpression Class > parse; Builtin Classes > Utility Classes > RegularExpression Class > pos; Builtin Classes > Utility Classes > RegularExpression Class > position"
---

# RegularExpression

A `RegularExpression` holds a pattern and tells whether a string matches it,
and where in a string it is found. The pattern language is small: literal
characters, a single-character wildcard, repetition, alternatives, groups and
sets of characters. The class is not preloaded. A program makes it available
with `::requires "rxregexp.cls"`, and the package that file names is part of
every installation; every example on this page does so. A traceback that runs
through that file names interrexx's own line numbers, not ooRexx's, and
nothing else differs ([Deviation 89](r:class.regularexpression.deviation.89)).

The page first states the class and how a pattern is read, then the grammar
and what it matches, and last the matching calls. Characters are bytes: the
pattern and the string are compared byte by byte, and no case is folded. A
`'00'x` ends a pattern, and what `match` and `pos` answer for a string that
contains one is not specified ([the grammar
rule](r:class.regularexpression.grammar) gives the first).

## The class

r[class.regularexpression.creation]
`RegularExpression` has the class method `new` and the instance methods
`init`, `uninit`, `parse`, `match`, `pos` and `position`, besides the methods
that every class has. `uninit` is public (`hasMethod('uninit')` answers 1),
but what it does is not part of this specification, except for the calls that
follow it ([Deviation 88](r:class.regularexpression.deviation.88)). The class
may have private and undocumented methods besides these; they are not part of
this specification either. Its only superclass is `Object` and it inherits no
mixin class ([the hierarchy rule](r:class.index.hierarchy)). `new` makes an
instance of the class that received it, so a subclass makes instances of
itself, and sends `init` the arguments of `new` ([the new
rule](r:class.index.new)). A subclass that declares its own `init` forwards to
this one, which holds the pattern. Until `parse`, `match` or `pos` has run,
`position` answers the string `!POS`, also after a pattern was given to `new`.

```rexx
re = .RegularExpression~new('ab')
say re~class~id re~class~superclass~id .RegularExpression~queryMixinClass
say re~hasMethod('uninit') re~hasMethod('position')
say re~position
re~match('ab')
say re~position
t = .tagged~new('x*', 'first')
say t~class~id t~tag t~match('xxx') t~position
::class tagged subclass RegularExpression
::method init
  expose tag
  use arg pattern, tag
  self~init:super(pattern)
::method tag
  expose tag
  return tag
::requires "rxregexp.cls"
```

```output
RegularExpression Object 0
1 1
!POS
2
TAGGED first 1 3
```

r[class.regularexpression.new]
`new(pattern, mode)` makes an expression. Both arguments are optional. A
`pattern` that is omitted, or an omitted argument in its place as in
`new(, 'MINIMAL')`, leaves the expression without a pattern; a pattern that is
given is parsed as [`parse`](r:class.regularexpression.parse) parses it, and
an error in it raises `SYNTAX 38.0` instead of an answer code, also for the
empty pattern. The `mode` is `MAXIMAL` or `MINIMAL` in any case and nothing
else: any other string, an abbreviation, a string with a blank around it and
the empty string, raise `SYNTAX 93.0`. It chooses [how a match is
read](r:class.regularexpression.mode); omitted, it is `MAXIMAL`. The arguments
are read and checked as [the argument rule](r:class.regularexpression.args)
says.

```rexx
say .RegularExpression~new('a', 'minimal')~match('ab')
say .RegularExpression~new('a', 'Maximal')~match('ab')
say .RegularExpression~new(12)~match(12)
say .RegularExpression~new(, 'MINIMAL')~parse('a')
call try ".RegularExpression~new('[')"
call try ".RegularExpression~new('(a')"
call try ".RegularExpression~new('')"
call try ".RegularExpression~new('a', 'MIN')"
call try ".RegularExpression~new('a', 'bogus')"
call try ".RegularExpression~new('a', '')"
call try ".RegularExpression~new('a', 1)"
call try ".RegularExpression~new('a', 'MAXIMAL', 3)"
call try ".RegularExpression~new(.nil)"
call try ".RegularExpression~new('a', .object~new)"
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=> made'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::requires "rxregexp.cls"
```

```output
1
0
1
0
.RegularExpression~new('[') => 38.0
.RegularExpression~new('(a') => 38.0
.RegularExpression~new('') => 38.0
.RegularExpression~new('a', 'MIN') => 93.0
.RegularExpression~new('a', 'bogus') => 93.0
.RegularExpression~new('a', '') => 93.0
.RegularExpression~new('a', 1) => 93.0
.RegularExpression~new('a', 'MAXIMAL', 3) => 88.922
.RegularExpression~new(.nil) => 88.909
.RegularExpression~new('a', .object~new) => 88.909
```

r[class.regularexpression.args]
`new`, `parse`, `match` and `pos` check their arguments in one order, which is
not the order of [the count rule](r:class.index.args.count). First each
argument that the method takes is checked from the left (the pattern and the
mode of `new` and `parse`, the string of `match` and `pos`): a required one
that is missing raises `SYNTAX 88.901`, and one that cannot be read as a
string `88.909`. Then the number of arguments is checked, one too many raising
`88.922`; an argument beyond the last one the method takes is not read. Then
the mode is checked, `93.0`, and only then the pattern, so that a bad pattern
given to `new` raises `38.0` last. An argument that is not a string is
converted by sending it `request` with `STRING` ([the request
rule](r:class.object.request)), which answers what its `makeString` answers,
or the object itself when it has no such method; when that answer is not a
string the argument raises `SYNTAX 88.909`, and its `string` method is not
used, unlike [the builtins](r:conv.string.object). A number is its digits, and
an `Array` is read as its `makeString` answer.

```rexx
re = .RegularExpression~new('xy')
call try "re~match('a', .nil)"
call try "re~parse('a', 'bogus', 3)"
call try ".RegularExpression~new('[', 'MAXIMAL', 3)"
call try ".RegularExpression~new('[', 'bogus')"
call try "re~parse('[', 'bogus')"
call try "re~match(.str~new)"
call try "re~match(.object~new)"
call try "re~match(.mk~new)"
say .RegularExpression~new(.array~of('a', 'b'))~match('a'||'0a'x||'b')
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=>' x
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class str
::method string
  return 'xy'
::class mk
::method makeString
  return 'xy'
::requires "rxregexp.cls"
```

```output
re~match('a', .nil) => 88.922
re~parse('a', 'bogus', 3) => 88.922
.RegularExpression~new('[', 'MAXIMAL', 3) => 88.922
.RegularExpression~new('[', 'bogus') => 38.0
re~parse('[', 'bogus') => 93.0
re~match(.str~new) => 88.909
re~match(.object~new) => 88.909
re~match(.mk~new) => 1
1
```

```rexx
re = .RegularExpression~new('xy')
call try ".RegularExpression~new(.nil, , 3)"
call try "re~parse(.nil, , 1)"
call try "re~match(.nil, 1)"
call try "re~match(, 1)"
call try "re~pos(, 1)"
call try "re~parse(.nil, 'bogus')"
call try ".RegularExpression~new('a', .nil, 1)"
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=> done'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::requires "rxregexp.cls"
```

```output defect=#202
.RegularExpression~new(.nil, , 3) => 88.909
re~parse(.nil, , 1) => 88.909
re~match(.nil, 1) => 88.909
re~match(, 1) => 88.901
re~pos(, 1) => 88.901
re~parse(.nil, 'bogus') => 88.909
.RegularExpression~new('a', .nil, 1) => 88.909
```

```rexx
re = .RegularExpression~new('xy')
say re~match(.rq~new) re~position
::class rq
::method request
  return 'xy'
::requires "rxregexp.cls"
```

```output defect=#164
1 2
```

r[class.regularexpression.mode]
The mode decides what a match is when several are possible. In `MAXIMAL` mode
a match is as long as it can be: [`match`](r:class.regularexpression.match)
answers 1 only when the whole string matches the pattern, and
[`pos`](r:class.regularexpression.pos) finds the longest match at the leftmost
place where there is one. In `MINIMAL` mode a match is as short as it can be:
`match` answers 1 when the pattern matches a leading part of the string, the
shortest one, and `pos` finds the shortest match at the leftmost place. The
mode of an expression is the one given to `new` or the last one given to
`parse`, and `MAXIMAL` before any was given.

```rexx
text = '<p>one</p><p>two</p>'
big = .RegularExpression~new('<p>?*</p>')
small = .RegularExpression~new('<p>?*</p>', 'MINIMAL')
say big~match(text) big~position small~match(text) small~position
say big~pos(text) big~position small~pos(text) small~position
say big~pos('x'text'y') big~position
exit
::requires "rxregexp.cls"
```

```output
1 20 1 10
1 20 1 10
2 21
```

## Reading a pattern

r[class.regularexpression.parse]
`parse(pattern, mode)` replaces the pattern of the expression and, when `mode`
is given, its mode; without it the mode stays what it was. It answers a code:
0 when the pattern was read, and otherwise one of 1 to 6 that [the next
rule](r:class.regularexpression.parse.codes) lists. The pattern is required.
The arguments are checked as [the argument
rule](r:class.regularexpression.args) says, and a `mode` that is not `MAXIMAL`
or `MINIMAL`, in any case, raises `SYNTAX 93.0`. `position` answers how many
characters of the pattern were read. When the code is 0 that is the length of
the pattern, unless the text after a complete pattern was left unread, as the
[grammar rule](r:class.regularexpression.grammar) says. When the code is not 0
the old pattern is gone and the expression matches every string: a later
`match` answers 1 and `pos` answers 1, both with position 0.

```rexx
re = .RegularExpression~new('ab', 'MINIMAL')
say re~match('abc') re~position
say re~parse('abc') re~position re~match('abc') re~position
say re~parse('abc', 'MAXIMAL') re~match('abcd') re~position
say re~parse('[') re~position
say re~match('zzz') re~position re~match('') re~position
say re~pos('zzz') re~position
call try 're~parse'
call try 're~parse("a", "MAXIMAL", 1)'
call try 're~parse(.nil)'
call try 're~parse("a", "bogus")'
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=> done'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::requires "rxregexp.cls"
```

```output
1 2
0 3 1 3
0 0 3
3 1
1 0 1 0
1 0
re~parse => 88.901
re~parse("a", "MAXIMAL", 1) => 88.922
re~parse(.nil) => 88.909
re~parse("a", "bogus") => 93.0
```

r[class.regularexpression.parse.codes]
The code of `parse` says what was wrong with the pattern. Code 1 is a symbol
where none is allowed, or a pattern that ends where something has to follow: a
pattern that is empty or starts with `*`, `+`, `|`, `)`, `]`, `{` or `}`, an
empty group `()`, and an alternative with nothing in it, as in `a|`, `a||b`
and `(a|)`. Code 2 is a group that is not closed, as in `(a`. Code 3 is a set
that is not valid or not closed: `[`, `[a`, `[]`, `[a-` and `[a-]`, a range
that has no end, and a `\` that ends the pattern inside a set, as in `[a\`.
Code 4 is a pattern that ends after an escape outside a set, as in `a\`, or
inside a count: `a{` and `a{3`. Code 5 is a count that is not a number or is
below 2, as in `a{}`, `a{x}` and `a{1}`, and a number too large for the
pattern to be built. Code 6 is a symbolic name in a set that is not one of the
[defined names](r:class.regularexpression.grammar.names), or has no closing
colon: `[:foo:]` and `[:alpha]`. `position` answers the number of characters
before the one at fault, so 0 when the first character is at fault (`*`), 1
for `()` and 2 for `a|*`, and the length of the pattern when it ends where
something has to follow (`(a`, `a|`, `a\`, `[`, `[a`, `[a-c`). Two kinds of
error count differently. An error in a set other than a missing `]`, which is
codes 3 and 6 and a `\` that ends the pattern inside a set, counts the
characters up to and including the `[` (`[]` 1, `xy[a-]` 3, `[a\` 1,
`xy[:foo:]` 3). An error in a count, codes 4 and 5, counts those before its
`{` (`a{` 1, `xyz{3` 3, `xyz{x}` 3). The example shows it for a pattern of
each kind.

```rexx
re = .RegularExpression~new
patterns = .array~of('', '*', ')', '()', 'a|', '(a|)', '(a', '[', '[a', '[]')
patterns~appendAll(.array~of('[a-]', 'a\', 'a{', 'a{3', 'a{}', 'a{1}'))
patterns~append('a{x}')
patterns~appendAll(.array~of('[:foo:]', '[:alpha]', 'a|*', '[a-c', 'xy[a-]'))
patterns~appendAll(.array~of('[a\', 'xyz{3', 'xyz{x}', 'xy[:foo:]'))
do p over patterns
  say p~left(9) re~parse(p) re~position
end
exit
::requires "rxregexp.cls"
```

```output
          1 0
*         1 0
)         1 0
()        1 1
a|        1 2
(a|)      1 3
(a        2 2
[         3 1
[a        3 2
[]        3 1
[a-]      3 1
a\        4 2
a{        4 1
a{3       4 1
a{}       5 1
a{1}      5 1
a{x}      5 1
[:foo:]   6 1
[:alpha]  6 1
a|*       1 2
[a-c      3 4
xy[a-]    3 3
[a\       3 1
xyz{3     4 3
xyz{x}    5 3
xy[:foo:] 6 3
```

## The pattern language

r[class.regularexpression.grammar]
A pattern is one or more alternatives separated by `|`, and an alternative is
one or more elements. An element is a character that stands for itself, the
wildcard `?`, a set in square brackets, or a group in parentheses, and it may
be followed by one quantifier. Any character that is not special stands for
itself, a blank, `.`, `^`, `$`, `-`, `:` and `,` among them; case counts. `?`
stands for any one character. `\` makes the character after it stand for
itself, whatever it is; a `\` that ends the pattern is error 4. An alternative
may not be empty, and neither may a group. `|` separates whole alternatives:
it ranks below joining elements, so `ab|cd` is `ab` or `cd`, and a group is
how the choice is limited to part of a pattern. A `*` or `+` or `{n}` applies
to the element before it. Reading stops, without an error, at a symbol that
cannot continue a pattern that is complete: a `)` with no group open, a second
quantifier after a quantifier (`a**`, `a+*`, `a{2}+`), a `]`, or a `}` after
an element; `parse` answers 0 and `position` says how many characters were
read, so the rest is not part of the pattern and has no effect. Where an
element is needed, such a symbol is error 1. A `'00'x` in the pattern ends it
in the same way: what follows is not read, and `position` counts the
characters before it.

```rexx
re = .RegularExpression~new
say 'literals' re~parse('a b.c') re~match('a b.c') re~match('a bxc')
say 'case' re~parse('abc') re~match('ABC')
say 'wildcard' re~parse('a?c') re~match('abc') re~match('a-c') re~match('ac')
say 'escape' re~parse('a\?c\\') re~match('a?c\') re~match('abc\')
say 'colon' re~parse('a:b-c') re~match('a:b-c')
say 'plain' re~parse('a^b$c,d') re~match('a^b$c,d')
say 'alternatives' re~parse('ab|cd|e') re~match('ab') re~match('cd'),
  re~match('e') re~match('abcd')
say 'groups' re~parse('(a|b)(c|d)') re~match('ad') re~match('bc'),
  re~match('ab')
say 'nested' re~parse('((a)(b|(c)))') re~match('ab') re~match('ac')
say 'one char' re~match('a')
say 'stops' re~parse('ab)cd') re~position re~match('ab')
say 'stops' re~parse('a**') re~position re~match('aaa') re~match('a')
say 'stops' re~parse('a]b') re~position re~match('a')
say 'bytes' re~parse('?') re~match('é') re~parse('??') re~match('é')
say 'nul' re~parse('a'||'00'x||'b') re~position re~match('a')
say 'nul' re~parse('['||'a'||'00'x||']') re~position
say 'nul' re~parse('00'x) re~position
exit
::requires "rxregexp.cls"
```

```output
literals 0 1 0
case 0 0
wildcard 0 1 1 0
escape 0 1 0
colon 0 1
plain 0 1
alternatives 0 1 1 1 0
groups 0 1 1 0
nested 0 1 1
one char 0
stops 0 2 1
stops 0 2 1 1
stops 0 1 1
bytes 0 0 0 1
nul 0 1 1
nul 3 2
nul 1 0
```

r[class.regularexpression.grammar.repeat]
`*` after an element matches it any number of times, none included, `+` one or
more times and `{n}` exactly `n` times. The count is read as a whole number
from the text between the braces: blanks before it and a sign are allowed, it
ends at the first character that is not a digit, and the text after it up to
the `}` is ignored, so `a{2,3}` is `a{2}`. A count with no digit, or one below
2, is error 5, and so is one too large for the pattern to be built; how large
that is is not specified. The element may be a character, `?`, a set or a
group, so `?*` matches any string, the empty one included, and `(ab){2}`
matches `abab`. A group that can match the empty string, as in `(a*)`, may
itself be repeated; [Deviation 87](r:class.regularexpression.deviation.87)
gives examples.

```rexx
re = .RegularExpression~new
say 'star' re~parse('ab*c') re~match('ac') re~match('abbbc') re~match('abd')
say 'plus' re~parse('ab+c') re~match('ac') re~match('abc') re~match('abbbc')
say 'count' re~parse('ab{3}c') re~match('abbbc') re~match('abbc'),
  re~match('abbbbc')
say 'group' re~parse('(ab){2}') re~match('abab') re~match('ab'),
  re~match('ababab')
say 'any' re~parse('?*') re~match('') re~match('anything at all')
say 'mixed' re~parse('x?*y') re~match('xy') re~match('xaby') re~match('xab')
say 'set' re~parse('[ab]+') re~match('abba') re~match('abca')
say 'braces' re~parse('a{ 3}') re~match('aaa') re~parse('a{2,3}'),
  re~match('aa') re~match('aaa') re~parse('a{+2}') re~match('aa')
say 'start' re~parse('a{0}') re~parse('a{1}') re~parse('a{2}'),
  re~parse('a{-2}')
exit
::requires "rxregexp.cls"
```

```output
star 0 1 1 0
plus 0 0 1 1
count 0 1 0 0
group 0 1 0 0
any 0 1 1
mixed 0 1 1 0
set 0 1 0
braces 0 1 0 1 0 0 1
start 5 5 0 5
```

r[class.regularexpression.grammar.set]
A set in square brackets matches one character that is one of its members. The
members are characters, ranges and [symbolic
names](r:class.regularexpression.grammar.names). A `^` right after the opening
bracket negates the set: it matches one character that is not a member; a `^`
anywhere else is an ordinary member. A `-` between two characters makes a
range. For a range the bytes count as numbers from -128 for `'80'x` up to 127
for `'7f'x`, so `'80'x` to `'ff'x` come before `'00'x`. When the first
character is not above the second, the range is all characters from the first
to the second; when it is above, it is the characters above the second up to
the first, so `[d-a]` is `b`, `c` and `d`. In a chain such as `[a-c-e]` the
character that ends a range also starts the next one, so that set is `a` to
`e`. A `\` right after the `-` of a range is the end of the range and not an
escape, and a `-` after a symbolic name starts a range at the closing colon of
the name, which is left out when the end is above it: `[:digit:-z]` is the
digits and the characters from `;` to `z`. A `[` inside a set is an ordinary
member. A `-` that opens the set, or follows the `^` that opens it, is an
ordinary member, and a `-` that ends the set is error 3. Inside a set the
characters `?`, `*`, `+`, `|`, `(`, `)`, `{`, `}` and `.` stand for
themselves; `\` makes the next character an ordinary member, which is how `]`,
`-` and `\` itself are members, and a `:` that is not preceded by `\` begins a
symbolic name. A set must have a member, and its closing bracket must be
present: error 3.

```rexx
re = .RegularExpression~new
say 'members' re~parse('[xyz]') re~match('y') re~match('w')
say 'range' re~parse('[a-d]') re~match('c') re~match('e')
say 'backward' re~parse('[d-a]') re~match('b') re~match('e') re~match('a')
say 'signed' re~parse('[a-'||'c3'x||']') re~match('ff'x) re~match('c3'x),
  re~match('b')
say 'high' re~parse('['||'80'x||'-'||'ff'x||']') re~match('90'x) re~match('a')
say 'chain' re~parse('[a-c-e]') re~match('d') re~match('b') re~match('f')
say 'late end' re~parse('[a-\c]') re~match('a') re~match(']') re~match('b'),
  re~match('\') re~match('c')
say 'after name' re~parse('[:digit:-z]') re~match(';') re~match(':'),
  re~match('5') re~match('{')
say 'bracket' re~parse('[[]') re~match('[') re~match('a')
say 'ranges' re~parse('[a-cx-z]') re~match('b') re~match('y') re~match('m')
say 'negated' re~parse('[^a-c]') re~match('d') re~match('b')
say 'caret' re~parse('[a^]') re~match('^') re~parse('[^^]') re~match('^'),
  re~match('a')
say 'dash' re~parse('[-a]') re~match('-') re~match('b')
say 'after caret' re~parse('[^-a]') re~match('-') re~match('b')
say 'plain' re~parse('[?*+|(){}.]') re~match('?') re~match('(') re~match('.'),
  re~match('x')
say 'escaped' re~parse('[\]\-\\]') re~match(']') re~match('-'),
  re~match('\') re~match('a')
say 'colon' re~parse('[\:a]') re~match(':') re~match('a')
say 'errors' re~parse('[a-]') re~parse('[]') re~parse('[abc')
exit
::requires "rxregexp.cls"
```

```output
members 0 1 0
range 0 1 0
backward 0 1 0 0
signed 0 1 0 0
high 0 1 0
chain 0 1 1 0
late end 0 1 1 0 0 1
after name 0 1 0 1 0
bracket 0 1 0
ranges 0 1 1 0
negated 0 1 0
caret 0 1 0 0 1
dash 0 1 0
after caret 0 0 1
plain 0 1 1 1 0
escaped 0 1 1 1 0
colon 0 1 1
errors 3 3 3
```

r[class.regularexpression.grammar.names]
A symbolic name in a set stands for a group of characters. A name is written
between two colons and is read without regard to case; it may be mixed with
other members, and `[^:digit:]` negates it. The names and what they stand for:
`alpha` is the letters `A` to `Z` and `a` to `z`, `lower` is `a` to `z`,
`upper` is `A` to `Z`, `digit` is `0` to `9`, `alnum` is `alpha` and `digit`,
`xdigit` is `digit`, `A` to `F` and `a` to `f`, `blank` is space and tab,
`space` is `'09'x` to `'0d'x` and space, `cntrl` is `'00'x` to `'1f'x` and
`'7f'x`, `print` is `'20'x` to `'7e'x`, `graph` is `print` without the space,
and `punct` is `print` without the space, the letters and the digits. A `:` in
a set that does not begin a name this way, such as the colon in `[a:b]`, is
error 6. A name outside a set is plain text: `:alpha:` is the seven
characters.

```rexx
re = .RegularExpression~new
names = 'alpha lower upper digit alnum xdigit blank space cntrl print'
names = names 'graph punct'
do name over names~makeArray(' ')
  re~parse('[:'name':]')
  hits = ''
  chars = .array~of('a', 'Z', '7', 'f', 'g', ' ', '09'x, '0a'x, '1f'x, '7f'x)
  chars~append('!')
  do ch over chars
    hits = hits re~match(ch)
  end
  say name~left(6) hits
end
say 'mixed' re~parse('[:digit:x]') re~match('x') re~match('5') re~match('y')
say 'negated' re~parse('[^:digit:]') re~match('x') re~match('5')
say 'case' re~parse('[:ALPHA:]') re~match('q')
say 'outside' re~parse(':alpha:') re~match(':alpha:') re~match('a')
say 'errors' re~parse('[a:b]') re~parse('[:]') re~parse('[:alpha'),
  re~parse('[:abc:]')
exit
::requires "rxregexp.cls"
```

```output
alpha   1 1 0 1 1 0 0 0 0 0 0
lower   1 0 0 1 1 0 0 0 0 0 0
upper   0 1 0 0 0 0 0 0 0 0 0
digit   0 0 1 0 0 0 0 0 0 0 0
alnum   1 1 1 1 1 0 0 0 0 0 0
xdigit  1 0 1 1 0 0 0 0 0 0 0
blank   0 0 0 0 0 1 1 0 0 0 0
space   0 0 0 0 0 1 1 1 0 0 0
cntrl   0 0 0 0 0 0 1 1 1 1 0
print   1 1 1 1 1 1 0 0 0 0 1
graph   1 1 1 1 1 0 0 0 0 0 1
punct   0 0 0 0 0 0 0 0 0 0 1
mixed 0 1 1 0
negated 0 1 0
case 0 1
outside 0 1 0
errors 6 6 6 6
```

## Matching

r[class.regularexpression.match]
`match(string)` answers 1 when the pattern matches the string as [the
mode](r:class.regularexpression.mode) says, and 0 when it does not, and
`position` answers a number. When the answer is 1 the number is the length of
the part of the string that matched: the whole string in `MAXIMAL` mode, the
shortest leading part in `MINIMAL` mode. When it is 0 the number is the length
of the longest leading part of the string that can still be the beginning of a
match of the pattern, which is 0 when the first character already fails, and
the length of the string when the string ended too early. The string argument
is required; it is read and checked as [the argument
rule](r:class.regularexpression.args) says.

```rexx
re = .RegularExpression~new('ab?d')
say re~match('abcd') re~position
say re~match('abc') re~position
say re~match('abcde') re~position
say re~match('xbcd') re~position
say re~match('') re~position
say re~match('ab') re~position
mi = .RegularExpression~new('a?*d', 'MINIMAL')
say mi~match('abdcd') mi~position mi~match('ab') mi~position
call try 're~match'
call try 're~match("a", 1)'
call try 're~match(.nil)'
call try 're~match(.object~new)'
say re~match(12) re~position
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=> done'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::requires "rxregexp.cls"
```

```output
1 4
0 3
0 4
0 0
0 0
0 2
1 3 0 2
re~match => 88.901
re~match("a", 1) => 88.922
re~match(.nil) => 88.909
re~match(.object~new) => 88.909
0 0
```

r[class.regularexpression.pos]
`pos(string)` looks for the pattern in the string, and answers the position of
the first character of the leftmost place where it matches, counting from 1,
or 0 when there is none. `position` then answers the position of the last
character of that match, so a match of `n` characters from position `p` ends
at `p+n-1`, and an empty match, which a pattern that can match the empty
string has at position 1, ends at 0. When there is no match both are 0. The
match is the longest in `MAXIMAL` mode and the shortest in `MINIMAL` mode, and
among matches that begin at one place only that place counts: the leftmost
place wins over the length. The argument is checked as for `match`.

```rexx
text = 'in 2016, 42 and 7'
re = .RegularExpression~new('[0-9]+')
b = re~pos(text)
say b re~position text~substr(b, re~position - b + 1)
mi = .RegularExpression~new('[0-9]+', 'MINIMAL')
b = mi~pos(text)
say b mi~position text~substr(b, mi~position - b + 1)
say re~pos('none') re~position
nul = .RegularExpression~new('b*')
say nul~pos('abb') nul~position nul~pos('') nul~position
alt = .RegularExpression~new('ab|abcd')
say alt~pos('xabcd') alt~position
call try 're~pos'
call try 're~pos("a", 1)'
call try 're~pos(.nil)'
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=> done'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::requires "rxregexp.cls"
```

```output
4 7 2016
4 4 2
0 0
1 0 1 0
2 5
re~pos => 88.901
re~pos("a", 1) => 88.922
re~pos(.nil) => 88.909
```

r[class.regularexpression.position]
`position` answers the position that the last `parse`, `match` or `pos` left
behind, as those rules say, and the string `!POS` before any of them ran. It
takes no argument: any argument raises `SYNTAX 93.902`. Sending it does not
change the position; each of the three methods replaces it.

```rexx
re = .RegularExpression~new('ab')
say re~position
say re~parse('abc') re~position
say re~match('abd') re~position
say re~pos('xabcd') re~position re~position
call try 're~position(5)'
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=> done'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::requires "rxregexp.cls"
```

```output
!POS
0 3
0 2
2 4 4
re~position(5) => 93.902
```

## Deviations

r[class.regularexpression.deviation.87]
The pattern language is the one this page gives, and every call ends. ooRexx
differs in many patterns; among them, it answers wrongly when a `+` is
followed by a repeated element, as `c+a*`, which it does not match against
`cca`, and when an alternative that begins with a repeated element is followed
by another one, at the top or in a group: `b*c|a` is `b*c` or `a`, but ooRexx
reads the second alternative as following the repeated element, so it finds
`ba`. It does not end for a pattern that repeats a group that can match the
empty string, such as `(a*)*`, for one that begins with a group whose first
alternative can match the empty string and is followed by `|`, such as
`(c*|b)|d`, and for a set with a range that ends at `'7f'x`, and it crashes on
others, such as `(c*|b)c|d`. And for a count it cannot build, as in
`a{99999999999}`, it uses memory until it is stopped, where interrexx answers
code 5 from `parse`. None of the examples is run on ooRexx.

```rexx
re = .RegularExpression~new('b*c|a')
say re~match('ba') re~position re~pos('ba') re~position
say re~match('bc') re~position re~match('a') re~position
re~parse('c+a*')
say re~match('cca') re~position
re~parse('x(b*c|a)')
say re~match('xba') re~position
re~parse('(a*)*')
say re~match('aaa') re~position re~pos('baa') re~position
re~parse('((ab)*|c)*')
say re~match('ababcab') re~position
say re~parse('a{99999999999}')
exit
::requires "rxregexp.cls"
```

```output
0 1 2 2
1 2 1 1
1 3
0 2
1 3 1 0
1 7
5
```

r[class.regularexpression.deviation.88]
`match` or `pos` on an expression that has never been given a pattern raises
`SYNTAX 93.900`, and so do `match`, `pos` and `parse` sent to an expression
after `uninit` has been sent to it; `position` then still answers `!POS`, and
a second `uninit` answers no value. ooRexx crashes on a call of the class
after `uninit`. The example is not run on ooRexx.

```rexx
re = .RegularExpression~new
call try "re~match('abc')"
call try "re~pos('abc')"
gone = .RegularExpression~new('ab')
gone~uninit
call try "gone~match('ab')"
call try "gone~pos('ab')"
call try "gone~parse('ab')"
call try "gone~position"
call try "gone~uninit"
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=>' x
  return
bad: say arg(1) '=>' condition('o')~code
  return
::requires "rxregexp.cls"
```

```output
re~match('abc') => 93.900
re~pos('abc') => 93.900
gone~match('ab') => 93.900
gone~pos('ab') => 93.900
gone~parse('ab') => 93.900
gone~position => !POS
gone~uninit => 91.999
```

r[class.regularexpression.deviation.89]
`rxregexp.cls` is written for interrexx, so a traceback that runs through it
names interrexx's line numbers: the `position` method raises its error from a
line that interrexx numbers 20 and ooRexx numbers 56. The example is not run
on ooRexx.

```rexx
re = .RegularExpression~new('xy')
signal on syntax name bad
re~position(1)
bad:
say condition('o')~traceback~firstItem~word(1)
::requires "rxregexp.cls"
```

```output
20
```
