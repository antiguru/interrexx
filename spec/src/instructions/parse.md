---
element: PARSE
kind: instruction
syntax: "PARSE [UPPER | LOWER] [CASELESS] source [template_list]"
ansi: "6.3.2.71, 8.3.17"
rexxref: "Keyword Instructions > PARSE"
deviations: [77]
---

# PARSE

PARSE takes a string from a source, cuts it up as a template says, and
assigns the pieces to variables. This page specifies what the instruction
adds to the parsing itself: the sources it can read, the keywords that choose
them and change the case, and the errors of its form. What a template does
with the string is the subject of [the parsing chapter](../parsing/index.md),
and [ARG](arg.md) and [PULL](pull.md) are short forms of PARSE with a fixed
source.

## Form

r[instr.parse.form]
After PARSE come the case keywords, if any, then one source keyword, and then
the template list, which runs to the end of the clause. The source keywords
are ARG, LINEIN, PULL, SOURCE, VALUE, VAR and VERSION. Keywords are not case
sensitive. What is parsed is the string value of what the source gives; a
value that is not a string becomes one [as the conversion
rule](r:conv.string.object) says. The first token after the keywords starts
the template list, so in `parse var s with` the word `with` is a variable of
the template, and so is `upper` in `parse arg upper`. A template list may be
empty, and then the instruction
[assigns nothing](r:parse.templates.empty) but still obtains its source.

```rexx
parse var word word2
say '[' || word || ']'
s = 'a b'
PARSE Var s with upper
say with '|' upper
parse arg upper
say '[' || upper || ']'
parse linein
say 'no template'
```

```output
[WORD]
a | b
[]
no template
```

r[instr.parse.options]
The keyword UPPER or the keyword LOWER, at most one of them, and the keyword
CASELESS, at most once, may stand between PARSE and the source keyword. Of
CASELESS and the other one, either may come first, and the two orders mean
the same. What the options do to the string and to the matching is specified
with [the parsing of several strings](r:parse.multiple.convert), and
[CASELESS](r:parse.multiple.caseless) is there. Every source accepts every
option.

```rexx
s = 'Ab-Cd'
parse upper caseless var s a 'cd'
say a
parse caseless upper var s a 'cd'
say a
parse lower caseless var s a 'CD'
say a
parse caseless lower value 'XY-ZZ' with a '-' b
say a b
parse upper source a
say (a == translate(a))
```

```output
AB-
AB-
ab-
xy zz
1
```

## Sources

r[instr.parse.arg]
PARSE ARG reads the arguments of the routine, method or program that is
running, as [the ARG builtin](r:bif.arg.scope) defines them. The first
template parses the string value of the first argument, the next template the
second argument, and so on, as [the comma rule](r:parse.multiple.comma) says.
Nothing is converted unless UPPER or LOWER is given. An argument that is
missing is [the null string](r:parse.multiple.comma).

```rexx
call show 'Mixed  Case', 'two'
call show
exit
show:
  parse arg a b, c
  say '[' || a || '][' || b || '][' || c || ']'
  parse upper arg a b, c
  say '[' || a || '][' || b || '][' || c || ']'
  return
```

```output
[Mixed][ Case][two]
[MIXED][ CASE][TWO]
[][][]
[][][]
```

r[instr.parse.linein]
PARSE LINEIN reads the next line from the default input stream, as the
builtin [LINEIN](r:bif.linein.stdin) does with no stream name, and parses it.
It never takes a line from the external data queue, even when the queue holds
lines, and it leaves the queue as it was. At the end of the input the source
is the null string, and the [`NOTREADY`](r:cond.notready.information)
condition is raised with the description `STDIN`; when no trap is set that
ends nothing, and the instruction completes with the null string. The standard
input of the examples is empty.

```rexx
q = rxqueue('Create', 'spec_instr_parse_linein')
call rxqueue 'Set', q
queue 'in the queue'
call on notready name nr
parse linein a b
say '[' || a || '][' || b || ']' queued()
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', q
exit
nr: say 'notready' condition('d')
    return
```

```output
notready STDIN
[][] 1
```

> **Note.** Reading the default input stream from several activities at once
> is the subject of [Deviation 13](../deviations.md).

r[instr.parse.pull]
PARSE PULL takes its line from the current external data queue, and from the
default input stream when the queue has none, as [PULL](r:instr.pull.source)
specifies. The line keeps its case unless UPPER or LOWER is given.

```rexx
q = rxqueue('Create', 'spec_instr_parse_pull')
call rxqueue 'Set', q
queue 'Red Green'
queue 'Blue'
parse pull a b
say '[' || a || '][' || b || ']' queued()
parse upper pull a b
say '[' || a || '][' || b || ']' queued()
parse pull a b
say '[' || a || '][' || b || ']' queued()
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', q
```

```output
[Red][Green] 1
[BLUE][] 0
[][] 0
```

r[instr.parse.source]
PARSE SOURCE reads a string of three or more words. The first word names the
operating system, and its value is not specified. The second tells how the
running code was started: `COMMAND` for the program itself, and also for an
internal routine, since it runs in the program, `SUBROUTINE` for a routine
or a program started with CALL, `FUNCTION` for one that was called as a
function, and `METHOD` for a method. Any other start, such as the prolog of
a package that a [`::REQUIRES`](r:dir.requires.create) directive loads, is not
specified. The rest of the string is the name of the file or package that
holds the code, which may itself contain blanks; for code that comes from a
file it is the name that `.context~package~name` gives, and for a method or
routine made from source text it is the name given when it was made.

```rexx
parse source sys how file
say how (words(sys how file) >= 3) (file == .context~package~name)
call inner
say result
call second
say result
say second()
say .thing~new~go
exit
inner: parse source . how .
  return how
::routine second
  parse source . how .
  return how
::class thing
::method go
  parse source . how .
  return how
```

```output
COMMAND 1 1
COMMAND
SUBROUTINE
FUNCTION
METHOD
```

r[instr.parse.value]
PARSE VALUE evaluates the expression between VALUE and the keyword WITH,
and parses the string value of the result. Without an expression the source
is the null string. The expression ends at the first WITH that stands as a
symbol of the expression itself, outside parentheses and brackets. A WITH
that is a message name after `~` or `~~`, an argument inside `[ ]` or part
of a string does not end it. A variable named `with` can therefore be used
in the expression only in parentheses or as an argument, and a template may
use `with` as a variable. The expression is evaluated before anything is
assigned. A VALUE source with no WITH is a syntax error that [the error
rule](r:instr.parse.syntax) lists.

```rexx
w = 'W'
parse value (w) 'x' with a b
say a b
parse value 'a' 'with' w with a b c
say a b c
with = 'W-W'
parse value (with) with a '-' b
say a b
parse value with a with b
say '[' || a || '][' || b || ']'
parse value substr('abcdef', 2, 3) with a +1 b
say a b
parse value .o~new~with with a b
say '[' || a || '][' || b || ']'
exit
::class o
::method with
  return 'MSG'
```

```output
W x
a with W
W W
[][]
b cd
[MSG][]
```

r[instr.parse.var]
PARSE VAR names a variable, a simple symbol, a compound one or a stem, and
parses its string value; the string value of a stem is
[the stem rule's](r:expr.variables.stem). The variable is only read: it
changes only if it is also a target of the template, and the
[order of work](r:parse.index.order) says that its old value is the one
parsed. A variable with no value is treated like any other
[use of such a variable](r:expr.variables.value): its
name is the source, or the [`NOVALUE`](r:cond.novalue.raise) condition is
raised if it is enabled, with the name as its description, and then nothing is
assigned. A compound symbol is replaced by its derived name first.

```rexx
t = 'one two three'
parse var t first t
say first '|' t
i = 4
x.4 = 'a b'
parse var x.i p q
say p q
parse var nothing_here p q
say '[' || p || '][' || q || ']'
parse var st. m n
say '[' || m || '][' || n || ']'
st. = 'def val'
parse var st. m n
say '[' || m || '][' || n || ']'
signal on novalue name nv
parse var y.i p q
say 'not reached'
nv: say condition('c') condition('d') '[' || p || '][' || q || ']'
```

```output
one | two three
a b
[NOTHING_HERE][]
[ST.][]
[def][val]
NOVALUE Y.4 [NOTHING_HERE][]
```

r[instr.parse.version]
PARSE VERSION reads a string of five words. The first is one word that names
the interpreter and its version, the second is the language level as a
decimal number, and the last three are a date: the day as a whole number, the
month as a three-letter name and the year as four digits. The values of the
words are not specified, and neither is the date.

```rexx
parse version v
say words(v)
parse version what level day month year
say datatype(level, 'N') datatype(day, 'W') length(month) length(year)
```

```output
5
1 1 3 4
```

## Errors

r[instr.parse.syntax]
The form of the instruction is checked when the program is read, so a fault
stops the program before any clause runs, and an INTERPRET raises the error
when its text is read. A PARSE with no source keyword raises `SYNTAX 20.903`,
also when only the case keywords are there. A word that is not a case keyword
or a source keyword raises `25.12`, and so does a repeated or a conflicting
case keyword, such as UPPER with LOWER. A VAR with no symbol after it, or
something that is not a symbol, such as a string, raises `20.904`. A VAR
symbol that starts with a digit raises `31.2`, and one that starts with a
period `31.3`. A VALUE with no WITH raises `38.3`. The template errors,
`38.1`, `38.2` and `38.901`, are [theirs](r:parse.patterns.syntax), and a
fault in the VALUE expression is the error of the expression.

```rexx
say 'never shown'
parse upper caseless
```

```output rc=236
```

```rexx
say 'never shown'
parse value 'a' 'b'
```

```output rc=218
```

```rexx
call try 'parse'
call try 'parse x'
call try 'parse upper'
call try 'parse upper lower var s'
call try 'parse caseless caseless arg'
call try 'parse var'
call try "parse var 's' a"
call try 'parse var 2 a'
call try 'parse var .s a'
call try 'parse value 1 + with a'
call try 'parse value 1'
call try 'parse numeric a'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
parse => 20.903
parse x => 25.12
parse upper => 20.903
parse upper lower var s => 25.12
parse caseless caseless arg => 25.12
parse var => 20.904
parse var 's' a => 20.904
parse var 2 a => 31.2
parse var .s a => 31.3
parse value 1 + with a => 35.1
parse value 1 => 38.3
parse numeric a => 25.12
```

## Deviations

r[instr.parse.deviation.77]
The string value of a stem is its default value's, and
[NOSTRING](r:cond.nostring.raise) is raised, with the default's description,
only when the default has no string value, so a stem that holds compound
variables and no default parses as its own name. ooRexx raises
NOSTRING for every stem used as a source.

```rexx
t.1 = 'only element'
signal on nostring name ns
parse var t. a b
say 'parsed' '[' || a || '][' || b || ']'
exit
ns: say 'nostring' condition('d')
```

```output
parsed [T.][]
```
