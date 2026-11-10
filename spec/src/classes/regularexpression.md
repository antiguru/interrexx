---
element: "RegularExpression"
kind: class
deviations: [87, 88]
rexxref: "Builtin Classes > Utility Classes > RegularExpression Class; Builtin Classes > Utility Classes > RegularExpression Class > new (Class Method); Builtin Classes > Utility Classes > RegularExpression Class > match; Builtin Classes > Utility Classes > RegularExpression Class > parse; Builtin Classes > Utility Classes > RegularExpression Class > pos; Builtin Classes > Utility Classes > RegularExpression Class > position"
---

# RegularExpression

A `RegularExpression` holds a pattern and tells whether a string matches it,
and where in a string it is found. The pattern language is small: literal
characters, a single-character wildcard, repetition, alternatives, groups and
sets of characters. The class is not preloaded. A program makes it available
with `::requires "rxregexp.cls"`, and the package that file names is part of
every installation. The examples on this page do not use that directive,
because they have to run where the file is not on the search path: each one
declares the class itself, over the entry points of the library `rxregexp`
that make up the class. The declaration is the last eight lines of each
program, with a method `position` that answers the position the last call left
behind; a program that requires `rxregexp.cls` gets the same class.

The page first states the class and how a pattern is read, then the grammar
and what it matches, and last the matching calls. Characters are bytes: the
pattern and the string are compared byte by byte, and no case is folded.

## The class

r[class.regularexpression.creation]
`RegularExpression` has the class method `new` and the instance methods
`init`, `parse`, `match`, `pos` and `position`, besides the methods that every
class has. It may have private and undocumented methods besides these, such as
`uninit`; they are not part of this specification. Its only superclass is
`Object` and it inherits no mixin class ([the hierarchy
rule](r:class.index.hierarchy)). `new` makes an instance of the class that
received it, so a subclass makes instances of itself, and sends `init` the
arguments of `new` ([the new rule](r:class.index.new)). A subclass that
declares its own `init` forwards to this one, which holds the pattern. Until
`parse`, `match` or `pos` has run, `position` answers the string `!POS`, also
after a pattern was given to `new`.

```rexx
re = .RegularExpression~new('ab')
say re~class~id re~class~superclass~id .RegularExpression~queryMixinClass
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
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
```

```output
REGULAREXPRESSION Object 0
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
read](r:class.regularexpression.mode); omitted, it is `MAXIMAL`. A third
argument raises `SYNTAX 88.922`. Both arguments are read as strings: a number
is its digits, and `.nil` and an object without a string value raise
`SYNTAX 88.909`.

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
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
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
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
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
rule](r:class.regularexpression.parse.codes) lists. The pattern is required;
with no argument `SYNTAX 88.901` is raised, with a third one `88.922`, a
`pattern` or `mode` that has no string value `88.909`, and a `mode` that is
not `MAXIMAL` or `MINIMAL`, in any case, `93.0`. `position` answers how many
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
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
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
that has no end. Code 4 is a pattern that ends after an escape, as in `a\`, or
inside a count: `a{` and `a{3`. Code 5 is a count that is not a number or is
below 2, as in `a{}`, `a{x}` and `a{1}`, and a number too large for the
pattern to be built. Code 6 is a symbolic name in a set that is not one of the
[defined names](r:class.regularexpression.grammar.names), or has no closing
colon: `[:foo:]` and `[:alpha]`. `position` says how far reading had come when
the error was found: 0 when the first character is at fault, and the length of
the pattern when the pattern ends too early, as in `(a`, `a|`, `a\`, `[` and
`[a`. The example shows it for a pattern of each kind.

```rexx
re = .RegularExpression~new
patterns = .array~of('', '*', ')', '()', 'a|', '(a|)', '(a', '[', '[a', '[]')
patterns~appendAll(.array~of('[a-]', 'a\', 'a{', 'a{3', 'a{}', 'a{1}'))
patterns~append('a{x}')
patterns~appendAll(.array~of('[:foo:]', '[:alpha]'))
do p over patterns
  say p~left(8) re~parse(p) re~position
end
exit
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
```

```output
         1 0
*        1 0
)        1 0
()       1 1
a|       1 2
(a|)     1 3
(a       2 2
[        3 1
[a       3 2
[]       3 1
[a-]     3 1
a\       4 2
a{       4 1
a{3      4 1
a{}      5 1
a{1}     5 1
a{x}     5 1
[:foo:]  6 1
[:alpha] 6 1
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
element is needed, such a symbol is error 1.

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
exit
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
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
```

r[class.regularexpression.grammar.repeat]
`*` after an element matches it any number of times, none included, `+` one or
more times and `{n}` exactly `n` times, where `n` is a decimal number of 2 or
more; `{0}` and `{1}` are error 5, and so is a count with no digits at the
start or one that is too large. The element may be a character, `?`, a set or
a group, so `?*` matches any string, the empty one included, and `(ab){2}`
matches `abab`. The text between the braces after the number is not specified.
A group that can match the empty string, as in `(a*)`, may itself be repeated;
[Deviation 87](r:class.regularexpression.deviation.87) gives examples.

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
say 'start' re~parse('a{0}') re~parse('a{1}') re~parse('a{2}'),
  re~parse('a{-2}')
exit
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
```

```output
star 0 1 1 0
plus 0 0 1 1
count 0 1 0 0
group 0 1 0 0
any 0 1 1
mixed 0 1 1 0
set 0 1 0
start 5 5 0 5
```

r[class.regularexpression.grammar.set]
A set in square brackets matches one character that is one of its members. The
members are characters, ranges and [symbolic
names](r:class.regularexpression.grammar.names). A `^` right after the opening
bracket negates the set: it matches one character that is not a member; a `^`
anywhere else is an ordinary member. A `-` between two characters makes a
range of all characters from the first to the second, in the byte order, and
the two may be given in either order; a `-` that opens the set, or follows the
`^` that opens it, is an ordinary member, and a `-` that ends the set is error
3. Inside a set the characters `?`, `*`, `+`, `|`, `(`, `)`, `{`, `}` and `.`
stand for themselves; `\` makes the next character an ordinary member, which
is how `]`, `-` and `\` itself are members, and a `:` that is not preceded by
`\` begins a symbolic name. A set must have a member, and its closing bracket
must be present: error 3.

```rexx
re = .RegularExpression~new
say 'members' re~parse('[xyz]') re~match('y') re~match('w')
say 'range' re~parse('[a-d]') re~match('c') re~match('e')
say 'backward' re~parse('[d-a]') re~match('b') re~match('e')
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
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
```

```output
members 0 1 0
range 0 1 0
backward 0 1 0
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
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
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
is required and is read as a string: `SYNTAX 88.901` with none, `88.922` with
a second, `88.909` for `.nil` and objects without a string value.

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
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
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
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
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
takes no argument: any argument is ignored, and it does not change when the
expression is used again; each of the three replaces it.

```rexx
re = .RegularExpression~new('ab')
say re~position
say re~parse('abc') re~position
say re~match('abd') re~position
say re~pos('xabd') re~position re~position(5) re~position
exit
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
```

```output
!POS
0 3
0 2
0 0 0 0
```

## Deviations

r[class.regularexpression.deviation.87]
The pattern language is the one this page gives, and every call ends. ooRexx
differs in three ways. It answers wrongly for a pattern whose first element is
repeated and that has a `|` after it: `b*c|a` is `b*c` or `a`, but ooRexx
reads the second alternative as following the repeated element, as if the
pattern were `b*(c|a)`, so it finds `ba` where interrexx finds only the `a`.
It does not end for a pattern that repeats a group that can match the empty
string, such as `(a*)*`, `(a*)+`, `(?*)+` or `((ab)*|c)*`. And for a count it
cannot build, as in `a{99999999999}`, it uses memory until it is stopped,
where interrexx answers code 5 from `parse`. None of the examples is run on
ooRexx.

```rexx
re = .RegularExpression~new('b*c|a')
say re~match('ba') re~position re~pos('ba') re~position
say re~match('bc') re~position re~match('a') re~position
re~parse('(a*)*')
say re~match('aaa') re~position re~pos('baa') re~position
re~parse('((ab)*|c)*')
say re~match('ababcab') re~position
say re~parse('a{99999999999}')
exit
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
```

```output
0 1 2 2
1 2 1 1
1 3 1 0
1 7
5
```

r[class.regularexpression.deviation.88]
`match` or `pos` on an expression that has never been given a pattern raises
`SYNTAX 93.900`; ooRexx crashes. The example is not run on ooRexx.

```rexx
re = .RegularExpression~new
signal on syntax name bad
say re~match('abc')
bad: say condition('o')~code
signal on syntax name bad2
say re~pos('abc')
bad2: say condition('o')~code
exit
::class RegularExpression
::method init external 'LIBRARY rxregexp RegExp_Init'
::method parse external 'LIBRARY rxregexp RegExp_Parse'
::method match external 'LIBRARY rxregexp RegExp_Match'
::method pos external 'LIBRARY rxregexp RegExp_Pos'
::method position
  expose !pos
  return !pos
```

```output
93.900
93.900
```
