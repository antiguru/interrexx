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

> Each entry gives the major number, what kind of construct or operation
> raises it, and a rule that raises it where one is written. The grouping is
> by the usual kind of fault, and a number may have subcodes of another kind.
> The examples that follow each list run one clause for most rows and print
> the code that its trap reads.

### Faults in the form of the text

> These faults are usually found in the text of a clause, and a clause that is
> a directive or part of a block can raise them too. A fault in the text of a
> program is found when the program is read, and in the text of an INTERPRET
> when it runs, as [the reading rule](r:cond.index.read) says.

r[cond.errors.form]
The numbers below usually name a fault in the form of the text: a string, a
comment, a character, a structure of blocks or the form of one clause. Two of
them have subcodes that are raised by values when the clause runs, 7 for a
SELECT that finds no WHEN to take, as [the OTHERWISE
rule](r:instr.select.otherwise) says, and 25 for a NUMERIC FORM VALUE that
holds no form, as [the FORM rule](r:instr.numeric.form.error) says.
* 6: a string or a comment that is still open where the text ends,
  [rule](r:lex.index.read-first).
* 7: a SELECT that is built wrongly, or has no branch to take,
  [rule](r:instr.select.syntax).
* 8: a THEN or an ELSE with no IF or WHEN before it to pair with,
  [rule](r:instr.if.else).
* 9: a WHEN or an OTHERWISE met where no SELECT is open,
  [rule](r:instr.select.otherwise).
* 10: an END with no open block to close, or one that names another block,
  [rule](r:instr.do.end).
* 13: a character that the language does not allow in a program,
  [rule](r:lex.characters.set).
* 14: text that ends inside a block whose END has not come,
  [rule](r:instr.do.end).
* 15: a string written with an X or a B whose digits break the grouping or
  digit rules of that form, [rule](r:lex.literals.hex.digit).
* 18: an IF or WHEN test that no THEN follows, [rule](r:instr.if.syntax).
* 19: an operand that the instruction wants as a name or a string and does
  not find, [rule](r:instr.call.error).
* 20: an instruction that needs a symbol at some place and finds another
  kind of token, or none, [rule](r:instr.procedure.syntax).
* 21: more tokens after an instruction that has all its operands,
  [rule](r:instr.call.on.error).
* 22: a string literal whose bytes the character set refuses.
* 23: a computed string whose bytes the character set refuses.
* 25: a keyword that the instruction does not offer at that place,
  [rule](r:instr.numeric.form.error).
* 27: a DO or LOOP whose controller and keyword parts are put together in a
  way that the form forbids, [rule](r:instr.do.syntax).
* 30: a symbol longer than the limit of the symbol rule,
  [rule](r:lex.symbols.length).
* 31: an assignment whose target is spelled like a constant,
  [rule](r:lex.symbols.constant).
* 35: an expression that cannot be parsed, because an operand or an operator
  is missing or misplaced, [rule](r:expr.index.syntax).
* 36: an opening parenthesis or bracket that the clause never closes,
  [rule](r:expr.index.syntax).
* 37: a closing parenthesis or bracket that has no opener of its own kind,
  [rule](r:expr.index.syntax).
* 38: a template whose contents the template form does not allow,
  [rule](r:parse.patterns.syntax).
* 46: a variable reference in parentheses that is not closed or not a single
  name, [rule](r:instr.drop.syntax).
* 47: a label clause in text that cannot hold one, such as the text of an
  INTERPRET, [rule](r:lex.clauses.label).

> The numbers 22 and 23 would apply where a string holds a byte that the
> character set cannot hold. Every string of this book may hold any of the
> 256 character codes, as [the character rule](r:conv.string.chars) says, so
> this book names no clause that raises them.

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
call try "call on error name lbl extra"
call try copies("a", 251) "= 1"
call try "nop 1"
call try "numeric form foo"
call try "numeric form value 'foo'"
call try "select; when 0 then nop; end"
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
call on error name lbl extra             21.903
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 30.1
nop 1                                    21.901
numeric form foo                         25.11
numeric form value 'foo'                 25.11
select; when 0 then nop; end             7.3
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
The numbers below usually name a fault that the values or the flow of control
cause, at the clause that meets it.
* 16: a SIGNAL or a trap name that no label of the code carries,
  [rule](r:instr.signal.label).
* 17: a PROCEDURE reached with another clause before it, or outside an
  internal routine, [rule](r:instr.procedure.first).
* 24: a TRACE setting that is neither a number nor a valid letter,
  [rule](r:instr.trace.invalid).
* 26: a value used as a whole number, such as a setting or a count or an
  exponent, that is not one or lies outside the range allowed,
  [rule](r:num.settings.digits).
* 28: a LEAVE or an ITERATE with no block of that name, or none at all, to
  act on, [rule](r:instr.leave.target).
* 29: the name of an ADDRESS environment that is longer than the interpreter
  takes.
* 33: a NUMERIC DIGITS and FUZZ pair in which FUZZ is not below DIGITS,
  [rule](r:num.settings.fuzz).
* 34: an operand of a test or of a logical operator that is anything but a
  one-character `0` or `1`, [rule](r:expr.logical.value).
* 40: a call whose arguments a routine rejects, for their count (too
  [few](r:bif.index.call.too-few) or [many](r:bif.index.call.too-many)) or
  for a value outside what the routine takes.
* 41: an operand of arithmetic or of a numeric comparison that holds no
  number, [rule](r:num.index.use).
* 42: arithmetic whose result cannot be represented or is not defined, such
  as an overflow, an underflow, a division by zero or zero to a negative
  power, [rule](r:num.limits.overflow).
* 43: a routine name that nothing supplies, [rule](r:instr.call.search).
* 44: a clause that needs a value from a call that returned none,
  [rule](r:expr.terms.call.result).
* 45: none of this book's clauses: the engines raise 44 where it fits.

> The number 29 is raised for an ADDRESS name beyond a length that this book
> does not fix. The number 45 is meant for a program that is called for a
> value and ends without one; both engines raise 44 at the clause that makes
> such a call, so 45 has no example here.

```rexx
call try "signal nolabelhere"
call try "procedure"
call try "trace x1"
call try "numeric digits 'a'"
call try "leave"
call try "numeric digits 5; numeric fuzz 5"
call try "if 'a' then say 1"
call try "x = abs()"
call try "x = left('abc', 'q')"
call try "address value '"copies("E", 1000)"'"
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
x = left('abc', 'q')                     40.12
address value 'EEEEEEEEEEEEEEEEEEEEEEEEE 29.1
x = 1 + 'a'                              41.1
x = 9 // 0                               42.3
x = nosuchroutine_spec_errors()          43.1
x = nothing()                            44.1
```

### Faults of the interpreter and its services

r[cond.errors.system]
The numbers below usually name a fault that is neither in the text nor in a
value of the program: the interpreter that cannot start the program or go on,
a program that is stopped, and a call into the system that fails.
* 3: the program cannot be loaded to begin with.
* 4: the program was halted and nothing trapped it,
  [rule](r:cond.halt.untrapped).
* 5: the interpreter lacks a resource that it needs to go on.
* 11: nesting of parentheses or of calls beyond the limit,
  [rule](r:expr.index.nesting).
* 39: an expression that the evaluator gives up on for its size or depth.
* 48: a call that the interpreter makes into the system and that fails, such
  as one on the external queue, [rule](r:instr.queue.queue).
* 49: a consistency check of the interpreter itself that fails.

> The number 3 is raised for a program that cannot be loaded, such as a
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
The numbers from 88 up usually name faults of messages, of objects, of
directives and of native code.
* 88: an argument that is missing or surplus for a routine or method of the
  native family, [rule](r:bif.index.call.native).
* 89: a USE clause whose target is neither a variable nor a message term,
  [rule](r:instr.use.syntax).
* 90: a library routine or method entry that a directive names and the
  system cannot supply, [rule](r:dir.routine.external).
* 91: a message term used for its value whose method ended without one,
  [rule](r:expr.messages.cascade).
* 92: a failure in the object-linking interface of a platform.
* 93: a method, or a builtin that is a String method, given an argument that
  it cannot accept, [rule](r:conv.position.check).
* 97: a message that finds no method of its name in the class of the
  receiver, [rule](r:expr.messages.name).
* 98: a fault found while a clause or a directive runs that no lower number
  names, such as the load of what a directive requires
  ([rule](r:dir.requires.failure)) or a NOVALUE under ::OPTIONS
  ([rule](r:dir.options.condition)).
* 99: a fault in the form of a directive, or an instruction in a place where
  it cannot stand, found when the text is read,
  [rule](r:dir.class.duplicate).

> The numbers 40, 88 and 93 all mean arguments that do not fit, and which of
> them a builtin or a method raises depends on the fault and is named on its
> own page: `left('abc', 'q')` raises 40 and `left('abc', -2)` raises 93.
> The number 92 belongs to a platform interface that this book does not
> specify, so no rule or example raises it. The numbers 98 and 99 differ in
> when the fault is found: 99 is a fault of the form that is found when the
> text is read, and 98 is a fault found when the clause or directive takes
> effect. A directive in the text of an INTERPRET is no directive, so it
> raises 99 there.

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
