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

This book is the authority on the language interrexx implements. When the
interpreter and this book disagree, one of them has a defect, and the
disagreement is settled here before it is settled in code. Behavior the book
does not state is not promised.

Where interrexx knowingly differs from the reference it follows, the page
for the element says so and names the numbered deviation. Deviations are
listed in the project's register, and a page lists only the ones that touch
its element.

## Reading a rule

Every statement the book makes about behavior is a rule. A rule has an
identifier made of a chapter prefix and dotted lowercase words, such as the
`bif.` prefix for built-in functions, and it is a single short paragraph.
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

## Sources

Two documents are cited by section and are not copied: the ooRexx reference
manual and the ANSI Rexx standard. A page's front matter names the sections
that describe its element. The wording of this book is its own, and the
rules state what the language does, derived from those sources and from
reasoning about the semantics, not from observing any particular
implementation. Where the reference manual and the ANSI standard are both
silent, a rule follows the observed behavior of the reference
implementation, ooRexx, and its page says so.
