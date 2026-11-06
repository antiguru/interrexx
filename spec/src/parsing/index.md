---
element: "Parsing"
kind: chapter
ansi: "6.3.2.71, 8.3.17"
rexxref: "Parsing"
---

# Parsing

Parsing takes a string, the source, and a template, and assigns parts of the
source to variables. The template says where to cut the source and which
variable receives each part. It is the one mechanism behind PARSE, ARG and
PULL: those instructions differ only in where the source comes from and in
whether they change its case, and the instructions chapter specifies them.
This chapter specifies what a template does with a source once there is one.

A template is a list of targets and triggers. A target is a place that
receives a value, usually a variable. A trigger is a pattern that finds a
place in the source, or a position that names one. The pages of this chapter
are:

* [Templates](templates.md): templates made of targets alone, which cut the
  source into words, and the period that takes a word and discards it.
* [Patterns](patterns.md): the triggers: strings, positions, relative
  positions and patterns that come from a variable or an expression.
* [Several strings](multiple.md): the comma that moves on to the next
  template and the next source string, and the options that change the case
  of the source or of the comparison.

PARSE VALUE and PARSE VAR are the sources the examples use, because each
takes a single string. The ARG source is the only one that provides several
strings, and [its template list](r:parse.multiple.comma) is on the last page.

## What a template is made of

r[parse.index.elements]
A template holds these elements, in any order and number. A target is a
variable symbol of any of the three forms, a message term that can stand on
the left of an assignment, or a period. A trigger is a string in quotes, an
expression in parentheses, or a position: a whole number, or a sign or `=`
followed by a whole number or by an expression in parentheses. A comma ends
the template and begins the next one. Any other token, an operator character
for one, is a syntax error, and [the pattern
page](r:parse.patterns.syntax) describes it.

```rexx
data = 'tag:12:east'
parse var data kind ':' count ':' side
say kind count side
parse var data . 4 text 7 .
say text
```

```output
tag 12 east
:12
```

r[parse.index.pieces]
The triggers of a template divide the source into pieces. Each piece lies
between two triggers, or between the start or end of the source and a
trigger, and the targets that stand between those two triggers share it. A
template with no trigger has one piece, the whole source. The targets of a
piece take its words as [the templates page](r:parse.templates.words)
says, and a piece with no target is skipped. Where each trigger lies, and so
where a piece begins and ends, is the subject of [the patterns
page](r:parse.patterns.string).

```rexx
parse value 'alpha beta;gamma delta' with u v ';' w y
say u'|'v'|'w'|'y
parse value 'alpha beta;gamma delta' with u v w y
say u'|'v'|'w'|'y
```

```output
alpha|beta|gamma|delta
alpha|beta;gamma|delta|
```

## Order of work

r[parse.index.order]
The interpreter evaluates the source before it assigns anything, so a target
that is also the variable of the source does not change what is parsed. It
then goes through the template from left to right, and assigns each target,
and evaluates each parenthesized trigger, at the moment it reaches it. A
pattern or a position therefore sees the values that earlier targets of the
same template have just received, and so does the tail of a compound symbol
that is a target. When the same variable appears as a target twice, the later
assignment is the one that remains.

```rexx
u = 'ef'
parse value 'ab,cd,ab,ef' with u ',' v ',' (u) w
say u v w
text = 'one two three'
parse var text first text
say first'|'text
i = 1
parse value 'p,2,r' with a.i ',' i ',' a.i
say a.1 i a.2
parse value 'm n' with k k
say '['k']'
```

```output
ab cd ,ef
one|two three
p 2 r
[n]
```

r[parse.index.nothing]
A parse changes only its targets. No other variable changes, the special
variables `RC` and `RESULT` are not set, and a template with no
target changes nothing at all, even when it contains triggers that find
nothing. The source expression of PARSE VALUE is still evaluated.

```rexx
rc = 'keep'; result = 'also'
parse value 'abc' with
parse value 'abc' with 'b' "z"
say rc result
parse value tell() with
say 'done'
exit
tell: say 'evaluated'; return 'abc'
```

```output
keep also
evaluated
done
```
