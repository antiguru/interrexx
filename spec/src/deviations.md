---
element: "Deviations"
kind: chapter
---

# Deviations

A Deviation is a declared difference between interrexx and ooRexx. Every
example on which the two answer differently belongs to one; any other
difference is a defect. Each entry says what interrexx does and how ooRexx
differs. A number is never reused: a withdrawn Deviation keeps its number
and says what replaced it. This chapter is informative; where a Deviation
touches a page of this book, that page states it as a rule whose identifier
ends `deviation.N`.

## 0. Trace indentation

interrexx indents trace lines, and the indentation a program reads back
through `~traceLine` or a condition's traceback, by block structure alone.
ooRexx keeps the indent in a counter that drops a level too far when a loop
ends on a failing control test, so its later lines sit one level short until
an enclosing construct restores it.

## 1. DO OVER order on a stem

DO OVER on a stem visits its tails in an order this book does not specify,
and so do the `Stem` methods that list or search its tails: `allIndexes`,
`allItems`, `makeArray` and `supplier`, the tail that `index` and `removeItem`
choose, and the order in which `hasItem`, `index` and `removeItem` compare the
items. ooRexx visits them in an order fixed by its internal storage, which is
neither insertion nor sorted order.

## 2. Withdrawn

interrexx evaluates an expression of any number of terms at one level.

## 3. Withdrawn

POS past the end of the haystack: current ooRexx agrees with interrexx.

## 4. Identity of equal values

Whether two equal strings or numbers produced separately are one object, and
whether an operation that leaves a value unchanged answers the same object, is
not specified; it shows only through `identityHash` and the identity
collection `IdentityTable`. ooRexx's answers follow how each value was built.
An empty stem's name in `USE ARG` is compared by its characters, where ooRexx
compares identity.

## 5. Withdrawn

interrexx now runs a class object's UNINIT at the collection that reclaims
it, as ooRexx does.

## 6. Finalizer timing after a forced collection

A forced collection can reclaim an object dropped just after it was made, so
its UNINIT may run earlier relative to the program's other output. ooRexx
keeps its most recent allocations alive through a forced collection.
Only the position of the finalizer's output differs, and the same holds for
a `~subclasses` count and a `WeakReference`.

## 7. Order of concurrent trace lines

When two activities trace at once, the order in which their lines reach
standard error is not specified. ooRexx's order varies from run to run;
interrexx gives one of the orders ooRexx can give.

## 8. Withdrawn

Output that lists a hash collection now matches ooRexx; hash iteration order
is [Deviation 33](#33-hash-iteration-order).

## 9. MutableBuffer storage held by a native call

A native routine that holds the address of a MutableBuffer's data keeps
reading valid storage after the buffer grows, even across a collection. In
ooRexx a collection reclaims the old storage and the routine reads garbage.

## 10. Signals halt and wake

SIGINT and SIGTERM halt the program with `4.1` even when SIGHUP is ignored,
as under `nohup`; ooRexx installs no handler then and the signal kills it.
A halt wakes every waiting activity, including one in SysSleep, GUARD WHEN or
a semaphore wait, which ooRexx can leave waiting for ever. A halted wait for
a command sets RC to -4, where ooRexx's RC is unpredictable.

## 11. SIGPIPE in a command's child

A command started by ADDRESS starts with SIGPIPE at its default action. Under
ooRexx the child inherits SIGPIPE ignored, so a pipeline such as
`yes | head -1` reports a broken pipe there and nothing here.

## 12. Reads a halt does not end

A halt ends the wait for a command and for a read of the default input
stream, but not a read of another stream; the halt is taken when that read
returns. For a redirected command interrupted by a signal, ooRexx raises
`98.923`, while interrexx keeps the output so far and raises `4.1`.

## 13. Reading the default input stream

While one activity reads the default input stream, every other activity
waits; in ooRexx they keep running. A halt during the read therefore misses an
activity that has not yet started to run.

## 14. A nested wait only a buried activity can end

An activity waiting inside a nested construct, such as INTERPRET, for
something only an activity suspended beneath it can provide is refused, exit
code 120. ooRexx runs each activity on its own thread and completes.

## 15. Withdrawn

A REPLY inside a labelled DO group is a known defect of interrexx, #101.

## 16. Blocking operations hold every activity

While a stream builtin, SAY to a redirected `.OUTPUT`, trace output, ADDRESS
WITH redirection, condition handling, package loading or a library routine
call is blocked, every other activity waits. In ooRexx the others run.

## 17. HALT of an activity with a nested region

A HALT sent to an activity that has a nested region running beneath it is
taken only when that region ends. ooRexx takes it at the activity's next
clause.

## 18. Nested busy-waiters

Two activities that busy-wait on each other inside nested constructs never
end. ooRexx preempts both and they complete.

## 19. Callback from a foreign thread

A callback into the interpreter from a thread that runs no native call waits
for a native call that holds the interpreter; if that call joins the thread,
the program hangs. ooRexx aborts. Both fail.

## 20. A wait nothing can end

A wait that no other activity, timer or call can end, such as the result of
a message never sent or a GUARD WHEN nothing can make true, is refused, exit
code 120. ooRexx blocks for ever.

## 21. Activities at program end

Program end does not wait for activities that termination-time UNINIT
methods start, REPLY continuations included, nor for any activity once the
main program ends with a refusal. ooRexx's such activities race its exit and
often print.

## 22. One schedule for racing activities

Activities switch only at slice ends, waits and ends, chosen by one
scheduler, so a program whose ooRexx output depends on thread timing gets
one order here, which ooRexx may give rarely or not at all. This covers how
a REPLY's sender and continuation interleave and the clause at which a HALT
from another activity lands.

## 23. Threads of native calls

Successive native calls of one activity may run on different threads, so a
thread-affine extension sees more than one thread. ooRexx keeps each
activity on one thread. The depth at which recursion through native callbacks
raises `11.1` differs too.

## 24. A call context used by another activity

A native call context kept and used from another activity's call answers its
digits, fuzz, form and routine, and raises `98.983` for every other query.
In ooRexx some of those queries crash and others hang.

## 25. DO loop tests on a user comparison

When a DO loop's TO test or BY direction test calls a comparison method of an
object, the answer is read as a logical value, as WHILE reads one: `1` ends
the loop, `0` goes on, and anything else raises `34.901`. ooRexx ends the
loop only when the answer is the `.true` object itself, and goes on for any
other answer, a computed `1` included.

## 26. `=` at the debug pause after an empty DO

Entering `=` at the debug pause after a DO with no passes runs the DO again
and continues. ooRexx damages its record of open blocks and later raises
`10.1` at a valid END.

## 27. A `.DebugInput` answer that is not a string

An answer that is not a string is read as its text: an Array as its lines,
any other object as its default string. ooRexx reads the object as if it
were a string, running garbage or crashing.

## 28. A built-in method run on the wrong object

A built-in class's method run on an object of an unrelated class, through
`run`, `setMethod` or `enhanced`, is refused, exit code 120. ooRexx reads the
object with the wrong layout and answers garbage or crashes.

## 29. An entry assignment with no value

`d~"A="()` on a Directory, a StringTable or a Properties, with no argument,
is refused, exit code 120, and so is `d~unknown('A=', args)` sent directly
with an argument array that holds no item. ooRexx stores an unrelated leftover
value for the first and removes the entry for the second.

## 30. A STRING method answering a non-string

When an object's `STRING` method answers an object with no string value,
interrexx gives ooRexx's answer wherever that is defined, and elsewhere ends
the program with exit code 120 after raising NOSTRING if it is trapped.
ooRexx reads `.nil` as a string there, with garbage results or a crash. The
[conventions](r:conv.string.deviation.30) state it for builtin arguments.

## 31. Withdrawn

Replaced by [Deviation 36](#36-strings-and-numbers-are-value-based).

## 32. A NEW that answers another object

When a class-side NEW reached through `enhanced` answers an object it did
not build, that object comes back unchanged; ooRexx marks it enhanced, which
changes every object of its class, every String for example. A metaclass NEW
that answers a class that `subclass`, `mixinclass` or a `::CLASS` directive has
already completed, or a class of the library such as `Array`, while a class
with that metaclass is made, by `subclass`, by `mixinclass` or by a `::CLASS`
directive with METACLASS, is refused, exit code 120, where ooRexx rewires that
class in place.

## 33. Hash iteration order

The order in which a hash collection yields its entries, and the order of
anything that lists one, is not specified, and may differ from ooRexx's.
Which entries are yielded, and how often, is specified. Which stored indexes
or items a search compares, by `==` or `hashCode`, beyond those it must
compare, and how often, is not specified either.

## 34. `..` in a file name

In every name interrexx qualifies (QUALIFY, streams, `.File` and the
RexxUtil path arguments), each `..` removes exactly the one component before
it. ooRexx removes none after a doubled slash and two after a component that
starts with a period, so its answer can name a different file.

## 35. HANDLE: streams

`HANDLE:0`, `HANDLE:1` and `HANDLE:2` read and write standard input, output
and error; any other descriptor is refused when the stream is created. In
ooRexx a `HANDLE:` stream never reaches its descriptor: reads see end of
file, and writes crash.

## 36. Strings and numbers are value-based

A String with no subclass and no enhancement, every number included, is
value-based: whether equal values are one object, whether `~copy` answers a
new object, and when a WeakReference to one clears are not specified, and
`objectName=` on one is refused, exit code 120. ooRexx keeps per-object state
on strings, so a name set on one value can appear on another.

## 37. SysStemSort writes back by tail

SysStemSort assigns the sorted values to the stem by tail. If a STRING
method run during the sort empties the stem, ooRexx writes into variables the
stem no longer holds and leaves it empty.

## 38. TO or BY after the FOR of DO OVER or DO WITH

`TO` or `BY` after the FOR count of a DO OVER or DO WITH raises `25.927`,
naming FOR and the keyword found. ooRexx raises the internal error `49.2`
with an internal code in its message.

## 39. A label spelled THEN

A label spelled `THEN` where IF or WHEN expects THEN raises `18.1` or `18.2`.
ooRexx raises `35.1` at the colon.

## 40. Buffered standard output

A standard output that is a pipe or a file is buffered and handed over
before a read, before the program idles, before an ADDRESS command, at 64
KiB and at exit. ooRexx writes each line at once, so here output still
buffered at a kill or crash is lost, and standard output and standard error
sharing one pipe interleave differently. A write error on a buffered
standard output is reported by the writes after the hand-over that met it.

## 41. A damaged compiled image

An encoded program image whose decoded bytes lack the image tag raises
`3.903`. ooRexx runs the partly decoded buffer as source, with unpredictable
results.

## 42. SysFork is refused

A call to SysFork is refused and forks nothing. In ooRexx both processes
continue the program.

## 43. Named RexxUtil semaphores are refused

SysCreateEventSem and SysCreateMutexSem with a name, SysOpenEventSem and
SysOpenMutexSem are refused; unnamed semaphores work. ooRexx shares named
semaphores between processes.

## 44. MAX and MIN round whole numbers

`n~min` and `n~max` with no arguments, and MAX and MIN with a whole-number
argument wider than NUMERIC DIGITS, round to the digits in force in both the
function and the method form, and decide LOSTDIGITS by the value. ooRexx
answers some integers unrounded, depending on how the number is held.

## 45. Bag hasItem with an unequal index

`Bag~hasItem` with an index unequal to the item answers 0. ooRexx answers
`.nil`.

## 46. LINEOUT on a binary record stream

On a stream opened `BINARY RECLENGTH n`, positioning with LINEOUT and closing
clear no record, and only a LINEOUT with a string pads its record. ooRexx
clears or pads a record when LINEOUT positions or closes.

## 47. A DO count wider than NUMERIC DIGITS

A DO repetitor or FOR count wider than NUMERIC DIGITS raises `26.2` or
`26.3`, or LOSTDIGITS first when that is trapped. ooRexx refuses such a count
too while NUMERIC DIGITS is below 9. At 9 or more it runs the loop when the
count is an unquoted integer literal of at most 18 digits, written without a
sign, a leading zero, a decimal point or an exponent, or that literal's own
value carried unchanged through variables, arguments, returns and
collections.

## 48. Writing to a transient stream

A write to a transient stream never seeks. On a FIFO, ooRexx fails a write
that follows a still-buffered write, and a close after one, with NOTREADY
`ERROR:29`.

## 49. Read and write positions

A persistent stream keeps separate read and write positions as the reference
manual describes, buffered or not: a write starts at the write position, and
a relative SEEK moves from the position it names. ooRexx tracks a stale
position on a buffered stream, so a write can land on bytes the program never
named, and on an unbuffered stream its relative and from-the-end seeks land
elsewhere. A relative SEEK that names neither position, on a stream open for
reading and writing, counts from the read position. ooRexx counts it from the
position that the last read, write or SEEK used, and from the write position
when there has been none since the stream was opened.

## 50. A trace line that cannot be delivered

When the trace destination refuses a line, the same condition as ooRexx's is
raised, but some clauses may do their own work first. ooRexx raises it
before the clause does anything.

## 51. `.context~rs` inside INTERPRET

Inside an INTERPRET, `.context~rs` answers the same value as `.rs`. In ooRexx
it answers the caller's earlier value until the INTERPRET ends.

## 52. An ADDRESS target that fails

When a write to an output or error redirection target raises a condition,
that condition is raised whatever return code the command answered. ooRexx
raises it only when the command answered 0, and otherwise ends the clause
with the return code as if every write had succeeded.

## 53. Numeric rules from the reference manual

A comparison under NUMERIC FUZZ checks LOSTDIGITS whatever the signs, MAX and
MIN apply FUZZ to whole numbers, an omitted MAX or MIN argument is `40.5` for
the function and `93.903` for the method, and a divisor is checked for
LOSTDIGITS when the dividend is zero. A LOSTDIGITS description renders its
operand the same way however it was produced. ooRexx departs from each of
these.

## 54. CHARS and LINES on a transient stream

CHARS and LINES on a transient stream ask how much input waits and consume
none. With `/dev/null` as standard input, which holds nothing, ooRexx answers
1 to some of these questions where interrexx answers 0. With an empty pipe, a
pipe that holds data, or a file as standard input it answers as interrexx does,
and no character is lost.

## 55. A stream opened WRITE

A stream opened WRITE reads nothing: reads fail with ERROR, CHARS and LINES
answer 0, and line positioning on a text stream is NOTREADY. ooRexx opens it
for reading and writing.

## 56. Line positions

A line position is counted in line ends from the start of the data, whatever
an earlier operation left. ooRexx counts from a line number it tracks, which
a character operation resets.

## 57. QUERY POSITION by line

`QUERY POSITION ... LINE` answers the line the position is in, and 0 for a
READ stream's write position. ooRexx answers its tracked line number.

## 58. LINES Count

`lines(name, 'C')` answers the lines that remain after the read position.
ooRexx's count can be wrong after a character read or a line positioning,
negative included.

## 59. An invalid SEEK

A SEEK below position 1, or of a READ stream's write position, answers 0,
raises NOTREADY and moves nothing. ooRexx answers a position it did not
check.

## 60. Ctrl-Z is data

A `'1a'x` in a stream is data, at its end too. ooRexx treats a trailing one
as end of file when it places writes, and overwrites it.

## 61. LINEOUT on an APPEND record stream

On a record stream opened APPEND, LINEOUT completes the record at the end of
the stream. ooRexx can raise a bare `93` there.

## 62. A read past the end

A CHARIN that reads nothing at the end, and a record-stream LINEIN that reads
nothing, leave the stream NOTREADY with description `NOTREADY:EOF`. ooRexx
leaves `ERROR:0`.

## 63. COPY of a class, a RexxContext or `.RexxInfo`

Copying a class object, a RexxContext or `.RexxInfo` raises `93.970` on every
route. ooRexx copies them when the copy is reached through a scope override.

## 64. A failed trap handler call under SIGNAL ON SYNTAX

When the CALL a CALL ON trap makes fails under SIGNAL ON SYNTAX, the signal
happens at once and SIGL names the raising clause. ooRexx runs the next
clause first.

## 65. RAISE RETURN and EXIT values

A value on RAISE's RETURN or EXIT option is the condition object's RESULT
for every condition, and a RAISE PROPAGATE keeps it. ooRexx sets RESULT only
for SYNTAX.

## 66. Frame name for a 16.1 inside INTERPRET

When a condition occurs in INTERPRET text and the SIGNAL ON trap for it names
a label that does not exist, the one PROGRAM frame of the 16.1 condition keeps
the program's name. ooRexx reports that frame's name as the null string.

## 67. The line a started message's error names

The error report of a started message names the line that raised the
condition. ooRexx's report races with the waiter's re-raise and often names
the waiter's line.

## 68. UNKNOWN of a Stem subclass

A Stem subclass's own UNKNOWN method runs. ooRexx forwards the message to the
stem's value and raises `97.1`.

## 69. The method of a ::CONSTANT expression

The method that evaluates a ::CONSTANT expression is that directive's own,
with its own source. ooRexx joins every expression constant of a class into
one method, so its source and a run of it differ.

## 70. Variable lists run left to right

DROP, EXPOSE and PROCEDURE EXPOSE process their names and parenthesised
lists from left to right. ooRexx processes the parenthesised lists first, in
reverse order, then the plain names.

## 71. A USE ARG alias in `.context~variables`

`.context~variables` lists a USE ARG alias under the name it is reachable by.
ooRexx lists it under the name of the variable it aliases.

## 72. The string value of a variable reference

In every position that requires a string, a variable reference's string
value is the referenced value's, and NOSTRING is raised only when that value
has none. ooRexx uses the object name of the referenced value, its default
name unless one was set, and raises NOSTRING for it in most positions.

## 73. A standard descriptor closed at start

A standard descriptor closed when the interpreter starts is open on
`/dev/null`, so writes to it succeed. In ooRexx they fail with `ERROR:9`.

## 74. APPEND on a write-only file

A stream opened APPEND on a file it can only write writes every line at the
file's current end. ooRexx writes at the end as it was when opened,
overwriting what another writer appended meanwhile.

## 75. A condition raised while its CALL trap is delayed

While a CALL ON trap is delayed, a condition raised for it again is held as
the reference manual describes, and ERROR, FAILURE and NOTREADY raised again
are ignored. ooRexx loses a held condition raised from a routine the handler
calls, and crashes in some such programs.

## 76. Package of an INTERPRET translation failure

The condition object of an INTERPRET that fails to translate names the
running package. ooRexx names another package object.

## 77. The string value of a stem

In every position that requires a string, a stem's string value is its
default value's, and NOSTRING is raised, with the default's description, only
when the default has no string value. ooRexx raises NOSTRING for every stem
there.

## 78. Search for the identical index or item

A lookup or PUT with the very index object already stored sends that object's
`==`. So does a search for an item with the very item object stored: `hasItem`,
`index`, `removeItem` and `allIndex` of the hash collections, and `hasItem`,
`index` and `removeItem` of an `Array`, `List`, `Queue` or `CircularQueue`. So
do a Relation's `uniqueIndexes` for an index object stored more than once, and
the check of a Set's or Bag's `put` and `[]=` that the index equals the item
when both are the very same object. ooRexx compares identity first and sends
nothing.

## 79. APPEND write positions

On a stream opened APPEND, every write goes after the last character, and
the write position cannot be moved into the file as it was when opened: such
a SEEK answers 0, and a CHAROUT or LINEOUT naming a position there writes
nothing, all NOTREADY. ooRexx moves the position there, and the write goes
after the last character as any other does. After a write, the write position
reported is just after the last character. ooRexx reports the position that
a CHAROUT start named, or that a SEEK set, moved past the characters written.

## 80. First SEEK of an unopened transient stream

The first SEEK of a transient stream that nothing has opened raises
`93.958`, as any positioning of a transient stream does. ooRexx answers 1.

## 81. A stem's STRING method

A stem's `STRING` sends `STRING` to its default value. ooRexx answers the
default's default name. A trace value line still shows the default's name.

## 82. Closing LINEOUT on a FIFO

A LINEOUT that closes a FIFO consumes no input. ooRexx consumes a byte.

## 83. When RAISE DESCRIPTION is converted

RAISE converts its DESCRIPTION to a string when it runs, so a conversion
that fails fails at RAISE. ooRexx converts at every `CONDITION('D')`.

## 84. Emptying a Table during REMOVE

When an index's equality method empties its Table, Relation, Set or Bag during
REMOVE, or the argument's empties a Table, Relation, StringTable, Set or Bag
during REMOVEITEM, or a Relation's or Bag's REMOVEALL or a Bag's ALLAT, the
search ends and ITEMS stays 0. ooRexx answers -1 for REMOVE and REMOVEITEM, and
its REMOVEALL and ALLAT go on comparing with an object that is no stored index.

## 85. `.environment` per interpreter

Each interpreter instance has its own `.environment`, and an object passed
between instances through the native API is refused. ooRexx shares one
`.environment` between every instance in a process.

## 86. The API daemon

The rxapi daemon starts only when a program first needs it, and prints its
own messages. ooRexx starts it with every interpreter.

## 87. rxregexp follows its documented grammar

rxregexp matches the documented pattern language and always terminates; a
pattern too large to compile is compile error 5. ooRexx answers some patterns
wrongly, never terminates on some and crashes on others.

## 88. Extension calls that crash ooRexx

RegularExpression `match` or `pos` with no pattern, and `match`, `pos` or
`parse` after `uninit`, SysGetgrnam or SysGetgrgid of a missing group,
SockSelect with a non-numeric entry and SysWordexp of a word list it rejects
raise a condition. ooRexx crashes.

## 89. Line numbers of rewritten extension packages

`rxregexp.cls`, `socket.cls`, `streamsocket.cls`, `mime.cls` and `smtp.cls`
are written for interrexx, so a traceback through them names interrexx's
lines of them, their numbers and text, and an error the package raises may be
reported from another clause of it or from the caller. Codes and results match
ooRexx.

## 90. SysWordexp is refused

SysWordexp stays registered, but every call raises `40.1` and expands
nothing. ooRexx expands the words with a shell, command substitution
included.

## 91. A command whose child was already reaped

A command whose process was reaped before the wait answers RC -10 as an
ERROR. ooRexx answers an unpredictable negative value.

## 92. The interpreter's own descriptors

The interpreter's own descriptors are numbered 1000 or above, and rxsock and
SysClose calls answer EBADF for them. In ooRexx a program can close them.

## 93. rxqueue with no session

rxqueue run outside a Rexx program, with the session queue as its queue,
prints an error and exits 9. ooRexx exits 0 and the lines go to a queue no
later process reaches.

## 94. A null TRANSLATE input table

A `tablei` given as the null string is a table with no characters, however
the null string was produced. ooRexx takes a null string from some builtins
as an omitted `tablei`. The [TRANSLATE page](r:bif.translate.deviation.94)
states it.

## 95. A native handle used after its call

An extension that keeps a local object reference past the native call that
received it, and uses it in a later call, ends the program with exit code
120 and a message that the handle is no longer held. The API does not
define such a use; ooRexx may still reach the object or one that replaced it.

## 96. Version and library banners

The version banner (`rexx -v`, `rexxc`, and what `RexxGetVersionInformation`
returns) names interrexx, its version, the ooRexx version it follows and its
licence. The `MathLoadFuncs` banner names the library and interrexx's version.
Neither prints ooRexx's product line, copyright or licence lines.
`PARSE VERSION` and `RexxInfo` are unchanged.

## 97. ABS under a changed NUMERIC FORM

ABS writes its answer in the current NUMERIC FORM, whatever form was in force
when the same string was converted to a number before. ooRexx may answer such
a string as it was written: under NUMERIC DIGITS 3, `abs('0.0000001')`
answers `1E-7`, and after a change to NUMERIC FORM ENGINEERING it answers
`0.0000001` where interrexx answers `100E-9`.

## 98. Closing a standard stream

A close of the standard input, output or error is a logical close, as for a
file, whether it is a `CLOSE` command of STREAM or a call of CHAROUT or
LINEOUT that gives no string. The state is then `UNKNOWN`, and the next stream
builtin that reads from it, writes to it, counts what is left in it (CHARS or
LINES) or opens it opens it again on the same input or output, so that later
reads and writes carry on. In ooRexx, output written to the standard output or
error after the close does not appear at once: it appears when the program
next closes, flushes or opens the stream, and what is written after the last
of these is lost. Output written before the close can be lost too, as when the
stream is a file and the later output replaces it. On the standard input, a
`CLOSE` has the same outcome on both until the program first reads from it;
after a read, ooRexx can lose the input not yet read. A CHAROUT or LINEOUT
that gives no string on the standard input leaves it in the `ERROR` state in
ooRexx and raises NOTREADY there, and LINEOUT answers 1.

## 99. Reading the standard output or error

A read of the standard output or the standard error, by CHARIN or LINEIN with
a count that is not 0, reads nothing and raises NOTREADY, whatever the stream
is connected to. ooRexx reads from such a stream when it is open for reading
as well, as a file opened for reading and writing is, and then answers the
characters there.

## 100. QUERY POSITION of a standard stream

`QUERY POSITION` of a transient standard stream answers the null string,
because a transient stream has no position. ooRexx answers 1.

## 101. Argument checks of the native builtins

DIRECTORY, FILESPEC, BEEP and the RexxUtil file and stem functions, such as
SysFileDelete and SysStemSort, check their arguments in the order that every
other builtin follows, the ANSI order: the count of arguments first, so that
an argument past the last one raises `SYNTAX 88.922` before an omitted or
wrong argument is reported. ooRexx reports an omitted argument, an object
whose `request` with the argument `STRING` answers no string, a stem name that
fails its check and an argument that is not a whole number before the count,
and the count before an argument's value: an option it does not allow or a
number out of range.

## 102. LOSTDIGITS before the right operand's errors

When LOSTDIGITS is trapped, a wide operand of an arithmetic operator raises it
before the operator raises `SYNTAX 42.3` for a zero divisor or `SYNTAX 26.8`
for an exponent that is not a whole number, as the ANSI standard orders the
operations. ooRexx raises the error for the right operand first.

## 103. TRACE() under ::OPTIONS TRACE ?

Under `::OPTIONS TRACE ?`, the TRACE builtin answers `?N` when the code
starts, as after the instruction `trace ?` at the start of a program: the
question mark turns interactive tracing on and the letter stays `N`, the
default. ooRexx answers the null string.

## 104. A pending CALL trap at an EXIT in an internal routine

When an EXIT inside an internal routine ends the program while a CALL trap is
pending, the routine of the trap is called before the program ends, as for an
EXIT in the main program, and it sees the variables of the routine that holds
the EXIT, with `SIGL` the line of the EXIT. ooRexx ends the program without
calling it.

## 105. Access checks of started messages

A message sent with START is checked for PRIVATE and PACKAGE access as the
same message sent directly by the code that starts it. A method of a class may
start a private method of its own class, code of a package may start a PACKAGE
method of that package, and a PACKAGE method started from another package
raises `SYNTAX 97.3`. ooRexx raises `97.2` for every started private message
and `97.1` for every started PACKAGE message, whichever code starts it.

## 106. CONDITION('S') after a CALL trap's routine renames the condition

In the routine of a CALL trap, the option `S` of CONDITION answers the state
of the trap that caught the condition, also after the routine has changed the
`CONDITION` entry of its argument. ooRexx answers the state of the trap for
the condition that the entry names at that moment, and `OFF` when no trap has
that name as the entry spells it.

## 107. Number literals are ordinary strings

A string that names a number is the same string to every message, however it
was written or computed. A number literal such as `12` used as the message
name of `send`, `sendWith`, `start`, `startWith` or `Message~new` names the
message `12`, and `request('ARRAY')` sent to it answers the array that
`makeArray` answers.
ooRexx raises `SYNTAX 93.946` for the first and answers `.nil` for the second
when the number is stored as a whole number, as a whole-number literal or the
result of arithmetic is. Equal literals of a program share one stored value,
so a quoted `'12'` behaves the same way when the literal `12` was used before
it. The same storage shows where a string is read as text: an item of an
`Array` whose `objectName` answers such a number is written by `makeString`
and `toString` as the digits; ooRexx writes bytes of its stored form.

## 108. A supplier that is not a Supplier

When `subclass`, `mixinclass`, `defineMethods` or `enhanced` takes the methods
of an object through its `supplier` method, the object that `supplier`
answers is used through the messages `available`, `index`, `item` and `next`
when it is not a `Supplier`; an object that does not answer one of them raises
`SYNTAX 97.1` as for any message. A `Supplier`, an instance of a subclass
included, is read directly, as a String subclass instance is read by its
characters
([Deviation 109](#109-string-subclass-instances-are-their-characters)). ooRexx
reads the answer as a Supplier without checking it and crashes when it is not
one.

## 109. String subclass instances are their characters

An instance of a subclass of `String` is read by its own characters wherever a
string, a number, a logical value or a whole number is needed, as receiver and
as argument, and as the operand of SAY; `REQUEST`, `MAKESTRING` and `STRING`
are not sent to it for that. ooRexx sends such an instance `request` with
`STRING` or `MAKESTRING` in many of these places, as an argument of a method,
an operator or a builtin function, as the receiver of the numeric and logical
methods and operators, `dataType` and `?`, as the operand of SAY, and in
`new`, so an override of either method can change what is read; elsewhere it
reads the instance's own characters, so one instance can answer differently to
`=` and to `==`. As the
message name of `send`, `sendWith`, `start`, `startWith` or `Message~new`,
ooRexx sends it `request('ARRAY')` instead and raises `SYNTAX 93.946`.

## 110. Instances follow every change to their class

Provisional. An object sees every change made, at any time, to the methods of
its class, of that class's superclasses and of their mixin classes, by
`define`, `defineMethods`, `delete`, `inherit` and `uninherit`, also when the
object has methods of its own from `setMethod` or `enhanced` and when the
class has been changed before. ooRexx copies the method table into existing
instances. A `define`, `defineMethods` or `delete` sent to a class, even one
that changes nothing, does not reach the instances that exist already, and no
later change reaches them after it: not to the class, not to its superclasses
or their mixin classes, and not `inherit` or `uninherit`. An object given a
method by `setMethod` sees no later change to its class or to the classes
behind it. A class object sees no change to its metaclass or to the classes
behind it that is made after the class itself.

## 111. A REQUIRES name replaced at the first offer chooses the file

A security manager is sent `requires` for a `::REQUIRES` name or a
`loadPackage` name first with the name as it was written, and then, when
nothing was replaced, with the full name of the file found for it. A NAME
replaced in the information of the first message chooses the file: that file
is loaded, and the manager is not sent a second `requires` message for it, so
it is never asked to approve its own replacement. A NAME replaced in the
second message chooses the file as well. ooRexx uses a replacement made at the
first message only to look for a package that is already loaded under that
name; otherwise it sends the second message with the full name of the file
the program asked for, and loads that file.

## 112. Array items of a loadPackage source are read through request

An item of the source array of `loadPackage` that is not a string is read as
`Method~new` and `Package~new` read one, by sending it `request` with the
argument `STRING`; an answer that is not a string raises the same error as
there, and a hole followed by an item raises the error that the source rule of
`new` gives for it. ooRexx reads an instance of a subclass of `String` and a
`MutableBuffer` by their text as well. Any other item it sends `objectName`,
not `request`, and uses the answer as the line, so an item whose `request`
answers no string raises no error; it crashes on a hole followed by an item.

## 113. A message keeps a copy of its argument array

`Message~new` with the option `A` keeps a copy of the argument array, so later
changes to the caller's array do not reach the message, and the number of
arguments is the position of the last item present, as for `Object~sendWith`;
the array of arguments that `Message~sendWith`, `startWith` and `replyWith`
take is copied and counted the same way. ooRexx keeps the caller's array
itself and passes its size as the number of arguments, trailing empty
positions included, in all four.

## 114. A metaclass NEW that answers an object that is not a class

When `subclass`, `mixinclass` or a `::CLASS` directive sends `new` to a
metaclass and the answer is not a class, `SYNTAX 98.900` is raised in the
program that made the request, and it can be trapped. ooRexx crashes.

## 115. A request('ARRAY') answer that is neither an array nor .nil

When `Method~new`, `Routine~new` or `Package~new` sends its source `request`
with the argument `ARRAY` and the answer is neither an array nor `.nil`,
`SYNTAX 88.913` is raised. ooRexx crashes.

## 116. A failing notification target is sent messageComplete once

When a `messageComplete` sent to a notification target of a started message
raises an error, that target is not sent `messageComplete` again. ooRexx sends
it to that target three times in all.

## 117. delete ends a hiding at once

`define` with `.nil` puts a mark in the class's method table that hides the
name, and `delete` of that name removes the mark: from then on the name is
looked up in the classes behind. ooRexx keeps the name hidden after the
`delete` until the class's table changes again.

## 118. A List subclass is sent makeArray and request

An instance of a subclass of `List` is read as any other collection is
wherever an array is wanted: `DO OVER` and the collection methods that read a
collection through `makeArray` send it `makeArray`, and the methods and
instructions that read an argument array or a source through `request` with
`ARRAY` (`sendWith`, `startWith`, `Message~new` with the option `A`,
`Message~replyWith`, FORWARD with ARGUMENTS, `Method~new`, `Routine~new`,
`Routine~callWith`, `Package~new`, `loadPackage`, `Supplier~new` and `init`,
the `unknown` of a `Stem`, a `VariableReference`, a `Directory`, a
`StringTable` or a `Monitor`, `File~writeLines` and `File~searchPath` with a
path that is no string) send it `request`, so an override is used. ooRexx
sends such an instance neither message and uses the items it holds, unlike the
subclasses of the other collection classes.

## 119. A second cancel of an Alarm or a Ticker sends nothing

A notification target of an `Alarm` or a `Ticker` is sent `cancel` once, when
the alarm or ticker is cancelled while it is waiting; a second `cancel` of the
same alarm or ticker sends it nothing. ooRexx sends the target `cancel` once or
twice for a second `cancel`, depending on timing.

## 120. previous of an index beyond the array answers the last item

`previous` of an index with a component beyond its dimension, one or all of
the components, answers the index of the last item in the order of `allItems`
whichever index is given, and `next` answers `.nil`. ooRexx answers an index
that depends on the position the beyond index names: in a 2 by 2 array holding
items at 1,1 and 2,2, `previous(4, 4)` answers 1,1 and `previous(5, 5)`
answers 2,2.

## 121. Queue put beyond the last item

`put` of a `Queue` or a `CircularQueue` with an index above the number of
items raises `SYNTAX 93.966` however far above it the index is, and the queue
keeps its items. ooRexx raises `93.966` or `93.918` according to the storage
the queue holds beyond its items, which its earlier history decides.

## 122. A right operand that is not a string is read once

A comparison operator, and the method of the same name, converts a right
operand that is not a string once. ooRexx converts the right operand twice for
the normal comparisons `=`, `\=`, `<>`, `><`, `<`, `>`, `<=`, `>=`, `\<` and
`\>` when the two operands are not both numbers.

## 123. isEmpty of a directory that holds only methods

`isEmpty` of a `Directory` agrees with `items`: it answers 0 for a directory
that holds a method of a name other than `UNKNOWN`, also when that is all it
holds, and 1 when `items` answers 0. ooRexx answers 1 for a directory that
holds only methods.

## 124. An argument array answered by request is an Array

An answer of `request` with `ARRAY` that is used as an array of arguments
(`sendWith`, `startWith` and `run` with the option `A` of `Object`, `new`,
`sendWith`, `startWith` and `replyWith` of `Message`, `callWith` of `Routine`,
`unknown` of `Stem`, `VariableReference`, `Directory`, `StringTable` and
`Monitor`, and the value of ARGUMENTS of FORWARD) or as an array of items or
indexes (`new` and `init` of `Supplier`) is accepted only when it is an
`Array` or an instance of a subclass of `Array` of one dimension; any other
object, a `Queue` included, raises the error that the reading method gives for
an answer that is no array. ooRexx uses the items of a `Queue` or a
`CircularQueue` as the arguments, or as the items or indexes of a `Supplier`,
except that `Message` passes as many arguments as the queue has storage, the
items followed by omitted ones, and for the arguments it crashes for a `List`,
a `Bag`, a `Table` or a `Directory`.

## 125. A List operand is read in its own order

A `List` that `allItems` or `makeArray` answers for the argument of `putAll`,
`union`, `intersection`, `xor` or `subset` of a `Set`, or of those and
`difference` of a `Bag`, gives every item of the list, in the order of the
list. ooRexx reads it as it reads an `Array`, by `items` and then `[]` with
each position from 1 up to that number, so the members are what `[]` answers
for those values rather than the items of the list: an item can be missed and
`.nil` can become a member.

## 126. compareTo with an argument of another class

`compareTo` of a `DateTime`, a `TimeSpan` or a `File`, and every comparison
operator that sends it, raises `SYNTAX 88.914` when the argument is not an
instance of `DateTime`, `TimeSpan` or `File` respectively (an instance of a
subclass is one). The error follows the `isA` that is sent to the argument and
precedes every other message. ooRexx answers 1 or -1 for an argument of another
class, whatever the receiver is.

## 127. Adding a span to a DateTime keeps the offset of the DateTime

`DateTime + TimeSpan`, `DateTime - TimeSpan` and `TimeSpan + DateTime` answer
an instance that has the offset of the `DateTime` operand. ooRexx gives the
answer of `-` with a span, and of a span plus a `DateTime`, the offset of the
host's zone, so that the same arithmetic on one instant answers another
instant on a host in another zone.

## 128. A DateTime hash code follows the instant

`hashCode` of a `DateTime` is computed from the instant, so two instances that
`=` finds equal have equal hash codes whatever their offsets, and an instance
equal to another is found in a `Set` or `Table` that holds the other. ooRexx
computes it from the fields, so equal instants at different offsets have
different hash codes.

## 129. elapsed lies between two instants

`elapsed` of a `DateTime` answers the span from the instant the receiver
stands for to the current instant, so the offset of the receiver does not
change it: two instances of one instant answer the same span, within the time
between the two calls. ooRexx subtracts the fields of the receiver from the
host's current local time, so the answer moves by the difference between the
offset of the receiver and the host's.

## 130. Whole-number arguments of DateTime and TimeSpan are read as a builtin's are

A method of `DateTime` or `TimeSpan` that reads a whole number (the fields and
the offset of `new`, the offset of `today`, of the `from` methods and of
`toTimezone`, the years of `addYears`, the days of `fromBaseDate`, the seconds
of `fromTicks`, and the count and the parts of `TimeSpan` `new`) reads it as a
whole-number argument of a builtin is read ([the form rule](r:conv.whole.form),
[the digits rule](r:conv.whole.digits)), with the method's own error for a value
those rules refuse: `2017.0`, `2.017E3` and `'2017 '` are 2017,
`2017.0000000001` is refused, and `2017.00000000000000000001` is 2017. ooRexx
reads some of these forms as an error, `2017.0` for a year among them, reads
`2017.0000000001` as year 1, accepts `1.0000000001` for an hour, and rounds the
days of `fromBaseDate` to nine digits.

## 131. An instance whose init never ran is zero

An instance of a subclass of `DateTime` or `TimeSpan` whose `init` did not
forward to the superclass, and so never gave it a value, is the first moment
of the class with the offset 0, or the span of no time, and every method of
the class works on it as it works on that value. ooRexx raises errors from
most of its methods, or answers text that is no value.

## 132. The ISO reader and the span reader are strict

`fromUtcIsoDate` and `fromStringFormat` of `TimeSpan` refuse with `SYNTAX
88.918` every string that is not exactly one the writing methods can produce
(for `fromUtcIsoDate`, the forms that `utcIsoDate` gives, and in addition a
fraction of one to six digits or none and the offsets `+00:00` and `-00:00`). A
blank anywhere, an offset whose minutes are above 59 or whose hour or minutes
have one digit, a fraction of seven digits, a point with no digits after it, and
a `Z` or an offset followed by anything are refused; so are, for a span, an
hour, minute or second that is out of range or written with one digit, days with
a leading zero or the days `0.`, a plus sign, and a minus sign before a span of
0. ooRexx accepts most of these spellings and reads them in its own way,
`+02:60` as `+03:00` and `24:00:00.000000` as one day.

## 133. A negative span under an hour keeps its sign

`fromStringFormat` of `TimeSpan` answers a negative span for the text of a
negative span of any size, `-00:01:30.000000` included. ooRexx answers the
positive span when the span has no days and no hours.

## 134. The lines of a method's source do not change when read

`source` of a `Method` answers an array with the same lines every time it is
sent, however often it was read before: an empty last line of the source, of a
`::METHOD` section or of what `new` or `newFile` is given, is left out once,
when the method is made, and no read drops another. ooRexx drops one more
trailing empty line after each read, until the last line is not empty, so the
answers get shorter; when every line of the source is empty, the number of lines
it answers changes from read to read, up as well as down, and a read can answer
a line that is not in the source. `source` of a `Routine` does the same.

## 135. An object whose init never ran raises an error

A class that a program declares as a subclass of `Alarm`, `Ticker`,
`RexxQueue`, `Supplier`, `StreamSupplier` or `Stream`, whose `init` does not
send `init` to its superclass, makes an object that is not set up, and every
instance method of the class other than `init` that the object is sent raises
`SYNTAX 98.901`. ooRexx answers `triggered` of such an alarm with the string
`TRIGGERED` and raises `SYNTAX 34.902` from its `cancel`, answers the readers
of such a ticker without an error and raises `SYNTAX 88.914` from its
`cancel`, and its `empty` of such a `RexxQueue` and its `available` of such a
`Supplier` end the interpreter with a segmentation fault, as do the `state`,
`description`, `qualify` and `query` of such a `Stream`.

## 136. A copy of a MutexSemaphore is free

`copy` of a `MutexSemaphore` answers a new semaphore that no activity owns,
whether the original is free or owned, and by which activity and how many times.
The two are separate from then on. ooRexx gives the copy the owner and the
count of the original, and a copy that another activity owns stays owned after
that activity ended, so that no activity can acquire it.

## 137. An empty TMPDIR is as an unset one

`temporaryPath` of `File` answers `/tmp` when the environment variable `TMPDIR`
is set to the null string, as it does when it is not set. ooRexx answers a
`File` for the null string.

## 138. allItems after a second init of a Supplier

`init` sent again to a `Supplier` also drops the arrays that an earlier
`allItems` or `allIndexes` collected, so the next call of either collects the
entries of the new arrays. ooRexx answers the arrays collected before the
`init`.

## 139. The size of a stream that another stream wrote to

A `Stream` answers `query('size')`, `chars` and `lines` for the file as it is
when it is asked, also after data that another stream on the same file wrote
and flushed. ooRexx answers the size that the stream found when it first read
or was first asked, so that its size and `chars` do not count the data written
since, and its `lines` can miss it.

## 140. A supplier of a stream read part of the way into a line

The `supplier` of a `Stream` that was read part of the way into a line is at
that line, with the number of the line as its index and the rest of the line,
as a plain `lineIn` answers it there, as its first item. ooRexx answers the
whole of the line when the read position is in the first line. It sends
`lineIn` with the number of the line, where this book sends it with no
argument.

## 141. The argument string of a program that rexx -e runs

A program that `rexx -e` runs gets its argument string built as a program in a
file gets it: the words after the text, joined by one blank, less the empty
words at the head. ooRexx drops one leading blank of that string when it is
longer than one character, so that the words `' '` and `x` give it `' x'`
where a file gets `'  x'`.

## 142. A copy of a Stream

A copy of a `Stream` is a new stream that has the name of the original and is
not open, as any new stream with that name is; the original is unchanged, and
the copy and the original do not share a read position. ooRexx's copy shares
the original's internal state, and the process ends abnormally when it ends.

## 143. A REGISTERED routine whose entry is not a classic function

A `::ROUTINE` directive with `EXTERNAL "REGISTERED library entry"`, whose
library loads and whose entry is not a function that the library registers
through the classic interface, is refused with `SYNTAX 90.999` when the
directive ends. ooRexx accepts the directive for some such entries,
`RxCalcTan` of `rxmath` among them, and the process ends when the routine is
called; for others, `SysSleep` of `rexxutil` among them, it raises the same
error.

## 144. LINES of a standard stream in normal mode

LINES with a name that denotes the standard input, output or error stream, and
no option or an option other than `C`, sends the message `LINES` to that entry
of `.local` and answers 0 when the string value of the answer is a number
equal to zero and 1 otherwise, whatever the answer is: `abc`, `1.5`, `1E10`
and a whole number of more than nine digits answer 1, and so does an object
that is not a string whose string value is one of these, such as an Array.
ooRexx answers each of
these as the object answered it, an object that is not a string included.

## 145. The state after a failed open with BINARY and no RECLENGTH

When `open` with `BINARY` and no `RECLENGTH` raises `SYNTAX 93.0`, as it does
for a file that is missing or empty, the stream is `UNKNOWN` afterwards.
ooRexx leaves it `READY`.

## 146. A command string with a NUL character

A command string that holds a NUL character is not run, in any native
environment: the command raises FAILURE with the return code 127. ooRexx cuts
the string at the NUL and runs the part before it.

## 147. A file without a #! line that a PATH search finds

ADDRESS PATH runs an executable file that its search of `PATH` finds and that
has no `#!` line with `/bin/sh`, which reads it as a script, as a shell does.
ooRexx cannot run such a file and answers FAILURE with the return code 127.

## 148. The descriptors that a command inherits

A command inherits no descriptor that the interpreter opened for its own work;
descriptors 0, 1 and 2, and any that the interpreter was started with, are
passed on. ooRexx also passes two descriptors that the interpreter itself
keeps open, so that a command sees two more open descriptors than the three of
the program.

## 149. Environment variables set after `export` and `unset`

Every set that VALUE makes in the `ENVIRONMENT` is kept, whatever `export` and
`unset` the interpreter did before, and `export` with an empty name changes
nothing. ooRexx loses some sets of VALUE that follow in-process `export` and
`unset` commands, which ones depending on what the program ran before for
any variable (the second `unset` of a name among them), so that the variable
has no value afterwards. For `export` with an empty name it stores an entry
with no name, which a `$` reference with no name then finds.

## 150. An INTERPRET in a line read at an interactive trace stop

The clauses of a string that an INTERPRET in such a line runs are not traced,
like the rest of the line, and no stop follows them. ooRexx traces them and
stops after each of them before the interpreter reads the next line.

## 151. RAISE SYNTAX in a line read at an interactive trace stop

A RAISE SYNTAX in such a line is reported as any error of the line is, and
tracing and the stops go on as before. In ooRexx the clauses that follow are
no longer shown, and the program may run on without a stop.

## 152. An empty line after a line that failed as it was read

An empty line that follows a line that failed as it was read, such as
`trace 1.5`, whose setting does not exist, or an expression that cannot be
parsed, goes on to the next stop. ooRexx ends interactive tracing at that empty
line.
