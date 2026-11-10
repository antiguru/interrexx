---
element: "MutableBuffer"
kind: class
rexxref: "Builtin Classes > Utility Classes > MutableBuffer Class; Builtin Classes > Utility Classes > MutableBuffer Class > new (Class Method); Builtin Classes > Utility Classes > MutableBuffer Class > []; Builtin Classes > Utility Classes > MutableBuffer Class > []=; Builtin Classes > Utility Classes > MutableBuffer Class > append; Builtin Classes > Utility Classes > MutableBuffer Class > caselessChangeStr; Builtin Classes > Utility Classes > MutableBuffer Class > caselessContains; Builtin Classes > Utility Classes > MutableBuffer Class > caselessContainsWord; Builtin Classes > Utility Classes > MutableBuffer Class > caselessCountStr; Builtin Classes > Utility Classes > MutableBuffer Class > caselessEndsWith; Builtin Classes > Utility Classes > MutableBuffer Class > caselessLastPos; Builtin Classes > Utility Classes > MutableBuffer Class > caselessMatch; Builtin Classes > Utility Classes > MutableBuffer Class > caselessMatchChar; Builtin Classes > Utility Classes > MutableBuffer Class > caselessPos; Builtin Classes > Utility Classes > MutableBuffer Class > caselessStartsWith; Builtin Classes > Utility Classes > MutableBuffer Class > caselessWordPos; Builtin Classes > Utility Classes > MutableBuffer Class > changeStr; Builtin Classes > Utility Classes > MutableBuffer Class > contains; Builtin Classes > Utility Classes > MutableBuffer Class > containsWord; Builtin Classes > Utility Classes > MutableBuffer Class > countStr; Builtin Classes > Utility Classes > MutableBuffer Class > delete / delStr; Builtin Classes > Utility Classes > MutableBuffer Class > delWord; Builtin Classes > Utility Classes > MutableBuffer Class > endsWith; Builtin Classes > Utility Classes > MutableBuffer Class > getBufferSize; Builtin Classes > Utility Classes > MutableBuffer Class > insert; Builtin Classes > Utility Classes > MutableBuffer Class > lastPos; Builtin Classes > Utility Classes > MutableBuffer Class > length; Builtin Classes > Utility Classes > MutableBuffer Class > lower; Builtin Classes > Utility Classes > MutableBuffer Class > makeArray; Builtin Classes > Utility Classes > MutableBuffer Class > match; Builtin Classes > Utility Classes > MutableBuffer Class > matchChar; Builtin Classes > Utility Classes > MutableBuffer Class > overlay; Builtin Classes > Utility Classes > MutableBuffer Class > pos; Builtin Classes > Utility Classes > MutableBuffer Class > replaceAt; Builtin Classes > Utility Classes > MutableBuffer Class > setBufferSize; Builtin Classes > Utility Classes > MutableBuffer Class > setText; Builtin Classes > Utility Classes > MutableBuffer Class > space; Builtin Classes > Utility Classes > MutableBuffer Class > startsWith; Builtin Classes > Utility Classes > MutableBuffer Class > string; Builtin Classes > Utility Classes > MutableBuffer Class > subChar; Builtin Classes > Utility Classes > MutableBuffer Class > substr; Builtin Classes > Utility Classes > MutableBuffer Class > subWord; Builtin Classes > Utility Classes > MutableBuffer Class > subWords; Builtin Classes > Utility Classes > MutableBuffer Class > translate; Builtin Classes > Utility Classes > MutableBuffer Class > upper; Builtin Classes > Utility Classes > MutableBuffer Class > verify; Builtin Classes > Utility Classes > MutableBuffer Class > word; Builtin Classes > Utility Classes > MutableBuffer Class > wordIndex; Builtin Classes > Utility Classes > MutableBuffer Class > wordLength; Builtin Classes > Utility Classes > MutableBuffer Class > wordPos; Builtin Classes > Utility Classes > MutableBuffer Class > words"
deviations: [4, 9, 109]
---

# MutableBuffer

A `MutableBuffer` holds a string of characters that a program changes in
place. Where a `String` is a value, so that every operation answers a new
string, a buffer is an object with an identity: `append`, `insert`, `upper`
and the other changing methods alter the characters it holds and answer the
buffer itself. Beside the characters a buffer has a size, the number of
characters it can hold before it has to grow, which a program can read and
set. A buffer is no `String`: it understands a part of the `String` methods,
and for those it states only what differs, the rest being linked to the page
that owns it.

## The class

r[class.mutablebuffer.creation]
`MutableBuffer` is a class whose only superclass is `Object`; it inherits no
mixin class ([the hierarchy rule](r:class.index.hierarchy)), and it is not a
subclass of [`String`](string.md). It defines the instance methods of this
page, and `new` is [stated below](r:class.mutablebuffer.new). A message that
names a `String` method the buffer does not define, such as `left` or
`abbrev`, and an arithmetic operator sent to a buffer, raise `SYNTAX 97.1`
([the error rule](r:class.index.errors)). The comparison operators and the
joining operators are covered by [the comparison
rule](r:class.mutablebuffer.comparison) and [the string value
rule](r:class.mutablebuffer.string).

```rexx
b = .mutablebuffer~new('abc')
say b~class~id b~isA(.string) b~isInstanceOf(.mutablebuffer)
do c over .mutablebuffer~superclasses
  say c~id
end
say b~hasMethod('APPEND') b~hasMethod('LEFT') b~hasMethod('ABBREV')
call try 'b~left(2)'
call try 'b + 1'
call try 'b~abbrev("a")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
MutableBuffer 0 1
Object
1 0 0
b~left(2) => 97.1
b + 1 => 97.1
b~abbrev("a") => 97.1
```

r[class.mutablebuffer.new]
`new` makes a buffer from at most two arguments, and sends the new buffer
`init` with the arguments after them ([the new rule](r:class.index.new)). The
first argument, the text, is read for its string value as [the argument
rule](r:class.index.string) says, and defaults to the null string; an object
with no string value, `.nil` included, raises `SYNTAX 88.909`. The second
argument, the size, is read after the first and must be a non-negative whole
number, or the call raises `SYNTAX 93.923`; it defaults to 256 when it is
omitted. A whole-number argument of a method of this class is read as that of
a builtin is ([the form rule](r:conv.whole.form), [the digits
rule](r:conv.whole.digits)), so NUMERIC DIGITS plays no part, and only the
error code differs. The buffer holds the text and has the size given, except
that a size smaller than the length of the text is raised to that length. A
subclass receiver gets an instance of itself. Both arguments are checked
before `init` is sent, so a failing `new` runs no `init`, and the reading of
the first argument comes before the check of the second.

```rexx
call show .mutablebuffer~new
call show .mutablebuffer~new('hello')
call show .mutablebuffer~new('hello', 10)
call show .mutablebuffer~new(, 40)
call show .mutablebuffer~new('hello', 2)
call show .mutablebuffer~new('hello', 0)
call show .mutablebuffer~new(copies('x', 300))
call try ".mutablebuffer~new('a', 'x')"
call try ".mutablebuffer~new('a', -1)"
call try ".mutablebuffer~new('a', 1.5)"
call try ".mutablebuffer~new(.nil)"
call try ".mutablebuffer~new(.nil, 'x')"
call try ".mutablebuffer~new('a', 1, 2)"
exit
show: procedure
  b = arg(1)
  say '<' || left(b~string, 8) || '>' b~length b~getBufferSize
  return
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
<        > 0 256
<hello   > 5 256
<hello   > 5 10
<        > 0 40
<hello   > 5 5
<hello   > 5 5
<xxxxxxxx> 300 300
.mutablebuffer~new('a', 'x') => 93.923
.mutablebuffer~new('a', -1) => 93.923
.mutablebuffer~new('a', 1.5) => 93.923
.mutablebuffer~new(.nil) => 88.909
.mutablebuffer~new(.nil, 'x') => 88.909
.mutablebuffer~new('a', 1, 2) => 93.902
```

r[class.mutablebuffer.new.init]
The buffer already holds its text and has its size when `init` is sent, and
`init` gets no argument unless `new` had more than two: it gets those after
the second, in order. The `init` of `Object` takes no argument, so a call of
`new` with a third argument raises `SYNTAX 93.902` unless a subclass declares
an `init` that takes it. An `init` of a subclass need not call its superclass:
the buffer is complete without it. `init` sent to a buffer that exists accepts
no argument either, and leaves it as it was.

```rexx
call try ".keep~new('abc', 40)"
call try ".keep~new('abc', 40, 'x', 'y')"
call try ".skip~new('abc')~string"
call try ".keep~new('abc', 'x')"
call try ".mutablebuffer~new('abc', 5, 'x')"
b = .mutablebuffer~new('abc', 9)
b~init
say b~string b~getBufferSize
call try 'b~init(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r~class~id r~string
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class keep subclass mutablebuffer
::method init
  say 'init got' arg() 'content' self~string 'size' self~getBufferSize
::class skip subclass mutablebuffer
::method init
  say 'init does not call its superclass'
```

```output
init got 0 content abc size 40
.keep~new('abc', 40) => KEEP abc
init got 2 content abc size 40
.keep~new('abc', 40, 'x', 'y') => KEEP abc
init does not call its superclass
.skip~new('abc')~string => String abc
.keep~new('abc', 'x') => 93.923
.mutablebuffer~new('abc', 5, 'x') => 93.902
abc 9
b~init(1) => 93.902
```

r[class.mutablebuffer.string]
`string` answers a `String` with the characters the buffer holds at that
moment, and takes no argument: one raises `SYNTAX 93.902`. A string taken
earlier does not follow later changes to the buffer. The string value of the
buffer, which `say`, the joining operators and the arguments of the other
methods use, is found as [the string-value rule](r:class.index.string) says,
and is the same text, since a buffer answers `makeString` with its characters.

```rexx
b = .mutablebuffer~new('abc')
say b b~string b~makeString b~length
s = b~string
b~append('def')
say s b~string
say b || '!' '<' || b || '>' length(b) left(b, 2)
say b~string~class~id b~makeString~class~id
say ('abcdef' = b) (b = 'abcdef') (b = b) (b == b~copy) (b \== b~copy)
call try 'b~string(1)'
call try 'b~makeString(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
abc abc abc 3
abc abcdef
abcdef! <abcdef> 6 ab
String String
1 0 1 0 1
b~string(1) => 93.902
b~makeString(1) => 93.902
```

r[class.mutablebuffer.makestring]
`makeString` answers a `String` with the characters of the buffer, as
`string` does, and takes no argument (`SYNTAX 93.902`). The `request` of
`Object` answers it for `STRING`, so a subclass that declares its own
`makeString` decides what `say b`, `b || x` and a buffer given as an argument
read, and an override of `string` changes only what `string` answers.
`request('ARRAY')` answers an array, that of
[`makeArray`](r:class.mutablebuffer.makearray). Whether two calls answer one
string object or two is not specified ([Deviation
4](r:class.mutablebuffer.deviation.4)).

```rexx
b = .shown~new('abc')
say b '|' b~string '|' b~makeString '|' 'xMADEx'~pos(b)
say b || '!' length(b) b~length
a = .mutablebuffer~new('abc')
say a~request('STRING') a~request('ARRAY')~class~id
::class shown subclass mutablebuffer
::method makestring
  return 'MADE'
::method string
  return 'STR'
```

```output
MADE | STR | MADE | 2
MADE! 4 3
abc Array
```

r[class.mutablebuffer.comparison]
A buffer compares as an object: `=`, `==`, `\=`, `\==`, `<>` and `><` are the
methods of `Object` ([the equality rule](r:class.object.eq)), which find two
buffers equal only when they are the same buffer, however alike their
characters are, and a buffer is never equal to a string when the buffer is the
receiver. The other comparison methods, `<`, `>`, `<=`, `>=`, `<<` and `>>`
with their negations, are not defined, and raise `SYNTAX 97.1`. A string that
is the receiver reads the string value of its argument, so `'abc' = b` and
`'abc' == b` are true when the buffer holds `abc` ([the string
rules](r:class.string.operators)).

```rexx
a = .mutablebuffer~new('abc')
b = .mutablebuffer~new('abc')
say (a = a) (a == a) (a = b) (a == b) (a \= b) (a <> b) (a \== b) (a >< b)
say ('abc' = a) ('abc' == a) (a = 'abc') (a == 'abc')
say ('abd' > a) ('abd' << a) ('abd' >> a)
call try 'rr = a < b'
call try "rr = a < 'abd'"
call try 'rr = a > b'
call try 'rr = a << b'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 1 0 0 1 1 1 1
1 1 0 0
1 0 1
rr = a < b => 97.1
rr = a < 'abd' => 97.1
rr = a > b => 97.1
rr = a << b => 97.1
```

r[class.mutablebuffer.copy]
`copy` answers a new buffer of the same class, holding the same characters and
having the same size, with the object variables of the receiver copied as [the
copy rule](r:class.object.copy) says. The copy is independent: a change to
either buffer does not reach the other, and the size of each grows by itself.
`copy` sends no `init` and no other message, takes no argument
(`SYNTAX 93.902`), and answers an instance of the class of the receiver, a
subclass included.

```rexx
b = .tagged~new('abc', 30)
b~tag = 'T1'
c = b~copy
say c~class~id c~string c~tag c~length c~getBufferSize
c~append('d')
c~tag = 'T2'
say b~string b~tag '|' c~string c~tag
say (b == c) (c~copy~class~id)
call try 'b~copy(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class tagged subclass mutablebuffer
::attribute tag
::method init
  say 'init'
  self~tag = 'none'
```

```output
init
TAGGED abc T1 3 30
abc T1 | abcd T2
0 TAGGED
b~copy(1) => 93.902
```

## Size

r[class.mutablebuffer.size]
A buffer has a length, the number of characters it holds, and a size, the number
of characters it can hold; the size is never smaller than the length. Every
change that makes the buffer longer than its size raises the size to the larger
of the new length and twice the old size; `append` makes one such change for
each argument, in turn. No other method changes the size but
[`setBufferSize`](r:class.mutablebuffer.setbuffersize), so deleting characters
leaves it as it was. `copy` gives the copy the size of the original.

```rexx
b = .mutablebuffer~new('abc', 5)
say b~length b~getBufferSize
b~append('de')
say b~length b~getBufferSize
b~append('f')
say b~length b~getBufferSize
b~append(copies('x', 20))
say b~length b~getBufferSize
b~delete(1, 20)
say b~length b~getBufferSize
b = .mutablebuffer~new
b~setText(copies('y', 300))
say b~length b~getBufferSize
b = .mutablebuffer~new('abcdefgh', 12)
b~setBufferSize(5)
say b~string b~length b~getBufferSize
b~setBufferSize(30)
say b~string b~length b~getBufferSize
b~setBufferSize(0)
say '<' || b~string || '>' b~length b~getBufferSize
b = .mutablebuffer~new('abc', 10)
c = b~copy
c~append(copies('z', 11))
say b~getBufferSize c~getBufferSize
b = .mutablebuffer~new('a', 1)
b~append('bc', 'de')
say b~length b~getBufferSize
c = .mutablebuffer~new('a', 1)
c~append('bcde')
say c~length c~getBufferSize
```

```output
3 5
5 5
6 10
26 26
6 26
300 512
abcde 5 5
abcde 5 30
<> 0 12
10 20
5 6
5 5
```

r[class.mutablebuffer.getbuffersize]
`getBufferSize` answers the size of the buffer as a whole number, and takes no
argument (`SYNTAX 93.902`). The size of a new buffer is given by
[`new`](r:class.mutablebuffer.new), and later changed as [the size
rule](r:class.mutablebuffer.size) says.

```rexx
say .mutablebuffer~new~getBufferSize,
  .mutablebuffer~new('abc', 7)~getBufferSize,
  .mutablebuffer~new('abc', 2)~getBufferSize
b = .mutablebuffer~new('abc', 7)
say b~getBufferSize~class~id
call try 'b~getBufferSize(1)'
exit
try:
  signal on syntax name bad
  interpret 'rr =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
256 7 3
String
b~getBufferSize(1) => 93.902
```

r[class.mutablebuffer.setbuffersize]
`setBufferSize` sets the size to its argument, a non-negative whole number
that is required: none raises `SYNTAX 93.903`, and a value that is not one
raises `SYNTAX 93.923`, a second argument `SYNTAX 93.902`. The argument is
read for its string value ([the argument rule](r:class.index.string)). A size
smaller than the length cuts the text to that many characters, and a larger
one keeps it. The size 0 empties the buffer and leaves the size as it was. The
method answers the buffer itself.

```rexx
b = .mutablebuffer~new('abcdef', 20)
r = b~setBufferSize(4)
say (r~identityHash == b~identityHash) b~string b~length b~getBufferSize
b~setBufferSize(9)
say b~string b~length b~getBufferSize
b~setBufferSize(1)
say b~string b~length b~getBufferSize
call try 'b~setBufferSize'
call try "b~setBufferSize('x')"
call try 'b~setBufferSize(-1)'
call try 'b~setBufferSize(1.5)'
call try 'b~setBufferSize(1, 2)'
call try 'b~getBufferSize(1)'
call try 'b~length(1)'
say b~string b~length b~getBufferSize
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 abcd 4 4
abcd 4 9
a 1 1
b~setBufferSize => 93.903
b~setBufferSize('x') => 93.923
b~setBufferSize(-1) => 93.923
b~setBufferSize(1.5) => 93.923
b~setBufferSize(1, 2) => 93.902
b~getBufferSize(1) => 93.902
b~length(1) => 93.902
a 1 1
```

r[class.mutablebuffer.length]
`length` answers the number of characters of the buffer, and takes no argument
(`SYNTAX 93.902`); [the builtin rule](r:bif.length.method) states what it
counts.

```rexx
b = .mutablebuffer~new('abc de')
say b~length b~length~class~id
b~append('f ')
say b~length
b~delete(1, 2)
say b~length
call try 'b~length(1)'
exit
try:
  signal on syntax name bad
  interpret 'rr =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
6 String
8
6
b~length(1) => 93.902
```

## Changing the text

r[class.mutablebuffer.append]
`append` appends the string value of each of its arguments, from left to
right, and answers the buffer itself. At least one argument is required
(`SYNTAX 93.903`), and an argument that is omitted before the last one present
raises `SYNTAX 93.903` too; trailing omitted arguments are ignored. An
argument is read just before it is appended, so the buffer itself as the
second argument is appended as it then is, with the first argument already in
it, and an argument that is omitted, or has no string value (`SYNTAX 88.909`,
`.nil` included), ends the call after the arguments before it were appended
and before those after it are read. The size grows as [the size
rule](r:class.mutablebuffer.size) says.

```rexx
b = .mutablebuffer~new('ab')
r = b~append('c', 'd', 'e')
say b~string (r~identityHash == b~identityHash)
b~append(12, 3.50, 1e3)
say b~string
b~append('')
b~append('x', )
say b~string b~length
b~append(b)
say b~string
-- the second b is read after the first was appended: 30 + 30 + 60
b~append(b, b)
say b~length
call try 'b~append'
call try "b~append(, 'd')"
call try "b~append('c', , 'e')"
call try 'b~append(.nil)'
say b~length
call try "b~append('!', .nil)"
say b~length
call try 'b~append(.object~new)'
say b~length
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
abcde 1
abcde123.501E3
abcde123.501E3x 15
abcde123.501E3xabcde123.501E3x
120
b~append => 93.903
b~append(, 'd') => 93.903
b~append('c', , 'e') => 93.903
b~append(.nil) => 88.909
121
b~append('!', .nil) => 88.909
122
b~append(.object~new) => 88.909
122
```

r[class.mutablebuffer.settext]
`setText` replaces the characters of the buffer with the string value of its
argument, which is required (`SYNTAX 93.903`; two arguments raise
`SYNTAX 93.902`), and answers the buffer itself. The buffer itself as the
argument leaves it unchanged. An argument without a string value raises
`SYNTAX 88.909` and leaves the buffer as it was. The size is not reduced, and
grows as [the size rule](r:class.mutablebuffer.size) says.

```rexx
b = .mutablebuffer~new('hello', 50)
r = b~setText('xyz')
say b~string b~length b~getBufferSize (r~identityHash == b~identityHash)
b~setText('')
say '<' || b~string || '>' b~length
b~setText(copies('q', 60))
say b~length b~getBufferSize
b~setText('abc')
b~setText(b)
say b~string
call try 'b~setText'
call try "b~setText('x', 'y')"
call try 'b~setText(.nil)'
say b~string
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
xyz 3 50 1
<> 0
60 100
abc
b~setText => 93.903
b~setText('x', 'y') => 93.902
b~setText(.nil) => 88.909
abc
```

r[class.mutablebuffer.brackets]
`[]` answers a `String` with a part of the buffer: the characters from the
position given as the first argument, a positive whole number that is
required, for as many characters as the second argument, a non-negative whole
number that defaults to 1. It is the method [`[]` of
`String`](r:class.string.brackets) applied to the characters of the buffer,
with its answers and its errors, and leaves the buffer as it was. The brackets
of a message term `b[3, 4]` and the method named `'[]'` are the same.

```rexx
b = .mutablebuffer~new('abcdefghij')
say b[3] b[3, 4] '<' || b[11] || '>' b[8, 10] '<' || b[2, 0] || '>'
say b[1.0] b[' 2 ', ' 3 '] b[3]~class~id
call try 'b[0]'
call try 'b[-1]'
call try "b['x']"
call try 'b[2, -1]'
call try "b[2, 'x']"
call try "b~'[]'"
call try 'b[, 2]'
call try "b~'[]'(2, 1, 1)"
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
c cdef <> hij <>
a bcd String
b[0] => 93.924
b[-1] => 93.924
b['x'] => 93.924
b[2, -1] => 93.923
b[2, 'x'] => 93.923
b~'[]' => 93.903
b[, 2] => 93.903
b~'[]'(2, 1, 1) => 93.902
```

r[class.mutablebuffer.brackets-set]
`[]=` is the method that an assignment `b[n, length] = text` sends, with the
text first: `'[]='(text, n, length)`. It replaces `length` characters from the
position `n` by the string value of `text`, and answers the buffer itself. `n`
is a positive whole number, and `length` a non-negative whole number that
defaults to the length of the text, so that the replacement is as long as the
text; a `length` of 0 inserts the text before the position. A replacement that
reaches past the end of the buffer cuts the buffer there. A position past the
end of the buffer first fills the gap with blanks. Too many arguments raise
`SYNTAX 93.902`. The other checks are made in the order of the arguments, each
argument being read just before its check: the text, which is required
(`SYNTAX 88.901`) and must have a string value (`88.909`); the position, which
is required (`88.901`) and must be a positive whole number (`88.912`); and the
length, which must be a non-negative whole number (`88.911`). A failing call
leaves the buffer as it was.

```rexx
t = 'abcdefghij'
call put "b[3] = 'XY'"
call put "b[3, 1] = 'XY'"
call put "b[3, 4] = 'X'"
call put "b[3, 0] = 'X'"
call put "b[3, 99] = 'X'"
call put "b[3] = ''"
call put "b[11] = 'X'"
call put "b[14] = 'X'"
call put "b[2, 3] = 5"
call put "b[0] = 'X'"
call put "b[-2] = 'X'"
call put "b['x'] = 'X'"
call put "b[2, -1] = 'X'"
call put "b[2, 'x'] = 'X'"
call put "b[, 1] = 'X'"
call put "b[2] = .nil"
call put "b~'[]='('X')"
call put "b~'[]='"
call put "b~'[]='('X', 2, 1, 1)"
call put "b~'[]='('Q', 2)"
b = .mutablebuffer~new(t)
r = b~'[]='('Q', 2)
say r~class~id (r~identityHash == b~identityHash) b~string
exit
put:
  signal on syntax name bad
  b = .mutablebuffer~new(t)
  interpret arg(1)
  say arg(1) '=> <' || b~string || '>'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
b[3] = 'XY' => <abXYefghij>
b[3, 1] = 'XY' => <abXYdefghij>
b[3, 4] = 'X' => <abXghij>
b[3, 0] = 'X' => <abXcdefghij>
b[3, 99] = 'X' => <abX>
b[3] = '' => <abcdefghij>
b[11] = 'X' => <abcdefghijX>
b[14] = 'X' => <abcdefghij   X>
b[2, 3] = 5 => <a5efghij>
b[0] = 'X' => 88.912
b[-2] = 'X' => 88.912
b['x'] = 'X' => 88.912
b[2, -1] = 'X' => 88.911
b[2, 'x'] = 'X' => 88.911
b[, 1] = 'X' => 88.901
b[2] = .nil => 88.909
b~'[]='('X') => 88.901
b~'[]=' => 88.901
b~'[]='('X', 2, 1, 1) => 93.902
b~'[]='('Q', 2) => <aQcdefghij>
MutableBuffer 1 aQcdefghij
```

r[class.mutablebuffer.replaceat]
`replaceAt` replaces the characters of the buffer from a position, and answers
the buffer itself. Its arguments are the new text, the position, the length
and a pad character, and it is the same method as `[]=` with the arguments in
the order of [the `replaceAt` of `String`](r:class.string.replaceat) and with
a pad: the characters from the position, as many as the length, are replaced
by the text, the length defaults to the length of the text, a position past
the end of the buffer first fills the gap with the pad, which defaults to a
blank, and a length of 0 inserts. Too many arguments raise `SYNTAX 93.902`.
The other checks are made as for `[]=`, in the order of the arguments, with
the pad last: the text is required (`SYNTAX 88.901`) and must have a string
value (`88.909`), the position is required (`88.901`) and must be a positive
whole number (`88.912`), the length must be a non-negative whole number
(`88.911`) and the pad a single character (`88.910`), the null string
excluded. A failing call leaves the buffer as it was.

```rexx
t = 'abcdefghij'
call put "b~replaceAt('XY', 3)"
call put "b~replaceAt('XY', 3, 1)"
call put "b~replaceAt('XY', 3, 0)"
call put "b~replaceAt('XY', 3, 99)"
call put "b~replaceAt('XY', 3.0, 1.0)"
call put "b~replaceAt('XY', 11)"
call put "b~replaceAt('XY', 13)"
call put "b~replaceAt('XY', 14, 2, '-')"
call put "b~replaceAt('X', 2, , '*')"
call put "b~replaceAt('', 2, 3)"
call put "b~replaceAt"
call put "b~replaceAt('XY')"
call put "b~replaceAt(, 2)"
call put "b~replaceAt('XY', 0)"
call put "b~replaceAt('XY', 2.5)"
call put "b~replaceAt('XY', 'x')"
call put "b~replaceAt('XY', 2, -1)"
call put "b~replaceAt('XY', 2, 'x')"
call put "b~replaceAt('XY', 2, 1, '**')"
call put "b~replaceAt('XY', 2, 1, '')"
call put "b~replaceAt('XY', 0, -1, '**')"
call put "b~replaceAt('XY', 2, 1, '*', 5)"
call put "b~replaceAt(.nil, 0)"
exit
put:
  signal on syntax name bad
  b = .mutablebuffer~new(t)
  interpret 'r =' arg(1)
  say arg(1) '=> <' || b~string || '>' (r~identityHash == b~identityHash)
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
b~replaceAt('XY', 3) => <abXYefghij> 1
b~replaceAt('XY', 3, 1) => <abXYdefghij> 1
b~replaceAt('XY', 3, 0) => <abXYcdefghij> 1
b~replaceAt('XY', 3, 99) => <abXY> 1
b~replaceAt('XY', 3.0, 1.0) => <abXYdefghij> 1
b~replaceAt('XY', 11) => <abcdefghijXY> 1
b~replaceAt('XY', 13) => <abcdefghij  XY> 1
b~replaceAt('XY', 14, 2, '-') => <abcdefghij---XY> 1
b~replaceAt('X', 2, , '*') => <aXcdefghij> 1
b~replaceAt('', 2, 3) => <aefghij> 1
b~replaceAt => 88.901
b~replaceAt('XY') => 88.901
b~replaceAt(, 2) => 88.901
b~replaceAt('XY', 0) => 88.912
b~replaceAt('XY', 2.5) => 88.912
b~replaceAt('XY', 'x') => 88.912
b~replaceAt('XY', 2, -1) => 88.911
b~replaceAt('XY', 2, 'x') => 88.911
b~replaceAt('XY', 2, 1, '**') => 88.910
b~replaceAt('XY', 2, 1, '') => 88.910
b~replaceAt('XY', 0, -1, '**') => 88.912
b~replaceAt('XY', 2, 1, '*', 5) => 93.902
b~replaceAt(.nil, 0) => 88.909
```

r[class.mutablebuffer.delete]
`delete` is `delStr` under another name: it takes the same arguments, gives
the same answers and errors, deletes the same characters from the buffer and
answers the buffer itself ([`delStr`](r:bif.delstr.method) is stated with the
other changing methods).

```rexx
t = 'abcdefghij'
do a over .array~of('', '(1)', '(3)', '(3, 2)', '(3, 0)', '(20)', '(2, 20)',,
    '(0)', '(2, -1)', '(2, 1, 1)', "('x')")
  call two a
end
exit
two:
  b1 = .mutablebuffer~new(t)
  b2 = .mutablebuffer~new(t)
  r1 = ev('b1~delete' || arg(1))
  r2 = ev('b2~delStr' || arg(1))
  say left('delete' || arg(1), 18) r1 (r1 == r2)
  return
ev: procedure expose b1 b2
  signal on syntax name bad
  interpret 'r =' arg(1)
  if r~identityHash == b1~identityHash | r~identityHash == b2~identityHash,
    then return '<' || r~string || '> self'
  return '<' || r~string || '>'
bad: return 'E' || condition('o')~code
```

```output
delete             <> self 1
delete(1)          <> self 1
delete(3)          <ab> self 1
delete(3, 2)       <abefghij> self 1
delete(3, 0)       <abcdefghij> self 1
delete(20)         <abcdefghij> self 1
delete(2, 20)      <a> self 1
delete(0)          E93.924 1
delete(2, -1)      E93.923 1
delete(2, 1, 1)    E93.902 1
delete('x')        E93.924 1
```

r[class.mutablebuffer.changing]
`changeStr`, `caselessChangeStr`, `delStr`, `delWord`, `insert`, `lower`,
`overlay`, `space`, `translate` and `upper` do what the function or the method
of that name of `String` does ([`changeStr`](r:bif.changestr.method),
[`caselessChangeStr`](r:class.string.caselesschangestr),
[`delStr`](r:bif.delstr.method), [`delWord`](r:bif.delword.method),
[`insert`](r:bif.insert.method), [`lower`](r:bif.lower.method),
[`overlay`](r:bif.overlay.method), [`space`](r:bif.space.method),
[`translate`](r:bif.translate.method), [`upper`](r:bif.upper.method)), with
the same arguments, defaults, checks and errors, in the same order. The
difference is the answer: the buffer takes the characters that the string
method would answer for its characters as its new text, and the method answers
the buffer itself, not a new string. A call that raises an error leaves the
buffer as it was. The size grows as [the size
rule](r:class.mutablebuffer.size) says. A `tablei` of `translate` that is the
null string is not covered by this rule.

```rexx
t = 'Alpha beta  GAMMA delta'
call chg "changeStr('a', 'AA')"
call chg "caselessChangeStr('a', '-', 2)"
call chg "delStr(7, 5)"
call chg "delWord(2, 1)"
call chg "insert('<>', 5, 4, '.')"
call chg "lower(7)"
call chg "upper(1, 3)"
call chg "overlay('XYZ', 3, 5, '*')"
call chg "space(2)"
call chg "translate('ab', 'AL')"
call chg "insert('x', -1)"
call chg "delWord(0)"
call chg "overlay('x', 0)"
call chg "space(1, '**')"
call chg "changeStr('a')"
call chg "upper(1, 2, 3)"
call chg "lower('x')"
exit
chg:
  b = .mutablebuffer~new(t)
  signal on syntax name bad
  interpret 'r = b~' || arg(1)
  interpret 's = t~' || arg(1)
  say arg(1) '=> <' || b~string || '>' (b~string == s),
    (r~identityHash == b~identityHash)
  return
bad: say arg(1) '=>' condition('o')~code '<' || b~string || '>',
    (b~string == t)
  return
```

```output
changeStr('a', 'AA') => <AlphAA betAA  GAMMA deltAA> 1 1
caselessChangeStr('a', '-', 2) => <-lph- beta  GAMMA delta> 1 1
delStr(7, 5) => <Alpha  GAMMA delta> 1 1
delWord(2, 1) => <Alpha GAMMA delta> 1 1
insert('<>', 5, 4, '.') => <Alpha<>.. beta  GAMMA delta> 1 1
lower(7) => <Alpha beta  gamma delta> 1 1
upper(1, 3) => <ALPha beta  GAMMA delta> 1 1
overlay('XYZ', 3, 5, '*') => <AlXYZ**eta  GAMMA delta> 1 1
space(2) => <Alpha  beta  GAMMA  delta> 1 1
translate('ab', 'AL') => <alpha beta  GaMMa delta> 1 1
insert('x', -1) => 93.906 <Alpha beta  GAMMA delta> 1
delWord(0) => 93.924 <Alpha beta  GAMMA delta> 1
overlay('x', 0) => 93.924 <Alpha beta  GAMMA delta> 1
space(1, '**') => 93.922 <Alpha beta  GAMMA delta> 1
changeStr('a') => 93.903 <Alpha beta  GAMMA delta> 1
upper(1, 2, 3) => 93.902 <Alpha beta  GAMMA delta> 1
lower('x') => 93.924 <Alpha beta  GAMMA delta> 1
```

## Reading the text

r[class.mutablebuffer.builtins]
`countStr`, `lastPos`, `pos`, `subWord`, `substr`, `verify`, `word`,
`wordIndex`, `wordLength`, `wordPos` and `words` answer what the function of
that name answers for the characters of the buffer, with the same arguments,
defaults, checks and errors, as [the builtin rule](r:class.string.builtins)
lists them: [`countStr`](r:bif.countstr.method),
[`lastPos`](r:bif.lastpos.method), [`pos`](r:bif.pos.method),
[`subWord`](r:bif.subword.method), [`substr`](r:bif.substr.method),
[`verify`](r:bif.verify.method), [`word`](r:bif.word.method),
[`wordIndex`](r:bif.wordindex.method),
[`wordLength`](r:bif.wordlength.method), [`wordPos`](r:bif.wordpos.method) and
[`words`](r:bif.words.method). They leave the buffer as it was, and, when the
function answers a string, answer a `String`. The errors are those of the
message form of the string methods, as [the message
rule](r:class.string.builtins.errors) says.

```rexx
t = 'Alpha beta  GAMMA delta'
b = .mutablebuffer~new(t)
call cmp "pos('ta', 3)"
call cmp "lastPos('a')"
call cmp "countStr('a')"
call cmp "substr(8, 4)"
call cmp "subWord(3, 1)"
call cmp "word(4)"
call cmp "wordIndex(3)"
call cmp "wordLength(2)"
call cmp "wordPos('GAMMA delta')"
call cmp "words"
call cmp "verify('abAL', 'N')"
call cmp "substr(0)"
call cmp "pos"
call cmp "word(2, 3)"
call cmp "wordPos('x', 'y')"
call cmp "verify('a', 'Q')"
say b~string
exit
cmp:
  r1 = ev('b~' || arg(1))
  r2 = ev('t~' || arg(1))
  say arg(1) '=>' r1 (r1 == r2)
  return
ev: procedure expose b t
  signal on syntax name bad
  interpret 'r =' arg(1)
  if r~isA(.array) then return r~makeString('l', '|')
  return r
bad: return 'E' || condition('o')~code
```

```output
pos('ta', 3) => 9 1
lastPos('a') => 23 1
countStr('a') => 3 1
substr(8, 4) => eta  1
subWord(3, 1) => GAMMA 1
word(4) => delta 1
wordIndex(3) => 13 1
wordLength(2) => 4 1
wordPos('GAMMA delta') => 3 1
words => 4 1
verify('abAL', 'N') => 2 1
substr(0) => E93.924 1
pos => E93.903 1
word(2, 3) => E93.902 1
wordPos('x', 'y') => E93.924 1
verify('a', 'Q') => E93.915 1
Alpha beta  GAMMA delta
```

r[class.mutablebuffer.stringmethods]
`caselessChangeStr` is a changing method, stated above; the others that
`String` has beside the builtin functions read the buffer and answer as the
`String` method does, with the same arguments, defaults, checks and errors:
[`caselessContains`](r:class.string.caselesscontains),
[`caselessContainsWord`](r:class.string.caselesscontainsword),
[`caselessCountStr`](r:class.string.caselesscountstr),
[`caselessEndsWith`](r:class.string.caselessendswith),
[`caselessLastPos`](r:class.string.caselesslastpos),
[`caselessMatch`](r:class.string.caselessmatch),
[`caselessMatchChar`](r:class.string.caselessmatchchar),
[`caselessPos`](r:class.string.caselesspos),
[`caselessStartsWith`](r:class.string.caselessstartswith),
[`caselessWordPos`](r:class.string.caselesswordpos),
[`contains`](r:class.string.contains),
[`containsWord`](r:class.string.containsword),
[`endsWith`](r:class.string.endswith), [`match`](r:class.string.match),
[`matchChar`](r:class.string.matchchar),
[`startsWith`](r:class.string.startswith), [`subChar`](r:class.string.subchar)
and [`subWords`](r:class.string.subwords). They leave the buffer as it was,
and answer a `String`, or a new `Array` for `subWords`.

```rexx
t = 'Alpha beta  GAMMA delta'
b = .mutablebuffer~new(t)
call cmp "caselessPos('GA')"
call cmp "caselessLastPos('A')"
call cmp "caselessContains('HA B')"
call cmp "caselessContainsWord('BETA')"
call cmp "caselessStartsWith('alp')"
call cmp "caselessEndsWith('TA')"
call cmp "caselessMatch(7, 'BE')"
call cmp "caselessMatchChar(7, 'BX')"
call cmp "caselessWordPos('Gamma')"
call cmp "caselessCountStr('A')"
call cmp "contains('ha b')"
call cmp "contains('HA B')"
call cmp "containsWord('beta')"
call cmp "startsWith('Alp')"
call cmp "endsWith('ta')"
call cmp "match(7, 'be')"
call cmp "matchChar(7, 'bx')"
call cmp "subChar(3)"
call cmp "subWords(2, 2)"
call cmp "contains"
call cmp "match(1)"
call cmp "subChar(0)"
call cmp "startsWith('a', 'b')"
say b~string
exit
cmp:
  r1 = ev('b~' || arg(1))
  r2 = ev('t~' || arg(1))
  say arg(1) '=>' r1 (r1 == r2)
  return
ev: procedure expose b t
  signal on syntax name bad
  interpret 'r =' arg(1)
  if r~isA(.array) then return r~makeString('l', '|')
  return r
bad: return 'E' || condition('o')~code
```

```output
caselessPos('GA') => 13 1
caselessLastPos('A') => 23 1
caselessContains('HA B') => 1 1
caselessContainsWord('BETA') => 1 1
caselessStartsWith('alp') => 1 1
caselessEndsWith('TA') => 1 1
caselessMatch(7, 'BE') => 1 1
caselessMatchChar(7, 'BX') => 1 1
caselessWordPos('Gamma') => 3 1
caselessCountStr('A') => 6 1
contains('ha b') => 1 1
contains('HA B') => 0 1
containsWord('beta') => 1 1
startsWith('Alp') => 1 1
endsWith('ta') => 1 1
match(7, 'be') => 1 1
matchChar(7, 'bx') => 1 1
subChar(3) => p 1
subWords(2, 2) => beta|GAMMA 1
contains => E93.903 1
match(1) => E93.903 1
subChar(0) => E93.924 1
startsWith('a', 'b') => E93.902 1
Alpha beta  GAMMA delta
```

r[class.mutablebuffer.makearray]
`makeArray` answers a new array of strings, the lines of the buffer or, with a
separator, the parts between its occurrences, exactly as [the `makeArray` of
`String`](r:class.string.makearray) does for a string with the same
characters: the same optional separator, the same cuts and the same errors.
The array does not change when the buffer does, and `DO OVER` a buffer visits
the elements of this array ([the DO OVER rule](r:instr.do.over)).

```rexx
b = .mutablebuffer~new('one' || '0A'x || 'two' || '0D0A'x || 'three' || '0D'x)
a = b~makeArray
say a~class~id a~items a~at(1) a~at(2) a~at(3)~c2x
c = .mutablebuffer~new('abXYcdXYef')~makeArray('XY')
say c~items c~at(1) c~at(2) c~at(3)
say .mutablebuffer~new('a b')~makeArray(' ')~items,
  .mutablebuffer~new('')~makeArray~items
b~append('!' || '0A'x || 'four')
say a~items b~makeArray~items
do line over .mutablebuffer~new('x' || '0A'x || 'y')
  say 'line' line
end
call try "b~makeArray(',', 1)"
call try 'b~makeArray(.nil)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
Array 3 one two 74687265650D
3 ab cd ef
2 0
3 4
line x
line y
b~makeArray(',', 1) => 93.902
b~makeArray(.nil) => 88.909
```

## Messages that a program can override

r[class.mutablebuffer.sends]
No method of the class sends a message to the buffer it works on: not `length`,
`string`, `makeString`, `append`, `insert`, `setText`, `substr`, `delStr`,
`getBufferSize`, `setBufferSize` or any other, so that a subclass that overrides
one of these changes only the calls that name it. `new` sends `init` as [the
init rule](r:class.mutablebuffer.new.init) says and `copy` sends nothing. What a
method sends to its arguments is the reading of their string values, once for
each argument that is present and in the order of the arguments, each reading
followed at once by the check of that argument: the first argument that fails
raises its error and the arguments after it are not read. The reading is [the
string-value rule](r:class.index.string)'s: the argument is sent `request` with
`STRING`, and the `request` of `Object` sends it `makeString`, so an argument
that is a buffer of a subclass is sent both. `append` reads and appends one
argument at a time. A call with too many arguments reads none. Nothing but that
reading is sent to an argument. The first example prints, for each call, the
messages that the subclass saw; the second passes objects that answer a string
and log the reading, named by their first argument, and would log `isA`,
`string` and `class` too; their `request` is that of `Object`. It prints the log
in order, with the error that ended the call. The third example shows the
`request` that comes before each `makeString`.

```rexx
b = .spy~new('Hello world', 40)
call run "b~append('x', 'y')"
call run "b~setText('Hello world')"
call run "b~insert('Q', 1)"
call run "b~overlay('Q', 2)"
call run "b~replaceAt('Q', 2)"
call run "b~'[]='('Q', 2)"
call run "b~delete(1, 1)"
call run "b~delStr(1, 1)"
call run "b~upper"
call run "b~changeStr('l', 'L')"
call run "b~setBufferSize(100)"
call run "b~substr(2, 3)"
call run "b~pos('o')"
call run "b~word(1)"
call run "b~words"
call run "b~contains('o')"
call run "b~makeArray"
call run "b~copy"
call run "b[2]"
call run "b~length"
call run "b~string"
call run "b~makeString"
call run "b || '!'"
call run ".spy~new('abc')~append(b)"
exit
run:
  .local~log = ''
  interpret 'r =' arg(1)
  say left(arg(1), 28) '=>' .local~log
  return
::class spy subclass mutablebuffer
::method new class
  .local~log = .local~log 'NEW'arg()
  forward class (super)
::method init
  .local~log = .local~log 'INIT'arg()
  forward class (super)
::method length
  .local~log = .local~log 'length'
  forward class (super)
::method string
  .local~log = .local~log 'string'
  forward class (super)
::method makestring
  .local~log = .local~log 'makestring'
  forward class (super)
::method makearray
  .local~log = .local~log 'makearray'
  forward class (super)
::method append
  .local~log = .local~log 'append'
  forward class (super)
::method insert
  .local~log = .local~log 'insert'
  forward class (super)
::method settext
  .local~log = .local~log 'setText'
  forward class (super)
::method getbuffersize
  .local~log = .local~log 'getBufferSize'
  forward class (super)
::method setbuffersize
  .local~log = .local~log 'setBufferSize'
  forward class (super)
::method '[]'
  .local~log = .local~log 'at'
  forward class (super)
::method '[]='
  .local~log = .local~log 'put'
  forward class (super)
::method replaceat
  .local~log = .local~log 'replaceAt'
  forward class (super)
::method delstr
  .local~log = .local~log 'delStr'
  forward class (super)
::method delete
  .local~log = .local~log 'delete'
  forward class (super)
::method substr
  .local~log = .local~log 'substr'
  forward class (super)
::method pos
  .local~log = .local~log 'pos'
  forward class (super)
::method copy
  .local~log = .local~log 'copy'
  forward class (super)
::method overlay
  .local~log = .local~log 'overlay'
  forward class (super)
::method upper
  .local~log = .local~log 'upper'
  forward class (super)
::method changestr
  .local~log = .local~log 'changeStr'
  forward class (super)
::method word
  .local~log = .local~log 'word'
  forward class (super)
::method words
  .local~log = .local~log 'words'
  forward class (super)
::method contains
  .local~log = .local~log 'contains'
  forward class (super)
```

```output
b~append('x', 'y')           =>  append
b~setText('Hello world')     =>  setText
b~insert('Q', 1)             =>  insert
b~overlay('Q', 2)            =>  overlay
b~replaceAt('Q', 2)          =>  replaceAt
b~'[]='('Q', 2)              =>  put
b~delete(1, 1)               =>  delete
b~delStr(1, 1)               =>  delStr
b~upper                      =>  upper
b~changeStr('l', 'L')        =>  changeStr
b~setBufferSize(100)         =>  setBufferSize
b~substr(2, 3)               =>  substr
b~pos('o')                   =>  pos
b~word(1)                    =>  word
b~words                      =>  words
b~contains('o')              =>  contains
b~makeArray                  =>  makearray
b~copy                       =>  copy
b[2]                         =>  at
b~length                     =>  length
b~string                     =>  string
b~makeString                 =>  makestring
b || '!'                     =>  makestring
.spy~new('abc')~append(b)    =>  NEW1 INIT0 append makestring
```

```rexx
call run 'replaceAt all', "bb~replaceAt(a('Q'), b(2), c(1), d('*'))"
call run 'replaceAt pos 0', "bb~replaceAt(a('Q'), b(0), c(1), d('*'))"
call run 'replaceAt len -1', "bb~replaceAt(a('Q'), b(2), c(-1), d('*'))"
call run 'replaceAt pad **', "bb~replaceAt(a('Q'), b(2), c(1), d('**'))"
call run 'replaceAt no pos', "bb~replaceAt(a('Q'))"
call run 'replaceAt count', "bb~replaceAt(a('Q'), 2, 1, '*', e(5))"
call run '[]= pos 0', "bb~'[]='(a('Q'), b(0), c(1))"
call run '[] len -1', "bb[b(2), c(-1)]"
call run 'insert pos -2', "bb~insert(a('Q'), b(-2), c(1), d('*'))"
call run 'insert all', "bb~insert(a('Q'), b(2), c(1), d('*'))"
call run 'append 3', "bb~append(a('x'), b('y'), c('z'))"
call run 'append nil', "bb~append(a('x'), .nil, c('z'))"
call run 'changeStr count -1', "bb~changeStr(a('b'), b('Q'), c(-1))"
call run 'pos start 0', "bb~pos(a('b'), b(0), c(5))"
call run 'setBufferSize', "bb~setBufferSize(a(9))"
call run 'makeArray', "bb~makeArray(a(' '))"
call run 'new size -1', ".mutablebuffer~new(a('z'), b(-1))"
call run 'new', ".mutablebuffer~new(a('z'), b(9))"
exit
run:
  .local~log = ''
  bb = .mutablebuffer~new('abc def')
  signal on syntax name bad
  interpret 'r =' arg(2)
  say left(arg(1), 20) '=>' .local~log
  return
bad: say left(arg(1), 20) '=>' .local~log 'SYNTAX' condition('o')~code
  return
a: return .probe~new('a', arg(1))
b: return .probe~new('b', arg(1))
c: return .probe~new('c', arg(1))
d: return .probe~new('d', arg(1))
e: return .probe~new('e', arg(1))
::class probe
::method init
  expose n v
  use arg n, v
::method makestring
  expose n v
  .local~log = .local~log n
  return v
::method string
  expose n v
  .local~log = .local~log n'-string'
  return v
::method isA
  .local~log = .local~log 'isA'
  return 0
::method class
  .local~log = .local~log 'class'
  return .object
```

```output
replaceAt all        =>  a b c d
replaceAt pos 0      =>  a b SYNTAX 88.912
replaceAt len -1     =>  a b c SYNTAX 88.911
replaceAt pad **     =>  a b c d SYNTAX 88.910
replaceAt no pos     =>  a SYNTAX 88.901
replaceAt count      =>  SYNTAX 93.902
[]= pos 0            =>  a b SYNTAX 88.912
[] len -1            =>  b c SYNTAX 93.923
insert pos -2        =>  a b SYNTAX 93.906
insert all           =>  a b c d
append 3             =>  a b c
append nil           =>  a SYNTAX 88.909
changeStr count -1   =>  a b c SYNTAX 93.906
pos start 0          =>  a b SYNTAX 93.924
setBufferSize        =>  a
makeArray            =>  a
new size -1          =>  a b SYNTAX 93.923
new                  =>  a b
```

```rexx
.local~log = ''
b = .mutablebuffer~new('abc')
b~insert(.probe~new('t', 'X'), .probe~new('n', 1))
say b~string '|' .local~log
::class probe
::method init
  expose n v
  use arg n, v
::method request
  expose n
  .local~log = .local~log n'-request:'arg(1)
  forward class (super)
::method makestring
  expose n v
  .local~log = .local~log n'-makeString'
  return v
```

```output defect=#164
aXbc |  t-request:STRING t-makeString n-request:STRING n-makeString
```

## Deviations

r[class.mutablebuffer.deviation.4]
Whether two calls of `string` or of `makeString` answer one string object or
two is not specified, and shows only through `identityHash` and the identity
collections ([Deviation 4](../deviations.md)). The characters are specified,
and a string taken from a buffer never follows the buffer's later changes.
ooRexx answers a new string at each call of `makeString`, where interrexx
answers the same one.

```rexx
b = .mutablebuffer~new('abc')
x = b~makeString
y = b~string
say (x == y) (x == 'abc') (x~hashCode == y~hashCode)
b~append('d')
say x b~makeString
```

```output
1 1 1
abc abcd
```

r[class.mutablebuffer.deviation.9]
A native routine that holds the address of the characters of a buffer keeps
reading valid storage when the buffer grows, even across a collection
([Deviation 9](../deviations.md)). In ooRexx a collection after the growth
reclaims the old storage, and the routine then reads garbage. A program in Rexx
sees only that the characters survive the growth, which the example shows: the
buffer grows several times and still holds what was appended before.

```rexx
b = .mutablebuffer~new('abc', 3)
do 5
  b~append(copies('x', 200))
end
say b~length b~substr(1, 6) b~substr(1001)
say b~string~length
```

```output
1003 abcxxx xxx
1003
```

r[class.mutablebuffer.deviation.109]
An instance of a subclass of `String` that is given to a method of the buffer
as a string, a number or a whole number, for the text of `new`, the size, a
position, a length or the pad of the methods above, is read by its own
characters, and `REQUEST`, `MAKESTRING` and `STRING` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an instance
`request` with `STRING` and reads the answer, so an override can change what
the method reads. In the example the subclass answers other strings from all
three methods and counts the sends, and the instance `n` is the number 2, `s`
the string `ab`, so the buffer is made from the characters `ab`, with size 2.

```rexx
.local~sent = 0
s = .shout~new('ab')
n = .shout~new('2')
b = .mutablebuffer~new(s, n)
say b~string b~length b~getBufferSize
b~append(s)
b~insert(s, n)
b[n] = s
say b~string b~length
say b[n] b~pos(s) b~substr(n, n)
say 'sends' .local~sent
::class shout subclass string
::method makestring
  .local~sent = .local~sent + 1
  return 'MADE'
::method string
  .local~sent = .local~sent + 1
  return 'STR'
::method request
  .local~sent = .local~sent + 1
  return 'REQ'
```

```output defect=#140
ab 2 2
aabbab 6
a 2 ab
sends 0
```
