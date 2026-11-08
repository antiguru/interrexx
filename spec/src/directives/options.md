---
element: "::OPTIONS"
kind: directive
syntax: "::OPTIONS option ..."
rexxref: "Directives > ::OPTIONS"
---

# ::OPTIONS

::OPTIONS sets the defaults that the code of its package starts with. The
defaults are the three NUMERIC settings, the TRACE setting, whether a routine
takes the NUMERIC settings of its caller, the way that six conditions are
raised, and whether the prolog runs when another package requires this one.
The settings themselves are specified with [NUMERIC](r:num.settings.effect)
and [TRACE](r:instr.trace.scope), and the conditions with their own pages in
the conditions chapter. This page specifies the form of the directive, what
each option sets, and how far it reaches.

## The directive

r[dir.options.syntax]
The words after `::OPTIONS` are options. An option is a keyword, in any case,
followed by its value, except PROLOG and NOPROLOG, which stand alone. Any
number of options may be written in any order, on one directive or on several.
The options are processed in text order, so an option written again replaces
the earlier one and options of different directives add up. A directive with
no option does nothing. A word that is no keyword of an option, which a
second value after a keyword also is, raises `SYNTAX 25.924`. The directive
takes no code of its own: a clause after it that is not a null clause raises
`SYNTAX 99.916`, as [the section rule](r:dir.index.section) says. The faults
are found when the program is read, so a program file with one writes nothing
and exits with 256 minus the error number, 231 for `25.924`.

```rexx
src = .array~of('say digits() fuzz() form()', -
  '::options digits 20 form engineering', -
  '::options fuzz 3 digits 15', -
  '::OPTIONS Digits 11')
p = .package~new('spec-dir-options-syntax', src)
src = .array~of('say digits() fuzz() form()', '::options', '::options ;')
p = .package~new('spec-dir-options-syntax2', src)
call try '::options bogus'
call try '::options 5'
call try '::options "digits" 5'
call try '::options digits 5 6'
call try '::options form engineering digits 5'
call try '::options prolog 5'
call try '::options digits 5', 'say 1'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-options-try', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
11 3 ENGINEERING
9 0 SCIENTIFIC
::options bogus => 25.924
::options 5 => 25.924
::options "digits" 5 => 25.924
::options digits 5 6 => 25.924
::options form engineering digits 5 => ok
::options prolog 5 => 25.924
::options digits 5 / say 1 => 99.916
```

```rexx
say 'before'
::options bogus
```

```output rc=231
```

r[dir.options.scope]
The options of a package are the starting values of all its code. They reach
the prolog, every `::ROUTINE` and every method of the package, wherever the
directive stands in the file, so that code written before the directive has
them too. The NUMERIC and TRACE settings that they give are the defaults of
the package that [DIGITS](r:bif.digits.default), [FUZZ](r:bif.fuzz.default)
and [FORM](r:bif.form.default) name: a NUMERIC instruction without a value
returns the setting to them. A NUMERIC or TRACE instruction changes only the
setting of the code that runs it, as [the scope rule](r:num.settings.call)
says, and the next routine or method that starts is not affected. An internal
routine and the text of an INTERPRET take the setting of the code that runs
them, as the scope rules of the settings say, and not the option. The
instruction TRACE with no value sets `N`, not the option, as [the bare
form](r:instr.trace.form.bare) says.

```rexx
say 'prolog' digits() fuzz() form() trace()
call internal
say 'routine' r()
say 'method' .box~new~m()
numeric digits 20
numeric fuzz 3
numeric form scientific
trace n
say 'changed' digits() fuzz() form() trace()
call internal
say 'routine' r()
numeric digits
numeric fuzz
numeric form
trace
say 'reset' digits() fuzz() form() trace()
interpret 'say "interpret" digits()'
exit
internal: say 'internal' digits() fuzz() form() trace(); return
::options digits 12 fuzz 2 form engineering trace o
::routine r
  return digits() fuzz() form() trace()
::class box
::method m
  return digits() fuzz() form() trace()
```

```output
prolog 12 2 ENGINEERING O
internal 12 2 ENGINEERING O
routine 12 2 ENGINEERING O
method 12 2 ENGINEERING O
changed 20 3 SCIENTIFIC N
internal 20 3 SCIENTIFIC N
routine 12 2 ENGINEERING O
reset 12 2 ENGINEERING N
interpret 12
```

r[dir.options.package]
The options of a package belong to the code that is written in it. They do
not reach the code of a package that this one requires, and the options of a
required package do not reach the code that requires it. A routine of one
package that another package calls starts with the options of the package
that holds the routine, as [the scope rule](r:bif.digits.scope) says for the
NUMERIC settings.

```rexx
f = 'spec-dir-options-lib.tmp'
call sysfiledelete f
call lineout f, '::options digits 15 fuzz 3 form engineering'
call lineout f, '::routine libr public'
call lineout f, '  return digits() fuzz() form()'
call lineout f
src = .array~of('say "main" digits() fuzz() form()', -
  'say "lib" libr()', -
  '::requires "'f'"', -
  '::options digits 6')
p = .package~new('spec-dir-options-main', src)
call sysfiledelete f
```

```output
main 6 0 SCIENTIFIC
lib 15 3 ENGINEERING
```

## The numeric options

r[dir.options.numeric]
DIGITS, FUZZ and FORM set the NUMERIC settings that the code of the package
starts with. DIGITS and FUZZ take a symbol or a string that the [NUMERIC
instruction](r:num.settings.digits) would accept as its value, so that `"7"`,
`05`, `1e2` and `5.0` are whole numbers. A value that is not a whole
number of at least 1 raises `SYNTAX 26.5` for DIGITS, and one that is not a
whole number of at least 0 raises `26.6` for FUZZ. A symbol is never looked
up as a variable. A missing value, or one that is not a symbol or a string,
such as a sign or parentheses, raises `19.917` for DIGITS and `19.918` for
FUZZ. FORM takes the symbol SCIENTIFIC or ENGINEERING in any case, and no
abbreviation. A missing value or a string raises `SYNTAX 20.925`, and any
other symbol `25.11`. The faults are found when the program is read.

```rexx
call try '::options digits "7" fuzz 05'
call try '::options digits 1e2'
call try '::options digits 5.0'
call try '::options digits abc'
call try '::options digits 1.5'
call try '::options digits 0'
call try '::options digits -1'
call try '::options digits (5)'
call try '::options digits'
call try '::options fuzz abc'
call try '::options fuzz -1'
call try '::options fuzz'
call try '::options form Engineering'
call try '::options form sci'
call try '::options form "scientific"'
call try '::options form'
exit
try:
  signal on syntax name e
  src = .array~of('say digits() fuzz() form()', arg(1))
  p = .package~new('spec-dir-options-num', src)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
7 5 SCIENTIFIC
::options digits "7" fuzz 05 => ok
100 0 SCIENTIFIC
::options digits 1e2 => ok
5 0 SCIENTIFIC
::options digits 5.0 => ok
::options digits abc => 26.5
::options digits 1.5 => 26.5
::options digits 0 => 26.5
::options digits -1 => 19.917
::options digits (5) => 19.917
::options digits => 19.917
::options fuzz abc => 26.6
::options fuzz -1 => 19.918
::options fuzz => 19.918
9 0 ENGINEERING
::options form Engineering => ok
::options form sci => 25.11
::options form "scientific" => 20.925
::options form => 20.925
```

r[dir.options.fuzz]
The two settings must stay apart: after each DIGITS or FUZZ is processed,
DIGITS must exceed FUZZ, or the directive raises `SYNTAX 33.1`, as [the fuzz
rule](r:bif.fuzz.result) says for the instruction. The check is made on the
value that the other setting holds at that point, which is its default of 9 or
0 until an option sets it, and it is made after every option, so the order of
the options matters. The error is found when the program is read.

```rexx
call try '::options fuzz 8'
call try '::options fuzz 9'
call try '::options digits 4'
call try '::options digits 5 fuzz 5'
call try '::options digits 5 fuzz 4'
call try '::options fuzz 8 digits 10'
call try '::options digits 10 fuzz 8 digits 8'
call try '::options digits 10 fuzz 8', '::options digits 7'
call try '::options digits 20', '::options fuzz 19', '::options digits 19'
exit
try:
  signal on syntax name e
  src = .array~of('say digits() fuzz()')
  text = arg(1)
  src~append(arg(1))
  do i = 2 to arg()
    src~append(arg(i))
    text = text '/' arg(i)
  end
  p = .package~new('spec-dir-options-fuzz', src)
  say text '=> ok'
  return
e: say text '=>' condition('o')~code
  return
```

```output
9 8
::options fuzz 8 => ok
::options fuzz 9 => 33.1
4 0
::options digits 4 => ok
::options digits 5 fuzz 5 => 33.1
5 4
::options digits 5 fuzz 4 => ok
10 8
::options fuzz 8 digits 10 => ok
::options digits 10 fuzz 8 digits 8 => 33.1
::options digits 10 fuzz 8 / ::options digits 7 => 33.1
::options digits 20 / ::options fuzz 19 / ::options digits 19 => 33.1
```

r[dir.options.inherit]
NUMERIC INHERIT makes a routine or a method of the package start with the
three NUMERIC settings of the code that calls it, and NUMERIC NOINHERIT, which
is the default, makes it start with the options and defaults of its own
package. The word after NUMERIC must be INHERIT or NOINHERIT, and the last of
the two in text order is used. A missing word, or a string, raises
`SYNTAX 20.935`, and any other symbol `25.935`, found when the program is
read. With INHERIT the settings that DIGITS, FUZZ and FORM give still start
the prolog, but a `::ROUTINE` or a method does not use them. A routine of a
package with INHERIT that a caller from another package calls takes the
settings of that caller.

```rexx
say 'start' digits()
numeric digits 5
numeric fuzz 1
numeric form engineering
say 'main' digits() fuzz() form()
say 'routine' r()
say 'method' .box~new~m()
numeric digits 7
say 'routine' r2()
f = 'spec-dir-options-inh.tmp'
call sysfiledelete f
call lineout f, '::options numeric inherit'
call lineout f, '::routine libr public'
call lineout f, '  return digits() fuzz() form()'
call lineout f
src = .array~of('numeric digits 6; numeric fuzz 2', -
  'say "other package" libr()', '::requires "'f'"')
p = .package~new('spec-dir-options-inh2', src)
call sysfiledelete f
call try '::options numeric inherit'
call try '::options numeric inherit numeric noinherit'
call try '::options numeric noinherit', '::options numeric inherit'
call try '::options numeric'
call try '::options numeric "inherit"'
call try '::options numeric bogus'
call try '::options numeric 5'
exit
try:
  signal on syntax name e
  src = .array~of('numeric digits 5', 'say r()')
  text = arg(1)
  src~append(arg(1))
  if arg() > 1 then do
    src~append(arg(2))
    text = text '/' arg(2)
  end
  src~append('::routine r')
  src~append('  return digits()')
  p = .package~new('spec-dir-options-inh3', src)
  say text '=> ok'
  return
e: say text '=>' condition('o')~code
  return
::options numeric inherit digits 15
::routine r
  return digits() fuzz() form()
::routine r2
  numeric digits 3
  return r()
::class box
::method m
  return digits() fuzz() form()
```

```output
start 15
main 5 1 ENGINEERING
routine 5 1 ENGINEERING
method 5 1 ENGINEERING
routine 3 1 ENGINEERING
other package 6 2 SCIENTIFIC
5
::options numeric inherit => ok
9
::options numeric inherit numeric noinherit => ok
5
::options numeric noinherit / ::options numeric inherit => ok
::options numeric => 20.935
::options numeric "inherit" => 20.935
::options numeric bogus => 25.935
::options numeric 5 => 25.935
```

## The trace option

r[dir.options.trace]
TRACE sets the tracing setting that the prolog, every routine and every method
of the package start with. Its value is a word that the [TRACE
instruction](r:instr.trace.form.name) accepts, chosen by its first letter, so
that `Off` and `O` are alike, and a letter that is not valid raises `SYNTAX
24.1`. A missing value raises `19.919`. The directive is stricter than the
instruction in one respect: a number is not a setting and raises `24.1`. The
setting is what `TRACE()` returns when the code starts, and a TRACE
instruction changes it for the running code only, as [the scope
rule](r:instr.trace.scope) says.

```rexx
call try 'o'
call try 'Off'
call try 'normal'
call try 'N'
call try '"n"'
call try 'x'
call try '3'
call try ''
exit
try:
  signal on syntax name e
  src = .array~of('say "start" trace()', 'trace n', -
    'say "changed" trace() .box~new~m r()', -
    '::options trace' arg(1), -
    '::routine r', '  return trace()', -
    '::class box', '::method m', '  return trace()')
  p = .package~new('spec-dir-options-trace', src)
  return
e: say 'trace' arg(1) '=>' condition('o')~code
  return
```

```output
start O
changed N O O
start O
changed N O O
start N
changed N N N
start N
changed N N N
start N
changed N N N
trace x => 24.1
trace 3 => 24.1
trace  => 19.919
```

## The condition options

r[dir.options.condition]
ERROR, FAILURE, LOSTDIGITS, NOSTRING, NOTREADY and NOVALUE each take the value
SYNTAX or CONDITION. CONDITION is the default and leaves the condition as the
conditions chapter specifies it. SYNTAX makes the condition that no trap
answers raise a SYNTAX condition instead, with these codes: `98.970` for
ERROR, `98.971` for FAILURE, `98.972` for LOSTDIGITS, `98.973` for NOSTRING,
`98.974` for NOTREADY and `98.986` for NOVALUE. The clause that raised it ends
there as [the SYNTAX rule](r:cond.syntax.raise) says, and `RC` is 98 in the
trap that catches it. Without a trap the program ends with the error report
and the exit status 158. A FAILURE that SYNTAX turns into an error is not
passed on to the ERROR trap, as the second example shows. A missing value or a
string raises `SYNTAX 20.929` and any other symbol `25.927`, both when the
program is read.

```rexx
call try 'error'
call try 'failure'
call try 'lostdigits'
call try 'nostring'
call try 'notready'
call try 'novalue'
exit
try:
  which = arg(1)
  signal on syntax name e
  select
    when which = 'error' then address system 'exit 3'
    when which = 'failure' then address nosuchenv 'echo x'
    when which = 'lostdigits' then do; numeric digits 3; say 12345 + 0; end
    when which = 'nostring' then say 'a' || .nstr~new
    when which = 'notready' then say linein('nosuchfile.zzz')
    when which = 'novalue' then say undefinedvar
  end
  say which 'went on'
  return
e: say which condition('o')~code rc
  return
::class nstr
::method string
  return .nil
::options error syntax failure syntax lostdigits syntax
::options nostring syntax notready syntax novalue syntax
```

```output
error 98.970 98
failure 98.971 98
lostdigits 98.972 98
nostring 98.973 98
notready 98.974 98
novalue 98.986 98
```

```rexx
signal on syntax name s
signal on error name t
address nosuchenv 'echo x'
say 'went on'
exit
t: say 'error trap' condition('c')
exit
s: say 'syntax' condition('o')~code rc
::options failure syntax
```

```output
syntax 98.971 98
```

```rexx
address system 'exit 3'
say 'not reached'
::options error syntax
```

```output rc=158
```

```rexx
call try '::options error'
call try '::options error "syntax"'
call try '::options error bogus'
call try '::options novalue 5'
call try '::options notready condition'
exit
try:
  signal on syntax name e
  src = .array~of('say "ok"', arg(1))
  p = .package~new('spec-dir-options-cond', src)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
::options error => 20.929
::options error "syntax" => 20.929
::options error bogus => 25.927
::options novalue 5 => 25.927
ok
::options notready condition => ok
```

r[dir.options.condition.trap]
A trap still answers the condition with its own name: with SIGNAL ON, CALL ON
or the ANY trap on, the condition is raised as ERROR, FAILURE, LOSTDIGITS,
NOSTRING, NOTREADY or NOVALUE and not as SYNTAX, and the answering trap is the
one that [the choice rule](r:cond.traps.choice) picks. The option applies only
while the trap instructions have left the condition alone: a SIGNAL or CALL
instruction that turns the trap of the condition `ON` or `OFF`, or the trap of
ANY, ends the option for that condition for the rest of the routine or method
in which it ran, so that an `OFF` condition is
[ignored](r:cond.index.untrapped) again. An internal routine takes this state
from its caller. A `::ROUTINE` and a method start without the state, and the
option applies there afresh. A SYNTAX condition that the option raises is an
ordinary one, which SIGNAL ON SYNTAX answers.

```rexx
call try 1
call try 2
call try 3
call try 4
call try 5
call try 6
call try 7
call try 8
call try 9
call try 10
exit
try:
  which = arg(1)
  signal on syntax name e
  select
    when which = 1 then say 'one' unset1
    when which = 2 then do; signal on novalue name t; say 'two' unset2; end
    when which = 3 then do; signal off novalue; say 'three' unset3; end
    when which = 4 then do; signal on any name t; say 'four' unset4; end
    when which = 5 then do; signal off any; say 'five' unset5; end
    when which = 6 then do
      call on error name tc; address system 'exit 3'
    end
    when which = 7 then do; call off error; address system 'exit 3'; end
    when which = 8 then call inner
    when which = 9 then do; signal off novalue; call inner; end
    when which = 10 then do; signal off novalue; call routine10; end
  end
  say which 'went on'
  return
t: say which 'trap' condition('c')
  return
tc: say which 'call trap' condition('c')
  return
e: say which 'syntax' condition('o')~code
  return
inner: say 'inner' unset8
  return
::routine routine10
  say 'routine' unset10
::options novalue syntax error syntax
```

```output
1 syntax 98.986
2 trap NOVALUE
three UNSET3
3 went on
4 trap NOVALUE
five UNSET5
5 went on
6 call trap ERROR
6 went on
7 went on
8 syntax 98.986
8 went on
inner UNSET8
9 went on
10 syntax 98.986
```

r[dir.options.condition.all]
ALL stands for the six condition options at once: `ALL SYNTAX` sets all of
them to SYNTAX and `ALL CONDITION` sets all of them to CONDITION. It is
processed in text order like any option, so an option that follows it
overrides its value for one condition, and one that precedes it is overridden.
A missing value or a string after ALL raises `SYNTAX 20.929`, and any other
symbol `25.927`.

```rexx
call try 'error'
call try 'novalue'
call try 'lostdigits'
call try 'nothing'
exit
try:
  which = arg(1)
  signal on syntax name e
  select
    when which = 'error' then address system 'exit 3'
    when which = 'novalue' then say undefinedvar
    when which = 'lostdigits' then do; numeric digits 3; say 12345 + 0; end
    otherwise nop
  end
  say which 'went on'
  return
e: say which condition('o')~code
  return
::options error condition all syntax novalue condition
```

```output
error 98.970
UNDEFINEDVAR
novalue went on
lostdigits 98.972
nothing went on
```

## The prolog option

r[dir.options.prolog]
PROLOG and NOPROLOG say whether the prolog of the package runs when the
package is not the program that was started: PROLOG, the default, runs it and
NOPROLOG leaves it out, so that a package of routines and classes can be
required without its start-up code. The last of the two in text order is
used. The installation of the package is not affected, so its routines and
classes exist all the same. The program that was started
runs its prolog whatever the option says. Loading the text of a package with
the `Package` class follows the same rule, as the example shows.

```rexx
f = 'spec-dir-options-pro1.tmp'
call sysfiledelete f
call lineout f, 'say "prolog of one"'
call lineout f, '::options noprolog'
call lineout f, '::routine one public'
call lineout f, '  return "one"'
call lineout f, '::class kone public'
call lineout f, '::method activate class'
call lineout f, '  say "activate kone"'
call lineout f
g = 'spec-dir-options-pro2.tmp'
call sysfiledelete g
call lineout g, 'say "prolog of two"'
call lineout g, '::options noprolog prolog'
call lineout g, '::routine two public'
call lineout g, '  return "two"'
call lineout g
src = .array~of('say "main prolog"', 'say one() two() .kone~id', -
  '::requires "'f'"', '::requires "'g'"')
p = .package~new('spec-dir-options-main3', src)
src = .array~of('say "never written"', '::options noprolog')
p = .package~new('spec-dir-options-main4', src)
call sysfiledelete f
call sysfiledelete g
```

```output
activate kone
prolog of two
main prolog
one two KONE
```

```rexx
say 'this prolog runs'
::options noprolog
```

```output
this prolog runs
```
