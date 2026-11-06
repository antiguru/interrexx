---
element: DIRECTORY
kind: builtin
syntax: "DIRECTORY([newdirectory])"
rexxref: "Functions > Built-in Functions > DIRECTORY"
---

# DIRECTORY

DIRECTORY answers the name of the current directory, and with an argument
first makes another directory the current one. The current directory is where
a relative file name starts: the [stream builtins](r:bif.index.stream.name),
[QUALIFY](qualify.md) and the commands the program starts all read it.

The rules here are those of a Unix-like system, where the components of a
name are separated by slashes. The names of other platforms are not
specified.

> **Note.** The current directory at the start of a program belongs to
> whoever started it, so the examples name no directory of their own. They
> build their expected values from the answer of a call without an argument,
> or use the root directory, `/`, which every Unix-like system has.

## Arguments

r[bif.directory.args]
`newdirectory` is a [string](r:conv.string.null), and it is optional.
DIRECTORY does not run as a String method: sending the message `DIRECTORY`
to a string raises `SYNTAX 97.1`. It does not follow the [call
rules](r:bif.index.call.count) for the number of arguments: a second
argument raises `SYNTAX 88.922`, even when the first is omitted, and an
argument that is an object with no string value, such as `.nil`, raises
`SYNTAX 88.909`. A trailing omitted place adds nothing to the count, so
`DIRECTORY(,)` is a call without an argument.

```rexx
here = directory()
signal on syntax name a
say directory('/', 'x')
a: say condition('o')~code
signal on syntax name b
say directory(, 'x')
b: say condition('o')~code
signal on syntax name c
say directory(.nil)
c: say condition('o')~code
signal on syntax name d
say '/'~directory
d: say condition('o')~code
signal off syntax
say (directory(,) == here) (directory() == here)
```

```output
88.922
88.922
88.909
97.1
1 1
```

## The result

r[bif.directory.get]
A call without an argument answers the full name of the current directory
and changes nothing. The name is absolute: it starts with a slash, it has no
component of one or two periods, no run of slashes and no slash at the end,
except that the name of the root directory is the single slash. Symbolic
links in it are resolved.

```rexx
here = directory()
say left(here, 1) (directory() == here) (pos('//', here) = 0) (pos('/./', here || '/') = 0)
call directory '/'
say directory()
```

```output
/ 1 1 1
/
```

r[bif.directory.set]
A call with an argument that names an existing directory makes that
directory the current one and answers its full name, in the form of [the
rule above](r:bif.directory.get). A name that does not start with a slash is
taken from the current directory. Components of one and two periods, repeated
slashes and a slash at the end are allowed in the argument, and a symbolic
link is followed, so the answer can differ from the argument in all of
these. The change stays until another call changes it again: ending a routine
does not undo it.

```rexx
here = directory()
parent = directory('..')
base = filespec('N', here)
say directory('/usr/../') directory('//')
say (directory(here || '/') == here) (directory(parent || '//' || base || '/./') == here)
call sub
say directory() == '/'
call directory here
say (directory(parent) == parent) (directory(base || '/.') == here)
exit
sub: call directory '/'
return
```

```output
/ /
1 1
1
1 1
```

r[bif.directory.fail]
A call with an argument answers the null string, and leaves the current
directory as it was, when the argument does not name a directory that can be
made the current one: a name that no file has, a name that is a file other
than a directory, and the null string. The null string is a name, not an
omitted argument, so it does not ask for the current directory. A name is
taken as written: blanks at its start or end are part of it.

```rexx
here = directory()
say '[' || directory('/spec-directory-none') || ']' '[' || directory('') || ']'
say '[' || directory('/dev/null') || ']' '[' || directory(' /') || ']' '[' || directory('/ ') || ']'
say directory() == here
```

```output
[] []
[] [] []
1
```

r[bif.directory.home]
An argument that is `~` alone, or that starts with `~/`, has that `~` replaced
by the value of the environment variable `HOME`, and the result is taken as
above. When `HOME` has no value, has the null string as its value or names no
directory, the call answers the null string and changes nothing. A `~` that
is followed by anything else, such as `~user`, is not replaced: it is the
first component of a relative name.

```rexx
call value 'HOME', '/', 'ENVIRONMENT'
say directory('~') directory('/usr') directory('~/') directory('~//tmp')
call value 'HOME', '/spec-directory-none', 'ENVIRONMENT'
say '[' || directory('~') || ']' directory()
call value 'HOME', '', 'ENVIRONMENT'
say '[' || directory('~') || ']' directory()
call value 'HOME', '/', 'ENVIRONMENT'
say '[' || directory('~spec-directory-none') || ']' directory()
```

```output
/ /usr / /tmp
[] /tmp
[] /tmp
[] /tmp
```

## What the current directory affects

r[bif.directory.effect]
A change of the current directory is seen by everything that takes a
relative name afterwards: QUALIFY puts the new directory in front of a
relative name, a stream builtin opens a relative file name in it, and a
command started with ADDRESS runs in it.

```rexx
call directory '/'
say qualify('x') qualify('./y/z')
say stream('spec-directory-none.tmp', 'c', 'query exists') == ''
address system 'pwd' with output stem out.
say out.1
```

```output
/x /y/z
1
/
```
