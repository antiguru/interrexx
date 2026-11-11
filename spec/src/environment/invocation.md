---
element: "Invocation"
kind: concept
rexxref: "Rexx Runtime Objects > The SYSCARGS Array (.SYSCARGS); Rexx Runtime Objects > The Local Directory (.LOCAL)"
deviations: [96, 141]
---

# Invocation

A program is started by the command `rexx`, which reads the words after its
name, finds the program, runs it, and ends with a status. This page specifies
what the command does with its words, what a program sees of how it was
started, and what happens when it ends. The words are the ones the host hands
to the command: a program that is started from a shell sees them after the
shell has quoted and split them, and a word may be empty.

The command has three forms. `rexx` with a program name and any arguments runs
a file. `rexx -e` with a string runs the string as a program. `rexx -v` prints
a version banner.

> **Note.** The test harness starts each example's program without any words
> of its own, so an example cannot show the words of the command. The rules
> about them are tested by the recorded runs of the command in
> `rust/crates/interrexx/tests/rexx_command/cases`, which run interrexx and
> ooRexx on the same words and compare what they write and their status; each
> such rule is cited there. The examples of this page show what a program sees
> when it was started without words, and what happens when it ends.

## The forms and the option words

r[env.invocation.forms]
A command with no words, or whose words end before a program name or a string
is found, runs nothing: it writes a usage text, a blank line and then three
lines, to its standard error stream, and ends with the status 255. The text
names the three forms. The words that lead up to the end are the option words
of the next rule, so `rexx -x`, `rexx -e`, `rexx -o` and `rexx -o` with its
string and nothing more all give the usage text.

r[env.invocation.words]
The words from the first one are option words as long as they begin with a
minus sign; the first word that does not is the program name, and every word
after it is an argument, even one that begins with a minus sign. Only the
letter after the minus sign of an option word says which option it is, in
either case: `e` is [the expression option](r:env.invocation.expression), `o`
is [the options option](r:env.invocation.optstring), and `v` is [the version
option](r:env.invocation.version). Any later letter of the word is ignored,
except the `d` that may follow `o`, so `-ev` is an expression option and `-vo`
a version option. An option word whose letter is none of these, a minus sign
alone, and a word of two minus signs are ignored, and do not end the option
words, so that `-x` before a program name has no effect other than [the place
it takes in `.syscargs`](r:env.invocation.syscargs).

r[env.invocation.version]
An option word `-v`, in either case, makes the command print the version
banner to its standard output stream and end with the status 0. It does so
wherever it stands among the option words, including after the string that
`-o` takes, and whatever the other words are: no program is looked for or run.
What the banner says is [stated below](r:env.invocation.deviation.96). A `-v`
that stands after the program name, or is the word after `-e`, is not an
option word: the first is an argument and the second is the text of the
program, and neither prints the banner.

## Running a string

r[env.invocation.expression]
An option word `-e` takes the next word as the text of a program, and runs
that text as a program. The words after the text are its arguments, as they
are for a file, and a second `-e` among them is an argument like any other.
`-e` with no word after it is the usage text. An empty text is a program
without clauses, which ends with the status 0. The program has no file: its
`PARSE SOURCE` shows `INSTORE` where a file shows its name, and its error
reports name `INSTORE` where a file's name its path. The array `.syscargs`
does not exist for it, so `.local` holds one entry less than it does for a
file, and the symbol `.syscargs` has the value of an environment symbol that
nothing is stored under ([the lookup rule](r:env.objects.search.none)). A `-o`
option word before the `-e` is ignored; one after the text is an argument.

## The options string

r[env.invocation.optstring]
An option word `-o` takes the next word as a string of options for the
program, and `-o` with no word after it is the usage text. The string is read
as an `::OPTIONS` directive that stands at the head of the program. If the
byte after its first non-blank byte is a colon, the string is the text of that
directive as it is given, and it is refused with `SYNTAX 93.900` unless it
begins, after its blanks, with two colons; otherwise the command puts
`::OPTIONS` and a blank before it. Only its first 511 bytes are read. An empty
string followed by a word that begins with a colon is refused with `SYNTAX
93.900`; followed by any other word, it sets nothing. A directive other than
`::OPTIONS` is read too, but what it defines does not reach the program. A
second `-o` replaces the first: its string, and whether the letter `d` follows
the `o`. The options apply to the main
program only, and `-od`, with the letter `d` after the `o`, makes them apply
to every program that a [`::REQUIRES`](r:dir.requires.create) directive loads
as well. An `::OPTIONS` directive of the program itself comes after them and
replaces what it sets. A string that is refused ends the command before the
program is looked for, with [the status of an error](r:env.invocation.status)
for the error it raises, as an `::OPTIONS` directive in the program would: 231
for `bogus 9` (`SYNTAX 25.924`), 230 for `digits x` (`SYNTAX 26.5`), and 163
for a string that is no directive (`SYNTAX 93.900`).

## The program

r[env.invocation.program]
The program name is a file that the command finds in the way a program called
by name is found; the pages that follow in this chapter state that search. The
name that the program reads from `PARSE SOURCE` ([the
rule](r:instr.parse.source)), and the one that its error reports give, is the
absolute path of the file that was found, with every `.` and `..` of the path
resolved, and with a link in the path left as it is. A word that is not valid
text is a name all the same: names and arguments are the bytes that the host
gave.

r[env.invocation.notfound]
A program name that no file answers to, or that names a directory, runs
nothing and writes the error `3.901` to the standard error stream; a file that
is found and cannot be read gives the error `3.1`, which names the file. Both
end with the status 253.

## The arguments

r[env.invocation.arguments]
The program starts with its arguments as one argument: the words after the
program name, joined by one blank between each two. An empty word at the head
of the words adds nothing, not even its blank, so that the arguments `''`,
`''` and `x` make the string `x`, and every later empty word is kept with the
blank before it, so that `a`, `''` and `b` make `a  b`, and `a` and `''` make
`a `. A command with no words after the program name starts it with no
argument, so that `ARG()` is 0 and `ARG(1)` the empty string; a command with
one empty word starts it with one argument, the empty string. [`PARSE
ARG`](r:instr.parse.arg), [`ARG`](r:bif.arg.args) and [`USE
ARG`](r:instr.use.arg) read this one argument as they read any other. A
program that [`-e` runs](r:env.invocation.expression) gets its argument in the
same way ([Deviation 141](r:env.invocation.deviation.141)).

r[env.invocation.syscargs]
The array `.syscargs` holds the words after the program name one by one,
without the joining. A word is the entry whose index is its place among all
the words after `rexx`, counted from 1, less one. When the program name is the
first word, that is the index 1 for the first argument. Each word before the
program name, the string of `-o` included, moves the arguments one index to
the right and leaves the index below them without an entry: after `-x` before
the program name, `hasIndex(1)` is false, `items` counts only the entries that
exist, and `size` and `last` count the empty places too. A command with no
arguments leaves `.syscargs` an array with no entry at all, and an empty word
is an entry that is the empty string.

## The status

r[env.invocation.status]
The command ends with the status of the program: the value of its `EXIT` or of
the `RETURN` that ended it ([the exit code rule](r:instr.exit.code)), or, when
an error ended it, `256` minus the major number of the error ([the error
rule](r:cond.index.syntax)). On Linux the command's status is the low eight
bits of that integer, so that `exit 300` gives 44 and `exit 70000` gives 112.
A program that `-e` runs hands on its value as a 16-bit integer: a whole
number from -32768 to 32767 gives the same status as it does from a file, and
any other gives 0, so that `exit 33000` gives 0 under `-e` and 232 from a
file. The errors that the command finds itself have a status of the same
kind: 253 for a program that cannot be found or read, and that of its error
for a refused string of `-o`. The usage text, which is no error, has the
status 255.

## The start

r[env.invocation.start]
A program starts with the three standard streams `READY`, with `SESSION` as
its current queue ([the RXQUEUE rule](r:bif.rxqueue.get)), which holds
nothing when the program is started from a shell, and, when the command gave
it no words, with no argument and no entry in `.syscargs`. A program that a
command of another Rexx program starts shares that program's `SESSION` queue,
as [the child rule](r:env.commands.rxqueue.child) states.
The monitors that connect the streams to the program are [the
bindings](r:env.objects.bindings) of the environment objects. With
`/dev/null` as standard input, what CHARS and LINES answer is Deviation 54
([for CHARS](r:bif.chars.deviation.54), [for
LINES](r:bif.lines.deviation.54)).

```rexx
say stream('STDIN', 'S') stream('STDOUT', 'S') stream('STDERR', 'S')
say queued() rxqueue('Get')
say .syscargs~items arg() '['arg(1)']'
```

```output
READY READY READY
0 SESSION
0 0 []
```

## The end

r[env.invocation.end]
When the main program ends, by its last clause, by `RETURN`, by `EXIT`, or by
an error that no trap handles, the status of the command is the program's own
([the status](r:env.invocation.status)), whatever the objects that the program
holds do. Whether the interpreter sends `UNINIT` to those objects at that end,
and in what order, is not specified ([the rule of
`UNINIT`](r:env.invocation.uninit)), so a program that needs something done at
its end does it itself. The first example ends the program by `EXIT` and the
second by an error.

```rexx
say 'before exit'
call quit
say 'not reached'
quit: exit 5
```

```output rc=5
before exit
```

```rexx
say 'before error'
say 1 / 0
```

```output rc=214
before error
```

r[env.invocation.uninit]
The interpreter sends the message `UNINIT` to an object that has a method of
that name when it finds that the program cannot reach the object any more;
what it frees by doing so is up to the method. This is best effort, and a
program may rely on two things only: an object is sent `UNINIT` at most once
by the interpreter (a program that sends `UNINIT` itself does not count), and
it is never sent it while the running program can still reach the object,
for example through a variable, an argument or the receiver of a method or
routine that is running, an entry of `.local`, `.environment` or a package's
local directory, or another object that it can reach. An object that is held
when the main program ends is therefore not sent it before the end. It is not
specified whether or when an object that the program has stopped referring to
is sent `UNINIT`, whether the objects that are left when the main program ends
are sent it, or in what order. The first example holds three objects of a
class whose `UNINIT` counts the messages it gets: while they are held, the
count is 0 however often storage is reclaimed ([GC](r:bif.gc.args)), and after
one is dropped the count is at most 1. In the second, the `UNINIT` of the
object stores it again, so that the program can reach it once more, and the
object is still sent `UNINIT` at most once.

```rexx
.environment~count = 0
.local~one = .guard~new
.environment~two = .guard~new
keep = .guard~new
do 3
  call gc 'force'
end
say 'held:' .environment~count
drop keep
do 5
  call gc 'force'
end
say 'dropped:' (.environment~count <= 1)
::class guard
::method uninit
  .environment~count += 1
```

```output
held: 0
dropped: 1
```

```rexx
.environment~count = 0
.environment~back = .nil
x = .phoenix~new
drop x
do 20 until .environment~count > 0
  call gc 'force'
end
.environment~back = .nil
do 20
  call gc 'force'
end
say 'at most once:' (.environment~count <= 1)
::class phoenix
::method uninit
  .environment~count += 1
  .environment~back = self
```

```output
at most once: 1
```

r[env.invocation.end.uninit]
An `EXIT`, or an error that no trap handles, in the `UNINIT` method of an
object ends that method only, and an error there writes no report, whether the
interpreter sent the message at the end of the main program or earlier. It
does not change the status of the command. In the first example, the `UNINIT`
of a dropped object exits with 9, and that of a second one divides by zero,
each after the object is dropped and storage is reclaimed; in the second, an
object that the program holds has an `UNINIT` that divides by zero when the
program ends, if the interpreter sends it. The status stays that of the main
program, with nothing written but what the program says.

```rexx
.error~destination(.stdout)
.environment~ran = 0
bad = .bad~new
drop bad
do 5
  call gc 'force'
end
say 'after' (.environment~ran <= 1)
bad2 = .bad2~new
drop bad2
do 5
  call gc 'force'
end
say 'after2'
return 3
::class bad
::method uninit
  .environment~ran += 1
  exit 9
::class bad2
::method uninit
  say 1 / 0
```

```output rc=3
after 1
after2
```

```rexx
.error~destination(.stdout)
bad = .bad~new
say 'main'
return 3
::class bad
::method uninit
  say 1 / 0
```

```output rc=3
main
```

> **Note.** What happens to an activity that is still running when the main
> program ends, and to the `UNINIT` messages that an `UNINIT` method starts,
> is the business of the concurrency chapter, and this page states nothing of
> it.

## Deviations

r[env.invocation.deviation.96]
The version banner that `-v` prints names interrexx, its version, the ooRexx
version it follows and its licence, and none of ooRexx's product, copyright
or licence lines ([Deviation
96](../deviations.md#96-version-and-library-banners)). Its other two lines,
`Build date:` followed by the date of the build, and `Addressing mode:`
followed by the width of an address in bits, are as ooRexx writes them.

r[env.invocation.deviation.141]
The argument string of a program that `rexx -e` runs is made as [the
arguments rule](r:env.invocation.arguments) makes it for a program in a file,
so that the words `' '` and `x` after the text give the string of two blanks
and `x`. ooRexx drops one leading blank of the string when it is longer than
one character, and gives `' x'` ([Deviation
141](../deviations.md#141-the-argument-string-of-a-program-that-rexx--e-runs)).
