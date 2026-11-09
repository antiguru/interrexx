---
element: "Error numbers"
kind: concept
ansi: "8.2.1, 8.4, 8.4.1"
rexxref: "Conditions and Condition Traps > Condition Information > The Special Variable RC; Error Numbers and Messages"
---

# Error numbers

Every fault that raises the SYNTAX condition has an error number. The number
says what kind of fault it is, so that a program can answer a kind of error
without reading a message. This page specifies how a number is built, what a
program sees of it, and what each of the numbers means. It does not list the
subcodes: the pages that raise an error name its subcode, and the texts of the
messages are not specified, as [the error rule](r:conv.error.name) of the
conventions says. The numbers run from 3 to 99, and the numbers in that range
that name no kind of fault are the ones that
[ERRORTEXT](../builtins/errortext.md) answers with the null string, as
[its rule](r:bif.errortext.undefined) lists them.

## How a number is built

r[cond.errors.scheme]
An error number has two parts, a major number and a subcode written after a
point without leading zeros: `40.3`, `93.924`. How a trap reads the two parts,
and why a program tells faults apart by the whole code, is the business of
[the code rule](r:cond.syntax.code); this page owns what the major numbers
mean. `SIGL` is set as [the SIGL rule](r:cond.information.sigl) says. An error
that no SIGNAL trap answers ends the program with the exit code `256` minus
the major number, as [the untrapped rule](r:cond.index.syntax) says, and the
subcode does not enter it. The error of a fault in the text that is read
before the program runs is no condition at all, as [the reading
rule](r:lex.index.read-first) says.

```rexx
rc = 'unset'
signal on syntax name e
say 'one'
x = 1 + 'a'
e: say condition('o')~code rc sigl
say word(translate(condition('o')~code, ' ', '.'), 1) == rc
```

```output
one
41.1 41 4
1
```

r[cond.errors.subcodes]
The subcodes of an error number are named on the pages of the clauses and
functions that raise it, in the rule that names the fault. This page names a
subcode only to show a kind of error.

```rexx
call try 'x = left("abc", -2)'
call try 'x = copies("ab", -2)'
call try 'x = d2c(-1)'
call try 'x = "abc"~substr(0)'
exit
try:
  signal on syntax name e
  interpret arg(1)
e: o = condition('o')
  say left(arg(1), 26) o~code o~rc
  return
```

```output
x = left("abc", -2)        93.923 93
x = copies("ab", -2)       93.906 93
x = d2c(-1)                93.927 93
x = "abc"~substr(0)        93.924 93
```

## The numbers

> Each entry gives the major number, a short label, constructs that raise it
> with the full code, and a rule of this book that raises it where one is
> written. The grouping is by the usual kind of fault, and a number may have
> subcodes of another kind. The examples run the listed constructs and print
> the code that the trap reads.

### Faults in the form of the text

> These faults are usually found in the text of a clause, and a clause that is
> a directive or part of a block can raise them too. A fault in the text of a
> program is found when the program is read, and in the text of an INTERPRET
> when it runs, as [the reading rule](r:cond.index.read) says.

r[cond.errors.form]
The numbers below usually name a fault in the form of the text, and some of
them also have subcodes that are raised by values when the clause runs, such
as 7.3 and 31.2.
* 6, unfinished literal or comment: `say 'a` 6.2, `/* open` 6.1;
  [rule](r:lex.index.read-first).
* 7, SELECT with no usable branch: `select; say 1; end` 7.2, `select; end`
  7.1, `select; when 0 then nop; end` 7.3; [rule](r:instr.select.syntax),
  [rule](r:instr.select.otherwise).
* 8, THEN or ELSE with no partner: `then say 1` 8.1, `else say 1` 8.2;
  [rule](r:instr.if.else).
* 9, WHEN or OTHERWISE outside SELECT: `when 1 then nop` 9.1, `otherwise nop`
  9.2; [rule](r:instr.select.otherwise).
* 10, END out of place: `end` 10.1, `do; end x` 10.3;
  [rule](r:instr.do.end).
* 13, foreign character: `say 1 #` 13.1, `x = 1 ^ 2` 13.1;
  [rule](r:lex.characters.set).
* 14, block left open: `do; say 1` 14.1, `if 1 then do; say 2` 14.1,
  `do 1 to 2 to 3` 14.5; [rule](r:instr.do.end).
* 15, malformed hex or binary literal: `say 'zz'x` 15.3, `say '12'b` 15.4,
  `say '1 2'x` 15.5; [rule](r:lex.literals.hex.digit).
* 18, IF with no THEN: `if 1 say 'a'` 18.1, `if 1; say 2` 18.1;
  [rule](r:instr.if.syntax).
* 19, missing operand: `call` 19.2, `signal` 19.4; [rule](r:instr.call.error).
* 20, wrong token in an operand: `drop a b 'c'` 20.901, `procedure expose 'a'`
  20.902, `parse var 'x'` 20.904, `numeric` 20.905, `drop ()` 20.906;
  [rule](r:instr.procedure.syntax).
* 21, tokens left over: `nop 1` 21.901, `call on error name lbl extra` 21.903;
  [rule](r:instr.call.on.error).
* 25, keyword not offered: `signal on foo` 25.3, `numeric form foo` 25.11,
  `parse foo x` 25.12, `numeric 1` 25.15; [rule](r:instr.numeric.form.error).
* 27, clashing DO parts: `do forever x` 27.901, `do i=1 to 2 until 1 while 1`
  27.1; [rule](r:instr.do.syntax).
* 30, over-long symbol: `x = 1` where `x` is a symbol of 251 characters 30.1;
  [rule](r:lex.symbols.length).
* 31, constant used as a variable: `1x = 2` 31.2, `drop 1a` 31.2,
  `parse value 'x' with .a` 31.3, `x = '1a'; drop (x)` 31.2;
  [rule](r:lex.symbols.constant).
* 35, expression that does not parse: `x = 1 +` 35.1, `x = * 2` 35.1, `say ()`
  35.1; [rule](r:expr.index.syntax).
* 36, bracket left open: `x = (1` 36.901, `x = a[1` 36.902;
  [rule](r:expr.index.syntax).
* 37, closer with no opener: `x = ]` 37.901, `x = (1]` 37.901, `x = 1)` 37.2;
  [rule](r:expr.index.syntax).
* 38, malformed template: `parse var x a 3 3 b b ) c` 38.1,
  `parse var x 'a' +` 38.901; [rule](r:parse.patterns.syntax).
* 46, malformed name in parentheses: `drop (x y` 46.1;
  [rule](r:instr.drop.syntax).
* 47, label in a text that cannot hold one: `x: say 1` 47.1;
  [rule](r:lex.clauses.label).

> The numbers 22 and 23 would apply where a string holds a byte that the
> character set cannot hold. Every string of this book may hold any of the
> 256 character codes, as [the character rule](r:conv.string.chars) says, so
> this book names no clause that raises them.

```rexx
call try "say 'a"
call try "/* open"
call try "select; say 1; end"
call try "select; end"
call try "select; when 0 then nop; end"
call try "then say 1"
call try "else say 1"
call try "when 1 then nop"
call try "otherwise nop"
call try "end"
call try "do; end x"
call try "say 1 #"
call try "x = 1 ^ 2"
call try "do; say 1"
call try "if 1 then do; say 2"
call try "do 1 to 2 to 3"
call try "say 'zz'x"
call try "say '12'b"
call try "say '1 2'x"
call try "if 1 say 'a'"
call try "if 1; say 2"
call try "call"
call try "signal"
call try "drop a b 'c'"
call try "procedure expose 'a'"
call try "parse var 'x'"
call try "numeric"
call try "drop ()"
call try "nop 1"
call try "call on error name lbl extra"
call try "signal on foo"
call try "numeric form foo"
call try "parse foo x"
call try "numeric 1"
call try "do forever x"
call try "do i=1 to 2 until 1 while 1"
call try copies('a', 251) '= 1', '(251 a) = 1'
call try "1x = 2"
call try "drop 1a"
call try "parse value 'x' with .a"
call try "x = '1a'; drop (x)"
call try "x = 1 +"
call try "x = * 2"
call try "say ()"
call try "x = (1"
call try "x = a[1"
call try "x = ]"
call try "x = (1]"
call try "x = 1)"
call try "parse var x a 3 3 b b ) c"
call try "parse var x 'a' +"
call try "drop (x y"
call try "x: say 1"
exit
try:
  signal on syntax name e
  interpret arg(1)
  say left(arg(1), 44) '=> none'
  return
e: label = arg(1)
  if arg(2, 'E') then label = arg(2)
  say left(label, 44) condition('o')~code
  return
```

```output
say 'a                                       6.2
/* open                                      6.1
select; say 1; end                           7.2
select; end                                  7.1
select; when 0 then nop; end                 7.3
then say 1                                   8.1
else say 1                                   8.2
when 1 then nop                              9.1
otherwise nop                                9.2
end                                          10.1
do; end x                                    10.3
say 1 #                                      13.1
x = 1 ^ 2                                    13.1
do; say 1                                    14.1
if 1 then do; say 2                          14.1
do 1 to 2 to 3                               14.5
say 'zz'x                                    15.3
say '12'b                                    15.4
say '1 2'x                                   15.5
if 1 say 'a'                                 18.1
if 1; say 2                                  18.1
call                                         19.2
signal                                       19.4
drop a b 'c'                                 20.901
procedure expose 'a'                         20.902
parse var 'x'                                20.904
numeric                                      20.905
drop ()                                      20.906
nop 1                                        21.901
call on error name lbl extra                 21.903
signal on foo                                25.3
numeric form foo                             25.11
parse foo x                                  25.12
numeric 1                                    25.15
do forever x                                 27.901
do i=1 to 2 until 1 while 1                  27.1
(251 a) = 1                                  30.1
1x = 2                                       31.2
drop 1a                                      31.2
parse value 'x' with .a                      31.3
x = '1a'; drop (x)                           31.2
x = 1 +                                      35.1
x = * 2                                      35.1
say ()                                       35.1
x = (1                                       36.901
x = a[1                                      36.902
x = ]                                        37.901
x = (1]                                      37.901
x = 1)                                       37.2
parse var x a 3 3 b b ) c                    38.1
parse var x 'a' +                            38.901
drop (x y                                    46.1
x: say 1                                     47.1
```

### Faults in the values and the flow

> These faults are found while a clause runs, because a value or the state
> of the program does not fit the clause.

r[cond.errors.run]
The numbers below usually name a fault that the values or the flow of control
cause, at the clause that meets it.
* 16, jump to a missing label: `signal nolabelhere` 16.1, `signal value 'zz'`
  16.1; [rule](r:instr.signal.label).
* 17, misplaced PROCEDURE: `procedure` 17.1; [rule](r:instr.procedure.first).
* 24, unknown TRACE setting: `trace x1` 24.1; [rule](r:instr.trace.invalid).
* 26, value that is no whole number: `numeric digits 'a'` 26.5,
  `numeric digits 0` 26.5, `numeric fuzz -1` 26.6, `x = 2 ** 1.5` 26.8;
  [rule](r:num.settings.digits).
* 28, LEAVE or ITERATE with no target: `leave` 28.1, `iterate` 28.2,
  `leave nosuchzz` 28.3, `iterate nosuchzz` 28.4;
  [rule](r:instr.leave.target).
* 29, over-long ADDRESS name: `address value copies('E', 1000)` 29.1.
* 33, FUZZ not below DIGITS: `numeric digits 5; numeric fuzz 5` 33.1;
  [rule](r:num.settings.fuzz).
* 34, operand that is no truth value: `if 'a' then say 1` 34.1, `x = 1 & 'a'`
  34.901, `x = \'a'` 34.901; [rule](r:expr.logical.value).
* 40, argument count or value refused: `x = abs()` 40.3, `x = abs(1, 2)` 40.4,
  `x = left('abc', 'q')` 40.12, `x = random(5, 1)` 40.33;
  [rule](r:bif.index.call.too-few), [rule](r:bif.index.call.too-many).
* 41, arithmetic on a non-number: `x = 1 + 'a'` 41.1, `x = -'a'` 41.1,
  `x = 'a' % 2` 41.1; [rule](r:num.index.use).
* 42, arithmetic with no result: `x = 9 // 0` 42.3, `x = 1e999999999 * 10`
  42.901, `x = 1e-999999999 / 10` 42.902, `x = 0 ** -1` 42.903;
  [rule](r:num.limits.overflow).
* 43, call to an unknown routine: `x = nosuchroutine_spec_errors()` 43.1,
  `call nosuchroutine_spec_errors` 43.1,
  `::requires 'nonexistent_spec_errors_zz.rex'` 43.901;
  [rule](r:instr.call.search).
* 44, call that gives no value: `x = nothing()` 44.1, `say nothing()` 44.1;
  [rule](r:expr.terms.call.result).

> The number 29 has no rule, and the length at which it is raised is not
> specified. The number 45 has no construct here: a call made for a value
> that reaches a RETURN without one raises 44, in an internal routine, a
> ::ROUTINE or a program file, as [the call rule](r:expr.terms.call.result)
> says.

```rexx
call try "signal nolabelhere"
call try "signal value 'zz'"
call try "procedure"
call try "trace x1"
call try "numeric digits 'a'"
call try "numeric digits 0"
call try "numeric fuzz -1"
call try "x = 2 ** 1.5"
call try "leave"
call try "iterate"
call try "leave nosuchzz"
call try "iterate nosuchzz"
call try "address value copies('E', 1000)"
call try "numeric digits 5; numeric fuzz 5"
call try "if 'a' then say 1"
call try "x = 1 & 'a'"
call try "x = \'a'"
call try "x = abs()"
call try "x = abs(1, 2)"
call try "x = left('abc', 'q')"
call try "x = random(5, 1)"
call try "x = 1 + 'a'"
call try "x = -'a'"
call try "x = 'a' % 2"
call try "x = 9 // 0"
call try "x = 1e999999999 * 10"
call try "x = 1e-999999999 / 10"
call try "x = 0 ** -1"
call try "x = nosuchroutine_spec_errors()"
call try "call nosuchroutine_spec_errors"
call try "x = nothing()"
call try "say nothing()"
exit
try:
  signal on syntax name e
  interpret arg(1)
  say left(arg(1), 44) '=> none'
  return
e: label = arg(1)
  if arg(2, 'E') then label = arg(2)
  say left(label, 44) condition('o')~code
  return
nothing: return
```

```output
signal nolabelhere                           16.1
signal value 'zz'                            16.1
procedure                                    17.1
trace x1                                     24.1
numeric digits 'a'                           26.5
numeric digits 0                             26.5
numeric fuzz -1                              26.6
x = 2 ** 1.5                                 26.8
leave                                        28.1
iterate                                      28.2
leave nosuchzz                               28.3
iterate nosuchzz                             28.4
address value copies('E', 1000)              29.1
numeric digits 5; numeric fuzz 5             33.1
if 'a' then say 1                            34.1
x = 1 & 'a'                                  34.901
x = \'a'                                     34.901
x = abs()                                    40.3
x = abs(1, 2)                                40.4
x = left('abc', 'q')                         40.12
x = random(5, 1)                             40.33
x = 1 + 'a'                                  41.1
x = -'a'                                     41.1
x = 'a' % 2                                  41.1
x = 9 // 0                                   42.3
x = 1e999999999 * 10                         42.901
x = 1e-999999999 / 10                        42.902
x = 0 ** -1                                  42.903
x = nosuchroutine_spec_errors()              43.1
call nosuchroutine_spec_errors               43.1
x = nothing()                                44.1
say nothing()                                44.1
```

### Faults of the interpreter and its services

r[cond.errors.system]
The numbers below usually name a fault that is neither in the text nor in a
value of the program: the interpreter that cannot start the program or go on,
a program that is stopped, and a request to the system that fails.
* 3, a program file that does not exist: 3.901, with the exit status `253`.
* 4, a HALT that no trap answers: `raise halt` 4.1;
  [rule](r:cond.halt.untrapped).
* 11, parentheses or calls nested beyond the limit;
  [rule](r:expr.index.nesting).
* 48, a request to the system that fails: `call rxqueue 'Set', 'nosuchq'`
  followed by `queue 'x'` 48.1; [rule](r:instr.queue.queue).

> The number 3 comes before any clause runs, so no trap sees it. The numbers
> 5, 39 and 49 name states that this book does not tie to a clause: how much
> a program can ask for before 5 is raised, how complex an expression can be
> before 39, and what 49 reports are not specified. A RAISE HALT is a
> deterministic way to reach 4.

```rexx
call try "call gone; queue 'x'"
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> none'
  return
e: say left(arg(1), 40) condition('o')~code
  return
gone: call rxqueue 'Set', 'spec_errors_no_such_queue'
  return
```

```output
call gone; queue 'x'                     48.1
```

```rexx
signal on syntax name e
raise halt
say 'not reached'
e: say condition('o')~code rc
```

```output
4.1 4
```

### Faults of messages, objects and directives

r[cond.errors.objects]
The numbers from 88 up usually name faults of messages, of objects and of
directives.
* 88, argument value refused: `x = filespec('N')` 88.901,
  `x = .directory~new~put(1)` 88.901, `x = charin(, 0, -1)` 88.907;
  [rule](r:bif.index.call.native).
* 89, USE target of the wrong kind: `use arg a, 'b'` 89.1, `use arg (a)` 89.1;
  [rule](r:instr.use.syntax).
* 90, EXTERNAL entry that is not there:
  `::routine r external 'registered nonezz'` 90.999;
  [rule](r:dir.routine.external).
* 91, message term with no result: `say 'a' .m~new~bar 'b'` 91.999;
  [rule](r:expr.messages.cascade).
* 93, argument value refused: `x = left('abc', -2)` 93.923,
  `x = copies('ab', -2)` 93.906, `x = 'abc'~substr(0)` 93.924, `x = d2c(-1)`
  93.927, `x = .array~new('x')` 93.906,
  `x = stream('nofile_zz', 'c', 'bogus')` 93.914;
  [rule](r:conv.position.check).
* 97, message that is not answered: `x = 'a'~foo` 97.1, `x = .nil~foo` 97.1,
  `.m~new~p` 97.2; [rule](r:expr.messages.name), [rule](r:conv.string.object).
* 98, fault of a running clause or directive: `raise propagate` 98.918,
  `::class a subclass nosuchzzclass` 98.909,
  `::routine r external 'library nonezz'` 98.903;
  [rule](r:dir.requires.failure), [rule](r:dir.options.condition).
* 99, fault in a directive or its place: `::routine zz` 99.914, `expose x`
  99.908, `forward to (.nil)` 99.923, `use local a` 99.915,
  `::class a (twice)` 99.901; [rule](r:dir.class.duplicate).

> The numbers 40, 88 and 93 all refuse arguments, and which of them a builtin
> or a method raises is named on its own page. The number 92 belongs to a
> platform interface that this book does not specify, so no rule or example
> raises it. The numbers 98 and 99 differ in when the fault is found: 99 is a
> fault of form found when the text is read, and 98 is a fault found when
> the clause or directive takes effect. A directive in the text of an
> INTERPRET is no directive, so it raises 99 there.

```rexx
call try "x = filespec('N')"
call try "x = .directory~new~put(1)"
call try "x = charin(, 0, -1)"
call try "use arg a, 'b'"
call try "use arg (a)"
call try "x = left('abc', -2)"
call try "x = copies('ab', -2)"
call try "x = 'abc'~substr(0)"
call try "x = d2c(-1)"
call try "x = .array~new('x')"
call try "x = stream('nofile_zz', 'c', 'bogus')"
call try "x = 'a'~foo"
call try "x = .nil~foo"
call try "raise propagate"
call try "::routine zz"
call try "expose x"
call try "forward to (.nil)"
call try "use local a"
exit
try:
  signal on syntax name e
  interpret arg(1)
  say left(arg(1), 44) '=> none'
  return
e: label = arg(1)
  if arg(2, 'E') then label = arg(2)
  say left(label, 44) condition('o')~code
  return
```

```output
x = filespec('N')                            88.901
x = .directory~new~put(1)                    88.901
x = charin(, 0, -1)                          88.907
use arg a, 'b'                               89.1
use arg (a)                                  89.1
x = left('abc', -2)                          93.923
x = copies('ab', -2)                         93.906
x = 'abc'~substr(0)                          93.924
x = d2c(-1)                                  93.927
x = .array~new('x')                          93.906
x = stream('nofile_zz', 'c', 'bogus')        93.914
x = 'a'~foo                                  97.1
x = .nil~foo                                 97.1
raise propagate                              98.918
::routine zz                                 99.914
expose x                                     99.908
forward to (.nil)                            99.923
use local a                                  99.915
```

> The directive constructs run as the source lines of a package, so that the
> fault is found when the package is built.

```rexx
call pk "::requires 'nonexistent_spec_errors_zz.rex'"
call pk "::routine r external 'registered nonezz'"
call pk "::routine r external 'library nonezz'"
call pk "::class a subclass nosuchzzclass"
call pk "::class a", "::class a"
exit
pk:
  signal on syntax name pe
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('n', src)
  say arg(1) '=> none'
  return
pe: say left(arg(1), 44) condition('o')~code
  return
```

```output
::requires 'nonexistent_spec_errors_zz.rex'  43.901
::routine r external 'registered nonezz'     90.999
::routine r external 'library nonezz'        98.903
::class a subclass nosuchzzclass             98.909
::class a                                    99.901
```

> The numbers 91 and 97.2 need a class to hold the method, so their example
> is a program of its own.

```rexx
signal on syntax name e1
say 'a' .m~new~bar 'b'
e1: say condition('o')~code
signal on syntax name e2
.m~new~p
e2: say condition('o')~code
::class m
::method bar
  return
::method p private
  return
```

```output
91.999
97.2
```
