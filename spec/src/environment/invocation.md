---
element: "Invocation"
kind: concept
rexxref: "Rexx Runtime Objects > The SYSCARGS Array (.SYSCARGS); Rexx Runtime Objects > The Local Directory (.LOCAL)"
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
> `tests/rexx_command/cases`, which run interrexx and ooRexx on the same words
> and compare what they write and their status; each such rule is cited there.
> The examples of this page show what a program sees when it was started
> without words, and what happens when it ends.

## The forms and the option words

r[env.invocation.forms]
A command with no words, or whose words end before a program name or a string
is found, runs nothing: it writes a usage text of three lines, a blank line
first, to its standard error stream, and ends with the status 255. The text
names the three forms. The words that lead up to the end are the option words
of the next rule, so `rexx -x`, `rexx -e` and `rexx -o` followed by a string
and nothing more all give the usage text, and so does `rexx -o` alone.

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
The text of the banner names the interpreter and is [Deviation
96](../deviations.md#96-version-and-library-banners). A `-v` that stands after
the program name, or is the word after `-e`, is not an option word: the first
is an argument and the second is the text of the program, and neither prints
the banner.

## Running a string

r[env.invocation.expression]
An option word `-e` takes the next word as the text of a program, and runs
that text as a program. The words after the text are its arguments, as they
are for a file, and a second `-e` among them is an argument like any other.
`-e` with no word after it is the usage text. An empty text is a program
without clauses, which ends with the status 0. The program has no file: its
`PARSE SOURCE` shows `INSTORE` where a file shows its name, and its errors are
reported as in `INSTORE`. The array `.syscargs` does not exist for it, so
`.local` holds one entry less than it does for a file, and the symbol
`.syscargs` has the value of an environment symbol that nothing is stored
under ([the lookup rule](r:env.objects.search)). A `-o` option word before the
`-e` is ignored; one after the text is an argument.

## The options string

r[env.invocation.optstring]
An option word `-o` takes the next word as a string of options for the
program, and `-o` with no word after it is the usage text. The string is read
as an `::OPTIONS` directive that stands at the head of the program. If the
byte after its first non-blank byte is a colon, the string is the text of that
directive as it is given, and it is refused with `SYNTAX 93.900` unless it
begins, after its blanks, with two colons; otherwise the command puts
`::OPTIONS` and a blank before it. Only its first 511 bytes are read. The
options apply to the main program only, and `-od`, with the letter `d` after
the `o`, makes them apply to every program that a
[`::REQUIRES`](r:dir.requires.create) directive loads as well. An `::OPTIONS`
directive of the program itself comes after them and replaces what it sets. A
string that is refused ends the command before the program is looked for, with
[the status of an error](r:env.invocation.status): 231 for an unknown keyword
(`SYNTAX 25.924`), 230 for a bad value (`SYNTAX 26.5`), and 163 for a string
that is no directive.

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
one empty word starts it with one argument, the empty string.
[`PARSE ARG`](r:instr.parse.arg), [`ARG`](r:bif.arg.args) and `USE ARG` read
this one argument as they read any other.

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
rule](r:cond.index.syntax)). A program that `-e` runs ends the same way. The
errors that the command finds itself have a status of the same kind: 253 for a
program that cannot be found or read, and 231 and 230 for the refused string
of `-o`. The usage text, which is no error, has the status 255.

## The start

r[env.invocation.start]
A program starts with the three standard streams `READY`, with its queue
`SESSION` empty, and, when the command gave it no words, with no argument and
no entry in `.syscargs`. The monitors that connect these to the program are
[the bindings](r:env.objects.bindings) of the environment objects.

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
an error that no trap handles, every object that has a method named `UNINIT`
and has not yet been sent it is sent it, whether the program refers to the
object by a variable or by an entry of `.local`. An object that the program
has stopped referring to may be sent it earlier, when it is collected, and
when is not specified. The messages come after the program's last clause and
after the output that the program wrote; the status of the command is the
program's own. The order in which the objects are sent the message is not
specified, so the examples keep one such object each. What the objects hold is
still alive, and `.output` is still the standard output.

```rexx
keep = .guard~new('kept in a variable')
say 'main ends'
::class guard
::method init
  expose what
  use arg what
::method uninit
  expose what
  say 'uninit:' what
```

```output
main ends
uninit: kept in a variable
```

```rexx
.local~keep = .guard~new('held by .local')
say 'main ends'
::class guard
::method init
  expose what
  use arg what
::method uninit
  expose what
  say 'uninit:' what
```

```output
main ends
uninit: held by .local
```

```rexx
keep = .guard~new('kept')
say 'before exit'
call quit
say 'not reached'
quit: exit 5
::class guard
::method init
  expose what
  use arg what
::method uninit
  expose what
  say 'uninit:' what
```

```output rc=5
before exit
uninit: kept
```

```rexx
keep = .guard~new('kept')
say 'before error'
say 1 / 0
::class guard
::method init
  expose what
  use arg what
::method uninit
  expose what
  say 'uninit:' what
```

```output rc=214
before error
uninit: kept
```

r[env.invocation.end.uninit]
An `EXIT`, or an error that no trap handles, in the `UNINIT` method of one
object ends that method only. The other objects are still sent the message,
and neither changes the status of the command. In the examples, the first
object's `UNINIT` exits with 9 in the one and divides by zero in the other,
the second says that it ran, and the status stays that of the main program.

```rexx
bad = .bad~new
quiet = .guard~new('second')
say 'main'
return 3
::class bad
::method uninit
  exit 9
::class guard
::method init
  expose what
  use arg what
::method uninit
  expose what
  say 'uninit:' what
```

```output rc=3
main
uninit: second
```

```rexx
bad = .bad~new
quiet = .guard~new('second')
say 'main'
return 3
::class bad
::method uninit
  say 1 / 0
::class guard
::method init
  expose what
  use arg what
::method uninit
  expose what
  say 'uninit:' what
```

```output rc=3
main
uninit: second
```

> **Note.** What happens to an activity that is still running when the main
> program ends, and to the `UNINIT` messages that an `UNINIT` method starts,
> is the business of the concurrency chapter, and this page states nothing of
> it.
