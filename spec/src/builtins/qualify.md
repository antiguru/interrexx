---
element: QUALIFY
kind: builtin
syntax: "QUALIFY(name)"
rexxref: "Functions > Built-in Functions > QUALIFY"
deviations: [34]
---

# QUALIFY

QUALIFY answers the full name of a file: the name made absolute, with the
current directory put in front of a relative one and the parts of the path
that go nowhere taken out. It looks at no file, so the file need not exist.
The rules here are those of a Unix-like system, where a name is
separated into its components by slashes; the names of other platforms are
not specified.

> **Note.** The result of a relative name depends on the current directory. The
> examples build their expected value from the current directory, which the
> DIRECTORY builtin answers when it has no argument, and name no directory.

## Arguments

r[bif.qualify.args]
`name` is a [string](r:conv.string.null), the name of a file or directory.
QUALIFY is not a String method. It takes one argument, by the
[call rules](r:bif.index.call.count): a call with none raises `SYNTAX 40.3`,
and a call with a second one raises `SYNTAX 40.4`. A name that is not a string
is used by its string value.

```rexx
signal on syntax name a
say qualify()
a: say condition('o')~code
signal on syntax name b
say qualify('x', 'y')
b: say condition('o')~code
signal on syntax name c
say 'x'~qualify
c: say condition('o')~code
signal off syntax
p = directory() || '/'
say (qualify(12) == p || '12')
```

```output
40.3
40.4
97.1
1
```

## The result

r[bif.qualify.relative]
A name that does not begin with a slash is relative to the current
directory: QUALIFY answers the current directory, a slash and the name. The
name is not looked up, so the answer is the same whether or not the file
exists, and QUALIFY creates nothing. A name with a blank in it, or one
that is a number, is a name like any other.

```rexx
p = directory() || '/'
say (qualify('spec-qualify-none.tmp') == p || 'spec-qualify-none.tmp')
say (qualify('a b/c d') == p || 'a b/c d')
say (qualify('.a') == p || '.a') (qualify('a.') == p || 'a.') (qualify('..a') == p || '..a')
say stream('spec-qualify-none.tmp', 'c', 'query exists') == ''
```

```output
1
1
1 1 1
1
```

r[bif.qualify.absolute]
A name that begins with a slash is absolute, and QUALIFY answers it with the
changes of [the next rule](r:bif.qualify.normal) and no more: the current
directory plays no part.

```rexx
say qualify('/spec-qualify/x') qualify('/') qualify('/spec-qualify/x y')
```

```output
/spec-qualify/x / /spec-qualify/x y
```

r[bif.qualify.normal]
The answer is cleaned up. A run of slashes is one slash. A component that is
a single period is dropped, and so is a slash at the end of the name. A
component of two periods, and the component before it, are dropped
together; at the root there is no component to drop, and the double period
goes alone. The root is the answer `/` when nothing else is left, and a
component of three or more periods, or of periods and other characters, is
a name.

```rexx
p = directory() || '/'
say qualify('/a//b///c') qualify('/a/./b/.') qualify('/a/b/') qualify('/a/b//')
say qualify('/a/b/../c') qualify('/a/b/c/../..') qualify('/a/..') qualify('/..') qualify('/../../a')
say qualify('//') qualify('/./') qualify('///a')
say qualify('/a/.../b') qualify('/a/..b/c') qualify('/a/.b/c/../d')
say (qualify('x//y') == p || 'x/y') (qualify('./x/') == p || 'x') (qualify('x/..') == directory())
say (qualify('.') == directory()) (qualify('./') == directory()) (qualify('x/../y') == p || 'y')
```

```output
/a/b/c /a/b /a/b /a/b
/a/c /a / / /a
/ / /a
/a/.../b /a/..b/c /a/.b/d
1 1 1
1 1 1
```

r[bif.qualify.null]
A name that is the null string has no full name: QUALIFY answers the null
string for it. A name of blanks is a name.

```rexx
p = directory() || '/'
say '[' || qualify('') || ']' '[' || qualify(copies('x', 0)) || ']'
say (qualify(' ') == p || ' ') (qualify('  ') == p || '  ')
```

```output
[] []
1 1
```

## Deviations

r[bif.qualify.deviation.34]
In every name interrexx qualifies, each `..` removes exactly the one component
before it, however the component was written. ooRexx removes none after a
doubled slash and two after a component that starts with a period, so its answer can
name a different file.

```rexx
say qualify('/a//..') qualify('/a//../b') qualify('/a/.b/../c')
```

```output
/ /b /a/c
```
