---
element: FILESPEC
kind: builtin
syntax: "FILESPEC(option, filespec)"
rexxref: "Functions > Built-in Functions > FILESPEC"
---

# FILESPEC

FILESPEC takes a file specification apart and answers one piece of it: its
directory, its file name or the extension of that name. It works on the
characters of the specification alone. It looks at no file, so the file need
not exist, and it never asks the file system anything.

The rules here are those of a Unix-like system, where the components of a
specification are separated by slashes. The specifications of other
platforms, such as those with a drive letter or with backslashes, are not
specified.

## Arguments

r[bif.filespec.args]
`option` is a string whose first character picks the piece, as [the next
rule](r:bif.filespec.option) says, and `filespec` is a
[string](r:conv.string.null); both are required. FILESPEC does not run as a
[String method](r:class.string.builtins): sending the message `FILESPEC` to a
string raises `SYNTAX 97.1`. It follows [the native call
rule](r:bif.index.call.native), both for the number of arguments and for an
argument that is an object other than a string, such as `.nil`.

```rexx
signal on syntax name a
say filespec()
a: say condition('o')~code
signal on syntax name b
say filespec('N')
b: say condition('o')~code
signal on syntax name c
say filespec(, 'a/b')
c: say condition('o')~code
signal on syntax name d
say filespec('N', 'a/b', 'c')
d: say condition('o')~code
signal on syntax name e
say filespec('N', .nil)
e: say condition('o')~code
signal on syntax name f
say 'a/b'~filespec('N')
f: say condition('o')~code
signal on syntax name g
say filespec('N', .told~new)
g: say condition('o')~code
signal off syntax
say filespec('N', 'a/b', ) filespec(.array~of('N'), .mutablebuffer~new('a/b'))
::class told
::method string
  return 'a/b'
```

```output
88.901
88.901
88.901
88.922
88.909
97.1
88.909
b b
```

r[bif.filespec.option]
The first character of `option` picks the piece, in either case, and the
characters after it are ignored. The pieces are `Drive`, `Extension`,
`Location`, `Name` and `Path`, so the first character is one of `D`, `E`,
`L`, `N` and `P`. Any other first character, and the null string, raises
`SYNTAX 40.904`. A blank is a character: it does not pick a piece, and
nothing strips it. The checks of [the arguments](r:bif.filespec.args) come
first, so a call with a wrong `option` and a wrong argument raises the
error of the argument.

```rexx
f = '/usr/lib/libm.so'
say filespec('Name', f) filespec('NAMEXYZ', f) filespec('n', f)
say filespec('path', f) filespec('Location', f) filespec('P', f)
say '[' || filespec('drive', f) || ']' filespec('ext', f) filespec('e', f)
signal on syntax name a
say filespec('', f)
a: say condition('o')~code
signal on syntax name b
say filespec('X', f)
b: say condition('o')~code
signal on syntax name c
say filespec(' N', f)
c: say condition('o')~code
signal on syntax name d
say filespec('X', .nil)
d: say condition('o')~code
signal on syntax name e
say filespec('X', f, 'x')
e: say condition('o')~code
signal on syntax name g
say filespec('X')
g: say condition('o')~code
```

```output
libm.so libm.so libm.so
/usr/lib/ /usr/lib/ /usr/lib/
[] so so
40.904
40.904
40.904
88.909
88.922
88.901
```

## The pieces

r[bif.filespec.path]
`Path` is the part of `filespec` up to and including its last slash, and the
null string when `filespec` has no slash. `Location` is the same string. A
specification that ends in a slash is all path, and one that is only slashes
is its own path.

```rexx
say '[' || filespec('P', '/usr/lib/libm.so') || ']' '[' || filespec('L', 'a/b') || ']'
say '[' || filespec('P', 'libm.so') || ']' '[' || filespec('L', 'libm.so') || ']'
say '[' || filespec('P', '/usr/lib/') || ']' '[' || filespec('P', 'a/b/') || ']'
say '[' || filespec('P', '/') || ']' '[' || filespec('L', '//') || ']'
say '[' || filespec('P', '/x') || ']' '[' || filespec('P', 'a//b') || ']'
```

```output
[/usr/lib/] [a/]
[] []
[/usr/lib/] [a/b/]
[/] [//]
[/] [a//]
```

r[bif.filespec.name]
`Name` is the part of `filespec` after its last slash. It is the whole of
`filespec` when there is no slash, and the null string when `filespec` ends
in a slash. A name of periods, such as `.` or `..`, is a name like another.
A `'00'x` ends `filespec`: every piece is taken from the characters before
the first `'00'x`, and the characters from it on are part of no piece. A line
feed, a tab and every other character are ordinary characters. The path and
the name together are `filespec` up to its first `'00'x`, with nothing lost
between them.

```rexx
say '[' || filespec('N', '/usr/lib/libm.so') || ']' '[' || filespec('N', 'libm.so') || ']'
say '[' || filespec('N', '/usr/lib/') || ']' '[' || filespec('N', '/') || ']'
say '[' || filespec('N', '.') || ']' '[' || filespec('N', '/x/..') || ']'
say '[' || filespec('N', 'x y/z w') || ']' '[' || filespec('P', 'x y/z w') || ']'
r = ''
do s over .array~of('/a/b.c', 'a//b', '/', '', 'a/', ' a/b ')
  r = r (filespec('P', s) || filespec('N', s) == s)
end
say strip(r)
s = 'x/y.v' || '00'x || '/z.w'
say '[' || filespec('P', s) || ']' '[' || filespec('N', s) || ']' '[' || filespec('E', s) || ']'
say filespec('N', 'a/b' || '0a'x || 'c') == 'b' || '0a'x || 'c'
```

```output
[libm.so] [libm.so]
[] []
[.] [..]
[z w] [x y/]
1 1 1 1 1 1
[x/] [y.v] [v]
1
```

r[bif.filespec.extension]
`Extension` is the part of the name after its last period. It is the null
string when the name has no period and when its last period is its last
character. A period that starts the name counts like any other, so a name
that is only an extension has it as its extension. Periods in the path
before the name play no part.

```rexx
say '[' || filespec('E', 'libm.so') || ']' '[' || filespec('E', 'a.b.c') || ']'
say '[' || filespec('E', 'archive.tar.gz') || ']' '[' || filespec('E', '/x/.hidden') || ']'
say '[' || filespec('E', 'noext') || ']' '[' || filespec('E', 'dot.') || ']'
say '[' || filespec('E', '/a.b/c') || ']' '[' || filespec('E', 'a.b/') || ']'
say '[' || filespec('E', '/a/b.c.') || ']' '[' || filespec('E', '..x') || ']'
say '[' || filespec('E', 'x/b.c d') || ']' '[' || filespec('E', '.') || ']'
```

```output
[so] [c]
[gz] [hidden]
[] []
[] []
[] [x]
[c d] []
```

r[bif.filespec.drive]
`Drive` is the null string, whatever `filespec` is: a Unix-like system has no
drives, and a colon or a letter that looks like a drive is part of the name.

```rexx
say '[' || filespec('D', '/usr/lib') || ']' '[' || filespec('D', 'c:/x.txt') || ']'
say '[' || filespec('D', 'C:\win\f.txt') || ']' '[' || filespec('D', '') || ']'
say filespec('N', 'c:/x.txt') filespec('N', 'C:\win\f.txt')
```

```output
[] []
[] []
x.txt C:\win\f.txt
```

r[bif.filespec.null]
The null string has no pieces: every option answers the null string for it.
A specification that is a number or has blanks in it is a specification like
any other, taken by its string value.

```rexx
r = ''
do o over .array~of('D', 'E', 'L', 'N', 'P')
  r = r || '[' || filespec(o, '') || ']'
end
say strip(r)
say filespec('N', 12) filespec('E', 1.5) filespec('N', 'a b.c d')
```

```output
[][][][][]
12 5 a b.c d
```
