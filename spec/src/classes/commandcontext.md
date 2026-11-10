---
element: "CommandContext"
kind: class
deviations: [109]
rexxref: "Builtin Classes > Utility Classes > CommandContext Class; Builtin Classes > Utility Classes > CommandContext Class > [ ]; Builtin Classes > Utility Classes > CommandContext Class > [ ]=; Builtin Classes > Utility Classes > CommandContext Class > dropVar; Builtin Classes > Utility Classes > CommandContext Class > environment; Builtin Classes > Utility Classes > CommandContext Class > error; Builtin Classes > Utility Classes > CommandContext Class > getVar; Builtin Classes > Utility Classes > CommandContext Class > hasVar; Builtin Classes > Utility Classes > CommandContext Class > input; Builtin Classes > Utility Classes > CommandContext Class > isRedirected; Builtin Classes > Utility Classes > CommandContext Class > isValid; Builtin Classes > Utility Classes > CommandContext Class > output; Builtin Classes > Utility Classes > CommandContext Class > raise; Builtin Classes > Utility Classes > CommandContext Class > raiseError; Builtin Classes > Utility Classes > CommandContext Class > raiseFailure; Builtin Classes > Utility Classes > CommandContext Class > setVar; Builtin Classes > Utility Classes > CommandContext Class > variables"
---

# CommandContext

When a command reaches a handler written in Rexx, the interpreter makes a
`CommandContext` for that one command and passes it to the method `command` of
the [`CommandHandler`](commandhandler.md) as the second argument. The context is
the handler's view of the command's surroundings: the variables of the code that
sent the command, the `ADDRESS ... WITH` redirections of its input, output and
error, and the one condition that the command may raise when the handler is
done. The streams that a context hands out are [`CommandInput`](commandinput.md)
and [`CommandOutput`](commandoutput.md) objects. The redirections themselves,
`ADDRESS ... WITH`, are specified with [the ADDRESS
instruction](../instructions/address.md). Which activity runs the handler, and
how commands to the environments of a host work, is specified in the environment
chapter; the examples here register environments of their own under names that
no host has, so that nothing depends on the host.

## The class

r[class.commandcontext.creation]
`CommandContext` has the instance methods `[]`, `[]=`, `dropVar`, `environment`,
`error`, `getVar`, `hasVar`, `input`, `isRedirected`, `isValid`, `output`,
`raise`, `raiseError`, `raiseFailure`, `setVar` and `variables`, besides the
methods that every class has. Its only superclass is `Object` and it inherits no
mixin class ([the hierarchy rule](r:class.index.hierarchy)). The interpreter
makes one for each command that is sent to a handler written in Rexx, and a
program receives it as the second argument of `command`; a program does not make
one. The `new` of a program raises `SYNTAX 93.901` with fewer than two arguments
and `SYNTAX 93.902` with more than two, and what an object made with two
arguments is good for is not specified: it belongs to no command, so its methods
have nothing to work with. The class has other methods, `finish` and `pending`
among them, that the interpreter uses; they are not part of this specification.

```rexx
say .commandcontext~id .commandcontext~superclasses~items,
  .commandcontext~superclass~id .commandcontext~queryMixinClass
have = .set~new
s = .commandcontext~methods(.nil)
do while s~available
  have~put(s~index)
  s~next
end
names = ('[] []= dropVar environment error getVar hasVar input isRedirected',
  'isValid output raise raiseError raiseFailure setVar',
  'variables')~makeArray(' ')
count = 0
do name over names
  if have~hasItem(name~upper) then count = count + 1
end
say count 'of' names~items
call try '.commandcontext~new'
call try '.commandcontext~new(1)'
call try '.commandcontext~new(1, 2, 3)'
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
```

```output
CommandContext 1 Object 0
16 of 16
.commandcontext~new 93.901
.commandcontext~new(1) 93.901
.commandcontext~new(1, 2, 3) 93.902
```

r[class.commandcontext.valid]
A context is good for the command that it stands for, on the activity that is
running the handler, and for as long as that command runs. `isValid` answers `1`
while the command runs, and `0` once the handler has returned. It takes no
argument (`SYNTAX 88.922`). After the command, every method that works with the
code that sent it, `[]`, `getVar`, `hasVar`, `setVar`, `[]=`, `dropVar`,
`variables`, `isRedirected`, `raise`, `raiseError` and `raiseFailure`, raises
`SYNTAX 98.900`, and so do the methods that read from the stream that `input`
answered and `lineOut` with a string, of the streams that `output` and `error`
answered, and `input`, `output` and `error` themselves when they were not asked
during the command; one that was asked answers the same object again.
`environment` and `isValid` work after the command, and they work on another
activity too, where the methods of the first list raise `SYNTAX 98.900` also
while the command runs, and `input`, `output` and `error` behave as they do
after the command; `copy` works in both cases. The example keeps the context,
asks it from another activity that a message starts, and asks it again after the
command.

```rexx
x = .commandhandler~register('spec_cc_valid', .keeper~new)
address spec_cc_valid 'go' with input using (.array~of('l')),
  output stem o. error stem e.
c = .environment~kept
say 'after the command:' c~isValid c~environment
do m over ('hasVar getVar variables isRedirected input output error',
    'raiseError copy')~makeArray(' ')
  call try m
end
say 'streams:'
call tryStream 'linein'
call tryStream 'lines'
call tryStream 'chars'
call tryStream 'charin'
signal on syntax name bad
x = c~isValid(1)
bad: say condition('o')~code
x = .commandhandler~deregister('spec_cc_valid')
exit
try:
  signal on syntax name e
  if arg(1) = 'hasVar' | arg(1) = 'getVar' then x = c~send(arg(1), 'a')
  else if arg(1) = 'isRedirected' then x = c~send(arg(1), 'I')
  else if arg(1) = 'raiseError' then x = c~send(arg(1), 1)
  else x = c~send(arg(1))
  say arg(1) 'answered' x~class~id
  return
e: say arg(1) condition('o')~code
  return
tryStream:
  signal on syntax name e2
  x = .environment~input~send(arg(1))
  say arg(1) 'answered'
  return
e2: say arg(1) condition('o')~code
  return
::class keeper inherit CommandHandler
::method command
  use arg command, caller
  .environment~kept = caller
  .environment~input = caller~input
  .environment~output = caller~output
  .environment~error = caller~error
  say 'running:' caller~isValid
  m = .message~new(.other~new, 'ask', 'I', caller)
  m~start
  say 'other activity:' m~result
  return 0
::class other
::method ask unguarded
  use arg c
  r = c~isValid c~environment c~copy~isValid c~input~class~id
  signal on syntax name e
  x = c~hasVar('a')
  return r 'hasVar answered'
e: return r 'hasVar' condition('o')~code
```

```output
running: 1
other activity: 1 SPEC_CC_VALID 1 CommandInput hasVar 98.900
after the command: 0 SPEC_CC_VALID
hasVar 98.900
getVar 98.900
variables 98.900
isRedirected 98.900
input answered CommandInput
output answered CommandOutput
error answered CommandOutput
raiseError 98.900
copy answered CommandContext
streams:
linein 98.900
lines 98.900
chars 98.900
charin 98.900
88.922
```

r[class.commandcontext.environment]
`environment` answers the name of the environment that the command was sent to,
as the instruction that sent it gave it: a symbol is in upper case, as symbols
are, and a string is as it was written, so the case of the environment name that
a command used shows. This is how a handler that serves several names learns
which of them the command used. The method takes no argument (`SYNTAX 93.902`)
and, as [the validity rule](r:class.commandcontext.valid) says, works after the
command too.

```rexx
h = .who~new
x = .commandhandler~register('spec_cc_one', h)
x = .commandhandler~register('Spec_Cc_Two', h)
address spec_cc_one 'a'
address SPEC_CC_TWO 'b'
address 'spec_cc_two' 'c'
address spec_cc_two
'd'
x = .commandhandler~deregister('spec_cc_one')
x = .commandhandler~deregister('spec_cc_two')
::class who inherit CommandHandler
::method command
  use arg command, caller
  say command 'sent to' caller~environment
  signal on syntax name bad
  x = caller~environment(1)
bad: say '  environment(1)' condition('o')~code
  return 0
```

```output
a sent to SPEC_CC_ONE
  environment(1) 93.902
b sent to SPEC_CC_TWO
  environment(1) 93.902
c sent to spec_cc_two
  environment(1) 93.902
d sent to SPEC_CC_TWO
  environment(1) 93.902
```

## Variables of the code that sent the command

r[class.commandcontext.name]
The `name` that `[]`, `getVar`, `[]=`, `setVar`, `hasVar` and `dropVar` take
names a variable of the code that sent the command, a simple variable, a stem or
a compound variable, as [a variable symbol](r:lex.symbols.variable) does, and
its case plays no part. A string is the name as it is; another object, an
instance of a subclass of `String` apart ([the deviation
rule](r:class.commandcontext.deviation.109)), is replaced by what it answers to
the message `request` with the argument `String`. The `request` of `Object`
answers what `makeString` answers, so an object with a `makeString`, an `Array`
or a `MutableBuffer` is read through it. A name is refused with `SYNTAX 93.900`
when it is no variable name: the null string, a string that starts with a digit
or with a period, a string with a blank or with a character that no symbol can
have, a number, and an object that does not answer a string. A name of a
compound variable has its tail resolved in the code that sent the command, as
[the tail rule](r:expr.variables.tail) says for a program; the example sets
`c.i` while `i` holds `3`, and reads the variable back as `c.3`. Special names
such as `RC`, `RESULT` and `SIGL` are names of variables like the others.

```rexx
x = .commandhandler~register('spec_cc_name', .namer~new)
i = 3; word = 'abc'; rc = 'my rc'
address spec_cc_name 'go'
x = .commandhandler~deregister('spec_cc_name')
::class namer inherit CommandHandler
::method command
  use arg command, caller
  caller['c.i'] = 'compound'
  caller['Plain'] = 'simple'
  say caller['C.3'] caller['c.3'] caller['PLAIN'] caller['word'] caller['rc']
  say caller[.logger~new] caller[.array~of('word')],
    caller[.mutablebuffer~new('word')]
  say '<'caller['absent']'>' '<'caller['stem.']'>' '<'caller['stem.a.b']'>'
  do n over .array~of('', '1x', '.x', 'a b', 'a-b', ' ', '3', 'a+', '.nil')
    self~try(caller, n)
  end
  self~try(caller, 12)
  self~try(caller, .nil)
  self~try(caller, .object~new)
  say caller['c.3'] caller['!x?_'] caller['x..y']
  return 0
::method try
  use arg caller, n
  signal on syntax name bad
  x = caller[n]
  say '<'n'> no error'
  return
bad: say '<'n'>' condition('o')~code
  return
::class logger
::method request
  say 'request' arg(1)
  return 'plain'
```

```output
compound compound simple abc my rc
request String
simple abc abc
<ABSENT> <STEM.> <STEM.A.B>
<> 93.900
<1x> 93.900
<.x> 93.900
<a b> 93.900
<a-b> 93.900
< > 93.900
<3> 93.900
<a+> 93.900
<.nil> 93.900
<12> 93.900
<The NIL object> 93.900
<an Object> 93.900
compound !X?_ X..Y
```

r[class.commandcontext.getvar]
`getVar(name)` and `[name]` answer the value of the variable, as the program
would read it: a simple variable's value, the value of a compound variable with
the tail resolved, and for a stem its default value. A read never raises
`NOVALUE`: a variable that has no value answers its name in upper case, with the
tail of a compound name resolved (`ABSENT`, `STEM.A`), the text that a program
sees for it, and [`hasVar`](r:class.commandcontext.hasvar) tells that answer
from the value of a variable that holds the same text. The two methods are the
same method, and `name` is required: `SYNTAX 93.901` for `getVar` and for `[]`
without it, and `SYNTAX 93.902` for more arguments. The value is the object that
the variable holds, not a string made of it: the same object each time, so that
a change the handler makes to it, such as an item appended to an array, is a
change to the variable.

```rexx
x = .commandhandler~register('spec_cc_getvar', .getter~new)
a = 'one'; arr = .array~of(1, 2); s. = 'stem default'; s.2 = 'two'; k = 2
nilvar = .nil
address spec_cc_getvar 'go'
say arr~items
x = .commandhandler~deregister('spec_cc_getvar')
::class getter inherit CommandHandler
::method command
  use arg command, caller
  say caller~getVar('a') caller['A'] caller~getVar('arr')~class~id,
    (caller['arr'] == caller['arr'])
  caller['arr']~append(3)
  say caller['s.'] caller['s.2'] caller['s.k'] caller['s.3'],
    caller['nosuch.1']
  say caller['nilvar']~isNil caller['nothing'] caller['nothing.k']
  do x over .array~of("caller~getVar", "caller~'[]'",,
      "caller~getVar('a', 'b')", "caller~'[]'('a', 'b')")
    self~try(caller, x)
  end
  return 0
::method try
  use arg caller, x
  signal on syntax name bad
  interpret 'y =' x
  say x 'no error'
  return
bad: say x condition('o')~code
  return
```

```output
one one Array 1
stem default two two stem default NOSUCH.1
1 NOTHING NOTHING.2
caller~getVar 93.901
caller~'[]' 93.901
caller~getVar('a', 'b') 93.902
caller~'[]'('a', 'b') 93.902
3
```

r[class.commandcontext.hasvar]
`hasVar(name)` answers `1` when the variable has a value in the code that sent
the command, and `0` when it has none. A variable that was never set, or was
dropped, has none; a compound variable has one when it was set or when its
stem has a default value; the name of a stem has a value whatever the stem
holds. A variable that holds `.nil` has a value. `name` is required
(`SYNTAX 93.901`) and one more argument raises `SYNTAX 93.902`. The name is
read as [the name rule](r:class.commandcontext.name) says.

```rexx
x = .commandhandler~register('spec_cc_hasvar', .asker~new)
a = 'one'; empty = ''; nilvar = .nil; k = 7
d. = 'default'; e.1 = 'one'
gone = 'will go'; drop gone
address spec_cc_hasvar 'go'
x = .commandhandler~deregister('spec_cc_hasvar')
::class asker inherit CommandHandler
::method command
  use arg command, caller
  say caller~hasVar('a') caller~hasVar('A') caller~hasVar('empty'),
    caller~hasVar('nilvar')
  say caller~hasVar('nosuch') caller~hasVar('gone') caller~hasVar('rc'),
    caller~hasVar('result')
  say caller~hasVar('d.') caller~hasVar('d.5') caller~hasVar('e.'),
    caller~hasVar('e.1') caller~hasVar('e.2') caller~hasVar('e.k')
  say caller~hasVar('unset.') caller~hasVar('unset.1')
  do x over .array~of("caller~hasVar", "caller~hasVar('a', 'b')",,
      "caller~hasVar('a b')")
    self~try(caller, x)
  end
  return 0
::method try
  use arg caller, x
  signal on syntax name bad
  interpret 'y =' x
  say x 'no error'
  return
bad: say x condition('o')~code
  return
```

```output
1 1 1 1
0 0 0 0
1 1 1 1 0 0
1 0
caller~hasVar 93.901
caller~hasVar('a', 'b') 93.902
caller~hasVar('a b') 93.900
```

r[class.commandcontext.setvar]
`[]=(value, name)` and `setVar(name, value)` store `value` in the variable
`name` of the code that sent the command, under the rules of an assignment
there: a simple variable gets the value, a compound variable is set with its
tail resolved ([the compound rule](r:instr.assignment.compound)), and a stem
gets the value as the default of every element, and all the elements it had are
gone ([the stem rule](r:instr.assignment.stem)). The value is stored as the
object it is: a string, `.nil` or any other object, not a string made from it.
The two methods are not two spellings of one: **the order of the arguments
differs**, the value first for the bracket form, which is what the assignment
`c[name] = value` sends, and the name first for `setVar`. Both need two
arguments (`SYNTAX 93.901`), and a third raises `SYNTAX 93.902`. The name is
read as [the name rule](r:class.commandcontext.name) says, and the value is not
read at all. Both answer nothing, so that using the answer as a value raises
`SYNTAX 91.999`. A name that is no variable name raises `SYNTAX 93.900`, and
nothing is assigned. The program sees the new values when the command is over,
and the handler at once.

```rexx
x = .commandhandler~register('spec_cc_setvar', .setter~new)
i = 4; old. = 'old'; old.1 = 'one'; old.2 = 'two'
address spec_cc_setvar 'go'
say a '|' b '|' c.4 '|' old. old.1 old.9
say s. s.5 '|' nilvar~isNil '|' obj~class~id
x = .commandhandler~deregister('spec_cc_setvar')
::class setter inherit CommandHandler
::method command
  use arg command, caller
  caller~setVar('a', 'by setVar')
  caller~'[]='('by bracket', 'b')
  caller~setVar('c.i', 'compound')
  caller['old.'] = 'cleared'
  caller~setVar('s.', 'stem default')
  caller~setVar('nilvar', .nil)
  caller~setVar('obj', .array~new)
  say 'in the handler:' caller['a'] caller['b'] caller['old.1']
  do x over .array~of("caller~setVar('a')", "caller~'[]='('a')",,
      "caller~setVar", "caller~setVar('a', 1, 2)", "caller~setVar('1x', 1)",,
      "caller~'[]='(1, '1x')", "caller~setVar('', 1)",,
      "x = caller~setVar('zz', 1)", "x = caller~'[]='(1, 'zz')")
    self~try(caller, x)
  end
  return 0
::method try
  use arg caller, x
  signal on syntax name bad
  interpret x
  say x 'no error'
  return
bad: say x condition('o')~code
  return
```

```output
in the handler: by setVar by bracket cleared
caller~setVar('a') 93.901
caller~'[]='('a') 93.901
caller~setVar 93.901
caller~setVar('a', 1, 2) 93.902
caller~setVar('1x', 1) 93.900
caller~'[]='(1, '1x') 93.900
caller~setVar('', 1) 93.900
x = caller~setVar('zz', 1) 91.999
x = caller~'[]='(1, 'zz') 91.999
by setVar | by bracket | compound | cleared cleared cleared
stem default stem default | 1 | Array
```

r[class.commandcontext.dropvar]
`dropVar(name)` drops the variable `name` of the code that sent the command,
under the rules of the DROP instruction: a simple variable has no value
afterwards, a compound variable has its derived name as value, even when its
stem has a default value ([the compound rule](r:instr.drop.compound)), and a
stem is dropped with all its elements ([the stem rule](r:instr.drop.stem)).
Dropping a variable that has no value is not an error. `name` is required
(`SYNTAX 93.901`) and one more argument raises `SYNTAX 93.902`; it is read as
[the name rule](r:class.commandcontext.name) says. The method answers nothing,
so using its answer as a value raises `SYNTAX 91.999`.

```rexx
x = .commandhandler~register('spec_cc_dropvar', .dropper~new)
a = 'one'; b = 'two'; k = 1
s. = 'default'; s.1 = 'one'; s.2 = 'two'
t. = 'tdef'; t.1 = 't1'
address spec_cc_dropvar 'go'
say symbol('a') symbol('b') symbol('k') symbol('s.1') s.1 s.2 symbol('t.'),
  t.1 t.9
x = .commandhandler~deregister('spec_cc_dropvar')
::class dropper inherit CommandHandler
::method command
  use arg command, caller
  caller~dropVar('a')
  caller~dropVar('nosuch')
  caller~dropVar('s.k')
  caller~dropVar('t.')
  say caller~hasVar('a') caller['a'] caller~hasVar('s.1') caller['s.1'],
    caller['s.2'] caller~hasVar('t.1')
  do x over .array~of("caller~dropVar", "caller~dropVar('a', 'b')",,
      "caller~dropVar('1x')", "x = caller~dropVar('b')")
    self~try(caller, x)
  end
  return 0
::method try
  use arg caller, x
  signal on syntax name bad
  interpret 'y =' x
  say x 'no error'
  return
bad: say x condition('o')~code
  return
```

```output
0 A 0 S.1 two 0
caller~dropVar 93.901
caller~dropVar('a', 'b') 93.902
caller~dropVar('1x') 93.900
x = caller~dropVar('b') 91.999
LIT LIT VAR LIT S.1 two VAR T.1 T.9
```

r[class.commandcontext.variables]
`variables` answers a new `Directory` of the variables of the code that sent
the command, in the form that
[`.context~variables`](r:class.rexxcontext.variables) answers for a piece of
code: each simple variable that has a value under its name in upper case with
its value, and each stem under its name in upper case with a period at the
end, with the stem object as its value; a compound variable is reached through
its stem. The directory is a snapshot: a change to it assigns nothing, and a
variable that the handler sets later is not in it, though a stem in it is the
stem of the code. The method takes no argument (`SYNTAX 88.922`).

```rexx
x = .commandhandler~register('spec_cc_variables', .lister~new)
plain = 'p'; num = 12; nothing = .nil; s. = 'dflt'; s.1 = 'one'; k = 1
gone = 'x'; drop gone
address spec_cc_variables 'go'
say s.7 late symbol('added')
x = .commandhandler~deregister('spec_cc_variables')
::class lister inherit CommandHandler
::method command
  use arg command, caller
  v = caller~variables
  say v~class~id v~hasIndex('PLAIN') v~hasIndex('plain') v~hasIndex('GONE'),
    v~hasIndex('S.1') v~hasIndex('UNSET')
  say v['PLAIN'] v['NUM'] v['NOTHING']~isNil v['S.']~class~id v['S.']['1'],
    v['S.']['7']
  caller['late'] = 'set later'
  say v~hasIndex('LATE') (v == caller~variables)
  v['ADDED'] = 'put in the snapshot'
  say caller~hasVar('added')
  v['S.']['7'] = 'seen by the program'
  signal on syntax name bad
  x = caller~variables(1)
bad: say condition('o')~code
  return 0
```

```output
Directory 1 0 0 0 0
p 12 1 Stem one dflt
0 0
0
88.922
seen by the program set later LIT
```

## Redirection

r[class.commandcontext.isredirected]
`isRedirected(which)` answers `1` when the command was sent with the
redirection that the first letter of `which` names, and `0` when it was not.
The letter is `I` for `WITH INPUT`, `O` for `WITH OUTPUT`, `E` for
`WITH ERROR` and `S` for output and error that go to the same target, and it
may be in either case; only the first letter is read, so `Input`, `Ixyz` and
`I` are the same. A redirection that names a stem, a stream or an object to
[`USING`](r:instr.address.with.using) is a redirection, and `NORMAL` is none.
The target is the same when both redirections name the same stem, the same
stream, or the same object, whether either appends or replaces. `S` is `0`
when only one of them is redirected. `which` is required (`SYNTAX 88.901`) and
one more argument raises `SYNTAX 88.922`. A `which` that starts with another
letter or has none, the null string and a number among them, raises
`SYNTAX 93.900`. An object that is not a string and answers no string to
`request` with `STRING`, `.nil` and an `Object` among them, raises
`SYNTAX 88.909`.

```rexx
x = .commandhandler~register('spec_cc_redir', .asker~new)
a = .array~new; b = .array~new
f = 'spec_cc_redir.tmp'
address spec_cc_redir 'none'
address spec_cc_redir 'normal' with input normal output normal error normal
address spec_cc_redir 'input' with input using (a)
address spec_cc_redir 'output' with output stem o.
address spec_cc_redir 'error' with error stem e.
address spec_cc_redir 'both' with output stem o. error stem e.
address spec_cc_redir 'same stem' with output stem o. error stem o.
address spec_cc_redir 'same stem, append',
  with output append stem o. error replace stem o.
address spec_cc_redir 'same stream' with output stream (f) error stream (f)
address spec_cc_redir 'same object' with output using (a) error using (a)
address spec_cc_redir 'two objects' with output using (a) error using (b)
address spec_cc_redir 'all' with input using (a) output stem o. error stem e.
x = .commandhandler~deregister('spec_cc_redir')
x = .file~new(f)~delete
::class asker inherit CommandHandler
::method command
  use arg command, caller
  say command~left(18) 'I' caller~isRedirected('I'),
    'O' caller~isRedirected('O'),
    'E' caller~isRedirected('E') 'S' caller~isRedirected('S'),
    'words' caller~isRedirected('Input') caller~isRedirected('output'),
    caller~isRedirected('e') caller~isRedirected('Samex'),
    caller~isRedirected('Ixyz')
  if command = 'all' then do
    do x over .array~of("caller~isRedirected",,
        "caller~isRedirected('I', 1)", "caller~isRedirected('')",,
        "caller~isRedirected('X')", "caller~isRedirected(1)",,
        "caller~isRedirected(.mutablebuffer~new('output'))")
      self~try(caller, x)
    end
  end
  return 0
::method try
  use arg caller, x
  signal on syntax name bad
  interpret 'y =' x
  say x '=>' y
  return
bad: say x '=>' condition('o')~code
  return
```

```output
none               I 0 O 0 E 0 S 0 words 0 0 0 0 0
normal             I 0 O 0 E 0 S 0 words 0 0 0 0 0
input              I 1 O 0 E 0 S 0 words 1 0 0 0 1
output             I 0 O 1 E 0 S 0 words 0 1 0 0 0
error              I 0 O 0 E 1 S 0 words 0 0 1 0 0
both               I 0 O 1 E 1 S 0 words 0 1 1 0 0
same stem          I 0 O 1 E 1 S 1 words 0 1 1 1 0
same stem, append  I 0 O 1 E 1 S 1 words 0 1 1 1 0
same stream        I 0 O 1 E 1 S 1 words 0 1 1 1 0
same object        I 0 O 1 E 1 S 1 words 0 1 1 1 0
two objects        I 0 O 1 E 1 S 0 words 0 1 1 0 0
all                I 1 O 1 E 1 S 0 words 1 1 1 0 1
caller~isRedirected => 88.901
caller~isRedirected('I', 1) => 88.922
caller~isRedirected('') => 93.900
caller~isRedirected('X') => 93.900
caller~isRedirected(1) => 93.900
caller~isRedirected(.mutablebuffer~new('output')) => 1
```

```rexx
x = .commandhandler~register('spec_cc_redir_obj', .asker~new)
address spec_cc_redir_obj 'go'
x = .commandhandler~deregister('spec_cc_redir_obj')
::class asker inherit CommandHandler
::method command
  use arg command, caller
  do x over .array~of("caller~isRedirected(.nil)",,
      "caller~isRedirected(.object~new)", "caller~isRedirected(.named~new)")
    self~try(caller, x)
  end
  return 0
::method try
  use arg caller, x
  signal on syntax name bad
  interpret 'y =' x
  say x '=>' y
  return
bad: say x '=>' condition('o')~code
  return
::class named
::method request
  return 'Input'
```

```output defect=#164
caller~isRedirected(.nil) => 88.909
caller~isRedirected(.object~new) => 88.909
caller~isRedirected(.named~new) => 0
```

r[class.commandcontext.input]
`input` answers the [`CommandInput`](commandinput.md) that reads the source of
`WITH INPUT`, the same object each time it is asked during the command. A
command that was sent without `WITH INPUT` has no such stream, and `input`
raises `SYNTAX 98.900`, which
[`isRedirected('Input')`](r:class.commandcontext.isredirected) tells beforehand.
Arguments to the method are ignored. The source is read line by line, as [the
page of the stream](commandinput.md) says; the example reads what `USING` gave.

```rexx
x = .commandhandler~register('spec_cc_input', .reader~new)
address spec_cc_input 'with input',
  with input using (.array~of('first', 'second'))
address spec_cc_input 'without input'
address spec_cc_input 'only output' with output stem o.
x = .commandhandler~deregister('spec_cc_input')
::class reader inherit CommandHandler
::method command
  use arg command, caller
  if caller~isRedirected('Input') then do
    in = caller~input
    say command ':' in~class~id (in == caller~input) in~linein in~linein,
      in~lines
  end
  else self~refused(caller, command)
  return 0
::method refused
  use arg caller, command
  signal on syntax name bad
  x = caller~input
bad: say command ':' condition('o')~code
  return
```

```output
with input : CommandInput 1 first second 0
without input : 98.900
only output : 98.900
```

r[class.commandcontext.output]
`output` answers the [`CommandOutput`](commandoutput.md) that writes to the
target of `WITH OUTPUT`, the same object each time it is asked during the
command. A command that was sent without `WITH OUTPUT` has no such stream, and
`output` raises `SYNTAX 98.900`, and a handler that is to write anyway uses
`SAY` or a stream of its own. Arguments to the method are ignored.

```rexx
x = .commandhandler~register('spec_cc_output', .writer~new)
address spec_cc_output 'a' with output stem o.
say o.0 o.1 o.2
address spec_cc_output 'b' with error stem e.
say e.0 symbol('o.3')
x = .commandhandler~deregister('spec_cc_output')
::class writer inherit CommandHandler
::method command
  use arg command, caller
  if command = 'a' then do
    out = caller~output
    out~lineout('from' command)
    say 'same object' (out == caller~output) out~class~id
    caller~output~lineout('second line')
  end
  else self~refused(caller, command)
  return 0
::method refused
  use arg caller, command
  signal on syntax name bad
  x = caller~output
bad: say command ':' condition('o')~code
  return
```

```output
same object 1 CommandOutput
2 from a second line
b : 98.900
0 LIT
```

r[class.commandcontext.error]
`error` answers the [`CommandOutput`](commandoutput.md) that writes to the
target of `WITH ERROR`, the same object each time it is asked during the
command, and raises `SYNTAX 98.900` for a command that was sent without
`WITH ERROR`. It is a different object from the one that `output` answers also
when both go to the same target; the lines that a handler writes through the
two go to the target in the order in which it wrote them. Arguments to the
method are ignored.

```rexx
x = .commandhandler~register('spec_cc_error', .writer~new)
address spec_cc_error 'a' with output stem both. error stem both.
say both.0 both.1 both.2 both.3
address spec_cc_error 'b' with output stem o.
x = .commandhandler~deregister('spec_cc_error')
::class writer inherit CommandHandler
::method command
  use arg command, caller
  if command = 'a' then do
    o = caller~output; e = caller~error
    say (o == e) (e == caller~error) e~class~id
    o~lineout('one'); e~lineout('two'); o~lineout('three')
  end
  else self~refused(caller, command)
  return 0
::method refused
  use arg caller, command
  signal on syntax name bad
  x = caller~error
bad: say command ':' condition('o')~code
  return
```

```output
0 1 CommandOutput
3 one two three
b : 98.900
```

## Raising conditions

r[class.commandcontext.raise]
`raise(condition, rc, description, additional)` makes the command raise a
condition. The condition is raised in the code that sent the command, when the
handler's method `command` returns, as if the command itself had raised it: a
`SIGNAL ON` or `CALL ON` trap for it answers it as for any condition, though a
CALL ON ANY trap does not see the conditions that only SIGNAL can trap ([the
CALL ON rule](r:instr.call.on)); the handler goes on running after the call. A
condition that no trap answers is treated as [the page of that
condition](../conditions/index.md) says. The method answers `rc`. What
`command` answers is not used when a condition was raised: `RC` holds `rc` as
it is, whatever object it is, the number or the text, `.nil` too, so a handler
may return the answer of the call or anything else. A HALT is delayed like any
HALT ([the HALT rule](r:cond.halt.raise)), so the clause after the command is
run before a trap for it. `condition` is a string: `ERROR`, `FAILURE`, `HALT`,
`LOSTDIGITS`, `NOMETHOD`, `NOSTRING`, `NOTREADY`, `NOVALUE`, or `USER`
followed by a name of one word, in either case; `SYNTAX` is the other form,
which [the next rule](r:class.commandcontext.raise.syntax) states. The string
is read as words separated by [whitespace](r:conv.string.whitespace), so the
blanks and tabs around and between the words play no part, and the name after
`USER` may be any word; the condition is named in upper case (`USER 1X`). A
command that raises ERROR or FAILURE has the status `.RS` of the [status
rule](r:instr.command.status). `description` is what `CONDITION('D')` answers
and the `DESCRIPTION` entry of the condition object holds; when it is omitted
or `.nil`, it is the command, and the null string is a description like
another. Another object is replaced by its string value as [the conversion
rule](r:conv.string.object) finds it: `request` with `STRING` is sent to it
first, and its `string` method when that answers no string, so that a class
with its own `request` decides the description. The condition holds that
string; an instance of a subclass of `String` is a string here ([the deviation
rule](r:class.commandcontext.deviation.109)). `additional` is the `ADDITIONAL`
entry of the condition object, the object as it was given, and when it is
omitted or `.nil` the condition object has no such entry. The condition object
has `RC`, holding the `rc` object. `condition` and `rc` are required: `SYNTAX
93.901` when they are fewer than two, `SYNTAX 93.903` when an omitted place
stands for one of them, and `SYNTAX 93.902` for more than four arguments. A
`condition` that is none of the forms above, `ANY`, `NONE`, `PROPAGATE`,
`USER` with no name or with more than one, any other text, the null string and
`.nil` among them, raises `SYNTAX 93.900`. A call that raises an error does
not use up the command's one condition, which [the second
rule](r:class.commandcontext.raise.once) says.

```rexx
x = .commandhandler~register('spec_cc_raise', .raiser~new)
call on error name trap
call on failure name trap
call on notready name trap
call on user spec name trap
do word over 'ERROR FAILURE NOTREADY USER DEFAULTS NILS'~makearray(' ')
  address spec_cc_raise word
  say word 'returned, rc' '<'rc'>' .rs
end
x = .commandhandler~deregister('spec_cc_raise')
exit
trap:
  o = condition('o')
  if o~hasIndex('ADDITIONAL') then add = o~additional~class~id
  else add = 'none'
  say '  trap' condition('c') '<'condition('d')'>' 'rc' rc 'entry' o~rc add
  return
::class raiser inherit CommandHandler
::method command
  use arg command, caller
  select
    when command = 'ERROR' then return,
      caller~raise('ERROR', 5, 'about error', 'more')
    when command = 'FAILURE' then return caller~raise('failure', 'six')
    when command = 'NOTREADY' then return caller~raise('NotReady', 7.5, '')
    when command = 'USER' then return,
      caller~raise('USER Spec', 8, 'user text', .array~of('x'))
    when command = 'DEFAULTS' then return caller~raise('ERROR', 9)
    when command = 'NILS' then return caller~raise('ERROR', 10, .nil, .nil)
  end
```

```output
  trap ERROR <about error> rc 5 entry 5 String
ERROR returned, rc <5> 1
  trap FAILURE <FAILURE> rc six entry six none
FAILURE returned, rc <six> -1
  trap NOTREADY <> rc 7.5 entry 7.5 none
NOTREADY returned, rc <7.5> 0
  trap USER SPEC <user text> rc 8 entry 8 Array
USER returned, rc <8> 0
  trap ERROR <DEFAULTS> rc 9 entry 9 none
DEFAULTS returned, rc <9> 1
  trap ERROR <NILS> rc 10 entry 10 none
NILS returned, rc <10> 1
```

```rexx
x = .commandhandler~register('spec_cc_halt', .halter~new)
call on halt name stopped
address spec_cc_halt 'go'
say 'clause after the command, rc' rc
say 'one more clause'
x = .commandhandler~deregister('spec_cc_halt')
exit
stopped: say 'halt trap' condition('c') '<'condition('d')'>' rc
  return
::class halter inherit CommandHandler
::method command
  use arg command, caller
  return caller~raise('Halt', 7, 'asked to stop')
```

```output
clause after the command, rc 7
halt trap HALT <asked to stop> 7
one more clause
```

```rexx
x = .commandhandler~register('spec_cc_raise_args', .tries~new)
address spec_cc_raise_args 'go'
x = .commandhandler~deregister('spec_cc_raise_args')
::class tries inherit CommandHandler
::method command
  use arg command, caller
  do x over .array~of("raise", "raise('ERROR')", "raise(, 1)",,
      "raise('ERROR', , 'text')", "raise('ERROR', 1, 'd', 'a', 'x')",,
      "raise('FOO', 1)", "raise('ANY', 1)", "raise('NONE', 1)",,
      "raise('PROPAGATE', 1)", "raise('USER', 1)", "raise('USER a b', 1)",,
      "raise('', 1)", "raise(.nil, 1)", "raise('SYNTAX', 98.900, 1, 2)",,
      "raiseError", "raiseFailure", "raiseError(1, 2)")
    self~try(caller, x)
  end
  return 0
::method try
  use arg caller, x
  signal on syntax name bad
  interpret 'y = caller~' || x
  say x '=> no error'
  return
bad: say x condition('o')~code
  return
```

```output
raise 93.901
raise('ERROR') 93.901
raise(, 1) 93.903
raise('ERROR', , 'text') 93.903
raise('ERROR', 1, 'd', 'a', 'x') 93.902
raise('FOO', 1) 93.900
raise('ANY', 1) 93.900
raise('NONE', 1) 93.900
raise('PROPAGATE', 1) 93.900
raise('USER', 1) 93.900
raise('USER a b', 1) 93.900
raise('', 1) 93.900
raise(.nil, 1) 93.900
raise('SYNTAX', 98.900, 1, 2) 93.900
raiseError 93.901
raiseFailure 93.901
raiseError(1, 2) 93.902
```

```rexx
x = .commandhandler~register('spec_cc_words', .t~new)
call on any name trap
do w over .array~of(' ERROR', 'USER  two', 'user x', 'USER 1x', 'USER a.b',,
    'USER error', 'USER a-b', 'USER' || '09'x || 'tab')
  .environment~w = w
  address spec_cc_words 'go'
end
x = .commandhandler~deregister('spec_cc_words')
exit
trap: say '  trapped' condition('c')
  return
::class t inherit CommandHandler
::method command
  use arg command, caller
  w = .environment~w
  say '['changestr('09'x, w, '<tab>')']' caller~raise(w, 1)
  return 0
```

```output
[ ERROR] 1
  trapped ERROR
[USER  two] 1
  trapped USER TWO
[user x] 1
  trapped USER X
[USER 1x] 1
  trapped USER 1X
[USER a.b] 1
  trapped USER A.B
[USER error] 1
  trapped USER ERROR
[USER a-b] 1
  trapped USER A-B
[USER<tab>tab] 1
  trapped USER TAB
```

```rexx
x = .commandhandler~register('spec_cc_desc', .raiser~new)
call on error name trap
address spec_cc_desc 'a'
address spec_cc_desc 'b'
address spec_cc_desc 'c'
address spec_cc_desc 'd'
x = .commandhandler~deregister('spec_cc_desc')
exit
trap:
  say '  trap <'condition('d')'>' condition('o')~description~class~id
  return
::class raiser inherit CommandHandler
::method command
  use arg command, caller
  if command = 'a' then return caller~raise('ERROR', 1, .lg~new)
  if command = 'b' then return caller~raise('ERROR', 1, .ms~new)
  if command = 'c' then
    return caller~raise('ERROR', 1, .mutablebuffer~new('mb'))
  return caller~raise('ERROR', 1, .rq~new)
::class lg
::method string
  say '  lg string'
  return 'lgtext'
::class ms
::method makestring
  say '  ms makestring'
  return 'mstext'
::class rq
::method request
  say '  rq request' arg(1)
  return 'rqtext'
::method makestring
  say '  rq makestring'
  return 'mstext'
```

```output defect=#184
  lg string
  trap <lgtext> String
  ms makestring
  trap <mstext> String
  trap <mb> String
  rq request STRING
  trap <rqtext> String
```

```rexx
x = .commandhandler~register('spec_cc_any', .h~new)
call on any name trap
address spec_cc_any 'NOVALUE'
say 'went on' rc
x = .commandhandler~deregister('spec_cc_any')
exit
trap: say 'trap' condition('c'); return
::class h inherit CommandHandler
::method command
  use arg cmd, ctx
  r = ctx~raise(cmd, 2, 'nv')
  return 0
```

```output defect=#203
went on 2
```

r[class.commandcontext.raise.syntax]
`raise('SYNTAX', code, additional)` makes the command raise a `SYNTAX` error
instead of the conditions of [the first form](r:class.commandcontext.raise), and
it takes at most three arguments, so that a fourth makes the call the first
form, which refuses `SYNTAX` with `SYNTAX 93.900`. The error is raised in the
code that sent the command, at the clause that sent it, when `command` returns,
as an error of a built-in function would be: the error number is `code`, the
clause is the one that holds the position of the error, and no frame of the
handler is among the frames of the error. The handler goes on running after the
call, the method answers `code` as it was given, and what `command` answers is
ignored. A `SYNTAX` trap in the code that sent the command answers the error as
for any `SYNTAX` condition. `code` is the number of the error as [a
`RAISE SYNTAX`](r:instr.raise.number) reads it, a number such as `40.3` whose
digits after the point are one whole number, so `40.003` and `40.03` are the
error `40.3`, with these differences: the point and a subcode from 1 to 999 are
required, and the major number is from 1 to 99. A `code` of any other form, `98`
and `40.0` and `100.1` and `40.1000` and `x` among them, raises `SYNTAX 93.900`
in the handler and raises nothing in the command. A well formed `code` for which
no error has a message raises `SYNTAX 98.941` at the clause. `additional` is for
the message of the error: an `Array` gives the values for its places in order,
and any other object is the one value, so that the `ADDITIONAL` entry of the
condition is always an `Array`, empty when `additional` is omitted or `.nil`.
The text of the message is not specified. `code` is required (`SYNTAX 93.901`).

```rexx
x = .commandhandler~register('spec_cc_syntax', .failer~new)
do code over ('40.3 40.003 040.03 99.999 5.5 98.900 98 40.0 40.1000',
    '100.1 x')~makeArray(' ')
  call try code
end
x = .commandhandler~deregister('spec_cc_syntax')
exit
try: procedure
  signal on syntax name raised
  address spec_cc_syntax arg(1)
  say arg(1)~left(8) 'went through, rc' rc
  return
raised:
  o = condition('o')
  say arg(1)~left(8) 'raised' o~code o~rc 'at line' sigl,
    'handler in the traceback:',
    (o~traceback~makeArray~makeString('l', '|')~pos('caller~raise') > 0)
  return
::class failer inherit CommandHandler
::method command
  use arg command, caller
  signal on syntax name refused
  x = caller~raise('SYNTAX', command, .array~of('a', 'b'))
  say '  after raise, the handler goes on:' x
  return 'ignored'
refused:
  say '  refused' command~left(7) condition('o')~code
  return 0
```

```output
  after raise, the handler goes on: 40.3
40.3     raised 40.3 40 at line 10 handler in the traceback: 0
  after raise, the handler goes on: 40.003
40.003   raised 40.3 40 at line 10 handler in the traceback: 0
  after raise, the handler goes on: 040.03
040.03   raised 40.3 40 at line 10 handler in the traceback: 0
  after raise, the handler goes on: 99.999
99.999   raised 98.941 98 at line 10 handler in the traceback: 0
  after raise, the handler goes on: 5.5
5.5      raised 98.941 98 at line 10 handler in the traceback: 0
  after raise, the handler goes on: 98.900
98.900   raised 98.900 98 at line 10 handler in the traceback: 0
  refused 98      93.900
98       went through, rc 0
  refused 40.0    93.900
40.0     went through, rc 0
  refused 40.1000 93.900
40.1000  went through, rc 0
  refused 100.1   93.900
100.1    went through, rc 0
  refused x       93.900
x        went through, rc 0
```

```rexx
x = .commandhandler~register('spec_cc_addl', .w~new)
do w over 'text none nil array two'~makeArray(' ')
  call try w
end
x = .commandhandler~deregister('spec_cc_addl')
exit
try: procedure
  signal on syntax name raised
  address spec_cc_addl arg(1)
  return
raised:
  a = condition('o')~additional
  say arg(1)~left(5) condition('o')~code a~class~id a~items,
    a~makeString('l', '|')
  return
::class w inherit CommandHandler
::method command
  use arg command, caller
  select
    when command = 'text' then x = caller~raise('SYNTAX', 98.900, 'some text')
    when command = 'none' then x = caller~raise('SYNTAX', 98.900)
    when command = 'nil' then x = caller~raise('SYNTAX', 98.900, .nil)
    when command = 'array' then x = caller~raise('SYNTAX', 98.900,,
      .array~of('in array'))
    when command = 'two' then x = caller~raise('SYNTAX', 98.900,,
      .array~of('a', 'b'))
  end
  return 0
```

```output
text  98.900 Array 1 some text
none  98.900 Array 0 
nil   98.900 Array 0 
array 98.900 Array 1 in array
two   98.900 Array 2 a|b
```

r[class.commandcontext.raise.once]
A command raises at most one condition. After a call of `raise`, `raiseError`
or `raiseFailure` that was accepted, in either form of `raise`, another call
of any of them for the same command raises `SYNTAX 93.900`, and the first
condition stays as it was asked for. A call that raised an error, because its
arguments were wrong or its condition unknown, asked for nothing, and a good
call may follow it. The condition is raised in the code that sent the command
only when the handler returns, so the handler sees no trap run in between.

```rexx
x = .commandhandler~register('spec_cc_once', .once~new)
call on error name trap
address spec_cc_once 'go'
say 'rc' rc .rs
x = .commandhandler~deregister('spec_cc_once')
exit
trap: say 'trap' condition('c') '<'condition('d')'>' rc
  return
::class once inherit CommandHandler
::method command
  use arg command, caller
  self~try(caller, "raise('FOO', 3)")
  self~try(caller, "raise('ERROR')")
  self~try(caller, "raise('ERROR', 4, 'the first')")
  self~try(caller, "raise('ERROR', 5, 'the second')")
  self~try(caller, "raiseError(6)")
  self~try(caller, "raiseFailure(7)")
  self~try(caller, "raise('SYNTAX', 98.900)")
  say 'handler ends'
  return 0
::method try
  use arg caller, x
  signal on syntax name bad
  interpret 'y = caller~' || x
  say x '=> answered' y
  return
bad: say x '=>' condition('o')~code
  return
```

```output
raise('FOO', 3) => 93.900
raise('ERROR') => 93.901
raise('ERROR', 4, 'the first') => answered 4
raise('ERROR', 5, 'the second') => 93.900
raiseError(6) => 93.900
raiseFailure(7) => 93.900
raise('SYNTAX', 98.900) => 93.900
handler ends
trap ERROR <the first> 4
rc 4 1
```

r[class.commandcontext.raiseerror]
`raiseError(rc)` is `raise('ERROR', rc)`: the command raises the condition
ERROR with the return code `rc`, answers `rc`, and its description is the
command. `rc` is required (`SYNTAX 93.901`) and one more argument raises
`SYNTAX 93.902`. It does what [the first form of
`raise`](r:class.commandcontext.raise) does for these two arguments, and [the
once rule](r:class.commandcontext.raise.once) holds for it. A program cannot
add methods to a context or to its class, so no program can see which message
does it.

```rexx
x = .commandhandler~register('spec_cc_raiseerror', .errors~new)
signal on error name trap
address spec_cc_raiseerror 'first command'
say 'not reached'
trap: say 'trap' condition('c') '<'condition('d')'>' rc .rs
x = .commandhandler~deregister('spec_cc_raiseerror')
::class errors inherit CommandHandler
::method command
  use arg command, caller
  return caller~raiseError(12)
```

```output
trap ERROR <first command> 12 1
```

r[class.commandcontext.raisefailure]
`raiseFailure(rc)` is `raise('FAILURE', rc)`: the command raises the condition
FAILURE with the return code `rc`, answers `rc`, and its description is the
command. `rc` is required (`SYNTAX 93.901`) and one more argument raises
`SYNTAX 93.902`. A command that raises FAILURE has the status `.RS` of `-1`
([the status rule](r:instr.command.status)). [The once
rule](r:class.commandcontext.raise.once) holds for it.

```rexx
x = .commandhandler~register('spec_cc_raisefailure', .failures~new)
signal on failure name trap
address spec_cc_raisefailure 'first command'
say 'not reached'
trap: say 'trap' condition('c') '<'condition('d')'>' rc .rs
x = .commandhandler~deregister('spec_cc_raisefailure')
::class failures inherit CommandHandler
::method command
  use arg command, caller
  return caller~raiseFailure(13)
```

```output
trap FAILURE <first command> 13 -1
```

## Copying

r[class.commandcontext.copy]
`copy` answers a new object that stands for the same command: it works with the
same variables and the same redirections as the original, and is valid while the
command runs, as the original is. A variable that one sets, the other reads. The
same holds for the streams that `input`, `output` and `error` answer: a copy of
a stream reads from the same source and writes to the same target, so that a
line that one reads is not read by the other. A copy keeps its own record of the
one condition. The record starts as the record of the object that was copied, as
it is at that time, so that a copy made after the original asked for a condition
refuses one with `SYNTAX 93.900`. From then on the two are apart: a copy answers
`raise`, `raiseError` and `raiseFailure` as the original does, refuses a second
one with `SYNTAX 93.900`, and does not count against the original, but the
condition that a copy asks for is never raised, and the command raises only what
the original asked for. When only a copy asked, what `command` answers is the
return code. The second example asks for a condition through the copy and
through the original in each order, through the copy twice, and through a copy
made after the original asked.

```rexx
x = .commandhandler~register('spec_cc_copy', .copier~new)
shared = 'program value'
address spec_cc_copy 'go' with input using (.array~of('i1', 'i2')),
  output stem o.
say 'after:' symbol('copied') copied
say o.0 o.1 o.2
x = .commandhandler~deregister('spec_cc_copy')
::class copier inherit CommandHandler
::method command
  use arg command, caller
  c = caller~copy
  say c~class~id (c == caller) c~isValid c~environment caller['shared'],
    c['shared']
  c['copied'] = 'by the copy'
  say 'original sees' caller['copied']
  in = caller~input; copyIn = in~copy
  say copyIn~class~id (copyIn == in) copyIn~linein in~linein in~lines
  out = caller~output; copyOut = out~copy
  copyOut~lineout('through the copy')
  out~lineout('through the original')
  say copyOut~class~id (copyOut == out)
  return 0
```

```output
CommandContext 0 1 SPEC_CC_COPY program value program value
original sees by the copy
CommandInput 0 i1 i2 0
CommandOutput 0
after: VAR by the copy
2 through the copy through the original
```

```rexx
x = .commandhandler~register('spec_cc_copy_once', .twice~new)
call on error name trap
call on failure name trap
do w over 'copy-first original-first copy-twice copy-later'~makeArray(' ')
  address spec_cc_copy_once w
  say w': rc' rc .rs
end
x = .commandhandler~deregister('spec_cc_copy_once')
exit
trap: say '  trap' condition('c') '<'condition('d')'>' rc
  return
::class twice inherit CommandHandler
::method command
  use arg command, caller
  c = caller~copy
  if command = 'copy-first' then do; first = c; second = caller; end
  if command = 'original-first' then do; first = caller; second = c; end
  if command = 'copy-twice' then do; first = c; second = c; end
  if command = 'copy-later' then do; first = caller; second = .nil; end
  say '  first' first~raiseError(1)
  if second == .nil then second = caller~copy
  self~try(second)
  return 0
::method try
  use arg o
  signal on syntax name bad
  say '  second' o~raiseFailure(2)
  return
bad: say '  second refused' condition('o')~code
```

```output
  first 1
  second 2
  trap FAILURE <copy-first> 2
copy-first: rc 2 -1
  first 1
  second 2
  trap ERROR <original-first> 1
original-first: rc 1 1
  first 1
  second refused 93.900
copy-twice: rc 0 0
  first 1
  second refused 93.900
  trap ERROR <copy-later> 1
copy-later: rc 1 1
```

## Deviations

r[class.commandcontext.deviation.109]
A `name` of `[]`, `getVar`, `[]=`, `setVar`, `hasVar` or `dropVar`, the `which`
of `isRedirected` and the `description` of `raise`, that is an instance of a
subclass of `String` is read by its own characters, and `request`, `makeString`
and `string` are not sent to it ([Deviation 109](r:class.string.deviation.109)).
ooRexx sends such an instance `request`, with `String` for a variable name and
with `STRING` for `which` and `description`, so an override can change what is
read. In the examples the overrides would change what is read; they count their
sends, and the string is read as it is without one.

```rexx
x = .commandhandler~register('spec_cc_dev109', .reader~new)
a = 'avalue'
.environment~sent = 0
address spec_cc_dev109 'go'
say a
x = .commandhandler~deregister('spec_cc_dev109')
::class reader inherit CommandHandler
::method command
  use arg command, caller
  n = .digits~new('a')
  say caller[n] caller~hasVar(n) .environment~sent
  caller[n] = 'changed'
  say caller['a'] caller~hasVar('other') .environment~sent
  return 0
::class digits subclass string
::method makestring
  .environment~sent = .environment~sent + 1
  return 'other'
::method request
  .environment~sent = .environment~sent + 1
  return 'other'
```

```output defect=#140
avalue 1 0
changed 0 0
changed
```

```rexx
x = .commandhandler~register('spec_cc_dev109_redir', .asker~new)
.environment~sent = 0
address spec_cc_dev109_redir 'go' with output stem o.
x = .commandhandler~deregister('spec_cc_dev109_redir')
::class asker inherit CommandHandler
::method command
  use arg command, caller
  w = .digits~new('Output')
  say caller~isRedirected(w) .environment~sent
  return 0
::class digits subclass string
::method makestring
  .environment~sent = .environment~sent + 1
  return 'Error'
::method request
  .environment~sent = .environment~sent + 1
  return 'Error'
```

```output
1 0
```

```rexx
x = .commandhandler~register('spec_cc_dev109_desc', .raiser~new)
.environment~sent = 0
call on error name trap
address spec_cc_dev109_desc 'go'
x = .commandhandler~deregister('spec_cc_dev109_desc')
exit
trap: say condition('d') .environment~sent
  return
::class raiser inherit CommandHandler
::method command
  use arg command, caller
  return caller~raise('ERROR', 1, .digits~new('own'))
::class digits subclass string
::method makestring
  .environment~sent = .environment~sent + 1
  return 'other'
::method request
  .environment~sent = .environment~sent + 1
  return 'other'
::method string
  .environment~sent = .environment~sent + 1
  return 'other'
```

```output
own 0
```
