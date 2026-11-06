---
element: "Introduction"
kind: chapter
---

# Introduction

This book specifies the language that interrexx implements: the Rexx
language as ooRexx extends it. It describes what a program means, and what
the interpreter must do when it runs one.

## Who it is for

The book is written for three readers: an engineer changing the
interpreter, who needs the exact behavior before touching code; an LLM
agent doing the same work, which may read one page at a time with no
history behind it; and a reviewer judging whether a change is correct. It
assumes you can read Rexx, and it does not teach the language.

## Authority

This book is normative in its own right: its rules define the language
interrexx implements, and no other document or implementation does. When
the interpreter and a rule disagree, one of them has a defect, and the
disagreement is settled here before it is settled in code. Behavior the book
does not state is not promised, and where a reader might expect a promise
the book says explicitly that a behavior is not specified.

Rules and examples are normative. Everything else is informative: the
overview at the top of a page, and every note, which is set as a block
quote that begins **Note.** A note explains where a rule comes from or how
another definition of Rexx differs; it never adds to or changes a rule.

## Terms

**ooRexx.** The open-source C++ implementation of Object Rexx, version 5. interrexx is
written to run ooRexx programs, and the book's examples are also run on
ooRexx to measure where the two agree.

**Deviation.** A declared difference between interrexx and ooRexx, numbered in
[the Deviations chapter](deviations.md). A page lists in its front matter the deviations that
touch its element, and states each as a rule whose identifier ends
`deviation.N`; a builtin's page puts these rules under a Deviations heading
at its end. Every example on which interrexx and ooRexx answer
differently belongs to such a rule; any other difference is a defect.

**String method.** A method of the `String` class. Many builtins run as one, as
[the conventions](r:conv.method) define.

**Identity.** Whether two values are the same object, as opposed to equal strings. It
is observable only through `identityHash` and the identity collections
(`IdentityTable`, `IdentitySet`), and the book says where it is not
specified.

## Reading a rule

Every statement the book makes about behavior is a rule. A rule has an
identifier of dotted lowercase words, and it is a single short paragraph.

An identifier names the page that holds its rule, so a reader of the raw
Markdown finds any rule from its identifier alone. The first word names the
chapter: `bif` for built-in functions in `builtins/`, `instr` for
`instructions/`, `expr` for `expressions/`, `parse` for `parsing/`, `cond`
for `conditions/`, `class` for `classes/`, `lex` for `lexical/`, `num` for
`numbers/`, `dir` for `directives/`, `util` for `rexxutil/` and `env` for
`environment/`. The second word is the page's file name: `bif.substr.pad` is
in `builtins/substr.md`, and the rules for a whole chapter, such as
`bif.index.call.count`, are in its `index.md`. The conventions are one page,
`conventions.md`, and every `conv.` rule is there. The checker rejects a rule
on any other page.

Identifiers never change and are never reused. Other pages refer to a rule by
that identifier, and so do comments in the test sources: a test that
exercises a rule cites it, and the checker reports any rule that has neither
a cited test nor an example.

## Reading an example

An example is a short program followed by the exact text it writes to
standard output. When the program ends with a non-zero exit code, the output
is marked with that code. The checker runs every example with an empty
standard input and compares the output byte for byte, so an example is
evidence, not decoration. An example belongs to the rule above it.

An output marked `defect=#N` (combinable with the exit code, as in
`output defect=#12 rc=1`) is an example interrexx fails today because of open
issue N. The output shown is still the specified one, and the checker accepts
the mismatch on interrexx; when the example starts to pass, it reports that
the marker is to be removed.

An output marked `escaped` (combinable with the other markers) writes a
character that cannot be shown as itself with an escape: `\a` or `\x07` is the
bell character, `\xNN` is the ASCII character of code `NN` in hexadecimal,
and `\\` is a backslash. The checker decodes the escapes before it compares,
and any other backslash in such a block is an error.

## Sources

Two documents are cited by section and are not copied: the ooRexx reference
manual and the ANSI Rexx standard (X3.274-1996, cited from its final public
draft, so the notes call it the ANSI draft). A page's front matter names the
sections that describe its element. Both are informative: the rules are
derived from them and from reasoning about the semantics, and where the two
disagree the rule follows ooRexx and a note says how ANSI differs. Where
both are silent, a rule matches ooRexx, and a note says so. The wording of
this book is its own.
