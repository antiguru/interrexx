---
element: "Environment"
kind: chapter
rexxref: "Rexx Runtime Objects"
---

# Environment

This chapter specifies what surrounds a program while it runs: the objects
that every program can reach by an environment symbol without declaring
them, and the places that a name such as `.output` is looked up in.

The pages of this chapter so far are:

* [Environment objects](objects.md): the directories `.environment` and
  `.local`, every entry that the language puts in them, what the entries are
  bound to when a program starts, the order in which an environment symbol is
  looked up, and what a program may change in them and where the change is
  seen.
* [Invocation](invocation.md): the `rexx` command, its words and options, how
  the arguments reach the program, the status it ends with, what a program
  sees at its start, and what is sent to its objects when it ends.

Pages on how external routines and programs are found, commands and their
environments, the security manager and interactive debugging follow. Until
they exist, the rules of the other chapters that mention those subjects in
plain text are the rules there are.

An example in this chapter changes the objects that its own program reads, as
a program does. Each example restores what it changed when a later line of the
example needs the original, and an object that an example adds to `.local` or
`.environment` has a name that no other example uses.
