---
element: "Invocation"
kind: concept
rexxref: "Rexx Runtime Objects > The SYSCARGS Array (.SYSCARGS); Rexx Runtime Objects > The Local Directory (.LOCAL); Useful Services > Distributing Programs without Source"
deviations: [41, 96, 141, 153]
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
a version banner. A second command, `rexxc`, [translates a program into a file
that holds no source](#programs-without-source), which `rexx` then runs.

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
the `o`. The options apply to the main program only, and `-od`, with the
letter `d` after the `o`, makes them apply to every program that is translated
during the run as well, one that a [`::REQUIRES`](r:dir.requires.create)
directive loads or one that is called by name. An `::OPTIONS` directive of the
program itself comes after them and replaces what it sets. A string that is
refused ends the command before the program is looked for, with [the status of
an error](r:env.invocation.status) for the error it raises, as an `::OPTIONS`
directive in the program would: 231 for `bogus 9` (`SYNTAX 25.924`), 230 for
`digits x` (`SYNTAX 26.5`), and 163 for a string that is no directive (`SYNTAX
93.900`).

## The program

r[env.invocation.program]
The program name is a file that the command finds in the way a program called
by name is found; as [the program rule](r:env.search.program) of the search
page states. The name that the program reads from `PARSE SOURCE` ([the
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

## Programs without source

> **Note.** The command `rexxc` translates a program and writes the result to
> a file, the image, so that the program can be handed on without its source;
> `rexx` runs an image as it runs the program it came from ([the run
> rule](r:env.invocation.rexxc.run)). The rules below state what `rexxc` takes
> and writes, what an image is to a reader, what a run of one shows, and what
> it is refused for.
>
> A program that the test harness starts cannot run `rexxc`, so the
> rules of this section have no examples. Each is tested by recorded runs of
> `rexxc` and of `rexx` on both interpreters, in
> `rust/crates/interrexx/tests/rexxc/cases` and
> `rust/crates/interrexx/tests/compiled/cases`, and by the tests of `rexxc.rs`
> and `compiled.rs`, which run both interpreters on copies of their images
> whose base64 text is laid out anew or holds a character that is not base64,
> and interrexx alone on images that are cut, extended or changed ([Deviation
> 153](r:env.invocation.deviation.153)); each rule is cited there.

### What rexxc takes

r[env.invocation.rexxc.words]
`rexxc` reads its words in order. The first is the name of the program,
whatever it looks like, so that `rexxc -x` looks for a program called `-x`. A
second word that does not begin with a minus sign is the name of the output
file. Every word after those is a flag, and so is a second word that begins
with a minus sign. A flag begins with a minus sign, and only the letter after
it counts, in either case: `s` and `e` are the two flags that exist ([the
banner rule](r:env.invocation.rexxc.banner) and [the image
rule](r:env.invocation.rexxc.image)), and a later letter of the word is
ignored, so `-S` and `-sx` are the flag `s`. A flag may stand more than once
and the flags may come in any order. A flag that is neither, a minus sign
alone, and a word after the flags that does not begin with a minus sign are
errors, and so is a name after the first flag, as in `rexxc a.rex -s b.rxo`.
The command then writes a line that says its parameters are incorrect and the
usage text, to its standard output stream, and ends with the status 255,
before it looks for the program. With no words at all it writes the usage text
alone, with the same status.

r[env.invocation.rexxc.check]
Without an output name `rexxc` only translates the program and keeps nothing
of the result: it ends with the status 0 when the program translates, and
reports the error when it does not. The flag `e` changes nothing then, and the
flag `s` only [leaves out the banner](r:env.invocation.rexxc.banner).

r[env.invocation.rexxc.banner]
Unless the flag `s` is given, `rexxc` writes the version banner of [the `-v`
option](r:env.invocation.version) to its standard output stream once its words
are accepted, and before anything else it writes, so that a command that ends
with an error writes it too. A command that ends with the usage text writes no
banner. The banner is as [the version rule](r:env.invocation.deviation.96)
states.

r[env.invocation.rexxc.names]
When the output name and the program name are the same string of bytes, the
command ends with the status 254 and the line `Output file name must be
different from input file name.` on its standard output stream, before it
looks for the program, so that it ends so even when no such program exists.
Words that are in error ([the words rule](r:env.invocation.rexxc.words)) are
reported before this check, so that `rexxc a a -x` ends with 255. The check
compares the two words and not the files they name: for it `a` and `a.rex`, or
`a.rex` and `./a.rex`, differ, and the translation of `a.rex` is then written
over the file `a.rex`.

r[env.invocation.rexxc.output]
The program is found as [`rexx` finds a program](r:env.invocation.program),
and a name that no file answers to, a directory, and a file that cannot be
read give the errors [of that rule](r:env.invocation.notfound), with the same
status. The output file is written after the program has translated, with the
name as it was given and no ending added. An existing file of that name has
its content replaced. A program that does not translate writes nothing and
leaves an existing file as it was. An output file that cannot be written gives
the error `3.902`, with the status 253, and leaves an existing file as it was.

r[env.invocation.rexxc.translation]
`rexxc` translates a program as it is translated before a run, and a run finds
the same errors in it, with the same report: the line of the clause, the error
numbers and the status of [the error rule](r:env.invocation.status), so that
`say (` ends the command with 221 (`SYNTAX 35.1`). The `EXTERNAL` option of a
`::ROUTINE`, `::METHOD` or `::ATTRIBUTE` directive is bound then, so that a
library that cannot be loaded (`SYNTAX 98.903`), a method entry of the `REXX`
library that does not exist (`SYNTAX 90.998`) and a `REGISTERED` routine that
cannot be found (`SYNTAX 90.999`) end the translation. Nothing else is looked
up: [a required file](r:dir.requires.create), a library that `::REQUIRES`
names with `LIBRARY`, and the superclass of a class are looked for when the
image runs, so a program that requires a file or a library that does not
exist, or whose class has a superclass that does not exist, translates.

r[env.invocation.rexxc.status]
`rexxc` ends with the status 0 when the program translated and, with an output
name, the file is written. Every other end has one of the statuses already
given: 255 for [the words](r:env.invocation.rexxc.words), 254 for [equal
names](r:env.invocation.rexxc.names), 253 for a program that cannot be found
or read and a file that cannot be written ([the output
rule](r:env.invocation.rexxc.output)), and for an error of the program, 256
minus its major number as [the status rule](r:env.invocation.status) gives it.

### What an image is

r[env.invocation.rexxc.image]
An image is the program in its translated form. Only its opening is specified,
and it is the same for both interpreters. The first line is `#!/usr/bin/env
rexx` and a line feed. The nine characters `/**/@REXX` follow, and then either
two bytes of 0, for the binary form that `rexxc` writes by default, or the
character `@` and a line feed, for the form that the flag `e` asks for. The
binary form is not text: it goes on with a header and the translated program,
which include the byte 0, and what they hold is not specified. In the encoded
form what follows is lines of base64 text, each of 72 characters and each
ended by a line feed, except the last, which may be shorter. Decoded, they
give bytes that begin with `/**/@REXX` and the same two bytes of 0, as the
binary form does after its first line.

### Running an image

r[env.invocation.rexxc.run]
`rexx` runs an image as it runs the program that it was made from: the words
are its arguments, a `RETURN` or `EXIT` gives [its
status](r:env.invocation.status), and a program it calls or requires is looked
for as for any program, beside the image first ([the caller
rule](r:env.search.caller)). A file is an image when its bytes begin, after a
first line that begins with `#!` or with no first line, with the characters
`/**/@REXX` and the two tag bytes of [the image
rule](r:env.invocation.rexxc.image). Nothing else about the file counts: its
name has no part in it, and the first line may say anything after the `#!`.
The ending `.rxo` is not among those that [the search
adds](r:env.search.extension), so `rexx name` does not find a file `name.rxo`,
while `rexx name.rxo` runs it, and `rexx name` runs an image that is called
`name`. A file that begins like an image but has another letter in place of
one of the four letters of `REXX`, or another byte in place of the byte that
follows them (the first byte of 0, or the `@`), or, in the encoded form,
another byte in place of the line feed after the `@`, is not an image: it is
source, and its `@` is the character that `SYNTAX 13.1` reports. A file that
is given to `rexxc` is always translated as source, an image too, so that it
ends with the same error.

r[env.invocation.rexxc.source]
A program that runs from an image has no source. `SOURCELINE()` answers 0, and
a call with a whole number `n` of 1 or more raises `SYNTAX 40.34`, as [the
past-the-end rule](r:bif.sourceline.past-end) says for a number above the
count. `.context~package~source` has no lines and `sourceSize` is 0 ([the
package rule](r:class.package.sourcesize)). The `source` of every routine and
method that the image defines has no lines either, and the package's
`sourceLine` answers the null string for every line, but the data of a
`::RESOURCE` directive and the values of `::ANNOTATE` directives are kept.
`PARSE SOURCE` gives the same three words as it does for a file ([the
rule](r:instr.parse.source)), and the name is the path of the image. A line
number is still the number of the clause's line in the program that was
translated: an error that ends the run, and the traceback of a condition, show
it, and show in place of the text of the clause a line that says no source is
available. The text of an `INTERPRET` string is source and is shown as it is
for any program.

r[env.invocation.rexxc.trace]
`TRACE` in an image is accepted and changes the setting that `TRACE()`
answers, but a clause of the image itself traces nothing: [the clause
lines](r:instr.trace.line) and the results and values that go with them are
not written, whatever [the setting](r:instr.trace.setting.clauses) is. The
`>I>` and `<I<` lines that [the tags rule](r:instr.trace.tags) gives to the
entry and the exit of a routine or method are shown, and the clauses that an
`INTERPRET` string holds trace as they do in any program, with the line number
of the `INTERPRET` clause. A pause of the [interactive
trace](r:instr.trace.form.interactive) happens only before such a clause.

r[env.invocation.rexxc.options]
An image takes no options from [the `-o` option](r:env.invocation.optstring):
it runs with what it was translated with, an `::OPTIONS` directive that was in
the program included, and the string is not spent on it. A string without the
letter `d` goes to the first program of the run that is translated from
source: for an image that requires two source files, the first of them, and
for one that calls a source file by name and requires none, that file; a
program translated after it takes none. With the letter `d` every program that
is translated from source takes the string.

r[env.invocation.rexxc.routes]
Besides the command, [a call by name](r:env.search.program), `::REQUIRES` and
`newFile` of `Routine` and `Method` restore an image from a file. The lines of
an encoded image, read as an array of lines, are restored by `new` of
`Routine`, `Method` and `Package` and by `loadPackage`; one string that holds
them is source, and so are the lines of a binary image, so that `new` of
`Routine` with them stops at the first `@` with `SYNTAX 13.1`. A routine or
method restored from lines sees nothing of the program whose `new` or
`loadPackage` restored it: a routine of that program that its code calls by
name is not found (`SYNTAX 43.1`), where the same code given to `new` as
source finds it. A routine that `newFile` or `new` restores is named by the
path of the file that `rexxc` translated, and its package by the name that the
call gave; a routine that a call by name restores is named by the name of the
call, and its package by the path of the image.

### Images that are refused

r[env.invocation.rexxc.foreign]
An image that the other interpreter wrote is refused when it is run, before
any of its code runs, with the error `3.903` and the status 253, although its
opening is that of any image. Which versions of an interpreter accept each
other's images is not specified.

r[env.invocation.rexxc.damaged]
In an encoded image a carriage return and a line feed are skipped between the
groups of four base64 characters, so that the lines may have any length that
is a multiple of four and may end with a carriage return as well. One that
stands inside a group makes the image refused with the error `3.905` and the
status 253, as does any other character that is not a base64 character, a
blank or a tab included, before any of its code runs. The skipping holds for
the base64 lines only: the line feed after the `@` stands alone, so a copy in
which that line too ends with a carriage return is source
([the run rule](r:env.invocation.rexxc.run)) and stops at its `@` with `SYNTAX
13.1`. An encoded image whose decoded bytes lack the binary tag is the case of
[Deviation 41](r:env.invocation.deviation.41). Otherwise an image that ends
too early, one that has a changed byte in its header or a changed second byte
of 0 in the binary form, and one that has bytes after its end, an encoded one
once its text is decoded, are refused with the error `3.903` and the status
253, before any of its code runs
([Deviation 153](r:env.invocation.deviation.153)).

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

r[env.invocation.deviation.41]
An encoded image whose decoded bytes lack the tag of an image, as the base64
of `say 'decoded'` does, is refused with the error `3.903` and the status
253, as [the damaged-image rule](r:env.invocation.rexxc.damaged) refuses other
damage. ooRexx translates the partly decoded bytes as source with
unpredictable results ([Deviation
41](../deviations.md#41-a-damaged-compiled-image)). The rule of
[`Method`](r:class.method.deviation.41) states the same for a program made by
`new`.

r[env.invocation.deviation.153]
An image that ends too early, one that has bytes after its end, and one that
has a changed byte in its header, the second byte of 0 of the binary form
included, are refused with the error `3.903` and the status 253, before any of
its code runs. An encoded image whose base64 text does not decode whole,
because characters follow the `=` that closes it, an `=` stands inside it, or
it ends with a group of fewer than four characters, is refused with the error
`3.905` and the status 253. ooRexx runs an image that has bytes after its end,
ignores some changed header bytes and crashes on others, and crashes on an
image that ends too early ([Deviation
153](../deviations.md#153-a-damaged-compiled-image-is-refused)).
