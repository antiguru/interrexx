---
element: "Security manager"
kind: concept
rexxref: "The Security Manager; The Security Manager > Calls to the Security Manager; Builtin Classes > Fundamental Classes > Routine Class > setSecurityManager; Builtin Classes > Fundamental Classes > Method Class > setSecurityManager; Builtin Classes > Fundamental Classes > Package Class > setSecurityManager"
---

# Security manager

A program can run code that it does not trust, and a security manager is the
way to keep that code inside bounds. The manager is an object of the
program's own. The interpreter asks it before the untrusted code does any of
a short list of things: calls a routine that lives outside the code, runs a
command, loads a file of Rexx code, reads a name from `.local` or
`.environment`, names a stream, or sends a message to a protected method. The
manager can let the action go on, can do it in its place, or can raise an
error.

The class pages own the methods that set a manager, and they own the rules of
each checkpoint that are tied to a class: [`setSecurityManager` of
`Routine`](r:class.routine.setsecuritymanager), of
[`Method`](r:class.method.setsecuritymanager) and of
[`Package`](r:class.package.setsecuritymanager), and [the checkpoint
rules](r:class.routine.setsecuritymanager.checkpoints) that follow the first
of them. This page ties those rules into one model: what a manager belongs to
and which code it governs, when a change of manager takes effect, what each
checkpoint sends and what its answer does, and what the code that is governed
sees when a manager answers with something wrong. A rule here links the class
rule it extends and states only the part that rule leaves out.

The examples call the code that they govern through `Routine` objects and
`Package` objects that they make from source text, so that the code is
another package than the example. Where a command is run, it is `exit` with a
number, which writes nothing and answers the number in `rc`.

## The checkpoints

r[env.security.model]
A security manager is any object. Each checkpoint is a message that the
interpreter sends to it, named for the checkpoint: `CALL`, `COMMAND`,
`REQUIRES`, `LOCAL`, `ENVIRONMENT`, `STREAM` and `METHOD`. The message has one
argument, a `Directory` that the interpreter makes for that message and that
holds the entries that the checkpoint names. The indexes of the entries are in
upper case, and the interpreter reads only entries stored under those
spellings: an entry stored under `'result'` is not the entry `RESULT` ([the
example of the call rule](r:env.security.call) shows it). The entries are
`NAME` and `ARGUMENTS` for `CALL`, and the manager may set `RESULT`; `COMMAND`
and `ADDRESS` for `COMMAND`, and the manager may set `RC`, `FAILURE` and
`ERROR`; `NAME` for `REQUIRES`, which the manager may replace, and it may set
`SECURITYMANAGER`; `NAME` for `LOCAL` and for `ENVIRONMENT`, and the manager
may set `RESULT`; `NAME` for `STREAM`, and the manager sets `STREAM`;
`OBJECT`, `NAME` and `ARGUMENTS` for `METHOD`, and the manager may set
`RESULT`. The manager answers `0`, which lets the action go on as if there
were no manager, or `1`, which says that it dealt with the action and that the
interpreter is to take what the action needs from the entries; any other
answer is an error ([the answer rule](r:env.security.answer)). A manager that
has no method of the name of the checkpoint is sent `UNKNOWN` as every object
is, with the name and an array of the one argument ([the checkpoint
rule](r:class.routine.setsecuritymanager.checkpoints)). The clause of the
governed code that reached the checkpoint goes on only when the manager has
answered. In the example, a routine reaches every checkpoint once. The manager
writes the name of each message it is sent and the indexes of the entries of
its directory; it answers the first two itself and the others with `0`.

```rexx
call sysfiledelete 'envsec_m1.tmp'
call sysfiledelete 'envsec_m2.tmp'
call lineout 'envsec_m1.tmp', 'first line'
call lineout 'envsec_m1.tmp'
call lineout 'envsec_m2.tmp', 'nop'
call lineout 'envsec_m2.tmp'
src = 'say envsecfun(1, 2)'
src = src || '; address system "exit 3"; say rc'
src = src || '; say .envsecname'
src = src || '; say linein("envsec_m1.tmp")'
src = src || '; say .package~new("envsec_m2.tmp")~class~id'
src = src || '; say .box~new~locked(7)'
r = .routine~new('agent', src)
r~setSecurityManager(.mgr~new)
r~call
call sysfiledelete 'envsec_m1.tmp'
call sysfiledelete 'envsec_m2.tmp'
exit
::class box
::method locked protected
  return 'ran'
::class mgr
::method call
  use arg info
  say 'CALL' entries(info) info~name info~arguments~items
  info~result = 'handled'
  return .true
::method command
  use arg info
  say 'COMMAND' entries(info) info~address info~command
  info~rc = 5
  return 1
::method unknown
  use arg name, args
  say name entries(args[1])
  return .false
::routine entries
  return arg(1)~allIndexes~sort~makearray~makestring('l', ',')
```

```output
CALL ARGUMENTS,NAME ENVSECFUN 2
handled
COMMAND ADDRESS,COMMAND SYSTEM exit 3
5
LOCAL NAME
ENVIRONMENT NAME
.ENVSECNAME
STREAM NAME
first line
REQUIRES NAME
REQUIRES NAME
Package
METHOD ARGUMENTS,NAME,OBJECT
ran
```

## What a manager governs

r[env.security.set]
A manager belongs to a package. `setSecurityManager` of a `Package` sets the
manager of that package. `setSecurityManager` of a `Routine` or a `Method`
sets the manager of the package that holds its code, so a routine or method
that `new` or `newFile` made, which has a package of its own, is governed
alone, and several routines and methods that one package holds share one
manager: setting it through one of them replaces what was set through another,
and a call with no argument through any of them removes the manager from the
code of all. In the example, the routine and the method of one package are
governed by the manager of the package, then by the one the routine sets, then
by the one the method sets, and the call with no argument through the method
leaves the package with none, so that both run their commands.

```rexx
src = .array~of('::routine zqr public', '  address system "exit 1"', -
  '  return rc', '::class zqk public', '::method m', -
  '  address system "exit 2"', '  return rc')
p = .package~new('envsec_set', src)
r = p~findRoutine('ZQR')
m = p~findClass('ZQK')~method('M')
k = p~findClass('ZQK')~new
p~setSecurityManager(.mgr~new('package'))
say 'package:' r~call k~m
r~setSecurityManager(.mgr~new('routine'))
say 'by the routine:' r~call k~m
m~setSecurityManager(.mgr~new('method'))
say 'by the method:' r~call k~m
m~setSecurityManager
say 'removed:' r~call k~m
exit
::class mgr
::method init
  expose tag
  use arg tag
::method command
  expose tag
  use arg info
  say tag 'manager:' info~command
  info~rc = 7
  return 1
```

```output
package manager: exit 1
package manager: exit 2
package: 7 7
routine manager: exit 1
routine manager: exit 2
by the routine: 7 7
method manager: exit 1
method manager: exit 2
by the method: 7 7
removed: 1 2
```

r[env.security.scope]
The code of a package is governed by the manager of the package, wherever it
runs: the main code, the routines, the labels that these call, the text that
`INTERPRET` runs for them, the methods of the classes, and what these run on
another activity, which is a method that `start` or a started `Message` runs,
and the rest of a method after `REPLY`.
Code of another package is not governed, whoever calls it: a routine of the
program that made the package, a routine of a package that it requires, an
external program that it calls. These run under the manager of their own
package, or under none. A package that the governed code makes with `new` or
`loadPackage` has no manager unless [the `REQUIRES`
rule](r:env.security.requires) gives it one. A routine of a governed package
that other code calls is governed as it runs: what is governed is the code,
not its caller. In the example, the package has a routine with a label, a
class whose method runs directly and as two started messages, and a required
file. The manager answers each command with its number plus 100, so that the
commands that reach it show as three-digit codes, and the command in the
required file, which does not reach it, shows its own number.

```rexx
call sysfiledelete 'envsec_other.tmp'
call lineout 'envsec_other.tmp', '::routine envsecother public'
call lineout 'envsec_other.tmp', '  address system "exit 6"'
call lineout 'envsec_other.tmp', '  return rc'
call lineout 'envsec_other.tmp'
src = .array~of('::requires "envsec_other.tmp"', -
  '::routine inpkg public', -
  '  address system "exit 1"; r = rc', -
  '  call lab; r = r result', -
  '  interpret "address system ""exit 3""; r = r rc"', -
  '  o = .zqk~new; r = r o~run(4)', -
  '  r = r o~start("run", 5)~result', -
  '  m = .message~new(o, "run", "i", 6); m~start; r = r m~result', -
  '  return r envsecother()', -
  'lab: address system "exit 2"; return rc', -
  '::class zqk', -
  '::method run unguarded', -
  '  use arg n', -
  '  address system "exit" n', -
  '  return rc')
p = .package~new('envsec_scope', src)
p~setSecurityManager(.mgr~new)
say p~findRoutine('INPKG')~call
call sysfiledelete 'envsec_other.tmp'
exit
::class mgr
::method command
  use arg info
  info~rc = 100 + word(info~command, 2)
  return 1
::method unknown
  return 0
```

```output
101 102 103 104 105 106 6
```

r[env.security.run]
A manager that is set, replaced or removed while code of the package is
running does not move the run that is going on, except for the two name
checkpoints. A run is a call of a routine, a run of a method, or the main code
of a program, and the labels that it calls and the text that it runs by
`INTERPRET` are part of it. It takes the
manager that its package has when it starts, and sends `CALL`, `COMMAND`,
`REQUIRES`, `STREAM` and `METHOD` to that manager until it ends: a manager
that the run removes or replaces keeps being asked, and a package that had no
manager when the run started is not asked about them, whatever the run sets.
(A run changes its manager by a message to a protected method, which is itself
a [`METHOD` checkpoint](r:env.security.method).) `LOCAL` and `ENVIRONMENT`
look at the manager that the package has when the name is read, so they follow
a change at once. A run that starts after the change takes the manager that
the package has by then. In the first example the routine replaces its manager
`old` by `new` after its first command: its second command still goes to `old`
and the name goes to `new`, and a second call of the routine, which starts
with `new`, sends everything to `new`. In the second example the program
governs itself. Its own run was already going when the manager was set, so its
commands and its label are not sent to the manager and its name lookups are; a
routine of the program, which starts after the change, is governed.

```rexx
src = 'address system "exit 1";'
src = src '.context~package~setSecurityManager(.mgr~new("new"));'
src = src 'address system "exit 2"; say .envsecname'
r = .routine~new('agent', src)
r~setSecurityManager(.mgr~new('old'))
say 'first run'
r~call
say 'second run'
r~call
exit
::class mgr
::method init
  expose tag
  use arg tag
::method command
  expose tag
  use arg info
  say tag 'COMMAND' info~command
  info~rc = 0
  return 1
::method local
  expose tag
  use arg info
  if info~name = 'ENVSECNAME' then say tag 'LOCAL' info~name
  return 0
::method unknown
  return 0
```

```output
first run
old COMMAND exit 1
old COMMAND exit 2
new LOCAL ENVSECNAME
.ENVSECNAME
second run
new COMMAND exit 1
new COMMAND exit 2
new LOCAL ENVSECNAME
.ENVSECNAME
```

```rexx
.context~package~setSecurityManager(.mgr~new)
address system 'exit 4'
say 'no manager yet:' rc
say .envsecname
call lab
call later
exit
lab:
  address system 'exit 5'
  say 'label:' rc
  return
::routine later
  address system 'exit 6'
  say 'routine:' rc
::class mgr
::method command
  use arg info
  say 'manager got' info~command
  info~rc = 9
  return 1
::method local
  use arg info
  if info~name = 'ENVSECNAME' then do
    info~result = 'provided'
    return 1
  end
  return 0
::method unknown
  return 0
```

```output
no manager yet: 4
provided
label: 5
manager got exit 6
routine: 9
```

r[env.security.own]
The methods of a manager are code of the package that defines the manager's
class. When that package is the governed one, the manager is governed too: a
command that one of its methods runs is a checkpoint, and the manager is sent
it. Nothing ends the repetition but the manager itself. In the example the
`command` method of the manager counts the commands it is sent and runs one
more while the count is below three.

```rexx
src = .array~of('::class zqm public', '::attribute depth public', -
  '::method init', '  self~depth = 0', '::method command', -
  '  use arg info', '  self~depth = self~depth + 1', -
  '  say "command" self~depth info~command', -
  '  if self~depth < 3 then address system "exit" self~depth + 10', -
  '  info~rc = 0', '  return 1', -
  '::routine zqgo public', '  address system "exit 1"')
p = .package~new('envsec_own', src)
p~setSecurityManager(p~findClass('ZQM')~new)
p~findRoutine('ZQGO')~call
```

```output
command 1 exit 1
command 2 exit 11
command 3 exit 12
```

## The checkpoints in detail

r[env.security.call]
The `CALL` checkpoint is sent as [the class
rule](r:class.routine.setsecuritymanager.call) says, and also for the built-in
functions `BEEP`, `DIRECTORY` and `FILESPEC`, which are offered like external
routines. No other built-in function is offered, and the stream functions have
[their own checkpoint](r:env.security.stream). A function of RexxUtil is
offered like one of a library, and so is a program file that the search finds,
at every call that finds it; the program that then runs is code of another
package and is not governed. `NAME` is the name as [the call
rule](r:instr.call.search) reads it: in capitals for a symbol, as it is
written for a string. `ARGUMENTS` is an array with one position for each
argument up to the last one that was written: an argument that was left out in
the middle is an absent position, and arguments left out at the end are not
counted, so the call `f(1, , "x")` has size 3 and two items and `f( , )` has
size 0. A manager that answers `1` takes the call over, and the value of its
`RESULT` entry is the value of the call, `.nil` included. With no entry a
function call raises `SYNTAX 44.1`, and a `CALL` instruction leaves the
variable `RESULT` dropped, as [a call that returns no
value](r:instr.call.result) does. A manager that answers `0` lets the search
go on, and nothing it set in the directory counts: a name that the search does
not find raises `SYNTAX 43.1` although `RESULT` was set, and a changed `NAME`
does not change the routine that is searched for. The first example shows the
name and the arguments, and each way that a call ends. In the second, each
built-in function of this book is called once and the manager notes the names
it is sent; it takes those calls over, so that nothing runs. In the third, a
program file is offered at each of two calls, and the command in it is not,
and so is a function of RexxUtil.

```rexx
tests = .array~of('return envsecfun(1, , "x")', -
  'call "EnvSecMix" 2; return result', -
  'call envsecnone; return symbol("RESULT")', -
  'return envsecnil() == .nil', 'return envsecnone()', -
  'return envsecpoor()', 'return envsecpass()')
do t over tests
  src = 'signal on syntax; ' t '; syntax: return "error" condition("o")~code'
  r = .routine~new('agent', src)
  r~setSecurityManager(.mgr~new)
  say t '=>' r~call
end
exit
::class mgr
::method call
  use arg info
  a = info~arguments
  say '  CALL' info~name a~size a~items
  select
    when info~name = 'ENVSECNONE' then return 1
    when info~name = 'ENVSECNIL' then info~result = .nil
    when info~name = 'ENVSECPOOR' then info['result'] = 'lower case index'
    when info~name = 'ENVSECPASS' then do
      info~result = 'ignored'
      info~name = 'LENGTH'
      return 0
    end
    otherwise info~result = 'R'
  end
  return 1
```

```output
  CALL ENVSECFUN 3 2
return envsecfun(1, , "x") => R
  CALL EnvSecMix 1 1
call "EnvSecMix" 2; return result => R
  CALL ENVSECNONE 0 0
call envsecnone; return symbol("RESULT") => LIT
  CALL ENVSECNIL 0 0
return envsecnil() == .nil => 1
  CALL ENVSECNONE 0 0
return envsecnone() => error 44.1
  CALL ENVSECPOOR 0 0
return envsecpoor() => error 44.1
  CALL ENVSECPASS 0 0
return envsecpass() => error 43.1
```

```rexx
names = .array~of('abbrev', 'abs', 'address', 'arg', 'b2x', 'beep', -
  'bitand', 'bitor', 'bitxor', 'c2d', 'c2x', 'center', 'changestr', -
  'charin', 'charout', 'chars', 'compare', 'condition', 'copies', -
  'countstr', 'd2c', 'd2x', 'datatype', 'date', 'delstr', 'delword', -
  'digits', -
  'directory', 'endlocal', 'errortext', 'filespec', 'format', 'form', -
  'fuzz', 'gc', 'insert', 'lastpos', 'left', 'length', 'linein', 'lineout', -
  'lines', 'lower', 'max', 'min', 'overlay', 'pos', 'qualify', 'queued', -
  'random', 'reverse', 'right', 'rxfuncadd', 'rxfuncdrop', 'rxfuncquery', -
  'rxqueue', 'setlocal', 'sign', 'sourceline', 'space', 'stream', 'strip', -
  'substr', 'subword', 'symbol', 'time', 'trace', 'translate', 'trunc', -
  'upper', 'userid', 'value', 'var', 'verify', 'wordindex', 'wordlength', -
  'word', 'wordpos', 'words', 'x2b', 'x2c', 'x2d', 'xrange')
m = .mgr~new
do n over names
  args = '()'
  if n = 'charout' | n = 'lineout' then args = '("envsecx")'
  src = 'signal on syntax; x =' n || args'; syntax: return 0'
  r = .routine~new('agent', src)
  r~setSecurityManager(m)
  r~call
end
say m~seen~allItems~sort~makearray~makestring('l', ' ')
exit
::class mgr
::attribute seen
::method init
  self~seen = .list~new
::method call
  use arg info
  self~seen~append(info~name)
  info~result = 'R'
  return 1
```

```output
BEEP DIRECTORY FILESPEC
```

```rexx
call sysfiledelete 'envsecprog.rex'
prog = 'address system "exit 8"; return "program" arg(1) rc'
call lineout 'envsecprog.rex', prog
call lineout 'envsecprog.rex'
src = 'say envsecprog("x"); call envsecprog "y"; say result;'
src = src 'address system "exit 9"; say rc;'
src = src 'say sysfileexists("envsec_nofile")'
r = .routine~new('agent', src)
r~setSecurityManager(.mgr~new)
r~call
call sysfiledelete 'envsecprog.rex'
exit
::class mgr
::method call
  use arg info
  say 'CALL' info~name
  return 0
::method command
  use arg info
  say 'COMMAND' info~command
  info~rc = 5
  return 1
```

```output
CALL ENVSECPROG
program x 8
CALL ENVSECPROG
program y 8
COMMAND exit 9
5
CALL SYSFILEEXISTS
0
```

r[env.security.command]
The `COMMAND` checkpoint is sent as [the class
rule](r:class.routine.setsecuritymanager.command) says, for the commands of
[the `ADDRESS` instruction](r:instr.address.command) and those of a clause
that is an expression alone. `ADDRESS` is the name of the environment as the
clause wrote it: in capitals for a symbol, as it is written for a string.
`COMMAND` is the string after its expression is evaluated, and an empty or
blank string is sent like any other. The directory holds just those two
entries, also for a command with a [redirection](r:instr.address.with.syntax).
A manager that answers `1` takes the redirection over with the command:
nothing is read from an `INPUT` source or written to an `OUTPUT` or `ERROR`
target. A clause that only names an environment sends nothing. A command that
names no environment is sent with the name of the current one, which a program
starts with as `sh`. What the manager sets
counts only when it answers `1`. `RC` is stored in the variable `rc` as it is,
whatever it is. `FAILURE` and `ERROR` raise their condition whatever their
value is (`0` and `.false` included) and whatever `RC` is, so an `ERROR` entry
with an `RC` of `0` raises ERROR with `rc` `0`; when both are set, only
FAILURE is raised. A manager that answers `0` leaves the command to run as it
was written, whatever else it set: its `RC` is not used, and a changed
`COMMAND` is not run. In the first example the commands of several clauses are
shown to the manager. In the second, each line is a manager that answers one
of the ways above, to a command that would end with `rc` `4` and has a handler
for both conditions.

```rexx
tests = .array~of('address system "exit 1"', 'address "EnvSec" "exit 2"', -
  'address envsec "exit" 1 + 2', -
  'address system "exit 4" with output normal', 'address system ""', -
  'address system "   "', -
  'o.0 = "untouched"; address system "echo hi" with output stem o.;' -
  'say o.0', -
  'address envsec')
do t over tests
  say '==' t
  r = .routine~new('agent', t)
  r~setSecurityManager(.mgr~new)
  r~call
end
exit
::class mgr
::method command
  use arg info
  say '  ADDRESS <'info~address'> COMMAND <'info~command'>' -
    info~allIndexes~sort~makearray~makestring('l', ',')
  info~rc = 0
  return 1
```

```output
== address system "exit 1"
  ADDRESS <SYSTEM> COMMAND <exit 1> ADDRESS,COMMAND
== address "EnvSec" "exit 2"
  ADDRESS <EnvSec> COMMAND <exit 2> ADDRESS,COMMAND
== address envsec "exit" 1 + 2
  ADDRESS <ENVSEC> COMMAND <exit 3> ADDRESS,COMMAND
== address system "exit 4" with output normal
  ADDRESS <SYSTEM> COMMAND <exit 4> ADDRESS,COMMAND
== address system ""
  ADDRESS <SYSTEM> COMMAND <> ADDRESS,COMMAND
== address system "   "
  ADDRESS <SYSTEM> COMMAND <   > ADDRESS,COMMAND
== o.0 = "untouched"; address system "echo hi" with output stem o.; say o.0
  ADDRESS <SYSTEM> COMMAND <echo hi> ADDRESS,COMMAND
untouched
== address envsec
```

```rexx
tests = .array~of('RC 7', 'RC text', 'answer 1 only', 'ERROR 0', -
  'FAILURE and ERROR', 'FAILURE false', 'answer 0 RC', 'answer 0 COMMAND')
src = 'signal on error name e; signal on failure name f;'
src = src 'address system "exit 4"; return "rc" rc;'
src = src 'e: return "error" rc; f: return "failure" rc'
do i = 1 to tests~items
  r = .routine~new('agent', src)
  r~setSecurityManager(.mgr~new(i))
  say left(tests[i], 20) r~call
end
exit
::class mgr
::method init
  expose i
  use arg i
::method command
  expose i
  use arg info
  select
    when i = 1 then info~rc = 7
    when i = 2 then info~rc = 'abc'
    when i = 3 then nop
    when i = 4 then do; info~rc = 0; info~error = 0; end
    when i = 5 then do; info~rc = 8; info~error = 1; info~failure = 1; end
    when i = 6 then do; info~rc = 9; info~failure = .false; end
    when i = 7 then do; info~rc = 3; return 0; end
    when i = 8 then do; info~command = 'exit 0'; return 0; end
  end
  return 1
```

```output
RC 7                 rc 7
RC text              rc abc
answer 1 only        rc 0
ERROR 0              error 0
FAILURE and ERROR    failure 8
FAILURE false        failure 9
answer 0 RC          error 4
answer 0 COMMAND     error 4
```

r[env.security.names]
The checkpoints `LOCAL` and `ENVIRONMENT` are sent as [the class
rules](r:class.routine.setsecuritymanager.local) say, for every environment
symbol that gets as far as `.local`. That includes the symbols that the
interpreter answers itself, `.environment`, `.local`, `.context`, `.methods`,
`.routines`, `.rs` and `.line`, so a manager can provide `.context` to the
code it governs. A name that `.local` already holds, such as `.stdout`, is
asked of `LOCAL` only. A manager that answers `1` and sets `RESULT` provides
the object that the symbol is, and the lookup stops there; `.nil` is an
object, so a manager can provide it, and the symbol is then found and is
`.nil`. A manager that answers `1` and sets no `RESULT` says that it found
nothing, and the lookup goes on to the next place. A `RESULT` that a manager
sets while it answers `0` is not used. In the example the manager notes the
messages it is sent for each symbol, except where it provides the name: it
provides `.envsecnil` and `.context` at `LOCAL`, finds nothing for
`.envsecnone` at `ENVIRONMENT`, and sets a `RESULT` for `.envsecignored` that
it does not use.
The function `VALUE` with the selector `ENVIRONMENT` reads a variable of the
host, not `.environment`, and sends no checkpoint.

```rexx
tests = .array~of('say (.envsecnil == .nil)', 'say .envsecnone', -
  'say .envsecignored', 'say .context~class~id', -
  'say .stdout~class~id')
do t over tests
  say '==' t
  r = .routine~new('agent', t)
  r~setSecurityManager(.mgr~new)
  r~call
end
exit
::class mgr
::method unknown
  use arg name, args
  info = args[1]
  if info~name = 'ENVSECNIL' & name = 'LOCAL' then do
    info~result = .nil
    return 1
  end
  if info~name = 'ENVSECNONE' & name = 'ENVIRONMENT' then return 1
  if info~name = 'ENVSECIGNORED' then info~result = 'not used'
  if info~name = 'CONTEXT' & name = 'LOCAL' then do
    info~result = 'provided context'
    return 1
  end
  say '  ' name info~name
  return 0
```

```output
== say (.envsecnil == .nil)
1
== say .envsecnone
   LOCAL ENVSECNONE
.ENVSECNONE
== say .envsecignored
   LOCAL ENVSECIGNORED
   ENVIRONMENT ENVSECIGNORED
.ENVSECIGNORED
== say .context~class~id
String
== say .stdout~class~id
   LOCAL STDOUT
Stream
```

r[env.security.stream]
The `STREAM` checkpoint is sent when one of the stream functions `CHARIN`,
`CHAROUT`, `CHARS`, `LINEIN`, `LINEOUT`, `LINES` and `STREAM`, with any
arguments, names a stream whose absolute name is not in the program's table of
streams, except for the standard streams, which [the next
rule](r:env.security.standard) takes out. `NAME` is the name that the function
was given, made absolute as [`QUALIFY` does](r:bif.qualify.relative). The
table is one for the whole program and is keyed by the absolute name, so `x`
and `./x` are one entry. A stream function that names a stream enters it in
the table, whether the code is governed or not and whether or not the stream
can be opened. The name is then not offered again, to any manager, until a
`close` command of `STREAM`, or an `open` command of `STREAM` that fails,
takes it out. A `close` sent to another `Stream` object of the same file does
not. A stream that a manager chose is in the table under that name, so the
code that no manager governs gets it as well. A manager that answers `1` and
sets `STREAM` gives the function that object, which must understand the
messages of a stream; one that answers `1` and sets no `STREAM` leaves the
function to open the name as a file, as an answer of `0` does. In the example
the program has the file open when the first manager is to be asked, so it is
not asked. After the program closes the stream, the first manager is asked and
provides a stream of another file; the second manager is not asked about that
name while the stream is open, and the program gets the same stream. After the
stream is closed once more the name is the real file again, and the second
manager is asked. The second example shows the table for a file that does not
exist: the first `lines` is offered and enters the name, the second and the
one that spells the name with `./` are not, a failing `open` takes the name
out, a `close` sent to a `Stream` object does not, and the `close` command
does.

```rexx
f = 'envsec_real.tmp'
call sysfiledelete f
call sysfiledelete 'envsec_repl.tmp'
call lineout f, 'real text'
call lineout f
call lineout 'envsec_repl.tmp', 'replacement text'
call lineout 'envsec_repl.tmp'
r1 = .routine~new('agent1', 'return linein("envsec_real.tmp", 1, 1)')
r1~setSecurityManager(.mgr~new('one'))
r2 = .routine~new('agent2', 'return charin("envsec_real.tmp", 1, 4)')
r2~setSecurityManager(.mgr~new('two'))
say 'the program opens it:' chars(f)
say 'managed, open:' r1~call
call stream f, 'c', 'close'
say 'managed, closed:' r1~call
say 'second manager:' r2~call
say 'the program:' charin(f, 1, 4)
call stream f, 'c', 'close'
say 'the program, closed:' charin(f, 1, 4)
call stream f, 'c', 'close'
say 'second manager:' r2~call
call stream f, 'c', 'close'
call sysfiledelete f
call sysfiledelete 'envsec_repl.tmp'
exit
::class mgr
::method init
  expose tag
  use arg tag
::method stream
  expose tag
  use arg info
  say tag 'asked, name is absolute:' (info~name == qualify(info~name))
  info~stream = .stream~new('envsec_repl.tmp')
  return 1
```

```output
the program opens it: 10
managed, open: real text
one asked, name is absolute: 1
managed, closed: replacement text
second manager: repl
the program: repl
the program, closed: real
two asked, name is absolute: 1
second manager: repl
```

```rexx
call sysfiledelete 'envsec_tab.tmp'
steps = .array~of('G:lines("envsec_tab.tmp")', 'G:lines("envsec_tab.tmp")', -
  'G:lines("./envsec_tab.tmp")', -
  'P:stream("envsec_tab.tmp", "c", "open read")', -
  'G:lines("envsec_tab.tmp")', 'G:lines("envsec_tab.tmp")', -
  'P:.stream~new("envsec_tab.tmp")~close', 'G:lines("envsec_tab.tmp")', -
  'P:stream("envsec_tab.tmp", "c", "close")', 'G:lines("envsec_tab.tmp")')
do s over steps
  parse var s who ':' t
  if who = 'P' then do
    interpret 'x =' t
    say 'program:' t
  end
  else do
    say 'governed:' t
    r = .routine~new('agent', 'x =' t)
    r~setSecurityManager(.mgr~new)
    r~call
  end
end
exit
::class mgr
::method unknown
  say '  offered'
  return 0
```

```output
governed: lines("envsec_tab.tmp")
  offered
governed: lines("envsec_tab.tmp")
governed: lines("./envsec_tab.tmp")
program: stream("envsec_tab.tmp", "c", "open read")
governed: lines("envsec_tab.tmp")
  offered
governed: lines("envsec_tab.tmp")
program: .stream~new("envsec_tab.tmp")~close
governed: lines("envsec_tab.tmp")
program: stream("envsec_tab.tmp", "c", "close")
governed: lines("envsec_tab.tmp")
  offered
```

r[env.security.standard]
A stream function that uses a standard stream sends no checkpoint at all:
neither `STREAM` nor `LOCAL`. This covers a function whose stream name is
omitted or empty, and one that names `STDIN`, `STDOUT` or `STDERR`, in any
case and with or without a colon after it. The symbols `.output`, `.input`
and `.error` send `LOCAL` when the code writes them, as [the name
rule](r:env.security.names) says, but the functions that reach those objects
without a symbol do not. In the example each line reaches a standard stream
from governed code, and the manager, which writes every message it is sent,
is sent none.

```rexx
tests = .array~of('call lineout , "one"', -
  'call charout "", "two" || "0a"x', 'call lineout "stdout:", "three"', -
  'x = stream("STDIN", "c", "query exists")', 'x = stream("stderr", "s")')
do t over tests
  r = .routine~new('agent', t)
  r~setSecurityManager(.mgr~new)
  r~call
end
say 'no checkpoint'
exit
::class mgr
::method unknown
  use arg name, args
  say '  ' name args[1]~name
  return 0
```

```output defect=#212
one
two
three
no checkpoint
```

r[env.security.requires]
The `REQUIRES` checkpoint is sent as [the class
rule](r:class.routine.setsecuritymanager.requires) says. A manager that
answers `1` and sets `SECURITYMANAGER` to an object gives the package that is
loaded a manager of its own: the main code of the package and every routine
and method of it is governed by that object from then on, also when other code
calls them, and the `::REQUIRES` directives of the package are sent to that
object and not to the manager that was asked first. When the file is offered
twice, a `SECURITYMANAGER` set in either message that the manager answered
with `1` gives the package its manager, whatever the manager answers to the
other message. A manager that answers
`0`, or that sets `SECURITYMANAGER` to `.nil`, leaves the package with none,
and so does a manager that sets nothing; a `::REQUIRES` of such a package is
sent to no manager. In the example three files are loaded, each with a command
in its main code; the first two also have a `::REQUIRES` of a file that does
nothing. The manager answers `1` with an inner manager for the first file, `0`
with one for the second and `1` with `.nil` for the third. It notes each
message it is sent, and so does the inner manager, which also answers every
command with `rc` `5`.

```rexx
files = .array~of('envsec_r0', 'envsec_q0', 'envsec_r1', 'envsec_r2', -
  'envsec_r3')
do f over files
  call sysfiledelete f'.tmp'
  call lineout f'.tmp', 'nop'
  call lineout f'.tmp'
end
do n = 1 to 3
  f = 'envsec_r'n'.tmp'
  call lineout f, 'address system "exit 4"; say "loaded' n 'rc" rc'
  if n < 3 then do
    call lineout f, '::requires "envsec_'substr('rq', n, 1)'0.tmp"'
  end
  call lineout f
end
do n = 1 to 3
  r = .routine~new('agent', 'x = .package~new("envsec_r'n'.tmp")')
  r~setSecurityManager(.mgr~new(n))
  r~call
end
do f over files
  call sysfiledelete f'.tmp'
end
exit
::class mgr
::method init
  expose n
  use arg n
::method requires
  expose n
  use arg info
  say 'manager' n 'asked'
  if n = 1 then info~securitymanager = .inner~new
  if n = 2 then info~securitymanager = .inner~new
  if n = 3 then info~securitymanager = .nil
  return n \= 2
::class inner
::method command
  use arg info
  info~rc = 5
  return 1
::method unknown
  use arg name, args
  say 'inner asked' name
  return 0
```

```output
manager 1 asked
manager 1 asked
inner asked REQUIRES
inner asked REQUIRES
loaded 1 rc 5
manager 2 asked
manager 2 asked
loaded 2 rc 4
manager 3 asked
manager 3 asked
loaded 3 rc 4
```

r[env.security.method]
The `METHOD` checkpoint is sent as [the class
rule](r:class.routine.setsecuritymanager.method) says. A method is protected
when [`PROTECTED` was declared](r:dir.method.protected) or
[`setProtected`](r:class.method.setprotected) was sent, and the built-in
methods that are protected are the ones that the second example prints. What
is checked is the send, by governed code, to a protected method, in any form
of a send: `send`, `sendWith`, the cascade `~~`, the `send` of a `Message` and
a send to `self` are all offered, and for an attribute the setter has the name
with `=` after it. The messages that the interpreter and the built-in methods
send while they carry out a clause of governed code are offered in the same
way: the `INIT` that `new` sends, the `MAKESTRING` that a string value asks
for (`SAY`, concatenation, `request`), the `MAKEARRAY` that `DO OVER` asks
for, and a message that a built-in method sends to an argument, such as the
`COMPARE` that `sortWith` sends. `FORWARD`, an operator, a message to the
superclass (`~name:super`) and a protected class method are sends like the
others, and for a class method `OBJECT` is the class. A message that the
receiver has no method for, and that goes on to the receiver's `UNKNOWN`
method, is not a checkpoint, even when `UNKNOWN` is protected: the message
that the code sent names no protected method. A message that is started is not
offered: the `start` of an object and of a `Message` run the method on another
activity with no checkpoint. A protected method is run without a checkpoint
when the sender is code with no manager, as the class rule says. The
`setSecurityManager` methods are protected, so governed code that changes its
manager is passed through the manager, which can stop the change by answering
`1`; the third example shows that the manager is still in place for the next
run. In the first example, a package that has a manager sends messages to a
protected method in the forms above, and the manager answers `1` with a result
for those it is sent. The fourth example shows the messages that the
interpreter and the built-in methods send, and the fifth the fallback to
`UNKNOWN`.

```rexx
src = .array~of('::class zqbox public', -
  '::method locked protected unguarded', '  return "locked ran" arg(1)', -
  '::attribute att protected', -
  '::method viaself unguarded', '  return self~locked(1)', -
  '::method viasend unguarded', '  return self~send("locked", 2)', -
  '::method viasendwith unguarded', -
  '  return self~sendWith("locked", .array~of(3))', -
  '::method viacascade unguarded', '  self~~locked(4); return "cascade"', -
  '::method viamessage unguarded', -
  '  return .message~new(self, "locked", "i", 5)~send', -
  '::method viastart unguarded', '  return self~start("locked", 6)~result', -
  '::method viamsgstart unguarded', -
  '  m = .message~new(self, "locked", "i", 7); m~start; return m~result', -
  '::method setatt unguarded', '  self~att = 8; return "att set"')
p = .package~new('envsec_method', src)
p~setSecurityManager(.mgr~new)
o = p~findClass('ZQBOX')~new
do m over .array~of('viaself', 'viasend', 'viasendwith', 'viacascade', -
    'viamessage', 'viastart', 'viamsgstart', 'setatt')
  say m '=>' o~send(m)
end
say 'outside the package:' o~locked(9)
exit
::class mgr
::method method
  use arg info
  say '  METHOD' info~name info~object~class~id info~arguments~items
  info~result = 'replaced'
  return 1
```

```output
  METHOD LOCKED ZQBOX 1
viaself => replaced
  METHOD LOCKED ZQBOX 1
viasend => replaced
  METHOD LOCKED ZQBOX 1
viasendwith => replaced
  METHOD LOCKED ZQBOX 1
viacascade => cascade
  METHOD LOCKED ZQBOX 1
viamessage => replaced
viastart => locked ran 6
viamsgstart => locked ran 7
  METHOD ATT= ZQBOX 1
setatt => att set
outside the package: locked ran 9
```

```rexx
seen = .set~new
list = .set~new
do name over .environment~allIndexes
  c = .environment[name]
  if \c~isA(.class) then iterate
  if seen~hasItem(c) then iterate
  seen~put(c)
  call scan c, c~methods(.nil)
  call scan c, c~instanceMethods(c)
end
do line over list~allItems~sort
  say line
end
exit
scan: procedure expose list
  use arg c, ms
  if ms = .nil then return
  do while ms~available
    if ms~item~isProtected then list~put(c~id ms~index)
    ms~next
  end
  return
```

```output
Class BASECLASS
Class DEFINE
Class DEFINEMETHODS
Class DELETE
Class METACLASS
Class SUBCLASSES
Class SUPERCLASS
Class SUPERCLASSES
Class UNINHERIT
Directory SETMETHOD
Directory UNSETMETHOD
Method SETPROTECTED
Method SETSECURITYMANAGER
Package DEFAULTOPTIONS
Package OPTIONS
Package SETSECURITYMANAGER
Routine SETSECURITYMANAGER
```

```rexx
src = '.context~package~setSecurityManager;'
src = src 'address system "exit 1"; return rc'
r = .routine~new('agent', src)
r~setSecurityManager(.mgr~new)
say 'first run:' r~call
say 'second run:' r~call
exit
::class mgr
::method method
  use arg info
  say '  METHOD' info~name
  return 1
::method command
  use arg info
  say '  COMMAND' info~command
  info~rc = 5
  return 1
::method unknown
  return 0
```

```output
  METHOD SETSECURITYMANAGER
  COMMAND exit 1
first run: 5
  METHOD SETSECURITYMANAGER
  COMMAND exit 1
second run: 5
```

```rexx
tests = .array~of('x = .envsecp~new', 'say .envsecq~new', -
  'say .envsecq~new || "!"', 'do i over .envsecs~new; say i; end', -
  'say .array~of(2, 1)~sortWith(.envsecc~new)~makestring("l", " ")')
do t over tests
  say '==' t
  r = .routine~new('agent', t)
  r~setSecurityManager(.mgr~new)
  r~call
end
exit
::class envsecp
::method init protected
  say 'init ran'
::class envsecq
::method makestring protected
  return 'makestring ran'
::class envsecs
::method makearray protected
  return .array~of('makearray ran')
::class envsecc
::method compare protected
  use arg a, b
  return a - b
::class mgr
::method method
  use arg info
  say '  METHOD' info~name info~arguments~items
  return 0
```

```output
== x = .envsecp~new
  METHOD INIT 0
init ran
== say .envsecq~new
  METHOD MAKESTRING 0
makestring ran
== say .envsecq~new || "!"
  METHOD MAKESTRING 0
makestring ran!
== do i over .envsecs~new; say i; end
  METHOD MAKEARRAY 0
makearray ran
== say .array~of(2, 1)~sortWith(.envsecc~new)~makestring("l", " ")
  METHOD COMPARE 2
1 2
```

```rexx
src = .array~of('::class zqu public', -
  '::method unknown protected unguarded', '  return "unknown ran" arg(1)', -
  '::method try unguarded', '  return self~nosuch')
p = .package~new('envsec_unknown', src)
p~setSecurityManager(.mgr~new)
say p~findClass('ZQU')~new~try
exit
::class mgr
::method method
  use arg info
  say '  METHOD' info~name
  info~result = 'replaced'
  return 1
```

```output defect=#225
unknown ran NOSUCH
```

## Answers and errors

r[env.security.answer]
Every checkpoint takes the same answers. The answer is `0` or `1` and nothing
else: the strings `0` and `1` and the objects `.false` and `.true` are
accepted, and any other value, such as `2`, `01`, ` 1`, the empty string or an
object, raises `SYNTAX 34.903`. A method that ends without an answer raises
`SYNTAX 91.999`, whether it falls off its end or ends with a `return` or an
`exit` that has no value. The error is raised by the clause that reached the
checkpoint, in the code that the manager governs, so a handler of that code
receives it as it does any error of the clause. In the example, one manager
per checkpoint answers each of five ways, and the table shows what the
governed code gets. The answers `0` and `1` are shown by the examples of the
class rules.

```rexx
names = .array~of('CALL', 'COMMAND', 'LOCAL', 'ENVIRONMENT', 'REQUIRES', -
  'STREAM', 'METHOD')
answers = .array~of('2', ' 1', '01', '', 'none')
n = 0
do cp over names
  line = left(cp, 12)
  do a over answers
    n = n + 1
    src = 'signal on syntax;' act(cp, n)
    src = src || '; return "no error";'
    src = src 'syntax: return "error" condition("o")~code'
    r = .routine~new('agent', src)
    r~setSecurityManager(.mgr~new(cp, a))
    line = line left(r~call, 12)
  end
  say line
end
exit
act: procedure
  use arg cp, n
  select
    when cp = 'CALL' then return 'x = envsecfun()'
    when cp = 'COMMAND' then return 'address system "exit 1"'
    when cp = 'LOCAL' | cp = 'ENVIRONMENT' then return 'x = .envsecname'
    when cp = 'REQUIRES' then return 'x = .package~new("envsec_gone'n'.tmp")'
    when cp = 'STREAM' then return 'x = lines("envsec_gone'n'.tmp")'
    otherwise return 'x = .envsecbox~new~locked'
  end
::class envsecbox
::method locked protected
  return 'ran'
::class mgr
::method init
  expose cp a
  use arg cp, a
::method unknown
  expose cp a
  use arg name, args
  if name \= cp then return 0
  if name = 'LOCAL' | name = 'ENVIRONMENT' then do
    if args[1]~name \= 'ENVSECNAME' then return 0
  end
  if name = 'CALL' then if args[1]~name \= 'ENVSECFUN' then return 0
  if name = 'METHOD' then if args[1]~name \= 'LOCKED' then return 0
  if a = 'none' then return
  return a
```

```output
CALL         error 34.903 error 34.903 error 34.903 error 34.903 error 91.999
COMMAND      error 34.903 error 34.903 error 34.903 error 34.903 error 91.999
LOCAL        error 34.903 error 34.903 error 34.903 error 34.903 error 91.999
ENVIRONMENT  error 34.903 error 34.903 error 34.903 error 34.903 error 91.999
REQUIRES     error 34.903 error 34.903 error 34.903 error 34.903 error 91.999
STREAM       error 34.903 error 34.903 error 34.903 error 34.903 error 91.999
METHOD       error 34.903 error 34.903 error 34.903 error 34.903 error 91.999
```

r[env.security.raise]
A manager may raise instead of answering, and the usual way is `RAISE SYNTAX
98.948`, the error for an action that is not allowed. The condition is raised
in the governed code, at the clause that reached the checkpoint, and the
handlers of that code are the ones that see it, as [the class
rule](r:class.routine.setsecuritymanager.checkpoints) says for `SYNTAX`. This
holds for the other conditions too: `RAISE ERROR`, `FAILURE`, `HALT`,
`NOTREADY` and `USER` in a manager method are handled by the matching `SIGNAL
ON` of the governed code, with the description that the `RAISE` gave. An error
of the manager's own code, such as a division by zero or a message that an
object does not understand, is raised there in the same way. In the example
the manager of each line raises one of these when it is asked about a command,
and the handlers of the governed code write what they got. The condition
object of a condition that the manager raises, and of an error in the
manager's code, has the position of the manager's clause that raised it, while
`SIGL` in the handler is the line of the governed clause that reached the
checkpoint. The errors of [a wrong answer](r:env.security.answer) have the
position of the governed clause. In the second example the manager raises in
the first run and answers wrongly in the second, and the program also prints
the line of the `RAISE`.

```rexx
acts = .array~of('syntax 98.948 array("blocked")', 'error 5', 'failure 6', -
  'user envsec description("a user condition")', -
  'halt description("a halt")', 'notready description("not ready")', -
  'zero', 'missing')
src = 'signal on syntax name s; signal on error name e;'
src = src 'signal on failure name f; signal on halt name h;'
src = src 'signal on notready name n; signal on user envsec name u;'
src = src 'address system "exit 1"; return "ran";'
src = src 's: return condition("c")"/"condition("o")~code;'
src = src 'e: return condition("c")"/"condition("d");'
src = src 'f: return condition("c")"/"condition("d");'
src = src 'h: return condition("c")"/"condition("d");'
src = src 'n: return condition("c")"/"condition("d");'
src = src 'u: return condition("c")"/"condition("d")'
do a over acts
  r = .routine~new('agent', src)
  r~setSecurityManager(.mgr~new(a))
  say left(a, 44) r~call
end
exit
::class mgr
::method init
  expose a
  use arg a
::method command
  expose a
  if a = 'zero' then x = 1 / 0
  if a = 'missing' then self~envsecmissing
  interpret 'raise' a
  return 1
```

```output
syntax 98.948 array("blocked")               SYNTAX/98.948
error 5                                      ERROR/
failure 6                                    FAILURE/
user envsec description("a user condition")  USER ENVSEC/a user condition
halt description("a halt")                   HALT/a halt
notready description("not ready")            NOTREADY/not ready
zero                                         SYNTAX/42.3
missing                                      SYNTAX/97.1
```

```rexx
src = .array~of('signal on syntax', 'address system "exit 1"', -
  'return "no error"', 'syntax: c = condition("o")', -
  '  return c~code "at line" c~position "sigl" sigl')
do mode over .array~of('raise', 'answer')
  r = .routine~new('agent', src)
  r~setSecurityManager(.mgr~new(mode))
  say mode '=>' r~call
end
say 'the raise is at line' .envsecline
exit
::class mgr
::method init
  expose mode
  use arg mode
::method command
  expose mode
  if mode = 'answer' then return 2
  .local~envsecline = .line + 1
  raise syntax 98.948 array('not allowed')
```

```output
raise => 98.948 at line 19 sigl 2
answer => 34.903 at line 2 sigl 2
the raise is at line 19
```
