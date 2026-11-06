---
element: ARG
kind: instruction
syntax: "ARG [template_list]"
ansi: "6.3.2.36, 8.3.2"
rexxref: "Keyword Instructions > ARG"
---

# ARG

ARG parses the arguments of the running routine, method or program, after it
has made them upper case. It is a short form of [PARSE](parse.md): the clause
`ARG template_list` does what `PARSE UPPER ARG template_list` does, and this
page says what follows from that. The builtin of the same name,
[ARG](../builtins/arg.md), is a function and is not related to the
instruction except through the arguments it reports.

r[instr.arg.short]
`ARG` followed by a template list is the instruction `PARSE UPPER ARG`
followed by the same list. Each argument, in order, is the source of the
template that stands in the same place after the commas, as [the comma
rule](r:parse.multiple.comma) says, and every one of them is made upper case
before it is parsed. An argument that was omitted, and every argument
beyond those passed, is the null string. A program that was started with no
argument parses the null string.

```rexx
call show 'Mixed  Case', 'two words'
call show 'only'
call show
exit
show:
  arg a b, c d
  say '[' || a || '][' || b || '][' || c || '][' || d || ']'
  parse upper arg x y, z
  say (x == a) (y == b) (z == strip(c d))
  return
```

```output
[MIXED][ CASE][TWO][WORDS]
1 1 1
[ONLY][][][]
1 1 1
[][][][]
1 1 1
```

r[instr.arg.template]
Everything after the keyword is the template list, and ARG has no keywords of
its own. A word that PARSE would read as an option, such as `upper` or
`caseless`, is a variable of the template, and so is a source keyword. The
case of the arguments is changed and nothing else about them is, so the
template patterns are written in upper case to match, and the arguments the
caller passed are not changed. The errors of the template are
[the template errors](r:parse.patterns.syntax). An ARG with no template
assigns [nothing](r:parse.templates.empty).

```rexx
call show 'key=Value', 'extra'
exit
show:
  arg upper caseless
  say '[' || upper || '][' || caseless || ']'
  arg k '=' v
  say k v
  arg k '=' v, e
  say k v e
  arg
  say arg(1) arg(2)
  return
```

```output
[KEY=VALUE][]
KEY VALUE
KEY VALUE EXTRA
key=Value extra
```
