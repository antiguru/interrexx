---
element: "Error numbers"
kind: concept
ansi: "8.4.1, 8.4.2, 9.2.4"
rexxref: "Conditions and Condition Traps > Condition Information > The Special Variable RC; Appendix > Error Numbers and Messages"
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
An error number has two parts, a major number and a subcode, which is a whole
number written after a point and without leading zeros: `40.3`, `93.924`.
The major number names the kind of error and the subcode the particular fault
of that kind, so one major number has many subcodes and a program that wants a
kind of error looks only at the major number. A trap sees the whole number as
the string `m.n` in `CONDITION('O')~CODE`, and the major number alone in the
`RC` entry and in the variable `RC`, as [the values
rule](r:cond.information.object.values) and [the RC
rule](r:cond.information.rc) say. `SIGL` is the line of the clause that was
running, as [the SIGL rule](r:cond.information.sigl) says. An error that no
SIGNAL trap answers ends the program with the exit code `256` minus the major
number, as [the untrapped rule](r:cond.index.syntax) says, and the subcode
does not enter it. The error of a fault in the text that is read before the
program runs is no condition at all, as [the reading
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
functions that raise it, in the rule that names the fault. Two faults of one
major number differ in their subcodes and in nothing that a program can rely
on in the message, so a program that must tell them apart compares `CODE`.
This page names a subcode only to show a kind of error.

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

> Each entry gives the major number, the kind of fault it names in a few
> words of this book's own, and a rule that raises it where one is written.
> The examples that follow each list run one clause for most rows and print
> the code that its trap reads.

### Faults in the form of the text

> These faults are found in the text of a clause, and a clause that is a
> directive or part of a block can raise them too. A fault in the text of a
> program is found when the program is read, and in the text of an INTERPRET
> when it runs, as [the reading rule](r:cond.index.read) says.

r[cond.errors.form]
The numbers below name a fault in the text, whether it is a string, a
comment, a character, a structure of blocks or the form of one clause.
* 6: a string or a comment that is not closed, [rule](r:lex.index.read-first).
* 7: a SELECT that has no WHEN where one belongs,
  [rule](r:instr.select.syntax).
* 8: a THEN or an ELSE that has no IF or WHEN to belong to,
  [rule](r:instr.if.else).
* 9: a WHEN or an OTHERWISE outside a SELECT,
  [rule](r:instr.select.otherwise).
* 10: an END that closes nothing, or that names the wrong block,
  [rule](r:instr.do.end).
* 13: a character outside the character set, [rule](r:lex.characters.set).
* 14: a block that is not closed, [rule](r:instr.interpret.complete).
* 15: a hexadecimal or binary string that is malformed,
  [rule](r:lex.literals.hex.digit).
* 18: an IF or WHEN condition that THEN does not follow,
  [rule](r:instr.if.syntax).
* 19: a string or a symbol is wanted and something else stands there,
  [rule](r:instr.call.error).
* 20: a symbol is wanted and something else stands there,
  [rule](r:instr.procedure.syntax).
* 21: data that remains after a clause is complete,
  [rule](r:instr.leave.error).
* 22: a literal string with character codes that are not valid.
* 23: a value with character codes that are not valid.
* 25: a keyword or option that the instruction does not have,
  [rule](r:instr.numeric.form.error).
* 27: the form of a DO or LOOP that is not valid, [rule](r:instr.do.syntax).
* 30: a name or a symbol that is too long, [rule](r:lex.symbols.length).
* 31: a name that starts with a digit or a point, where a target stands,
  [rule](r:lex.symbols.constant).
* 35: an expression that is not valid, [rule](r:expr.index.syntax).
* 36: a parenthesis or a bracket that is not matched,
  [rule](r:expr.index.syntax).
* 37: a comma, a parenthesis or a bracket where none belongs,
  [rule](r:expr.index.syntax).
* 38: a template or a pattern that is not valid,
  [rule](r:parse.patterns.syntax).
* 46: a variable reference in parentheses that is not valid,
  [rule](r:instr.drop.syntax).
* 47: a label where the text does not allow one, [rule](r:lex.clauses.label).

> The numbers 22 and 23 are for a string that holds a character code which
> the character set cannot hold. Every string of this book may hold any of
> the 256 character codes, as [the character rule](r:conv.string.chars)
> says, so this book names no clause that raises them.

```rexx
call try "say 'a"
call try "select; say 1; end"
call try "if 1 then say 'a'; else; else say 'b'"
call try "when 1 then say 'a'"
call try "end"
call try "say 1 #"
call try "do; say 1"
call try "say 'zz'x"
call try "if 1 say 'a'"
call try "call"
call try "drop a b 'c'"
call try "leave x y"
call try "numeric form foo"
call try "do forever x"
call try "1x = 2"
call try "x = 1 +"
call try "x = (1"
call try "x = ]"
call try "parse var x a 3 3 b b ) c"
call try "drop (x y"
call try "x: say 1"
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> none'
  return
e: say left(arg(1), 40) condition('o')~code
  return
```

```output
say 'a                                   6.2
select; say 1; end                       7.2
if 1 then say 'a'; else; else say 'b'    8.2
when 1 then say 'a'                      9.1
end                                      10.1
say 1 #                                  13.1
do; say 1                                14.1
say 'zz'x                                15.3
if 1 say 'a'                             18.1
call                                     19.2
drop a b 'c'                             20.901
leave x y                                21.907
numeric form foo                         25.11
do forever x                             27.901
1x = 2                                   31.2
x = 1 +                                  35.1
x = (1                                   36.901
x = ]                                    37.901
parse var x a 3 3 b b ) c                38.1
drop (x y                                46.1
x: say 1                                 47.1
```

### Faults in the values and the flow

> These faults are found while a clause runs, because a value or the state
> of the program does not fit the clause.

r[cond.errors.run]
The numbers below name a fault that the values or the flow of control cause,
at the clause that meets it.
* 16: a label that the program does not have, [rule](r:instr.signal.label).
* 17: a PROCEDURE that is not the first clause of a routine,
  [rule](r:instr.procedure.first).
* 24: a TRACE setting that is not valid, [rule](r:instr.trace.invalid).
* 26: a value that is no whole number, or out of the range allowed,
  [rule](r:num.settings.digits).
* 28: a LEAVE or an ITERATE that has no block to act on,
  [rule](r:instr.leave.target).
* 29: the name of an ADDRESS environment that is too long.
* 33: the value of a NUMERIC setting that the other settings forbid,
  [rule](r:num.settings.fuzz).
* 34: a value used as a truth value that is not 0 or 1,
  [rule](r:expr.logical.value).
* 40: a call of a routine with arguments that do not fit it,
  [rule](r:bif.index.call.count).
* 41: a value used as a number that is not a number,
  [rule](r:num.limits.not-number).
* 42: arithmetic that overflows, underflows or divides by zero,
  [rule](r:num.limits.overflow).
* 43: a routine that cannot be found, [rule](r:instr.call.search).
* 44: a clause that needs a value from a call that gave none,
  [rule](r:expr.terms.call.result).
* 45: none of this book's clauses: the engines raise 44 where it fits.

> The number 29 is raised for an ADDRESS name beyond a length that this book
> does not fix. The number 45 is the error of a RETURN that carries no data
> in a program called as a function; both engines raise 44 for the clause
> that calls such a program, so 45 has no example here.

```rexx
call try "signal nolabelhere"
call try "procedure"
call try "trace x1"
call try "numeric digits 'a'"
call try "leave"
call try "numeric digits 5; numeric fuzz 5"
call try "if 'a' then say 1"
call try "x = abs()"
call try "x = 1 + 'a'"
call try "x = 9 // 0"
call try "x = nosuchroutine_spec_errors()"
call try "x = nothing()"
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> none'
  return
e: say left(arg(1), 40) condition('o')~code
  return
nothing: return
```

```output
signal nolabelhere                       16.1
procedure                                17.1
trace x1                                 24.1
numeric digits 'a'                       26.5
leave                                    28.1
numeric digits 5; numeric fuzz 5         33.1
if 'a' then say 1                        34.1
x = abs()                                40.3
x = 1 + 'a'                              41.1
x = 9 // 0                               42.3
x = nosuchroutine_spec_errors()          43.1
x = nothing()                            44.1
```

### Faults of the interpreter and its services

r[cond.errors.system]
The numbers below name a fault that is not in the text or in a value of the
program: the interpreter that cannot start the program or go on, a program
that is stopped, and a service of the system that does not answer.
* 3: the program cannot be started or loaded.
* 4: the program was halted and nothing trapped it,
  [rule](r:cond.halt.untrapped).
* 5: the interpreter has no resource left that it needs.
* 11: nesting of parentheses or of calls that is too deep,
  [rule](r:expr.index.nesting).
* 39: an expression too complex for the interpreter to evaluate.
* 48: a service of the system that fails, such as the external queue,
  [rule](r:instr.queue.queue).
* 49: an internal check of the interpreter that fails.

> The number 3 is the error of a program that cannot be started, such as a
> program file that cannot be found. It comes before any clause runs, so no
> trap sees it. The numbers 5, 39 and 49 name states that this book does not
> tie to a clause: how much a program can ask for before 5 is raised, how
> complex an expression can be before 39, and what 49 reports are not
> specified. The number 4 is the error that a HALT becomes when nothing
> traps it, and a RAISE HALT is a deterministic way to reach it.

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

### Faults of objects, directives and native code

r[cond.errors.objects]
The numbers from 88 up name faults of messages, of objects, of directives
and of native routines.
* 88: a builtin of the native kind that is called with arguments that do not
  fit, [rule](r:bif.index.call.native).
* 89: a variable or a message term that is wanted and another thing stands,
  [rule](r:instr.use.syntax).
* 90: an external routine or method whose name cannot be found,
  [rule](r:dir.routine.external).
* 91: a message term that needs a result from a method that gave none,
  [rule](r:expr.messages.cascade).
* 92: a fault of the OLE interface of a platform.
* 93: a call of a method, or of a builtin function that is one, with arguments
  that do not fit, [rule](r:conv.position.check).
* 97: a message to an object that has no method of that name,
  [rule](r:conv.string.object).
* 98: a fault in running a directive or in loading what it names,
  [rule](r:dir.class.subclass).
* 99: a fault in a directive, found while the program is read,
  [rule](r:dir.class.duplicate).

> The number 92 belongs to a platform interface that this book does not
> specify, so no rule or example raises it. The numbers 98 and 99 differ in
> when the fault is found: 99 is a fault of the text of a directive and is
> found when the program is read, and 98 is a fault of what the directive
> asks for and is found when the directive takes effect. A directive in the
> text of an INTERPRET is no directive, so it raises 99 there.

```rexx
call try "x = filespec('N')"
call try "use arg a, 'b'"
call try "x = .array~new~foo~bar"
call try "x = 'abc'~substr(0)"
call try "x = .nil~foo"
call try "::routine zz"
call pk 'registered'
call pk 'library'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> none'
  return
e: say left(arg(1), 40) condition('o')~code
  return
pk:
  signal on syntax name pe
  use arg kind
  src = .array~of('::routine r external "'kind' spec_errors_none"')
  p = .package~new('n', src)
  return
pe: say 'external' left(kind, 11) condition('o')~code
  return
```

```output
x = filespec('N')                        88.901
use arg a, 'b'                           89.1
x = .array~new~foo~bar                   97.1
x = 'abc'~substr(0)                      93.924
x = .nil~foo                             97.1
::routine zz                             99.914
external registered  90.999
external library     98.903
```

> The number 91 needs a method that returns nothing and a class to hold it,
> so its example is a program of its own.

```rexx
signal on syntax name e
say 'a' .m~new~bar 'b'
e: say condition('o')~code
::class m
::method bar
  return
```

```output
91.999
```
