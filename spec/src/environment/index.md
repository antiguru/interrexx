---
element: "Environment"
kind: chapter
rexxref: "Rexx Runtime Objects"
---

# Environment

This chapter specifies what surrounds a program while it runs: the objects
that every program can reach by an environment symbol without declaring
them, and the places that a name such as `.output` is looked up in.

The pages of this chapter are:

* [Environment objects](objects.md): the directories `.environment` and
  `.local`, every entry that the language puts in them, what the entries are
  bound to when a program starts, the order in which an environment symbol is
  looked up, and what a program may change in them and where the change is
  seen.
* [Invocation](invocation.md): the `rexx` command, its words and options, how
  the arguments reach the program, the status it ends with, what a program
  sees at its start, what is sent to its objects when it ends, and the
  command `rexxc` and the images it writes.
* [Search](search.md): how an external routine, a required file, a program
  called by name and a library are found: the places, the extensions, the
  case of a name, what is read again at every call, and what a program that
  is called by name leaves in its caller.
* [Command environments](commands.md): the environments that run a command
  on Linux and the program each runs, how the return code of a command
  comes out of its exit status or signal, what a command starts with, the
  commands `cd`, `export`, `set` and `unset` that the interpreter does
  itself, and the `rxqueue` filter that puts the lines of a command on a
  queue of the program.
* [Security manager](security.md): the object that is asked before the code
  it governs calls an external routine, runs a command, loads a file, reads a
  name from `.local` or `.environment`, names a stream or sends a message to a
  protected method: what a manager belongs to, which code it governs, what
  each checkpoint sends and what the answer does.
* [Debugging](debugging.md): interactive tracing: how it starts and ends, when
  the interpreter stops, what it reads at a stop and from where, what each kind
  of line does, where its own lines are written, and the `RXTRACE` variable.

An example in this chapter changes the objects that its own program reads, as
a program does. Each example restores what it changed when a later line of the
example needs the original, and an object that an example adds to `.local` or
`.environment` has a name that no other example uses.
