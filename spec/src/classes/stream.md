---
element: "Stream"
kind: class
rexxref: "Builtin Classes > Stream Classes > Stream Class; Builtin Classes > Stream Classes > Stream Class > new (Inherited Class Method); Builtin Classes > Stream Classes > Stream Class > arrayIn; Builtin Classes > Stream Classes > Stream Class > arrayOut; Builtin Classes > Stream Classes > Stream Class > charIn; Builtin Classes > Stream Classes > Stream Class > charOut; Builtin Classes > Stream Classes > Stream Class > chars; Builtin Classes > Stream Classes > Stream Class > close; Builtin Classes > Stream Classes > Stream Class > command; Builtin Classes > Stream Classes > Stream Class > description; Builtin Classes > Stream Classes > Stream Class > flush; Builtin Classes > Stream Classes > Stream Class > init; Builtin Classes > Stream Classes > Stream Class > lineIn; Builtin Classes > Stream Classes > Stream Class > lineOut; Builtin Classes > Stream Classes > Stream Class > lines; Builtin Classes > Stream Classes > Stream Class > makeArray; Builtin Classes > Stream Classes > Stream Class > open; Builtin Classes > Stream Classes > Stream Class > position; Builtin Classes > Stream Classes > Stream Class > qualify; Builtin Classes > Stream Classes > Stream Class > query; Builtin Classes > Stream Classes > Stream Class > say; Builtin Classes > Stream Classes > Stream Class > seek; Builtin Classes > Stream Classes > Stream Class > state; Builtin Classes > Stream Classes > Stream Class > string; Builtin Classes > Stream Classes > Stream Class > supplier; Builtin Classes > Stream Classes > Stream Class > uninit"
deviations: [34, 55, 62, 109]
---

# Stream

A `Stream` is an object that stands for a stream: the file, or the standard
stream, that its name denotes. Its methods do what the stream built-in functions
do, with the stream as the receiver in place of a name argument, and it has some
that the functions have no counterpart for: the lines of a stream as an array or
as a supplier, and the writing of the items of an array. This page states what
is the method's own: how an object is made, the argument errors and the answers
of the methods, and the messages that a method sends to its receiver and to its
arguments, where a subclass or an argument can override them. The rules of the
functions are linked, never repeated. A name that is one of the standard names
denotes that standard stream, [as for the functions](r:bif.index.stream.std);
what the standard streams of a program are bound to when it starts, and the
objects `.input`, `.output` and `.error`, are specified in the environment
chapter. The examples create the files they use under names of their own, close
every stream they open, and remove the files at the end.

## The class

r[class.stream.creation]
`Stream` is a subclass of `InputOutputStream` and it inherits no other class. It
is a mixin class: [`queryMixinClass`](r:class.index.mixins) answers 1. It has
the class method `new`, which every class has, and the instance methods
`arrayIn`, `arrayOut`, `charIn`, `charOut`, `chars`, `close`, `command`,
`description`, `flush`, `init`, `lineIn`, `lineOut`, `lines`, `makeArray`,
`open`, `position`, `qualify`, `query`, `say`, `seek`, `state`, `string`,
`supplier` and `uninit`. It may have private methods besides these; they are not
part of this specification. It has no method `makeString`. It defines every
method that [`InputStream`](inputstream.md), [`OutputStream`](outputstream.md)
and [`InputOutputStream`](inputoutputstream.md) define, so a message to a
`Stream` never reaches the methods of those pages. An instance is not a string,
so a string method such as `left` raises `SYNTAX 97.1`. A copy of an instance is
not specified.

```rexx
say .stream~superclass~id .stream~superclasses~items .stream~queryMixinClass
names = 'arrayIn arrayOut charIn charOut chars close command description',
  'flush init lineIn lineOut lines makeArray open position qualify query',
  'say seek state string supplier uninit'
found = 0
do n over names~makearray(' ')
  found += .stream~method(n)~isA(.method)
end
say found 'of' names~words
f = 'spec-stream-creation-zq.tmp'
s = .stream~new(f)
say s~isA(.inputoutputstream) s~isA(.inputstream) s~isA(.outputstream)
signal on syntax name bad
say s~makeString
bad: say condition('o')~code
signal on syntax name bad2
say s~left(1)
bad2: say condition('o')~code
```

```output
InputOutputStream 1 1
24 of 24
1 1 1
97.1
97.1
```

r[class.stream.new]
`new` is the `new` that `Class` supplies ([the new rule](r:class.index.new)): it
makes an instance of the class that received it, a subclass included, and sends
it `init` with the arguments of `new`. The instance is not open, and nothing is
read or written until a method needs the stream; its state is `UNKNOWN`. `init`
takes the name of the stream as its one argument. A missing argument raises
`SYNTAX 93.901` and a second one `SYNTAX 93.902`. A name that is a string is
used as it is, with its blanks. The null string is a name too, though a stream
with that name cannot be opened (the functions refuse it with `SYNTAX 40.27`, as
[their rule](r:bif.stream.args) says). A name that is not a string is read as
the string value its `makeString` method answers, which is sent once, when the
stream is made, and never again, so a later change in what the object answers
does not change the name. An argument that has no `makeString` method, `.nil`
among them, raises `SYNTAX 93.938`; a `string` method alone does not do. A
`MutableBuffer`, an `Array` and a `File` have a `makeString` method, and the
name of the stream is its answer.

```rexx
f = 'spec-stream-new-zq.tmp'
call try '.stream~new'
call try '.stream~new(f, f)'
call try '.stream~new(.nil)'
call try '.stream~new(.object~new)'
call try '.stream~new(.only_string~new)'
call try '.stream~new(f)'
call try '.stream~new(12)'
call try '.stream~new("")'
call try '.stream~new(" a b ")'
call try '.stream~new(.mutablebuffer~new(f))'
call try '.stream~new(.array~of(f))'
call try '.stream~new(.has_makestring~new)'
call try '.stream~new(.file~new(f))~string~endsWith(f)'
say .stream~new(f)~state '['.stream~new(f)~description']'
say '['.stream~new(f)~query('exists')']'
e = .stream~new('')
say '['e~linein']' e~state e~open~left(6) e~state
o = .has_makestring~new
s = .stream~new(o)
say s~string s~string o~count
say s~qualify~endsWith('made-name') o~count
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  if r~isA(.stream) then say arg(1) '=> ['r~string']'
  else say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class only_string
::method string
  return 'from-string'
::class has_makestring
::attribute count
::method init
  self~count = 0
::method makestring
  self~count += 1
  return 'made-name'
```

```output
.stream~new => 93.901
.stream~new(f, f) => 93.902
.stream~new(.nil) => 93.938
.stream~new(.object~new) => 93.938
.stream~new(.only_string~new) => 93.938
.stream~new(f) => [spec-stream-new-zq.tmp]
.stream~new(12) => [12]
.stream~new("") => []
.stream~new(" a b ") => [ a b ]
.stream~new(.mutablebuffer~new(f)) => [spec-stream-new-zq.tmp]
.stream~new(.array~of(f)) => [spec-stream-new-zq.tmp]
.stream~new(.has_makestring~new) => [made-name]
.stream~new(.file~new(f))~string~endsWith(f) => 1
UNKNOWN [UNKNOWN:]
[]
[] ERROR ERROR: ERROR
made-name made-name 1
1 1
```

r[class.stream.new.sends]
`new` sends `init` to the new instance, with the arguments of `new` and no
others, so a subclass that declares `init` receives them all and decides what
the superclass `init` is given. The `init` of `Stream` checks the count of its
arguments, and then sends `makeString` to the name when it is not a string,
once, and it sends no other message. A subclass whose `init` does not send
`init` to its superclass has an instance that was never initialized, and what
that instance does is not specified.

```rexx
f = 'spec-stream-newsends-zq.tmp'
s = .loud~new(f)
say 'name' s~string
o = .counted~new
t = .stream~new(o)
say 'first sends' o~log
t = .loud~new(o)
say 'second sends' o~log
call try '.loud~new'
call try '.loud~new(.nil, 2)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class loud subclass stream
::method init
  say 'init gets' arg() 'argument(s)'
  forward class (super) continue
::class counted
::attribute log
::method init
  self~log = ''
::method makestring
  self~log = self~log 'makestring'
  return 'counted-name'
```

```output
init gets 1 argument(s)
name spec-stream-newsends-zq.tmp
first sends  makestring
init gets 1 argument(s)
second sends  makestring makestring
init gets 0 argument(s)
.loud~new => 93.901
init gets 2 argument(s)
.loud~new(.nil, 2) => 93.902
```

r[class.stream.init]
`init` may be sent again to a stream that exists. It takes the one argument that
[`new`](r:class.stream.new) gives it, and checks it as `new` does. The stream
then has the new name, no matter what it had, and is not open: its state is
`UNKNOWN`, and the first read of the stream starts at the first line of the file
that the new name denotes. Whether the file that it had open before stays open
is not specified.

```rexx
a = 'spec-stream-inita-zq.tmp'
b = 'spec-stream-initb-zq.tmp'
call lineout a, 'in a'
call lineout a
call lineout b, 'in b'
call lineout b
s = .stream~new(a)
say s~linein s~state
s~init(b)
say s~string s~state
say s~linein s~state
s~init(a)
say s~linein s~state
s~close
signal on syntax name bad
s~init
bad: say condition('o')~code
call sysfiledelete a
call sysfiledelete b
```

```output
in a READY
spec-stream-initb-zq.tmp UNKNOWN
in b READY
in a READY
93.901
```

## The name and the state

r[class.stream.string]
`string` answers the name that `init` read, as it was given: not qualified, and
with the blanks it had. It takes no argument: one raises `SYNTAX 93.902`. So
`say` and the joining operators show the name. The name is not the `objectName`
of the instance, which stays what [the object rule](r:class.index.objectname)
says.

```rexx
f = 'spec-stream-string-zq.tmp'
s = .stream~new('./' || f)
say s~string '|' s '|' s~objectName
say 'name:' || s
s~objectName = 'mine'
say s~string s~objectName
signal on syntax name bad
say s~string(1)
bad: say condition('o')~code
```

```output
./spec-stream-string-zq.tmp | ./spec-stream-string-zq.tmp | a Stream
name:./spec-stream-string-zq.tmp
./spec-stream-string-zq.tmp mine
93.902
```

r[class.stream.qualify]
`qualify` answers the full name of the stream as
[QUALIFY](r:bif.qualify.relative) makes it from the name, taken from the current
directory when it is not absolute (see [Deviation
34](r:class.stream.deviation.34)). The stream need not be open and the file need
not exist, and the state does not change. It takes no argument: one raises
`SYNTAX 88.922`. The null name has the null string as its full name.

```rexx
f = 'spec-stream-qualify-zq.tmp'
s = .stream~new(f)
say (s~qualify == qualify(f)) s~qualify~endsWith('/' || f) s~state
s2 = .stream~new('./' || f)
say (s2~qualify == s~qualify) '[' || .stream~new('')~qualify || ']'
say .stream~new(f)~query('exists') || '|'
signal on syntax name bad
say s~qualify(1)
bad: say condition('o')~code
```

```output
1 1 UNKNOWN
1 []
|
88.922
```

r[class.stream.state]
`state` answers the state of the stream, one of `UNKNOWN`, `READY`, `NOTREADY`
and `ERROR`, and `description` answers the state, a colon and what [the
description rule](r:bif.stream.description) adds to it. Each takes no argument:
one raises `SYNTAX 88.922`. An instance that has just been made is `UNKNOWN`,
and what changes the state is what [the rule for the state of a
stream](r:bif.stream.change) says. The state is that of the one instance: an
operation on another instance of the same file, or on the stream that a function
names with the same file name, does not change it.

```rexx
f = 'spec-stream-state-zq.tmp'
call lineout f, 'only'
call lineout f
a = .stream~new(f)
b = .stream~new(f)
say a~state '[' || a~description || ']' b~state
say a~linein a~state '[' || a~description || ']' b~state
say '[' || a~linein || ']' a~state '[' || a~description || ']' b~state
say '[' || b~charin(,2) || ']' a~state b~state stream(f)
a~close
say a~state b~state '[' || a~description || ']' '[' || b~description || ']'
b~close
n = .stream~new('spec-stream-state-none-zq.tmp')
say '[' || n~linein || ']' n~state n~description~left(6)
call try 'a~state(1)'
call try 'a~description("x")'
call sysfiledelete f
exit
try:
  signal on syntax name bad
  interpret arg(1)
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
UNKNOWN [UNKNOWN:] UNKNOWN
only READY [READY:] UNKNOWN
[] NOTREADY [NOTREADY:EOF] UNKNOWN
[on] NOTREADY READY UNKNOWN
UNKNOWN READY [UNKNOWN:] [READY:]
[] ERROR ERROR:
a~state(1) => 88.922
a~description("x") => 88.922
```

r[class.stream.separate]
Every instance is a stream of its own, with its own state and its own read and
write positions, also when two instances, or an instance and a function, name
the same file. They do not share what is read: each reads the file from its own
position. What an instance has written is in the file at the latest when it is
closed or flushed, or sent `uninit`.

```rexx
f = 'spec-stream-separate-zq.tmp'
call lineout f, 'one'
call lineout f, 'two'
call lineout f, 'three'
call lineout f
a = .stream~new(f)
b = .stream~new(f)
say a~linein b~linein linein(f)
say a~linein linein(f) a~query('position') b~query('position')
call stream f, 'c', 'close'
say a~linein b~linein b~linein
a~close
b~close
c = .stream~new(f)
say c~lineout('four') c~state
c~close
say .stream~new(f)~arrayin~items
call sysfiledelete f
```

```output
one one one
two two 9 5
three two three
0 READY
4
```

## What the stream functions do

r[class.stream.builtins]
These methods do what a stream function does, with the receiver as the stream
and the other arguments in the places they have after the name: `charIn`
([CHARIN](r:bif.charin.result)), `charOut` ([CHAROUT](r:bif.charout.result)),
`chars` ([CHARS](r:bif.chars.result)), `lineIn` ([LINEIN](r:bif.linein.result)),
`lineOut` ([LINEOUT](r:bif.lineout.result)), `lines`
([LINES](r:bif.lines.count)), `open` ([OPEN](r:bif.stream.open.result)), `close`
([CLOSE](r:bif.stream.close)), `flush` ([FLUSH](r:bif.stream.flush)), `seek` and
`position` ([SEEK](r:bif.stream.seek)), `query`
([QUERY](r:bif.stream.query.words)), `command` ([the
commands](r:bif.stream.command)), and `state` and `description` ([the
state](r:bif.stream.state)). The stream opens when a method first needs it, [as
the functions open it](r:bif.index.stream.open), and the answers, the states and
the NOTREADY outcomes are the functions'. The rules below state what is
different for a message: [the argument counts](r:class.stream.args), [the
arguments](r:class.stream.args.numbers), [the default of
`lines`](r:class.stream.lines), [the condition that a method
raises](r:class.stream.notready), and what the methods send. The answers are
strings. A method with no counterpart among the functions, or whose messages
another rule states, has a rule of its own.

```rexx
fa = 'spec-stream-builtins-a-zq.tmp'
fb = 'spec-stream-builtins-b-zq.tmp'
do f over fa, fb
  call lineout f, 'alpha'
  call lineout f, 'beta'
  call lineout f, 'gamma'
  call lineout f
end
s = .stream~new(fb)
say charin(fa, , 3) s~charin(, 3)
say linein(fa, 2) s~linein(2)
say chars(fa) s~chars
say lines(fa, 'C') s~lines('C')
say charout(fa, 'X', 1) s~charout('X', 1)
say lineout(fa, 'delta') s~lineout('delta')
say stream(fa, 'c', 'query size') s~query('size')
say stream(fa, 'c', 'seek 3') s~seek(3)
say stream(fa, 'c', 'query position read') s~position('=3 read')
say stream(fa) s~state stream(fa, 'd') s~description
say stream(fa, 'c', 'flush') s~flush
say stream(fa, 'c', 'close') s~close
say '['stream(fa, 'c', 'close')']' '['s~close']' s~command('close')
say stream(fa, 'c', 'open read') s~open('read')
say stream(fa, 'c', 'close') s~command('close')
say charin(fa, 1, 40) == charin(fb, 1, 40)
call sysfiledelete fa
call sysfiledelete fb
```

```output
alp alp
beta beta
6 6
1 1
0 0
0 0
17 17
3 3
3 3
READY READY READY: READY:
READY: READY:
READY: READY:
[] [] 
READY: READY:
READY: READY:
1
```

r[class.stream.args]
The count of arguments is checked before anything else, as [the count
rule](r:class.index.args.count) says, and before the stream is opened. A message
with more arguments than the method takes raises `SYNTAX 88.922` for `chars`,
`charIn`, `charOut`, `close`, `description`, `flush`, `lineIn`, `lineOut`,
`lines`, `open`, `position`, `qualify`, `seek`, `state` and `uninit`, and
`SYNTAX 93.902` for `arrayIn`, `arrayOut`, `command`, `init`, `makeArray`,
`query`, `say`, `string` and `supplier`. `charIn`, `charOut`, `lineIn`,
`lineOut` and `arrayOut` take two arguments, `lines`, `open`, `position`,
`seek`, `arrayIn`, `makeArray`, `command`, `init`, `query` and `say` one, and
the others none. `arrayOut`, `command`, `init` and `query` need their argument,
and a message without it raises `SYNTAX 93.901`; `position` and `seek` raise
`SYNTAX 88.901`. No argument is required by `say`, `lines`, `open`, `arrayIn` or
`makeArray`. In the example the state is still `UNKNOWN` after every message
that raised.

```rexx
f = 'spec-stream-args-zq.tmp'
s = .stream~new(f)
too = 'chars charIn charOut close description flush lineIn lineOut lines',
  'open position qualify seek state uninit arrayIn arrayOut command init',
  'makeArray query say string supplier'
none = 'chars close description flush qualify state uninit string supplier'
two = 'charIn charOut lineIn lineOut arrayOut'
do m over too~makearray(' ')
  if none~wordpos(m) > 0 then args = '1'
  else if two~wordpos(m) > 0 then args = '1, 2, 3'
  else args = '1, 2'
  call try 's~' || m || '(' || args || ')'
end
say '--'
do m over 'arrayOut command init query position seek'~makearray(' ')
  call try 's~' || m
end
say s~state
say '--'
call try 's~say'
call try 's~lines'
call try 's~open(, )'
call try 's~arrayIn(,)'
s~close
call sysfiledelete f
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
s~chars(1) => 88.922
s~charIn(1, 2, 3) => 88.922
s~charOut(1, 2, 3) => 88.922
s~close(1) => 88.922
s~description(1) => 88.922
s~flush(1) => 88.922
s~lineIn(1, 2, 3) => 88.922
s~lineOut(1, 2, 3) => 88.922
s~lines(1, 2) => 88.922
s~open(1, 2) => 88.922
s~position(1, 2) => 88.922
s~qualify(1) => 88.922
s~seek(1, 2) => 88.922
s~state(1) => 88.922
s~uninit(1) => 88.922
s~arrayIn(1, 2) => 93.902
s~arrayOut(1, 2, 3) => 93.902
s~command(1, 2) => 93.902
s~init(1, 2) => 93.902
s~makeArray(1, 2) => 93.902
s~query(1, 2) => 93.902
s~say(1, 2) => 93.902
s~string(1) => 93.902
s~supplier(1) => 93.902
--
s~arrayOut => 93.901
s~command => 93.901
s~init => 93.901
s~query => 93.901
s~position => 88.901
s~seek => 88.901
UNKNOWN
--
s~say => no error
s~lines => no error
s~open(, ) => no error
s~arrayIn(,) => no error
```

r[class.stream.args.numbers]
A start, a line number and a count are read as [the functions read
them](r:bif.index.stream.value): the argument's number rounded to 20 significant
digits, whatever `NUMERIC DIGITS` is in the caller, and it must be a whole
number that is not more than 9223372036854775807. An argument that is not, or
that has a form that is no number, raises `SYNTAX 88.907`; the functions raise
`SYNTAX 40.12` for it. A negative count raises `SYNTAX 88.907` too, and a
`lineIn` count above 1 raises `SYNTAX 93.0`. These are checked, from left to
right, before the stream is opened. Once it is open, a start or a line number
below 1 raises `SYNTAX 93.907`, the value `-0` among them. A stream that cannot
be opened is not asked: the call has the [NOTREADY
outcome](r:bif.index.stream.start) of a function that finds no stream, whatever
the start is.

```rexx
f = 'spec-stream-numbers-zq.tmp'
call lineout f, 'abcdef'
call lineout f
s = .stream~new(f)
numeric digits 3
say '['s~charin('1.99999999999999999999', 1)']' '['s~charin(' 3 ', '1e0')']'
say '['s~charin('+4', 1.0)']' '['s~charin(5, '+1')']'
say '['s~charin(9223372036854775807, 1)']' s~state
numeric digits 40
call try 's~charin("2.0000000000000000001", 1)'
call try 's~charin(1, "1.00000000000000000005")'
call try 's~charin(9223372036854775808, 1)'
call try 's~charin(1.5)'
call try 's~charin("1e")'
call try 's~charin(" ")'
call try 's~charin(1, -1)'
call try 's~linein(, -1)'
call try 's~linein(1, 2)'
say '--'
t = .stream~new('spec-stream-numbers-none-zq.tmp')
call try 't~charin(0, "x")'
say t~state
call try 't~charin(0, 1, 1)'
call try 't~linein(, 2)'
say t~state
call try 's~charin(0, 1)'
call try 's~charin(-3, 1)'
call try 's~charin("-0", 1)'
call try 's~linein(0)'
call try 's~charout("x", 0)'
call try 's~lineout("x", 0)'
call try 's~linein(0, 0)'
say s~state
u = .stream~new('spec-stream-numbers-dir-zq/none')
call try 'u~charin(0, 1)'
call try 'u~linein(0)'
call try 'u~charout("x", 0)'
s~close
call sysfiledelete f
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> [' || r || ']'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
[b] [c]
[d] [e]
[] NOTREADY
s~charin("2.0000000000000000001", 1) => 88.907
s~charin(1, "1.00000000000000000005") => 88.907
s~charin(9223372036854775808, 1) => 88.907
s~charin(1.5) => 88.907
s~charin("1e") => 88.907
s~charin(" ") => 88.907
s~charin(1, -1) => 88.907
s~linein(, -1) => 88.907
s~linein(1, 2) => 93.0
--
t~charin(0, "x") => 88.907
UNKNOWN
t~charin(0, 1, 1) => 88.922
t~linein(, 2) => 93.0
UNKNOWN
s~charin(0, 1) => 93.907
s~charin(-3, 1) => 93.907
s~charin("-0", 1) => 93.907
s~linein(0) => 93.907
s~charout("x", 0) => 93.907
s~lineout("x", 0) => 93.907
s~linein(0, 0) => 93.907
READY
u~charin(0, 1) => []
u~linein(0) => []
u~charout("x", 0) => [1]
```

r[class.stream.args.strings]
The string that `charOut`, `lineOut` and `say` write is read as [the string
value](r:class.index.string) that a method takes from an argument: the
`makeString` method of an object that is not a string is sent to it and its
answer is written. An argument that has no `makeString` method, `.nil` and an
object that has only `string` among them, raises `SYNTAX 88.909`; it is checked
before the stream is opened, and a count or number argument before it is read.
An argument that is a `String`, a number or a `MutableBuffer`, or an object with
a `makeString` method, is written as that string. The items that `arrayOut`
writes are read in the same way, one at a time, by the `lineOut` or `charOut`
that `arrayOut` sends; the items before a failing one are written ([the
`arrayOut` rule](r:class.stream.arrayout) has the details).

```rexx
f = 'spec-stream-strings-zq.tmp'
s = .stream~new(f)
call try 's~lineOut(.nil)'
call try 's~lineOut(.object~new)'
call try 's~lineOut(.only_string~new)'
call try 's~charOut(.nil)'
call try 's~say(.object~new)'
call try 's~lineOut(.nil, "x")'
call try 's~lineOut(.nil, 0)'
say s~state
call try 's~lineOut(12)'
call try 's~lineOut(.has_makestring~new)'
call try 's~lineOut(.mutablebuffer~new("buffer"))'
call try 's~charOut(1.50)'
call try 's~say'
call try 's~lineOut("")'
say s~state
s~close
say translate(charin(f, 1, 200), '~', '0a'x)
call sysfiledelete f
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> [' || r || ']'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class only_string
::method string
  return 'only-string'
::class has_makestring
::method makestring
  return 'made-string'
```

```output
s~lineOut(.nil) => 88.909
s~lineOut(.object~new) => 88.909
s~lineOut(.only_string~new) => 88.909
s~charOut(.nil) => 88.909
s~say(.object~new) => 88.909
s~lineOut(.nil, "x") => 88.909
s~lineOut(.nil, 0) => 88.909
UNKNOWN
s~lineOut(12) => [0]
s~lineOut(.has_makestring~new) => [0]
s~lineOut(.mutablebuffer~new("buffer")) => [0]
s~charOut(1.50) => [0]
s~say => [0]
s~lineOut("") => [0]
READY
12~made-string~buffer~1.50~~
```

r[class.stream.lines]
`lines` takes an option, as [LINES](r:bif.lines.args) does, but the option that
a message has when none is given is `C`: the answer is the number of lines left.
`N` answers 1 or 0 as the function does. Only the first character of the option
counts, in either case. An option that is the null string, a number or begins
with another character raises `SYNTAX 93.915`.

```rexx
f = 'spec-stream-lines-zq.tmp'
call lineout f, 'one'
call lineout f, 'two'
call lineout f, 'three'
call lineout f
s = .stream~new(f)
say lines(f) s~lines s~lines('N') s~lines('C') s~lines('normal') s~lines('cx')
x = s~linein
say lines(f) s~lines s~lines('N') s~lines('count')
x = s~charin(, 2)
say s~lines s~lines('N')
x = s~linein
x = s~linein
say s~lines s~lines('N') s~lines('C')
call try 's~lines("")'
call try 's~lines("x")'
call try 's~lines(1)'
call try 's~lines("N", 1)'
s~close
call sysfiledelete f
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> [' || r || ']'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 3 1 3 1 3
1 2 1 2
2 1
0 0 0
s~lines("") => 93.915
s~lines("x") => 93.915
s~lines(1) => 93.915
s~lines("N", 1) => 88.922
```

r[class.stream.notready]
A method that cannot read or write all that it was asked to raises the NOTREADY
condition in the code that sent the message, as [a function
does](r:bif.index.stream.notready): at the end of the data, when the stream
cannot be opened, and when a write fails. `condition('D')` is the name of the
stream as [`string`](r:class.stream.string) answers it, in the spelling that
`new` was given, and `condition('C')` is `NOTREADY`. The condition is not an
error: when it is not trapped the method goes on and answers as its rules say.

```rexx
f = 'spec-stream-notready-zq.tmp'
call lineout f, 'one'
call lineout f
s = .stream~new('./' || f)
n = .stream~new('spec-stream-notready-none-zq.tmp')
w = .stream~new('spec-stream-notready-nodir-zq/x')
call on notready name trap
say s~linein
say '[' || s~linein || ']'
say '[' || n~linein || ']' n~state
say '[' || s~charin(, 3) || ']'
say '[' || s~charin(9, 1) || ']'
say w~lineout('x') w~state
signal on notready name stop
x = s~linein
say 'not reached'
stop: say 'stopped:' condition('c') condition('d')
s~close
call sysfiledelete f
exit
trap: say '  NOTREADY' condition('d')
  return
```

```output
one
[]
  NOTREADY ./spec-stream-notready-zq.tmp
[] ERROR
  NOTREADY spec-stream-notready-none-zq.tmp
[]
  NOTREADY ./spec-stream-notready-zq.tmp
[]
  NOTREADY ./spec-stream-notready-zq.tmp
1 ERROR
  NOTREADY spec-stream-notready-nodir-zq/x
stopped: NOTREADY ./spec-stream-notready-zq.tmp
```

## Opening, closing and positioning

r[class.stream.open]
`open` opens the stream as [OPEN](r:bif.stream.open.result) does, with the
options of [the option rule](r:bif.stream.open.mode) in its one argument, a
string that may be left out, and answers `READY:`, or `ERROR:` with the host's
number and the NOTREADY condition when it cannot. An option that is not allowed,
such as a word that no option starts with, raises `SYNTAX 93.0`, and the stream
is not open after it, also when it was. The state is that of [the state
rule](r:class.stream.state). Opening a stream that is open opens it afresh, as
[the function does](r:bif.stream.open.again).

```rexx
f = 'spec-stream-open-zq.tmp'
call lineout f, 'abc'
call lineout f
s = .stream~new(f)
say s~open('read') s~state
say s~lineout('x') s~state
say s~open s~state '['s~charin(, 2)']'
say s~open('REP') s~query('size')
say s~close
say s~open s~state
signal on syntax name bad
say s~open('bogus')
bad: say condition('o')~code s~state
say s~open s~state
signal on syntax name bad2
say s~open('read write')
bad2: say condition('o')~code s~state
signal off syntax
n = .stream~new('spec-stream-open-nodir-zq/x')
call on notready name trap
say n~open~left(6) n~state
call sysfiledelete f
exit
trap: say 'NOTREADY' condition('d')
  return
```

```output
READY: READY
1 ERROR
READY: READY [ab]
READY: 0
READY:
READY: READY
93.0 UNKNOWN
READY: READY
93.0 UNKNOWN
ERROR: ERROR
NOTREADY spec-stream-open-nodir-zq/x
```

r[class.stream.close]
`close` closes the stream and answers `READY:`, and the stream is then
`UNKNOWN`; what was written is in the file, and the next use of the stream opens
it afresh, as [CLOSE](r:bif.stream.close) says. A stream that is not open has
nothing to close: `close` answers the null string and the state is `UNKNOWN`.

```rexx
f = 'spec-stream-close-zq.tmp'
s = .stream~new(f)
say '['s~close']' s~state
say s~lineout('first') s~state
say s~close s~state
say '['s~close']' s~state
say s~linein s~state
say s~close
say .stream~new(f)~chars
call sysfiledelete f
```

```output
[] UNKNOWN
0 READY
READY: UNKNOWN
[] UNKNOWN
first READY
READY:
6
```

r[class.stream.flush]
`flush` answers `READY:` and leaves the state as it was, [as FLUSH
does](r:bif.stream.flush), also for a stream that is not open. What it forces
out is in the file afterwards, so a stream that reads the file after the writing
stream was flushed sees it.

```rexx
f = 'spec-stream-flush-zq.tmp'
s = .stream~new(f)
say s~flush s~state
say s~lineout('data') s~flush s~state
r = .stream~new(f)
say r~linein r~state
say r~close s~close
call sysfiledelete f
```

```output
READY: UNKNOWN
0 READY: READY
data READY
READY: READY:
```

r[class.stream.query]
`query` answers what [QUERY](r:bif.stream.query.words) answers for the word in
its one argument, a string: the position, the size, the type, the file name that
exists, the times and the handle, with the options and the shortenings that the
rules of QUERY give. A word that is none of those, and words after it that the
query does not take, raise `SYNTAX 93.0`; a missing argument raises
`SYNTAX 93.901`. The answers are strings, the null string where the function
answers it. It sends no message.

```rexx
f = 'spec-stream-query-zq.tmp'
call charout f, 'abcdef' || '0a'x || 'gh'
call charout f
s = .stream~new(f)
say '['s~query('position')']' '['s~query('handle')']' s~query('size')
say s~query('streamtype') s~query('exists')~endsWith(f) s~state
x = s~charin(, 3)
say s~query('position') s~query('position read') s~query('position write')
say s~query('p w c') s~query('position read line') s~query('streamtype')
say s~query('size')~class~id s~query('handle')~datatype('W')
say s~query('timestamp')~length s~query('datetime')~length
call try 's~query("bogus")'
call try 's~query("size 1")'
call try 's~query("position x")'
call try 's~query'
call try 's~query("size", 1)'
s~close
say '['s~query('position')']' s~query('size') s~query('streamtype')
call sysfiledelete f
say '['s~query('size')']' '['s~query('exists')']' s~query('streamtype')
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> [' || r || ']'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
[] [] 9
UNKNOWN 1 UNKNOWN
4 4 10
10 1 PERSISTENT
String 1
19 17
s~query("bogus") => 93.0
s~query("size 1") => 93.0
s~query("position x") => 93.0
s~query => 93.901
s~query("size", 1) => 93.902
[] 9 UNKNOWN
[] [] UNKNOWN
```

r[class.stream.seek]
`seek` and `position` do the same: they set a position of the stream and answer
the new one, as [SEEK](r:bif.stream.seek) does, with the offset and the options
in their one argument, a string. A missing argument raises `SYNTAX 88.901`, an
offset that is not a whole number or an option that is none of those of SEEK
`SYNTAX 93.0`, and an argument that holds no offset, the null string for one,
`SYNTAX 93.903`. Neither method sends a message, and neither sends the other;
[`command`](r:class.stream.command.sends) sends `position` for both of its
commands.

```rexx
f = 'spec-stream-seek-zq.tmp'
call charout f, '0123456789'
call charout f
s = .stream~new(f)
say s~seek(3) s~query('position read') s~query('position write')
say s~position('=5 read') s~position('+2 read') s~position('-1 read')
say s~position('<2 read') '[' || s~charin(, 1) || ']'
say s~seek('4 write') s~seek('8 read') s~query('position')
say s~position(' 6  read  char ') s~seek('<0 write')
say s~seek('20 read') '[' || s~charin(, 1) || ']'
call try 's~seek'
call try 's~position'
call try 's~seek("x")'
call try 's~seek("2 bogus")'
call try 's~seek("2 read write")'
call try 's~seek("")'
call try 's~position(" ")'
call try 's~seek(1, 2)'
s~close
call sysfiledelete f
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> [' || r || ']'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
3 3 3
5 7 6
9 [8]
4 8 8
6 11
20 []
s~seek => 88.901
s~position => 88.901
s~seek("x") => 93.0
s~seek("2 bogus") => 93.0
s~seek("2 read write") => 93.0
s~seek("") => 93.903
s~position(" ") => 93.903
s~seek(1, 2) => 88.922
```

r[class.stream.command]
`command` carries out the command in its one argument, a string, as [the command
of STREAM](r:bif.stream.command) does: the first word names the command and may
be shortened, in either case, and the commands are `OPEN`, `CLOSE`, `FLUSH`,
`SEEK`, `POSITION` and `QUERY`, with the options and the answers that their
rules give. A first word that no command starts with, and so the name of another
method, raises `SYNTAX 93.914`; words after `CLOSE` or `FLUSH`, an option or a
word that the command does not know raise `SYNTAX 93.0`, and `SEEK` or
`POSITION` without an offset `SYNTAX 93.903`. A missing argument raises
`SYNTAX 93.901`. The answer is the answer of the method that carries the command
out, [which the next rule names](r:class.stream.command.sends). What `command`
does with a command that is the null string or only blanks is not specified.

```rexx
f = 'spec-stream-command-zq.tmp'
call charout f, 'abcdef'
call charout f
s = .stream~new(f)
say s~command('open read') s~command('seek 3 read') s~command('query position')
say s~command('QUERY size') s~command('q st') s~command('f') s~command('c')
say s~command('o') s~command('p =2') s~command('se <1 read') s~command('cl')
say '['s~command('close')']' s~state
call try 's~command("frob")'
call try 's~command("state")'
call try 's~command("description")'
call try 's~command("opened")'
call try 's~command("close now")'
call try 's~command("flush now")'
call try 's~command("seek")'
call try 's~command("seek x")'
call try 's~command("open bogus")'
call try 's~command("query bogus")'
call try 's~command'
call try 's~command(1)'
call try 's~command("query size", 2)'
s~close
call sysfiledelete f
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> [' || r || ']'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
READY: 3 3
6 PERSISTENT READY: READY:
READY: 2 6 READY:
[] UNKNOWN
s~command("frob") => 93.914
s~command("state") => 93.914
s~command("description") => 93.914
s~command("opened") => 93.914
s~command("close now") => 93.0
s~command("flush now") => 93.0
s~command("seek") => 93.903
s~command("seek x") => 93.0
s~command("open bogus") => 93.0
s~command("query bogus") => 93.0
s~command => 93.901
s~command(1) => 93.914
s~command("query size", 2) => 93.902
```

r[class.stream.command.sends]
`command` sends one message to the receiver for a command that it carries out,
after it has split the first word from the rest, and none for a command it
refuses. `CLOSE` sends `close` and `FLUSH` sends `flush`, with no argument.
`OPEN` sends `open`, `SEEK` and `POSITION` send `position`, and `QUERY` sends
`query`, each with one argument: the rest of the command after the word and one
blank, in upper case, with the other blanks as they were. `SEEK` sends
`position`, not `seek`. The answer of the message is the answer of `command`,
whatever it is, and an error that the message raises is the error of `command`.
A command that has words after `CLOSE` or `FLUSH`, or an unknown first word,
sends nothing. In the example a log lists each message as its name, the number
of its arguments after a `#`, and the argument in brackets.

```rexx
f = 'spec-stream-commandsends-zq.tmp'
call charout f, 'abcdef'
call charout f
s = .spy~new(f)
call show 'open read'
call show 'OPEN  Read'
call show 'seek =2 read'
call show 'SE 3'
call show 'position <1 write'
call show 'p'
call show 'query  size '
call show 'q st'
call show 'flush'
call show 'f'
call show 'close'
call show 'c'
call show 'close now'
call show 'flush now'
call show 'frob'
call show 'state'
call show 'open bogus'
s~log = ''
s~answer = 'made up'
say '[' || s~command('close') || ']' s~log
s~log = ''
s~answer = .nil
say '[' || s~command('query size') || ']' s~log
s~answer = 'RANS'
s~close
call sysfiledelete f
exit
show:
  s~log = ''
  signal on syntax name bad
  r = s~command(arg(1))
  say left(arg(1), 20) '=> [' || r || ']' s~log
  s~close
  return
bad: say left(arg(1), 20) '=>' condition('o')~code s~log
  return
::class spy subclass stream
::attribute log
::attribute answer
::method init
  expose log answer
  log = ''
  answer = 'RANS'
  forward class (super)
::method open
  expose log answer
  log = log 'open#' || arg() || '[' || arg(1) || ']'
  if answer \= 'RANS' then return answer
  forward class (super)
::method close
  expose log answer
  log = log 'close#' || arg()
  if answer \= 'RANS' then return answer
  forward class (super)
::method flush
  expose log
  log = log 'flush#' || arg()
  forward class (super)
::method position
  expose log
  log = log 'position#' || arg() || '[' || arg(1) || ']'
  forward class (super)
::method seek
  expose log
  log = log 'seek#' || arg() || '[' || arg(1) || ']'
  forward class (super)
::method query
  expose log answer
  log = log 'query#' || arg() || '[' || arg(1) || ']'
  if answer \= 'RANS' then return answer
  forward class (super)
```

```output
open read            => [READY:]  open#1[READ]
OPEN  Read           => [READY:]  open#1[ READ]
seek =2 read         => [2]  position#1[=2 READ]
SE 3                 => [3]  position#1[3]
position <1 write    => [6]  position#1[<1 WRITE]
p                    => 93.903  position#1[]
query  size          => [6]  query#1[ SIZE ]
q st                 => [UNKNOWN]  query#1[ST]
flush                => [READY:]  flush#0
f                    => [READY:]  flush#0
close                => []  close#0
c                    => []  close#0
close now            => 93.0 
flush now            => 93.0 
frob                 => 93.914 
state                => 93.914 
open bogus           => 93.0  open#1[BOGUS]
[made up]  close#0
[The NIL object]  query#1[SIZE]
```

## Writing lines and arrays

r[class.stream.say]
`say` writes a line, as `lineOut` does with one argument: the answer of `say` is
the answer of that `lineOut`, 0 when the line was written, as
[LINEOUT](r:bif.lineout.result) answers, and its string is checked [as the
string of `lineOut` is](r:class.stream.args.strings). The argument is optional;
without it `say` writes an empty line. It takes at most one argument. The
messages that `say` sends are those of [the next
rule](r:class.stream.say.sends).

```rexx
f = 'spec-stream-say-zq.tmp'
s = .stream~new(f)
say s~say('one') s~state
say s~say
say s~say(12)
say s~say('a b')
signal on syntax name bad
say s~say(.nil)
bad: say condition('o')~code
signal off syntax
signal on syntax name bad2
say s~say('x', 'y')
bad2: say condition('o')~code
s~close
say translate(charin(f, 1, 100), '~', '0a'x)
call sysfiledelete f
```

```output
0 READY
0
0
0
88.909
93.902
one~~12~a b~
```

r[class.stream.say.sends]
`say` sends `lineOut` to the receiver, with one argument, the argument of `say`
itself and not its string value, or the null string when it has none. It sends
nothing else and reads nothing from its argument: a subclass that overrides
`lineOut` sees every `say`, and it is `lineOut` that checks the string. The
answer of `say` is the answer of that message, whatever it is, or no value when
the message answers none, so that its answer as a term raises `SYNTAX 91.999`.

```rexx
f = 'spec-stream-saysends-zq.tmp'
s = .loud~new(f)
say s~say('one')
say s~say
say s~say(12)
x = s~say(.shown~new)
signal on syntax name bad0
x = s~say(.nil)
bad0: say condition('o')~code
s~answer = 'zz'
say s~say('two')
s~answer = 'NOANSWER'
signal on syntax name bad
say s~say('three')
bad: say condition('o')~code
signal off syntax
s~close
call sysfiledelete f
exit
::class shown
::method makestring
  return 'made'
::class loud subclass stream
::attribute answer
::method init
  expose answer
  answer = 'RANS'
  forward class (super)
::method lineout
  expose answer
  if arg(1) == .nil then kind = '.nil'
  else kind = arg(1)~class~id
  say '  lineout gets' arg() 'argument:' kind
  if answer = 'NOANSWER' then return
  if answer \= 'RANS' then return answer
  forward class (super)
```

```output
  lineout gets 1 argument: String
0
  lineout gets 1 argument: String
0
  lineout gets 1 argument: String
0
  lineout gets 1 argument: SHOWN
  lineout gets 1 argument: .nil
88.909
  lineout gets 1 argument: String
zz
  lineout gets 1 argument: String
91.999
```

r[class.stream.arrayin]
`arrayIn` reads the stream from its read position to its end and answers a new
`Array` of strings. With no argument, or with an argument whose first character
is `L` in either case, the items are the lines, without their [line
ends](r:bif.index.stream.line-end), the first one a partial line when the stream
was read part of the way into it; with an argument whose first character is `C`
they are the characters, a line end among them, one to an item. Any other
argument, the null string and a number among them, raises `SYNTAX 93.915`. The
stream is read as [LINEIN](r:bif.linein.result) and
[CHARIN](r:bif.charin.result) read, and the read position is at the end
afterwards: the state is `NOTREADY` (see [Deviation
62](r:class.stream.deviation.62) for the characters). A stream that is at its
end, and one that cannot be read, give an empty array, and a stream that cannot
be read is `ERROR`.

```rexx
f = 'spec-stream-arrayin-zq.tmp'
call charout f, 'alpha' || '0a'x || 'be' || '0d0a'x || '0a'x || 'gamma' || '0a'x
call charout f, 'last'
call charout f
s = .stream~new(f)
say s~charin(, 2)
a = s~arrayin
say a~class~id a~items '[' || a~makeString('L', '|') || ']' s~state
say s~arrayin~items s~state '['s~linein']'
s~close
a = s~arrayin('chars')
say a~items a~at(1) a~at(6)~c2x a~at(a~items)
do o over 'L', 'l', 'lines', 'LX', 'Chars', 'c', 'cats'
  s~close
  say o s~arrayin(o)~items
end
do o over '', 'x', 1, ' L'
  call try o
end
n = .stream~new('spec-stream-arrayin-none-zq.tmp')
say n~arrayin~items n~state n~description~left(6)
s~close
call sysfiledelete f
exit
try:
  signal on syntax name bad
  say '[' || arg(1) || ']' s~arrayin(arg(1))~items
  return
bad: say '[' || arg(1) || ']' condition('o')~code
  return
```

```output
al
Array 5 [pha|be||gamma|last] NOTREADY
0 NOTREADY []
21 a 0A t
L 5
l 5
lines 5
LX 5
Chars 21
c 21
cats 21
[] 93.915
[x] 93.915
[1] 93.915
[ L] 93.915
0 ERROR ERROR:
```

r[class.stream.arrayin.sends]
When `arrayIn` has an argument that is not a string, it sends it
`caselessStartsWith('L')` and, when that answers false,
`caselessStartsWith('C')`, and no other message, to find the option; when both
answer false the option is wrong and `SYNTAX 93.915` is raised. Then, for the
lines, it sends `lines` with no argument to the receiver once, before it reads,
and reads that answer as a whole number that is not negative, whatever it is
used for: another answer raises `SYNTAX 93.906`, and the array is the same
whatever the number is. The lines are then read from the stream directly; no
`lineIn` is sent. For the characters it sends `chars` with no argument once,
with the same check of the answer, and then `charIn`, with no argument, once for
each character and once more: it stops when a `charIn` raises NOTREADY, and the
answer of that one is not an item. The `charIn` of `Stream` raises it at the end
of the data, and so does one that a subclass writes with `RAISE NOTREADY`. The
answer of `arrayIn` is the array. Where a large `lines` answer starts to be
refused is not specified.

```rexx
f = 'spec-stream-arrayinsends-zq.tmp'
call lineout f, 'one'
call lineout f, 'two'
call lineout f
.local~log = ''
s = .spy~new(f)
s~count = 5
a = s~arrayin
say a~items a~makeString('L', ',') .local~log
do n over 0, ' 3 ', '1e1', 2.5, -1, 'x', .nil
  s~close
  s~count = n
  call try n
end
s~close
s~count = 0
.local~log = ''
a = s~arrayin('chars')
say a~items a~makeString('L', ',') .local~log
do o over .opt~new('1'), .opt~new('0 1'), .opt~new('0 0')
  .local~log = ''
  s~close
  call try2 o
end
s~close
call sysfiledelete f
exit
try:
  signal on syntax name bad
  .local~log = ''
  a = s~arrayin
  say '[' || arg(1) || ']' '->' a~items .local~log
  return
bad: say '[' || arg(1) || ']' '=>' condition('o')~code .local~log
  return
try2:
  signal on syntax name bad2
  a = s~arrayin(arg(1))
  say a~items .local~log
  return
bad2: say condition('o')~code .local~log
  return
::class spy subclass stream
::attribute count
::attribute reads
::method lines
  .local~log = .local~log 'lines'
  return self~count
::method chars
  .local~log = .local~log 'chars'
  self~reads = 0
  return self~count
::method linein
  .local~log = .local~log 'linein'
  forward class (super)
::method charin
  .local~log = .local~log 'charin'
  self~reads = self~reads + 1
  if self~reads > 3 then raise notready
  return 'c' || self~reads
::class opt
::attribute answers
::method init
  expose answers
  use arg answers
::method caselessStartsWith
  expose answers
  .local~log = .local~log 'caselessStartsWith(' || arg(1) || ')'
  parse var answers first answers
  return first
```

```output
2 one,two  lines
[0] -> 2  lines
[ 3 ] -> 2  lines
[1e1] -> 2  lines
[2.5] => 93.906  lines
[-1] => 93.906  lines
[x] => 93.906  lines
[The NIL object] => 93.906  lines
3 c1,c2,c3  chars charin charin charin charin
2  caselessStartsWith(L) lines
3  caselessStartsWith(L) caselessStartsWith(C) chars charin charin charin charin
93.915  caselessStartsWith(L) caselessStartsWith(C)
```

r[class.stream.makearray]
`makeArray` does what [`arrayIn`](r:class.stream.arrayin) does, with the same
argument: it answers a new `Array` of the lines of the stream, or of its
characters, and an argument that `arrayIn` refuses is refused with the same
error. It takes at most one argument. The message `request` with the argument
`ARRAY`, sent to a stream, is answered by `makeArray` too, so the lines of a
stream are what `arrayOut` of another stream writes when it is given the stream.

```rexx
f = 'spec-stream-makearray-zq.tmp'
g = 'spec-stream-makearray2-zq.tmp'
call lineout f, 'one'
call lineout f, 'two'
call lineout f
s = .stream~new(f)
a = s~makearray
say a~class~id a~items a~makeString('L', ',') s~state
s~close
say s~makearray('chars')~items
s~close
say s~makearray('c')~items
s~close
call try 's~makearray("x")'
call try 's~makearray(1)'
call try 's~makearray("l", 2)'
call try 's~request("ARRAY")~items'
s~close
d = .stream~new(g)
say d~arrayout(s) d~close
say .stream~new(g)~makearray~makeString('L', ',')
s~close
call sysfiledelete f
call sysfiledelete g
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
Array 2 one,two NOTREADY
8
8
s~makearray("x") => 93.915
s~makearray(1) => 93.915
s~makearray("l", 2) => 93.902
s~request("ARRAY")~items => 2
0 READY:
one,two
```

r[class.stream.makearray.sends]
`makeArray` sends `arrayIn` to the receiver, once, with the argument of
`makeArray` when it has one and with no argument when it has none, and sends
nothing else. It answers what `arrayIn` answers, an object that is no array
included.

```rexx
f = 'spec-stream-makearraysends-zq.tmp'
call lineout f, 'one'
call lineout f, 'two'
call lineout f
t = .spy~new(f)
say t~makearray~items t~log
t~close
say t~makearray('c')~items t~log
t~close
t~answer = 'zz'
say t~makearray('l') t~log
t~answer = .nil
say (t~makearray == .nil) t~log
t~answer = 'RANS'
t~close
call sysfiledelete f
exit
::class spy subclass stream
::attribute log
::attribute answer
::method init
  self~log = ''
  self~answer = 'RANS'
  forward class (super)
::method arrayin
  self~log = self~log 'arrayin#' || arg() || '[' || arg(1) || ']'
  if self~answer \= 'RANS' then return self~answer
  forward class (super)
```

```output
2  arrayin#0[]
8  arrayin#0[] arrayin#1[c]
zz  arrayin#0[] arrayin#1[c] arrayin#1[l]
1  arrayin#0[] arrayin#1[c] arrayin#1[l] arrayin#0[]
```

r[class.stream.arrayout]
`arrayOut` writes the items of the collection in its first argument to the
stream, one `lineOut` for each item, or one `charOut` for each when the second
argument says characters, and answers the number of items that were not written:
0 when all were. The first argument is sent `request` with the argument `ARRAY`,
which an `Array`, a `List`, a `Set`, a `Queue` and a stream answer with an array
of their items, and the items are written in the order of that array, an empty
slot giving none; an argument whose answer is not an array, `.nil` among them,
raises `SYNTAX 98.913`. The second argument is optional: it is sent `left` with
the argument 1, and the answer must be `L` for lines or `C` for characters, in
either case, or `SYNTAX 93.0` is raised; the option is checked before the first
argument is. An item is written as `lineOut` and `charOut` read their string, so
an item that has no `makeString` method raises `SYNTAX 88.909` in the write of
that item, after the items before it were written. When a write raises NOTREADY
the writing ends at that item, which was not written: the answer is the number
of items from it to the end. A missing first argument raises `SYNTAX 93.901`,
and more than two `SYNTAX 93.902`. An empty array writes nothing and does not
open the stream.

```rexx
f = 'spec-stream-arrayout-zq.tmp'
s = .stream~new(f)
say s~arrayout(.array~of('one', 'two')) s~state
say s~arrayout(.list~of('three')) s~arrayout(.array~of(4, 5), 'C')
say s~arrayout(.array~new(3)~~put('six', 1)~~put('eight', 3), 'lines')
say s~arrayout(.array~new) s~arrayout(.array~new(2))
say s~arrayout(.array~of('x', 'y'), .opt~new('L'))
say s~arrayout(.array~of('z'), 'c')
say s~arrayout(.set~of('set')) s~arrayout(.queue~of('q1', 'q2'))
e = .stream~new('spec-stream-arrayout-empty-zq.tmp')
say e~arrayout(.array~new) e~state '[' || e~query('exists') || ']'
s~close
say translate(charin(f, 1, 200), '~', '0a'x)
call try 's~arrayout'
call try 's~arrayout(.array~of(1), "x")'
call try 's~arrayout(.nil, "x")'
call try 's~arrayout(.nil)'
call try 's~arrayout(.object~new)'
call try 's~arrayout(.array~of(1), "L", 3)'
call try 's~arrayout(.array~of("p", .nil, "q"))'
call try 's~arrayout(.array~of("a", .object~new), "C")'
s~close
say translate(charin(f, 1, 200), '~', '0a'x)
r = .stream~new(f)
r~open('read')
say r~arrayout(.array~of('a', 'b', 'c')) r~state r~arrayout(.array~new)
r~close
call sysfiledelete f
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class opt
::attribute word
::method init
  self~word = arg(1)
::method left
  return self~word
```

```output
0 READY
0 0
0
0 0
0
0
0 0
0 UNKNOWN []
one~two~three~45six~eight~x~y~zset~q1~q2~
s~arrayout => 93.901
s~arrayout(.array~of(1), "x") => 93.0
s~arrayout(.nil, "x") => 93.0
s~arrayout(.nil) => 98.913
s~arrayout(.object~new) => 98.913
s~arrayout(.array~of(1), "L", 3) => 93.902
s~arrayout(.array~of("p", .nil, "q")) => 88.909
s~arrayout(.array~of("a", .object~new), "C") => 88.909
one~two~three~45six~eight~x~y~zset~q1~q2~p~a
3 ERROR 0
```

r[class.stream.arrayout.sends]
`arrayOut` sends, in this order: `left(1)` to the second argument when it has
one and it is not a string, `request('ARRAY')` to the first argument, and then
`lineOut`, or `charOut` for the characters, to the receiver for each item in
turn, with the item itself as the one argument. It sends no other message to the
receiver, the array or an item; what the sends answer is not looked at, except
that a NOTREADY that a send raises ends the writing, as [the rule of
`arrayOut`](r:class.stream.arrayout) says. That condition reaches `arrayOut`
when the method that answers the send hands the message on in place, as
`FORWARD` without `CONTINUE` does, and it does not when the method sends the
message on and answers afterwards, as `FORWARD CONTINUE` and a message to
`SUPER` do: the condition is then raised in that method, and `arrayOut` writes
every item and answers 0. An item is not read by `arrayOut`.

```rexx
f = 'spec-stream-arrayoutsends-zq.tmp'
call lineout f, 'orig'
call lineout f
items = .array~of('a', .shown~new, 'c')
do mode over 'record', 'forward', 'continue', 'super'
  s = .spy~new(f)
  s~mode = mode
  s~open('read')
  say left(mode, 9) s~arrayout(items) s~state s~log
  s~close
end
.local~log = ''
s = .spy~new(f)
s~mode = 'record'
say s~arrayout(.arr~of('p', 'q'), .opt~new) .local~log s~log
call sysfiledelete f
exit
::class shown
::method makestring
  return 'shown'
::class spy subclass stream
::attribute log
::attribute mode
::method init
  self~log = ''
  forward class (super)
::method lineout
  self~log = self~log 'lineOut(' || arg(1)~class~id || ')'
  .local~log = .local~log 'lineOut'
  select
    when self~mode = 'record' then return 0
    when self~mode = 'forward' then forward class (super)
    when self~mode = 'continue' then do
      forward class (super) continue
      return result
    end
    when self~mode = 'super' then return self~lineout:super(arg(1))
  end
::method charout
  self~log = self~log 'charOut'
  return 0
::class opt
::method left
  .local~log = .local~log 'left(' || arg(1) || ')'
  return 'L'
::class arr subclass array
::method request
  .local~log = .local~log 'request(' || arg(1) || ')'
  forward class (super)
```

```output
record    0 READY  lineOut(String) lineOut(SHOWN) lineOut(String)
forward   3 ERROR  lineOut(String)
continue  0 ERROR  lineOut(String) lineOut(SHOWN) lineOut(String)
super     0 ERROR  lineOut(String) lineOut(SHOWN) lineOut(String)
0  left(1) request(ARRAY) lineOut lineOut  lineOut(String) lineOut(String)
```

r[class.stream.supplier]
`supplier` answers a new [`StreamSupplier`](streamsupplier.md) that stands for
the lines of the receiver that are left, as [the rule of that
class](r:class.streamsupplier.entry) says: it is at the line that the stream
reads next, and making it reads that line. It takes no argument: one raises
`SYNTAX 93.902`. A stream that has no line left, or cannot be read, gives a
supplier that is not available. The messages that the stream is sent are those
of [the next rule](r:class.stream.supplier.sends).

```rexx
f = 'spec-stream-supplier-zq.tmp'
call lineout f, 'one'
call lineout f, 'two'
call lineout f, 'three'
call lineout f
s = .stream~new(f)
p = s~supplier
say p~class~id p~index p~item p~available
say s~linein
q = s~supplier
say q~index q~item
q~next
say q~available
p~next
say p~index p~item p~available
s~close
n = .stream~new('spec-stream-supplier-none-zq.tmp')
say n~supplier~available n~state
signal on syntax name bad
x = s~supplier(1)
bad: say condition('o')~code
s~close
call sysfiledelete f
```

```output
StreamSupplier 1 one 1
two
3 three
0
2 two 1
0 UNKNOWN
93.902
```

r[class.stream.supplier.sends]
The supplier that `supplier` makes sends the stream, in this order, `state`
twice and `lineIn` with one argument, the number of the line to read, when it is
made, and `lineIn` with the number of the next line for each `next`; the other methods of the supplier send nothing. `supplier` sends nothing else.

```rexx
f = 'spec-stream-suppliersends-zq.tmp'
call lineout f, 'one'
call lineout f, 'two'
call lineout f, 'three'
call lineout f
do pre over 0, 1
  s = .spy~new(f)
  do pre
    x = s~linein
  end
  s~log = ''
  p = s~supplier
  say 'read' pre p~class~id p~index p~item p~available s~log
  s~log = ''
  p~next
  say '  next' p~index p~item s~log
  s~close
end
s = .spy~new('spec-stream-suppliersends-none-zq.tmp')
p = s~supplier
l = s~log
say p~available s~state l
s~close
call sysfiledelete f
exit
::class spy subclass stream
::attribute log
::method init
  self~log = ''
  forward class (super)
::method state
  self~log = self~log 'state'
  forward class (super)
::method linein
  self~log = self~log 'lineIn(' || arg(1) || ')'
  forward class (super)
```

```output
read 0 StreamSupplier 1 one 1  state state lineIn(1)
  next 2 two  lineIn(2)
read 1 StreamSupplier 2 two 1  state state lineIn(2)
  next 3 three  lineIn(3)
0 UNKNOWN  state state lineIn(1)
```

r[class.stream.uninit]
`uninit` closes the stream when it is open, as `close` does, so that what was
written is in the file. The interpreter sends it to an instance that is no
longer used; when it does so is not specified. A program may send it too, and it
answers no value: its answer as a term raises `SYNTAX 91.999`. It takes no
argument: one raises `SYNTAX 88.922`. It sends no message. An instance that has
been sent `uninit` is not to be used again: what a method does then is not
specified.

```rexx
f = 'spec-stream-uninit-zq.tmp'
s = .stream~new(f)
x = s~lineout('kept')
call try 's~uninit(1)'
s~uninit
say .stream~new(f)~linein
t = .stream~new(f)
x = t~linein
signal on syntax name bad
say t~uninit
bad: say condition('o')~code
call sysfiledelete f
exit
try:
  signal on syntax name bad2
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad2: say arg(1) '=>' condition('o')~code
  return
```

```output
s~uninit(1) => 88.922
kept
91.999
```

## Deviations

r[class.stream.deviation.34]
In a name that holds `..`, `qualify` removes exactly the one component before
each, however it was written, as [QUALIFY](r:bif.qualify.deviation.34) does, and
the stream is the file that this full name denotes. ooRexx removes none after a
doubled slash and two after a component that starts with a period, so its
`qualify` answers another name and the stream can be another file ([Deviation
34](r:bif.stream.deviation.34)).

```rexx
say .stream~new('/a/.b/../c')~qualify .stream~new('/a//../b')~qualify
```

```output
/a/c /b
```

r[class.stream.deviation.55]
A stream opened `WRITE` reads nothing, as [the
function](r:bif.stream.deviation.55) says, and so every method that reads fails
on it: `lineIn` and `charIn` answer the null string and the state is `ERROR`,
`lines` and `chars` answer 0, a supplier of the stream is not available, and
`arrayIn` answers an empty array. ooRexx opens such a stream for reading and
writing, so that these methods read the file.

```rexx
f = 'spec-stream-dev55-zq.tmp'
call lineout f, 'one'
call lineout f, 'two'
call lineout f
s = .stream~new(f)
say s~open('write') s~state
say '[' || s~linein || ']' s~state s~description~left(6)
say s~lines s~chars s~supplier~available s~arrayin~items
say '[' || s~charin || ']' s~state
s~close
call sysfiledelete f
```

```output
READY: READY
[] ERROR ERROR:
0 0 0 0
[] ERROR
```

r[class.stream.deviation.62]
A read that finds nothing at the end of the data leaves the stream `NOTREADY`
with the description `NOTREADY:EOF`, [as for the
functions](r:bif.stream.deviation.62), and that holds for the `charIn` that
`arrayIn` sends to read the characters, at the end of them. ooRexx leaves
`ERROR:0` after a read of characters that finds none.

```rexx
f = 'spec-stream-dev62-zq.tmp'
call charout f, 'ab'
call charout f
s = .stream~new(f)
say s~arrayin('chars')~items s~state s~description
s~close
say s~charin(, 2) s~state '[' || s~charin || ']' s~state s~description
s~close
call sysfiledelete f
```

```output
2 NOTREADY NOTREADY:EOF
ab READY [] NOTREADY NOTREADY:EOF
```

r[class.stream.deviation.109]
An instance of a subclass of `String` that is given as a name, a string to
write, an option, an offset, a start, a count or any other string argument of a
`Stream` is read by its own characters, and `request`, `makeString` and `string`
are not sent to it ([Deviation 109](r:class.string.deviation.109)). ooRexx sends
such an instance `request` with `STRING`, so an override can change what is
read. The example prints what the own characters of the instances give, and the
number of messages that were sent to them, which is 0.

```rexx
f = 'spec-stream-dev109-zq.tmp'
.environment~sent = 0
s = .stream~new(.digits~new(f))
say s~string
x = s~lineout(.digits~new('abc'))
x = s~charout(.digits~new('def'))
x = s~say(.digits~new('ghi'))
x = s~arrayout(.array~of(.digits~new('jkl')))
s~close
say translate(charin(f, 1, 100), '~', '0a'x)
say s~open(.digits~new('read')) s~state
say s~seek(.digits~new('2')) s~position(.digits~new('=3 read'))
say s~lines(.digits~new('c')) s~charin(.digits~new('2'), .digits~new('1'))
say s~linein(.digits~new('1'), .digits~new('1'))
say s~query(.digits~new('size')) s~command(.digits~new('query size'))
s~close
say 'sent' .environment~sent
call sysfiledelete f
::class digits subclass string
::method makestring
  .environment~sent = .environment~sent + 1
  return 'zzz'
::method request
  .environment~sent = .environment~sent + 1
  return 'zzz'
::method string
  .environment~sent = .environment~sent + 1
  return 'zzz'
```

```output defect=#140
spec-stream-dev109-zq.tmp
abc~defghi~jkl~
READY: READY
2 3
3 b
abc
15 15
sent 0
```
