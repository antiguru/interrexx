---
element: "File"
kind: class
rexxref: "Builtin Classes > Utility Classes > File Class; Builtin Classes > Utility Classes > File Class > isCaseSensitive (Class Method); Builtin Classes > Utility Classes > File Class > listRoots (Class Method); Builtin Classes > Utility Classes > File Class > pathSeparator (Class Method); Builtin Classes > Utility Classes > File Class > readChars (Class Method); Builtin Classes > Utility Classes > File Class > readLines (Class Method); Builtin Classes > Utility Classes > File Class > searchPath (Class Method); Builtin Classes > Utility Classes > File Class > separator (Class Method); Builtin Classes > Utility Classes > File Class > temporaryPath (Class Method); Builtin Classes > Utility Classes > File Class > writeChars (Class Method); Builtin Classes > Utility Classes > File Class > writeLines (Class Method); Builtin Classes > Utility Classes > File Class > absoluteFile; Builtin Classes > Utility Classes > File Class > absolutePath; Builtin Classes > Utility Classes > File Class > canRead; Builtin Classes > Utility Classes > File Class > canWrite; Builtin Classes > Utility Classes > File Class > compareTo; Builtin Classes > Utility Classes > File Class > delete; Builtin Classes > Utility Classes > File Class > exists; Builtin Classes > Utility Classes > File Class > extension; Builtin Classes > Utility Classes > File Class > hashCode; Builtin Classes > Utility Classes > File Class > init; Builtin Classes > Utility Classes > File Class > isCaseSensitive; Builtin Classes > Utility Classes > File Class > isDirectory; Builtin Classes > Utility Classes > File Class > isFile; Builtin Classes > Utility Classes > File Class > isHidden; Builtin Classes > Utility Classes > File Class > lastAccessed (Attribute); Builtin Classes > Utility Classes > File Class > lastModified (Attribute); Builtin Classes > Utility Classes > File Class > length; Builtin Classes > Utility Classes > File Class > list; Builtin Classes > Utility Classes > File Class > listFiles; Builtin Classes > Utility Classes > File Class > makeDir; Builtin Classes > Utility Classes > File Class > makeDirs; Builtin Classes > Utility Classes > File Class > makeString; Builtin Classes > Utility Classes > File Class > name; Builtin Classes > Utility Classes > File Class > parent; Builtin Classes > Utility Classes > File Class > parentFile; Builtin Classes > Utility Classes > File Class > path; Builtin Classes > Utility Classes > File Class > pathSeparator; Builtin Classes > Utility Classes > File Class > renameTo; Builtin Classes > Utility Classes > File Class > separator; Builtin Classes > Utility Classes > File Class > setReadOnly; Builtin Classes > Utility Classes > File Class > setWritable; Builtin Classes > Utility Classes > File Class > string; Builtin Classes > Utility Classes > Orderable Class > Comparison Methods"
deviations: [34, 109, 118]
---

# File

A `File` stands for a path to a file or to a directory. The path may name
something that does not exist: a `File` holds only the path, and a method that
asks about the file or changes it looks at the file system when it is called.
The class also has methods that read and write whole files, search a list of
directories for a file, and name the places where files live. The rules of this
page are those of a Unix-like system, where the components of a path are
separated by slashes, as [QUALIFY](../builtins/qualify.md) assumes; the paths of
other systems are not specified. Nor are the choices of the file system and the
host: whether names that differ in case are different names, the home directory,
the temporary directory, the size of a directory, how precisely a file keeps its
times, and what the user may do with a file that the examples did not make. The
examples make files and directories under names of their own, derive every
expected value from what they made, and remove what they made. The class
inherits [`Comparable`](r:class.comparable.creation) and `Orderable`, and orders
its instances by their absolute paths; the comparison operators of an instance
are [those of `Orderable`](r:class.orderable.compare).

## The class

r[class.file.creation]
`File` has the instance methods `absoluteFile`, `absolutePath`, `canRead`,
`canWrite`, `compareTo`, `delete`, `exists`, `extension`, `hashCode`, `init`,
`isCaseSensitive`, `isDirectory`, `isFile`, `isHidden`, `lastAccessed`,
`lastAccessed=`, `lastModified`, `lastModified=`, `length`, `list`,
`listFiles`, `makeDir`, `makeDirs`, `makeString`, `name`, `parent`,
`parentFile`, `path`, `pathSeparator`, `renameTo`, `separator`, `setReadOnly`,
`setWritable` and `string`, and the class methods `isCaseSensitive`,
`listRoots`, `pathSeparator`, `readChars`, `readLines`, `searchPath`,
`separator`, `temporaryPath`, `writeChars` and `writeLines`, besides the `new`
that every class has. It may have private methods besides these; they are not
part of this specification. Its superclasses are `Object`, `Comparable` and
`Orderable`, in that order. An instance has no `unknown` method, so a message
that [the method search](r:class.methods.unknown) does not resolve raises
[NOMETHOD](r:cond.nomethod.raise), as [the error rule of the
chapter](r:class.index.errors) says. It is not a string either, so a string
method such as `left` raises `SYNTAX 97.1`.

```rexx
do c over .file~superclasses
  say c~id
end
names = 'absoluteFile absolutePath canRead canWrite compareTo delete exists',
  'extension hashCode init isCaseSensitive isDirectory isFile isHidden',
  'lastAccessed lastAccessed= lastModified lastModified= length list',
  'listFiles makeDir makeDirs makeString name parent parentFile path',
  'pathSeparator renameTo separator setReadOnly setWritable string'
found = 0
do n over names~makearray(' ')
  found += .file~method(n)~isA(.method)
end
say found 'of' names~words
cnames = 'isCaseSensitive listRoots pathSeparator readChars readLines',
  'searchPath separator temporaryPath writeChars writeLines new'
found = 0
do n over cnames~makearray(' ')
  found += .file~hasMethod(n)
end
say found 'of' cnames~words
say .file~new('x')~hasMethod('UNKNOWN')
signal on syntax name bad
say .file~new('x')~left(1)
bad: say condition('o')~code
```

```output
Object
Comparable
Orderable
34 of 34
11 of 11
0
97.1
```

## Making a File

r[class.file.new]
`new` sends `init` ([the new rule](r:class.index.new)). `init` takes a `path`
and an optional `dir`. The `path` is read as [a string
argument](r:class.index.string): a `File` given as `path` is read by its
`makeString`, which is its absolute path, and an argument with no string value
raises `SYNTAX 88.909`. A call with no argument raises `SYNTAX 93.901`, one
with a third argument `93.902`, and one with `dir` and no `path` `93.903`
([the argument rules](r:class.index.args.count)). The path that the instance
holds is made in these steps. First `path` and `dir` each lose their trailing
slashes, but a lone slash keeps its slash: `a//b///` is `a//b`, `///` is `/`,
and the empty string stays empty; slashes anywhere else are kept. Then, when
`dir` is omitted or `.nil`, the path is the `path` that is left. Otherwise it
is `dir`, a slash and `path`, where the slash is left out when `dir` ends in
one or `path` begins with one, and where both supply one a single slash
stands. A `dir` that is the empty string counts as a `dir`, so `a` with an
empty `dir` is `/a`. A `dir` that is a `File` is read as its absolute path,
and any other `dir` as a [string argument](r:class.index.string), with the
error that a `path` has. The text of the path is kept as it was written, apart
from these steps: `path` ([the path rule](r:class.file.path)) answers it, and
a `File` made with a relative `path` or `dir` is relative to the current
directory when it is made ([the absolute rule](r:class.file.absolute)). `init`
sent to an instance again changes that instance as `new` made it, and answers
nothing.

```rexx
say .file~new('a//b///')~path .file~new('///')~path '['.file~new('')~path']'
say .file~new('a', 'd')~path .file~new('a/', 'd//')~path,
    .file~new('/a', 'd')~path
say .file~new('a', '/d/')~path .file~new('', 'd')~path .file~new('a', '')~path
say .file~new('a', .nil)~path .file~new('a', )~path .file~new('/a', '/')~path
say .file~new('a', .file~new('/x/y'))~path
say (.file~new('a', .file~new('q'))~path == directory() || '/q/a')
say .file~new(12)~path .file~new('a b', 'c d')~path .file~new('a\b')~path
signal on syntax name b1
say .file~new
b1: say condition('o')~code
signal on syntax name b2
say .file~new('a', 'b', 'c')
b2: say condition('o')~code
signal on syntax name b3
say .file~new(, 'd')
b3: say condition('o')~code
signal on syntax name b4
say .file~new(.nil)
b4: say condition('o')~code
signal on syntax name b5
say .file~new('a', .object~new)
b5: say condition('o')~code
f = .file~new('first')
f~init('second', 'dir')
say f~path
signal on syntax name b6
x = f~init('third')
b6: say condition('o')~code
```

```output
a//b / []
d/a d/a d/a
/d/a d/ /a
a a /a
/x/y/a
1
12 c d/a b a\b
93.901
93.902
93.903
88.909
88.909
dir/second
91.999
```

r[class.file.new.sends]
`init` sends the receiver `separator` before it forms the path, and the answer
is the separator that it removes at the end of `path` and `dir` and puts between
them; it is sent once when `dir` is omitted or `.nil` and three times when a
`dir` is given. A `File` given as `dir` is sent `isA` with the argument `.File`
and, when that answers true, `absolutePath`, before the first `separator`; any
other `dir` is read as a string. A `File` given as `path` is read by its
`makeString`, after the first `separator`, and `path` is sent no `isA`. A `File`
that a method of `File` makes, such as the answer of `absoluteFile`, is made by
`new` sent to the class of the receiver, so these messages are sent to it too,
and a subclass of `File` gets instances of itself. The answer of `separator`
plays no part in the absolute path, which is made as [the absolute
rule](r:class.file.absolute) says.

```rexx
.environment~log = .array~new
f = .logfile~new('a')
call show 'new path'
g = .logfile~new('b', f)
call show 'new path, File dir'
h = .logfile~new(f)
call show 'new File path'
i = .logfile~new('b', 'dd')
call show 'new path, string dir'
x = f~absoluteFile
call show 'absoluteFile'
say .sharp~new('a##')~path .sharp~new('a/b', 'd#')~path
exit
show:
  line = ''
  last = ''
  do w over .environment~log
    if w \== last then line = line w
    last = w
  end
  say arg(1)':' line
  .environment~log~empty
  return
::class logfile subclass file
::method separator
  .environment~log~append('separator')
  forward class (super)
::method absolutePath
  .environment~log~append('absolutePath')
  forward class (super)
::method makeString
  .environment~log~append('makeString')
  forward class (super)
::method isA
  .environment~log~append('isA' arg(1)~id)
  forward class (super)
::class sharp subclass file
::method separator
  return '#'
```

```output
new path:  separator
new path, File dir:  isA File absolutePath separator
new File path:  separator makeString
new path, string dir:  separator
absoluteFile:  separator
a d#a/b
```

## The path and its text

r[class.file.path]
`path` answers the path as `init` made it ([the new rule](r:class.file.new)):
a string that is not changed when the current directory changes and that need
not be an absolute path. `string` answers the same string, and `makeString`
answers the absolute path ([the absolute rule](r:class.file.absolute)). Each
takes no argument; one raises `SYNTAX 93.902`. A `File` that is written by
`say`, joined to a string or given where a string argument is read has its
`makeString` as its text, since `makeString` is what [the string
rule](r:class.index.string) finds first, so those show the absolute path; the
answer of `string` is the path. `objectName`, `defaultName` and `objectName=`
are those of `Object`: the name of an instance is `a File` unless it was set,
and setting it changes neither `path` nor `string`.

```rexx
f = .file~new('p/q')
say f~path f~string (f~makeString == directory() || '/p/q')
say (f == .file~new('p/q')) (f~string == 'p/q')
say (('['f']') == '['directory() || '/p/q]') (f 'x' == f~makeString 'x')
say f~objectName '|' f~defaultName
f~objectName = 'nm'
say f~objectName f~string f~path
signal on syntax name b1
say f~path(1)
b1: say condition('o')~code
signal on syntax name b2
say f~string(1)
b2: say condition('o')~code
signal on syntax name b3
say f~makeString(1)
b3: say condition('o')~code
```

```output
p/q p/q 1
1 1
1 1
a File | a File
nm p/q p/q
93.902
93.902
93.902
```

r[class.file.absolute]
`absolutePath` answers the absolute path, a string, and `absoluteFile` a new
`File` that holds it as its path. Both take no argument. The absolute path is
worked out when the instance is made, from the current directory at that moment,
and a later change of the current directory does not change it. It is what
[QUALIFY](r:bif.qualify.relative) answers for the path at that moment: an
absolute path is [cleaned up](r:bif.qualify.normal) and a relative one is
qualified against the current directory, the empty path has the empty string as
its absolute path ([the null rule](r:bif.qualify.null)), and `..` follows
[Deviation 34](r:class.file.deviation.34). `absoluteFile` of an absolute `File`
holds the same path.

```rexx
here = directory()
say .file~new('/a/./b//c/../d')~absolutePath .file~new('/a/b/')~absolutePath
say .file~new('/..')~absolutePath .file~new('/x/y/../..')~absolutePath
say (.file~new('p/q')~absolutePath == here || '/p/q')
say (.file~new('./p/../q')~absolutePath == here || '/q')
say (.file~new('.')~absolutePath == here) (.file~new('')~absolutePath == '')
say (.file~new('~/x')~absolutePath \== here || '/~/x')
say (.file~new('x~')~absolutePath == here || '/x~')
f = .file~new('p/q')
g = f~absoluteFile
say g~class~id (g~path == f~absolutePath) (g~absoluteFile~path == g~path)
d = 'spec-file-abs.tmp'
junk = .file~new(d)~makeDirs
before = .file~new('x')
call directory d
after = .file~new('x')
say (before~absolutePath == here || '/x'),
    (after~absolutePath == here || '/' || d || '/x')
call directory here
say (after~absolutePath == here || '/' || d || '/x') after~path
junk = .file~new(d)~delete
signal on syntax name bad
say f~absolutePath(1)
bad: say condition('o')~code
```

```output
/a/b/d /a/b
/ /
1
1
1 1
1
1
File 1 1
1 1
1 x
93.902
```

r[class.file.separators]
`separator` answers the string that separates the components of a path, and
`pathSeparator` the string that separates the directories of a search path. On
a Unix-like system they are `/` and `:`. Each is an instance method and a
class method, and answers the same on both sides, a string. They take no
argument; one raises `SYNTAX 93.902`. The class methods are the ones of [the
class-side rule](r:class.file.separators.class-side).

```rexx
f = .file~new('x')
say f~separator f~pathSeparator f~separator~class~id
signal on syntax name b1
say f~separator(1)
b1: say condition('o')~code
signal on syntax name b2
say f~pathSeparator(1)
b2: say condition('o')~code
```

```output
/ : String
93.902
93.902
```

r[class.file.separators.class-side]
The class methods `separator` and `pathSeparator` answer the same strings as
the instance methods ([the separators rule](r:class.file.separators)). They
take no argument; one raises `SYNTAX 93.902`.

```rexx
say .file~separator .file~pathSeparator
say (.file~separator == .file~new('x')~separator)
say (.file~pathSeparator == .file~new('x')~pathSeparator)
signal on syntax name b1
say .file~separator(1)
b1: say condition('o')~code
signal on syntax name b2
say .file~pathSeparator(1)
b2: say condition('o')~code
```

```output
/ :
1
1
93.902
93.902
```

r[class.file.iscasesensitive]
`isCaseSensitive` answers 1 when the file system that holds the path tells
names apart by their case, and 0 when it does not; for a path that does not
exist it answers for the deepest directory of the path that does. Which it is
depends on the host and is not specified. It takes no argument; one raises
`SYNTAX 93.902`. The answer decides how [`compareTo`](r:class.file.compareto)
compares paths. The example derives what it prints from the answer.

```rexx
d = .file~new('spec-file-case.tmp')
junk = d~makeDir
n = .file~new('spec-file-case.tmp/no/such/place')
say (n~isCaseSensitive == d~isCaseSensitive)
a = .file~new('spec-file-case.tmp/A')
b = .file~new('spec-file-case.tmp/a')
if a~isCaseSensitive then want = -1
else want = 0
say (a~compareTo(b) == want)
say (.file~new('/')~isCaseSensitive == .file~isCaseSensitive)
signal on syntax name bad
say a~isCaseSensitive(1)
bad: say condition('o')~code
junk = d~delete
```

```output
1
1
1
93.902
```

## The parts of a path

r[class.file.name]
`name` answers the last component of the absolute path, and the empty string
for the root. `extension` answers what follows the last period of that name,
and the empty string when the name has no period, when its period is its last
character, or when its only period is its first character, as in a name such
as `.cfg`; a name such as `.b.c` has the extension `c`. A period in a
directory of the path plays no part. Both take no argument; one raises
`SYNTAX 93.902`. As they work on the absolute path, the name of `a/..` is
the name of the directory that holds `a`, and the name of `.` is the name of
the current directory. The messages that they send are those of [the next
rule](r:class.file.name.sends).

```rexx
f = .file~new('/usr/lib/x.tar.gz')
say f~name f~extension
say '['.file~new('/a/.cfg')~extension']' '['.file~new('/a/.b.c')~extension']'
say '['.file~new('/a/z.')~extension']' '['.file~new('/a.b/z')~extension']'
say '['.file~new('/a/z')~extension']' '['.file~new('/')~name']'
say .file~new('/a/b/')~name .file~new('/a/b/../c')~name,
    .file~new('/a/b/.')~name
say .file~new('rel')~name (.file~new('.')~name == .file~new(directory())~name)
signal on syntax name b1
say .file~new('x')~name(1)
b1: say condition('o')~code
signal on syntax name b2
say .file~new('x')~extension(1)
b2: say condition('o')~code
```

```output
x.tar.gz gz
[] [c]
[] []
[] []
b c b
rel 1
93.902
93.902
```

r[class.file.name.sends]
`name` sends the receiver `separator`, and `extension` sends it `separator`
and then `name`, which sends `separator` once more. The answer of `separator`
is the string that is looked for in the absolute path, so that a receiver
whose `separator` answers `#` has as its name what follows the last `#` of its
absolute path.

```rexx
.environment~log = .array~new
f = .logfile~new('/p/q.txt')
.environment~log~empty
x = f~name
call show 'name'
x = f~extension
call show 'extension'
say .sharp~new('a#b#')~name .sharp~new('a#b.c')~extension
exit
show:
  say arg(1)':' .environment~log~toString(,' ')
  .environment~log~empty
  return
::class logfile subclass file
::method separator
  .environment~log~append('separator')
  forward class (super)
::method name
  .environment~log~append('name')
  forward class (super)
::class sharp subclass file
::method separator
  return '#'
```

```output
name: name separator
extension: separator name separator
b c
```

r[class.file.parent]
`parent` answers the absolute path without its last component, a string,
and `.nil` when the absolute path is the root or is empty; the parent of `/a`
is `/`. `parentFile` answers a new `File` with that string as its path, made
by `new` sent to the class of the receiver, and `.nil` when `parent` answers
`.nil`. Both take no argument; one raises `SYNTAX 93.902`. The messages that
they send are those of [the rule after the next](r:class.file.parent.sends).

```rexx
here = directory()
say .file~new('/a/b/c')~parent .file~new('/a')~parent,
    .file~new('/a/b/../c')~parent
say (.file~new('/')~parent == .nil) (.file~new('')~parent == .nil)
say (.file~new('/')~parentFile == .nil),
    (.file~new('/a')~parentFile~parentFile == .nil)
say .file~new('/a/b/c')~parentFile~class~id,
    .file~new('/a/b/c')~parentFile~path
say (.file~new('rel')~parent == here),
    (.file~new('rel')~parentFile~path == here)
say .sub~new('/a/b')~parentFile~class~id
signal on syntax name b1
say .file~new('x')~parent(1)
b1: say condition('o')~code
signal on syntax name b2
say .file~new('x')~parentFile(1)
b2: say condition('o')~code
::class sub subclass file
```

```output
/a/b / /a
1 1
1 1
File /a/b
1 1
SUB
93.902
93.902
```

r[class.file.parent.sends]
`parent` sends the receiver `separator`, and `parentFile` sends it `parent`
and then `new` to its class, whose `init` sends `separator` as [the new
rule](r:class.file.new.sends) says.

```rexx
.environment~log = .array~new
f = .logfile~new('/p/q.txt')
.environment~log~empty
x = f~parent
say .environment~log~toString(,' ')
.environment~log~empty
x = f~parentFile
line = ''
last = ''
do w over .environment~log
  if w \== last then line = line w
  last = w
end
say line~strip
::class logfile subclass file
::method separator
  .environment~log~append('separator')
  forward class (super)
::method parent
  .environment~log~append('parent')
  forward class (super)
```

```output
parent separator
parent separator
```

## Asking about a file

r[class.file.exists]
`exists` answers 1 when something, a file or a directory, is at the absolute
path, and 0 otherwise, also for the empty path. `isFile` answers 1 when that is
a file and `isDirectory` when it is a directory, and each answers 0 where
nothing is. `length` answers the size of a file in bytes, and 0 where nothing
is; the size of a directory is not specified. `canRead` and `canWrite` answer 1
when the file exists and the process may read it or write to it, and 0
otherwise. `isHidden` answers 1 for an existing file or directory when its name
or the name of any directory above it in its absolute path starts with a period,
so for a relative path the directories of the current directory count, and 0
otherwise, so 0 where nothing is. None of them takes an argument; one raises
`SYNTAX 93.902`.

> **Note.** The example assumes that no directory above the examples' current
> directory has a name that starts with a period, and that the user is not the
> superuser, for whom `canWrite` would not follow the permissions.

```rexx
d = .file~new('spec-file-query.tmp')
f = .file~new('spec-file-query.tmp/data.txt')
n = .file~new('spec-file-query.tmp/none')
say d~exists d~isDirectory d~isFile f~exists n~exists
junk = d~makeDir
call lineout f~path, 'abc'
call lineout f~path
say f~exists f~isFile f~isDirectory f~length
say d~exists d~isDirectory d~isFile
say n~exists n~isFile n~isDirectory n~length n~canRead n~canWrite n~isHidden
say f~canRead f~canWrite d~canRead d~canWrite (d~length >= 0)
h = .file~new('spec-file-query.tmp/.dot')
say h~isHidden
call lineout h~path, 'x'
call lineout h~path
say h~isHidden f~isHidden d~isHidden
k = .file~new('spec-file-query.tmp/.hd/sub')
say k~isHidden
junk = k~makeDirs
say k~isHidden k~parentFile~isHidden
say .file~new('/')~exists .file~new('/')~isDirectory .file~new('')~exists
line = ''
names = 'exists isFile isDirectory length canRead canWrite isHidden'
do m over names~makearray(' ')
  line = line try(m)
end
say line~strip
junk = h~delete
junk = k~delete
junk = k~parentFile~delete
junk = f~delete
junk = d~delete
exit
try:
  signal on syntax name bad
  interpret 'r = f~' || arg(1) || '(1)'
  return 'none'
bad: return condition('o')~code
```

```output
0 0 0 0 0
1 1 0 4
1 1 0
0 0 0 0 0 0 0
1 1 1 1 1
0
1 0 0
0
1 1
1 1 0
93.902 93.902 93.902 93.902 93.902 93.902 93.902
```

## Changing files and directories

r[class.file.delete]
`delete` removes the file or the empty directory at the absolute path and
answers 1, and answers 0 when there is nothing there, when the directory is not
empty, when it cannot remove it, and when it is a file that the process may not
write to, a read-only one as `setReadOnly` makes: the permissions of a directory
do not keep it from being removed. It takes no argument; one raises
`SYNTAX 93.902`. It sends the receiver `isDirectory`, and [the next
rule](r:class.file.delete.isdirectory) says what its answer decides.

```rexx
d = .file~new('spec-file-delete.tmp')
f = .file~new('spec-file-delete.tmp/f')
say d~delete
junk = d~makeDir
call lineout f~path, 'x'
call lineout f~path
say d~delete d~exists
f~setReadOnly
say f~delete f~exists
f~setWritable
say f~delete f~exists f~delete
d~setReadOnly
say d~delete d~exists
signal on syntax name bad
say d~delete(1)
bad: say condition('o')~code
.environment~log = .array~new
junk = .logfile~new('spec-file-delete.tmp')~delete
say .environment~log~toString(,' ')
::class logfile subclass file
::method isDirectory
  .environment~log~append('isDirectory')
  forward class (super)
```

```output
0
0 1
0 1
1 0 0
1 0
93.902
isDirectory
```

r[class.file.makedir]
`makeDir` makes the directory that the last component of the absolute path
names, and answers 1. It makes no directory above it: it answers 0 when a
parent is missing, when something is at the path already, a file or a
directory, and when it cannot make it. `makeDirs` answers 0 without making
anything when something is at the path. Otherwise it sends the receiver
`makeDir`, and answers 1 when that answers 1. When it does not, it sends the
receiver `parentFile` and `makeDirs` to what that answers, and then sends
`makeDir` to the receiver again and answers what that answers; so it makes
every directory that is missing from the top down, and answers 0 when one of
them cannot be made. Neither takes an argument; one raises `SYNTAX 93.902`.

```rexx
a = .file~new('spec-file-mk.tmp')
b = .file~new('spec-file-mk.tmp/b/c')
say b~makeDir b~exists
say a~makeDir a~makeDir a~makeDirs
say b~makeDirs b~isDirectory b~makeDirs b~parentFile~makeDir
f = .file~new('spec-file-mk.tmp/file')
call lineout f~path, 'x'
call lineout f~path
say f~makeDir f~makeDirs .file~new('spec-file-mk.tmp/file/sub')~makeDirs
say f~isFile
signal on syntax name b1
say a~makeDir(1)
b1: say condition('o')~code
signal on syntax name b2
say a~makeDirs(1)
b2: say condition('o')~code
.environment~log = .array~new
x = .logfile~new('spec-file-mk.tmp/p/q')~makeDirs
say .environment~log~section(1, 4)~toString(,' ')
say .environment~log~section(5)~toString(,' ')
junk = f~delete
junk = .file~new('spec-file-mk.tmp/p/q')~delete
junk = .file~new('spec-file-mk.tmp/p')~delete
junk = b~delete
junk = b~parentFile~delete
junk = a~delete
::class logfile subclass file
::method makeDir
  .environment~log~append(self~name 'makeDir')
  forward class (super)
::method makeDirs
  .environment~log~append(self~name 'makeDirs')
  forward class (super)
::method exists
  .environment~log~append(self~name 'exists')
  forward class (super)
::method parentFile
  .environment~log~append(self~name 'parentFile')
  forward class (super)
```

```output
0 0
1 0 0
1 1 0 0
0 0 0
1
93.902
93.902
q makeDirs q exists q makeDir q parentFile
p makeDirs p exists p makeDir q makeDir
```

r[class.file.delete.isdirectory]
When the answer of `isDirectory` that `delete` is sent is false, `delete`
removes only a file, and a directory at the path is left, answering 0. When it
is true, `delete` removes an empty directory or a file.

```rexx
d = 'spec-file-del-dir.tmp'
junk = .file~new(d'/one')~makeDirs
say .no~new(d'/one')~delete .file~new(d'/one')~exists
call lineout d'/f', 'x'
call lineout d'/f'
say .yes~new(d'/f')~delete .file~new(d'/f')~exists
junk = .file~new(d'/f')~delete
junk = .file~new(d'/one')~delete
junk = .file~new(d)~delete
::class no subclass file
::method isDirectory
  return 0
::class yes subclass file
::method isDirectory
  return 1
```

```output defect=#179
0 1
1 0
```

r[class.file.renameto]
`renameTo` takes one argument, the `File` that is to take the place of the
receiver, and moves the file or directory at the absolute path of the receiver
to the absolute path of the argument. It answers 1 when that was done, and 0
when it was not: when nothing is at the receiver's path, when the argument's
parent directory is missing, or when a file is at the argument's path already,
the receiver's own path included, or a directory as [the next
rule](r:class.file.renameto.directory) says. The receiver keeps its own path, so
its `exists` answers 0 afterwards and the argument's answers 1; a directory
takes its contents along. The argument is sent `isA` with `.File`; one that
answers false raises `SYNTAX 88.914`, as does a string or `.nil`, and no
argument raises `SYNTAX 93.901`, a second one `93.902`. What happens when a
directory is at the argument's path is the subject of [the next
rule](r:class.file.renameto.directory).

```rexx
d = 'spec-file-ren.tmp'
junk = .file~new(d'/in')~makeDirs
call lineout d'/a.txt', 'one'
call lineout d'/a.txt'
call lineout d'/b.txt', 'two!'
call lineout d'/b.txt'
a = .file~new(d'/a.txt')
b = .file~new(d'/b.txt')
c = .file~new(d'/c.txt')
say a~renameTo(b) a~exists b~length
say a~renameTo(c) a~exists c~exists a~path
say c~renameTo(c) c~renameTo(.file~new(d'/nodir/c.txt')) c~exists
m = .file~new(d'/in/moved.txt')
say c~renameTo(m) c~exists m~length
call lineout d'/in/x', 'x'
call lineout d'/in/x'
say .file~new(d'/in')~renameTo(.file~new(d'/out'))
say .file~new(d'/in')~exists .file~new(d'/out/x')~exists
call try 'a~renameTo("text")'
call try 'a~renameTo(.nil)'
call try 'a~renameTo'
call try 'a~renameTo(b, b)'
.environment~log = .array~new
call try 'a~renameTo(.spy~new)'
say .environment~log~toString(,' ')
junk = .file~new(d'/out/x')~delete
junk = .file~new(d'/out/moved.txt')~delete
junk = .file~new(d'/out')~delete
junk = b~delete
junk = .file~new(d)~delete
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class spy
::method isA
  .environment~log~append('isA' arg(1)~id)
  return 0
```

```output
0 1 5
1 0 1 spec-file-ren.tmp/a.txt
0 0 1
1 0 4
1
0 1
a~renameTo("text") => 88.914
a~renameTo(.nil) => 88.914
a~renameTo => 93.901
a~renameTo(b, b) => 93.902
a~renameTo(.spy~new) => 88.914
isA File
```

r[class.file.renameto.directory]
A directory takes the place of an empty directory that is at the argument's
path: `renameTo` replaces it and answers 1. It answers 0, and moves nothing,
when the directory at the argument's path is not empty, when a file is to take
the place of a directory or a directory the place of a file, and when the
argument's path lies inside the receiver's directory.

```rexx
d = 'spec-file-ren-dir.tmp'
junk = .file~new(d'/from/inner')~makeDirs
junk = .file~new(d'/empty')~makeDirs
junk = .file~new(d'/full/x')~makeDirs
call lineout d'/f', 'x'
call lineout d'/f'
say .file~new(d'/from')~renameTo(.file~new(d'/empty')),
    .file~new(d'/from')~exists .file~new(d'/empty/inner')~exists
say .file~new(d'/empty')~renameTo(.file~new(d'/full')),
    .file~new(d'/f')~renameTo(.file~new(d'/full/x')),
    .file~new(d'/full')~renameTo(.file~new(d'/f')),
    .file~new(d'/full')~renameTo(.file~new(d'/full/x/in'))
junk = .file~new(d'/f')~delete
junk = .file~new(d'/full/x')~delete
junk = .file~new(d'/full')~delete
junk = .file~new(d'/empty/inner')~delete
junk = .file~new(d'/empty')~delete
junk = .file~new(d'/from/inner')~delete
junk = .file~new(d'/from')~delete
junk = .file~new(d)~delete
```

```output defect=#177
1 0 1
0 0 0 0
```

r[class.file.permissions]
`setReadOnly` clears the write permission of the file or directory at the
absolute path for its owner, its group and everybody else, and `setWritable`
sets it for all three; neither changes the other permissions, and neither
does anything where nothing is. Neither answers anything, so a program that
uses what one answers raises `SYNTAX 91.999`, and neither takes an argument;
one raises `SYNTAX 93.902`. For a user other than the superuser, `canWrite`
answers 0 after `setReadOnly` and 1 after `setWritable`.

```rexx
f = .file~new('spec-file-perm.tmp')
call lineout f~path, 'x'
call lineout f~path
address system 'chmod 600' f~path
call mode 'start'
f~setReadOnly
call mode 'read only'
say f~canRead f~canWrite
f~setWritable
call mode 'writable'
say f~canRead f~canWrite
address system 'chmod 644' f~path
f~setReadOnly
call mode 'from 644'
f~setWritable
call mode 'again'
.file~new('spec-file-perm-none.tmp')~setReadOnly
.file~new('spec-file-perm-none.tmp')~setWritable
say .file~new('spec-file-perm-none.tmp')~exists
signal on syntax name b1
x = f~setReadOnly
b1: say condition('o')~code
signal on syntax name b2
f~setWritable(1)
b2: say condition('o')~code
f~setWritable
junk = f~delete
exit
mode:
  address system 'ls -ld' f~path with output stem o.
  say arg(1)':' o.1~left(10)
  return
```

```output
start: -rw-------
read only: -r--------
1 0
writable: -rw--w--w-
1 1
from 644: -r--r--r--
again: -rw-rw-rw-
0
91.999
93.902
```

## Times

r[class.file.lastmodified]
`lastModified` answers when the file or directory at the absolute path was
last changed, as a [`DateTime`](r:class.datetime.creation) in the time zone of
the host, with the host's offset, which is not specified; it answers `.nil`
where nothing is. It takes no argument; one raises `SYNTAX 93.902`. A file
system may keep a time with less precision than a microsecond, so the examples
use whole seconds. The answer after [the set
rule](r:class.file.lastmodified-set) has the date and the time of day that
were set.

```rexx
f = .file~new('spec-file-time.tmp')
say (f~lastModified == .nil) (f~lastAccessed == .nil)
call lineout f~path, 'x'
call lineout f~path
f~lastModified = .datetime~new(2003, 1, 15, 12, 5, 6, 0, 0)
m = f~lastModified
say m~class~id m~isoDate
signal on syntax name bad
say f~lastModified(1)
bad: say condition('o')~code
junk = f~delete
```

```output
1 1
DateTime 2003-01-15T12:05:06.000000
93.902
```

r[class.file.lastmodified-set]
`lastModified=` takes a `DateTime` and sets the time at which the file or
directory at the absolute path was last changed to the date and the time of
day of its fields, as read in the host's time zone: the offset of the argument
plays no part. It does nothing where nothing is, and makes no file. The
argument is sent `isA` with `.DateTime`; one that answers false raises
`SYNTAX 88.914`, and so does a string, a number, `.nil` or a `TimeSpan`. Sent
as a message, `lastModified=` takes exactly one argument: none raises
`SYNTAX 93.901` and a second one `93.902`.

```rexx
f = .file~new('spec-file-time.tmp')
call lineout f~path, 'x'
call lineout f~path
d = .datetime~new(2003, 1, 15, 12, 5, 6, 0, 0)
e = .datetime~new(2004, 2, 16, 7, 8, 9, 0, 120)
f~lastModified = d
say f~lastModified~isoDate
f~lastModified = e
say f~lastModified~isoDate
n = .file~new('spec-file-time-none.tmp')
n~lastModified = d
say n~exists
call try 'f~"LASTMODIFIED="("text")'
call try 'f~"LASTMODIFIED="(.nil)'
call try 'f~"LASTMODIFIED="(5)'
call try 'f~"LASTMODIFIED="(.timespan~new(1))'
call try 'f~"LASTMODIFIED="'
.environment~log = .array~new
call try 'f~"LASTMODIFIED="(.spy~new)'
say .environment~log~toString(,' ')
call try 'f~"LASTMODIFIED="(d, d)'
say f~lastModified~isoDate
junk = f~delete
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class spy
::method isA
  .environment~log~append('isA' arg(1)~id)
  return 0
```

```output
2003-01-15T12:05:06.000000
2004-02-16T07:08:09.000000
0
f~"LASTMODIFIED="("text") => 88.914
f~"LASTMODIFIED="(.nil) => 88.914
f~"LASTMODIFIED="(5) => 88.914
f~"LASTMODIFIED="(.timespan~new(1)) => 88.914
f~"LASTMODIFIED=" => 93.901
f~"LASTMODIFIED="(.spy~new) => 88.914
isA DateTime
f~"LASTMODIFIED="(d, d) => 93.902
2004-02-16T07:08:09.000000
```

r[class.file.lastaccessed]
`lastAccessed` answers when the file or directory at the absolute path was
last used, as a [`DateTime`](r:class.datetime.creation) as [`lastModified`
does](r:class.file.lastmodified), and `.nil` where nothing is. Not every file
system keeps the time of last use, and some do not update it when a file is
read; an example can rely on the answer only after the time was set. It takes
no argument; one raises `SYNTAX 93.902`.

```rexx
f = .file~new('spec-file-access.tmp')
say (f~lastAccessed == .nil)
call lineout f~path, 'x'
call lineout f~path
f~lastAccessed = .datetime~new(2002, 3, 4, 5, 6, 7, 0, 0)
a = f~lastAccessed
say a~class~id a~isoDate
signal on syntax name bad
say f~lastAccessed(1)
bad: say condition('o')~code
junk = f~delete
```

```output
1
DateTime 2002-03-04T05:06:07.000000
93.902
```

r[class.file.lastaccessed-set]
`lastAccessed=` takes a `DateTime` and sets the time of last use of the file
or directory at the absolute path as [`lastModified=`
does](r:class.file.lastmodified-set), with the same checks and errors, and
does nothing where nothing is.

```rexx
f = .file~new('spec-file-access.tmp')
call lineout f~path, 'x'
call lineout f~path
f~lastAccessed = .datetime~new(2002, 3, 4, 5, 6, 7, 0, 120)
say f~lastAccessed~isoDate
n = .file~new('spec-file-access-none.tmp')
n~lastAccessed = .datetime~new(2002, 3, 4, 5, 6, 7, 0, 0)
say n~exists
signal on syntax name b1
f~"LASTACCESSED="("text")
b1: say condition('o')~code
signal on syntax name b2
f~"LASTACCESSED="
b2: say condition('o')~code
signal on syntax name b3
f~"LASTACCESSED="(1, 2)
b3: say condition('o')~code
junk = f~delete
```

```output
2002-03-04T05:06:07.000000
0
88.914
93.901
93.902
```

## The contents of a directory

r[class.file.list]
`list` answers an `Array` of the names of the entries of the directory at the
absolute path, as strings, without the entries `.` and `..`; hidden entries are
included. The names are in no order that is specified. An empty directory
answers an empty array, and a path that is no directory, a file or nothing,
answers `.nil`. `listFiles` answers an `Array` of `File` objects for the same
entries, in the same order, and `.nil` where `list` does. It sends the receiver
`list`, and makes each `File` by sending `new` to the class of the receiver,
with the name of the entry as `path` and the receiver itself as `dir`, which
`init` sends `isA` and `absolutePath` ([the sends
rule](r:class.file.new.sends)), so the path of each is the absolute path of the
directory, a slash and the name. Neither takes an argument; one raises
`SYNTAX 93.902`.

```rexx
d = .file~new('spec-file-list.tmp')
junk = .file~new('spec-file-list.tmp/sub')~makeDirs
do n over '.hid b.txt a.txt'~makearray(' ')
  call lineout d~path'/'n, 'x'
  call lineout d~path'/'n
end
l = d~list
say l~class~id l~items l~sort~toString(,' ')
f = d~listFiles
say f~class~id f~items f[1]~class~id
p = .array~new
do e over f
  p~append(e~path)
end
q = d~absolutePath
say (p~sort~toString(,' ') == q'/.hid' q'/a.txt' q'/b.txt' q'/sub')
l2 = d~list
f2 = d~listFiles
same = 1
do i = 1 to l2~items
  if f2[i]~name \== l2[i] then same = 0
end
say same
say (.file~new('spec-file-list.tmp/a.txt')~list == .nil),
    (.file~new('spec-file-list.tmp/none')~list == .nil)
say (.file~new('spec-file-list.tmp/none')~listFiles == .nil)
e = .file~new('spec-file-list.tmp/sub')~list
say e~class~id e~items
say .file~new('spec-file-list.tmp/sub')~listFiles~items
signal on syntax name b1
say d~list(1)
b1: say condition('o')~code
signal on syntax name b2
say d~listFiles(1)
b2: say condition('o')~code
.environment~log = .array~new
x = .logfile~new('spec-file-list.tmp')~listFiles
say .environment~log~toString(,' ') x~items x[1]~class~id
do n over '.hid b.txt a.txt'~makearray(' ')
  junk = .file~new(d~path'/'n)~delete
end
junk = .file~new('spec-file-list.tmp/sub')~delete
junk = d~delete
::class logfile subclass file
::method list
  .environment~log~append('list')
  forward class (super)
```

```output
Array 4 .hid a.txt b.txt sub
Array 4 File
1
1
1 1
1
Array 0
0
93.902
93.902
list 4 LOGFILE
```

## Comparing files

r[class.file.compareto]
`compareTo`, which takes the place of [the `compareTo` of
`Comparable`](r:class.comparable.compareto), takes one argument and answers -1,
0 or 1 as the absolute path of the receiver is less than, equal to or greater
than the absolute path of the argument. The paths are compared character by
character, so that a path is less than a longer path that it is the start of.
The argument is sent `isA` with `.File` first, then the receiver is sent
`isCaseSensitive`, and then `absolutePath` is sent to the receiver and to the
argument, in that order. When `isCaseSensitive` answers true the comparison
tells upper and lower case letters apart. When it answers false two paths whose
letters differ only in case are equal, and the order of the others is that of
the paths with their letters from `a` to `z` written as capitals. No argument
raises `SYNTAX 93.901`, and a second one `93.902`. The comparison operators of a
`File` answer from `compareTo` as [those of
`Orderable`](r:class.orderable.compare) say.

```rexx
a = .file~new('/t/a'); b = .file~new('/t/b')
say a~compareTo(b) b~compareTo(a) a~compareTo(.file~new('/t//a/')),
    a~compareTo(a)
say .file~new('/t/ab')~compareTo(.file~new('/t/a/b')),
    .file~new('/t')~compareTo(a)
say .cs~new('/t/a')~compareTo(.file~new('/t/A')),
    .cs~new('/t/A')~compareTo(.cs~new('/t/a'))
say .ci~new('/t/a')~compareTo(.file~new('/t/A')),
    .ci~new('/t/A')~compareTo(.file~new('/t/a'))
say .ci~new('/t/B')~compareTo(a) .ci~new('/t/_')~compareTo(a),
    .ci~new('/t/a')~compareTo(.cs~new('/t/_'))
say .cs~new('/t/B')~compareTo(a) .cs~new('/t/_')~compareTo(a)
signal on syntax name b1
say a~compareTo
b1: say condition('o')~code
signal on syntax name b2
say a~compareTo(b, b)
b2: say condition('o')~code
.environment~log = .array~new
x = .spy~new('/s')~compareTo(.spy~new('/z'))
say .environment~log~toString(,' ')
::class cs subclass file
::method isCaseSensitive
  return 1
::class ci subclass file
::method isCaseSensitive
  return 0
::class spy subclass file
::method isCaseSensitive
  .environment~log~append(self~path 'isCaseSensitive')
  forward class (super)
::method absolutePath
  .environment~log~append(self~path 'absolutePath')
  forward class (super)
::method isA
  .environment~log~append(self~path 'isA' arg(1)~id)
  forward class (super)
```

```output
-1 1 0 0
1 -1
1 -1
0 0
1 1 -1
-1 -1
93.901
93.902
/z isA File /s isCaseSensitive /s absolutePath /z absolutePath
```

r[class.file.hashcode]
`hashCode` answers the same string that [`hashCode`](r:class.string.hashcode) of
the absolute path answers, so that two files with the same absolute path have
the same hash code, whatever the spelling of their paths. The hash code does not
depend on `isCaseSensitive`, so two files that `=` finds equal when the file
system ignores case can have different hash codes. It sends the receiver none of
`absolutePath`, `makeString`, `path` and `string`, and it takes no argument; one
raises `SYNTAX 93.902`. A `File` is found in a collection that looks for its
index by `==` and `hashCode`.

```rexx
a = .file~new('/t/a')
b = .file~new('/t//a/')
c = .file~new('/t/A')
say (a~hashCode == b~hashCode)
say (a~hashCode == a~absolutePath~hashCode)
x = .ci~new('/t/a')
y = .ci~new('/t/A')
say (x = y) (x~hashCode == y~hashCode)
s = .set~new
s~put(a); s~put(b); s~put(c)
say s~items
t = .table~new
t[a] = 'found'
say t[b] t[c]
signal on syntax name bad
say a~hashCode(1)
bad: say condition('o')~code
.environment~log = .array~new
x = .logfile~new('/t/a')~hashCode
say '['.environment~log~toString(,' ')']'
::class ci subclass file
::method isCaseSensitive
  return 0
::class logfile subclass file
::method absolutePath
  .environment~log~append('absolutePath')
  forward class (super)
::method makeString
  .environment~log~append('makeString')
  forward class (super)
::method path
  .environment~log~append('path')
  forward class (super)
::method string
  .environment~log~append('string')
  forward class (super)
```

```output
1
1
1 0
2
found The NIL object
93.902
[]
```

r[class.file.operators]
The eighteen comparison methods `=`, `==`, `<`, `<=`, `<<`, `<<=`, `<>`, `>`,
`>=`, `><`, `>>`, `>>=`, `\=`, `\==`, `\<`, `\<<`, `\>` and `\>>` are those of
[`Orderable`](r:class.orderable.compare): each sends `compareTo` to the receiver
and answers from its answer, so for a `File` they compare absolute paths as [the
compare rule](r:class.file.compareto) says, and a strict operator is not strict.

```rexx
a = .file~new('/t/a')
b = .file~new('/t/b')
say (a = .file~new('/t/./a')) (a == .file~new('/t/./a')) (a \= b) (a \== b)
say (a < b) (a <= b) (a << b) (a <<= b) (a <> b) (a >< b)
say (a > b) (a >= b) (a >> b) (a >>= b) (a \< b) (a \> b)
say (a \<< b) (a \>> b)
```

```output
1 1 1 1
1 1 1 1 1 1
0 0 0 0 0 1
0 1
```

## Places and searching

r[class.file.iscasesensitive.class-side]
The class method `isCaseSensitive` answers what the instance method answers
for the root of the file system, `/` ([the instance
rule](r:class.file.iscasesensitive)). It takes no argument; an argument,
whatever it is, raises `SYNTAX 88.922`.

```rexx
say (.file~isCaseSensitive == .file~new('/')~isCaseSensitive)
call try '.file~isCaseSensitive(1)'
call try '.file~isCaseSensitive(.nil)'
call try '.file~isCaseSensitive(1, 2)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1
.file~isCaseSensitive(1) => 88.922
.file~isCaseSensitive(.nil) => 88.922
.file~isCaseSensitive(1, 2) => 88.922
```

r[class.file.listroots]
`listRoots` answers an `Array` of strings, one for each root of the file
system. On a Unix-like system that is one string, `/`. It takes no argument;
an argument, whatever it is, raises `SYNTAX 88.922`.

```rexx
r = .file~listRoots
say r~class~id r~items r[1] r~dimension
call try '.file~listRoots(1)'
call try '.file~listRoots(.nil)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
Array 1 / 1
.file~listRoots(1) => 88.922
.file~listRoots(.nil) => 88.922
```

r[class.file.temporarypath]
`temporaryPath` answers a new `File`, made by `new` sent to the receiver, for
the directory in which temporary files are to be put: the value of the
environment variable `TMPDIR` when that is set, and `/tmp` when it is not. It
takes no argument; one raises `SYNTAX 93.902`. The example sets `TMPDIR` between
[SETLOCAL](../builtins/setlocal.md) and ENDLOCAL, which put it back.

```rexx
call setlocal
call value 'TMPDIR', 'spec-file-tmpdir.tmp', 'ENVIRONMENT'
t = .file~temporaryPath
say t~class~id t~path,
    (t~absolutePath == directory() || '/spec-file-tmpdir.tmp')
call value 'TMPDIR', '/spec-file/tmp//', 'ENVIRONMENT'
say .file~temporaryPath~path
say .sub~temporaryPath~class~id
call value 'TMPDIR', .nil, 'ENVIRONMENT'
say .file~temporaryPath~path
call endlocal
signal on syntax name bad
say .file~temporaryPath(1)
bad: say condition('o')~code
::class sub subclass file
```

```output
File spec-file-tmpdir.tmp 1
/spec-file/tmp
SUB
/tmp
93.902
```

r[class.file.searchpath]
`searchPath` takes a file `name` and an optional `path`, and answers a new
`File`, or `.nil` when nothing is found. The `path` lists directories. A string
`path` is cut at every `:`, the class is sent `pathSeparator`, whose answer
plays no part, and the empty pieces are dropped. Any other `path` is sent
`request` with the argument `ARRAY`, and the items of the array it answers are
the directories, each read as [a string argument](r:class.index.string), so a
`.nil` item raises `SYNTAX 88.909`; an answer that is not an array raises
`SYNTAX 98.913`. An omitted `path` stands for the directories of the environment
variable `PATH`. The directories are tried in order, and the answer is the first
one that holds a file of that name, as `new` of `File` with `name` as `path` and
the directory as `dir` would make it: the class of the answer is `File`,
whatever the class that received `searchPath`. The `name` may name a file below
the directory, as `sub/c.txt`. A directory that holds a directory of that name
does not count, and a `name` that has a wildcard character `*` or `?` finds
nothing, nor does an empty `name`. No argument raises `SYNTAX 93.901` and a
third one `93.902`. The `name` is read as [a string
argument](r:class.index.string), so an object whose `makeString` answers a name
is searched for by that name and `.nil` raises `SYNTAX 88.909`. A `name` that
begins with a slash is the absolute path of the file: the answer is a `File` for
it when the file exists, whatever the directories. A relative directory is
relative to the current directory, and the answer holds the absolute path of the
file it finds, as [the next rule](r:class.file.searchpath.relative) shows.

```rexx
here = directory()
d1 = here'/spec-file-search.tmp/d1'
d2 = here'/spec-file-search.tmp/d2'
junk = .file~new(d1'/sub')~makeDirs
junk = .file~new(d2)~makeDirs
list = 'd1/a.txt d1/sub/c.txt d2/a.txt d2/b.txt'~makearray(' ')
do n over list
  call lineout 'spec-file-search.tmp/'n, 'x'
  call lineout 'spec-file-search.tmp/'n
end
call show 'a.txt', d1':'d2
call show 'a.txt', d2':'d1
call show 'b.txt', d1':'d2
call show 'b.txt', d1
call show 'a.txt', d1'::'d2':'
call show 'a.txt', .array~of(d2, d1)
call show 'b.txt', .list~of(d1, d2)
call show 'sub/c.txt', d1':'d2
call show 'sub', d1':'d2
call show 'a.*', d1
call show 'a.t?t', d1
call show '', d1
call show 'a.txt', ''
call show 'a.txt', .array~new
call show 'a.txt', 5
call show 'a.txt', .nil
call show 'a.txt', .array~of(.nil)
call show 'a.txt', .object~new
call setlocal
call value 'PATH', d2':'d1, 'ENVIRONMENT'
r = .file~searchPath('a.txt')
say r~class~id (r~path == d2'/a.txt')
r = .sub~searchPath('b.txt', )
say r~class~id (r~path == d2'/b.txt')
call endlocal
call try '.file~searchPath'
call try '.file~searchPath("a", "b", "c")'
do n over list
  junk = .file~new('spec-file-search.tmp/'n)~delete
end
junk = .file~new(d1'/sub')~delete
junk = .file~new(d1)~delete
junk = .file~new(d2)~delete
junk = .file~new('spec-file-search.tmp')~delete
exit
show:
  signal on syntax name bad
  r = .file~searchPath(arg(1), arg(2))
  if r == .nil then say '['arg(1)'] nil'
  else say '['arg(1)']' r~path~changestr(here, '.')
  return
bad: say '['arg(1)']' condition('o')~code
  return
try:
  signal on syntax name bad2
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad2: say arg(1) '=>' condition('o')~code
  return
::class sub subclass file
```

```output
[a.txt] ./spec-file-search.tmp/d1/a.txt
[a.txt] ./spec-file-search.tmp/d2/a.txt
[b.txt] ./spec-file-search.tmp/d2/b.txt
[b.txt] nil
[a.txt] ./spec-file-search.tmp/d1/a.txt
[a.txt] ./spec-file-search.tmp/d2/a.txt
[b.txt] ./spec-file-search.tmp/d2/b.txt
[sub/c.txt] ./spec-file-search.tmp/d1/sub/c.txt
[sub] nil
[a.*] nil
[a.t?t] nil
[] nil
[a.txt] nil
[a.txt] nil
[a.txt] nil
[a.txt] 98.913
[a.txt] 88.909
[a.txt] 98.913
File 1
File 1
.file~searchPath => 93.901
.file~searchPath("a", "b", "c") => 93.902
```

r[class.file.searchpath.relative]
A directory of `path` that is relative is the current directory, a slash and
the directory, and the path of the answer is then absolute. An absolute
`name` is found whatever the directories, and a `name` that is an object is
read by its `makeString`.

```rexx
here = directory()
d = 'spec-file-search-rel.tmp'
junk = .file~new(d'/sub')~makeDirs
call lineout d'/a.txt', 'x'
call lineout d'/a.txt'
call show .file~searchPath('a.txt', d)
call show .file~searchPath(here'/'d'/a.txt', here'/'d'/sub')
call show .file~searchPath(.named~new, here'/'d)
signal on syntax name bad
r = .file~searchPath(.nil, here'/'d)
say 'no error'
signal cleanup
bad: say condition('o')~code
cleanup:
junk = .file~new(d'/a.txt')~delete
junk = .file~new(d'/sub')~delete
junk = .file~new(d)~delete
exit
show:
  if arg(1) == .nil then say 'nil'
  else say (arg(1)~path == here'/'d'/a.txt')
  return
::class named
::method makeString
  return 'a.txt'
```

```output defect=#178
1
1
1
88.909
```

r[class.file.readchars]
`readChars` takes the name of a file, and answers the whole content of the
file as a string, bytes unchanged, line ends included. The name is a `File`
or a string: an argument that is sent `isA` with `.File` and answers true is
read by its `absolutePath`, and any other is read as a [string
argument](r:class.index.string). An argument that has no string value, as
`.nil` has none, raises `SYNTAX 93.938`. A file that cannot be read, as one
that does not exist or a directory, answers the empty string. No argument
raises `SYNTAX 93.901` and a second one `93.902`.

```rexx
f = 'spec-file-readchars.tmp'
.file~writeChars(f, 'ab' || '0d0a'x || 'cd' || '0a1a'x || 'ef', 'Replace')
x = .file~readChars(f)
say x~class~id x~length c2x(x)
say (.file~readChars(.file~new(f)) == x),
    (.file~readChars(.file~new(f)~absolutePath) == x)
say '['.file~readChars('spec-file-readchars-none.tmp')']'
say '['.file~readChars('.')~length']' .file~readChars('')~length
call try '.file~readChars'
call try '.file~readChars(f, 1)'
call try '.file~readChars(.nil)'
call try '.file~readChars(.object~new)'
.environment~log = .array~new
say (.file~readChars(.logfile~new(f)) == x)
say .environment~log~toString(,' ')
junk = .file~new(f)~delete
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class logfile subclass file
::method isA
  .environment~log~append('isA' arg(1)~id)
  forward class (super)
::method absolutePath
  .environment~log~append('absolutePath')
  forward class (super)
```

```output
String 10 61620D0A63640A1A6566
1 1
[]
[0] 0
.file~readChars => 93.901
.file~readChars(f, 1) => 93.902
.file~readChars(.nil) => 93.938
.file~readChars(.object~new) => 93.938
1
isA File absolutePath
```

r[class.file.readlines]
`readLines` takes the name of a file as [`readChars`](r:class.file.readchars)
does, with the same errors, and answers an `Array` of the lines of the file,
each a string without its line end. A line ends at a line feed, and one
carriage return just before the line feed is dropped with it; a carriage
return anywhere else is part of the line. The last line need not end in a line
feed, and no empty line is added after one that does. A file that is empty,
cannot be read or is a directory answers an empty array.

```rexx
f = 'spec-file-readlines.tmp'
text = 'a' || '0d0a'x || 'b' || '0d'x || 'c' || '0a0a'x,
    || 'd' || '0d0d0a'x || 'e'
.file~writeChars(f, text, 'Replace')
x = .file~readLines(f)
say x~class~id x~items x~dimension
do i = 1 to x~items
  say i c2x(x[i])
end
.file~writeChars(f, 'z' || '0a'x, 'Replace')
y = .file~readLines(f)
say y~items c2x(y[y~items])
say .file~readLines('spec-file-readlines-none.tmp')~items,
    .file~readLines('.')~items
.file~writeChars(f, '', 'Replace')
say .file~readLines(f)~items
.file~writeChars(f, '0a'x, 'Replace')
z = .file~readLines(f)
say z~items '['z[1]']'
junk = .file~new(f)~delete
```

```output
Array 5 1
1 61
2 620D63
3 
4 640D
5 65
1 7A
0 0
0
1 []
```

r[class.file.writechars]
`writeChars` takes the name of a file, a string and an optional `mode`, and
writes the string to the file, bytes unchanged, creating the file when it does
not exist. It answers nothing, so a program that uses what it answers raises
`SYNTAX 91.999`. The `mode` is sent `strip`, and the first character of the
string that answers, in upper or lower case, decides: `A` appends the string
to the content of the file and `R` replaces the content, whatever follows the
letter, so `Replace`, `r` and `repl!` replace. An omitted `mode` appends. Any
other `mode`, the empty string among them, raises `SYNTAX 88.916`, and one
that does not understand `strip`, `.nil` for one, raises `SYNTAX 97.1`. The
name is read as [`readChars`](r:class.file.readchars) reads it, with the same
error, after the `mode` has been checked. The string is read as the string
value of an argument ([the string rule](r:class.index.string)), so `.nil` is
written as `The NIL object` and no object is refused. A file that cannot be
opened is not written, and nothing is reported. No argument raises
`SYNTAX 93.901`, a call with only the name `93.901` as well, and a fourth
argument `93.902`.

```rexx
f = 'spec-file-writechars.tmp'
.file~writeChars(f, 'abc')
.file~writeChars(f, 'def')
say .file~readChars(f)
.file~writeChars(f, 'X', 'Replace')
say .file~readChars(f)
.file~writeChars(f, 'Y', )
.file~writeChars(f, 'Z', 'a')
say .file~readChars(f)
.file~writeChars(f, 'W', 'r')
.file~writeChars(f, 'V', ' repl!')
say .file~readChars(f)
.file~writeChars(.file~new(f), '', 'Replace')
say .file~new(f)~length
.file~writeChars(f, .nil)
.file~writeChars(f, 12, 'append')
say .file~readChars(f)
.file~writeChars('spec-file-writechars-none.tmp/x', 'q')
say .file~new('spec-file-writechars-none.tmp')~exists
call try '.file~writeChars(f, "x", "")'
call try '.file~writeChars(f, "x", "Q")'
call try '.file~writeChars(f, "x", .nil)'
call try '.file~writeChars(.nil, "x", "Q")'
call try '.file~writeChars(.nil, "x")'
call try '.file~writeChars'
call try '.file~writeChars(f)'
call try '.file~writeChars(f, "x", "A", 1)'
call try 'f2 = .file~writeChars(f, "x")'
junk = .file~new(f)~delete
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
abcdef
X
XYZ
V
0
The NIL object12
0
.file~writeChars(f, "x", "") => 88.916
.file~writeChars(f, "x", "Q") => 88.916
.file~writeChars(f, "x", .nil) => 97.1
.file~writeChars(.nil, "x", "Q") => 88.916
.file~writeChars(.nil, "x") => 93.938
.file~writeChars => 93.901
.file~writeChars(f) => 93.901
.file~writeChars(f, "x", "A", 1) => 93.902
f2 = .file~writeChars(f, "x") => 91.999
```

r[class.file.writelines]
`writeLines` takes the name of a file, an ordered collection and an optional
`mode`, and writes each item of the collection to the file followed by a line
feed, creating the file when it does not exist. It answers nothing, as
[`writeChars`](r:class.file.writechars) does, reads the `mode` as it does,
with the same error, and appends when `mode` is omitted. The collection is
sent `isA` with `.OrderedCollection`; one that answers false raises
`SYNTAX 88.914`, as do a string, `.nil`, a `Set`, a `Bag`, a `Directory`, a
`Table`, a `Stem` and a `MutableBuffer`. The checks are made in this order:
the collection, the `mode`, the name, with the error of
[`readChars`](r:class.file.readchars) for the name. The collection is then
sent `request` with the argument `ARRAY`, a `List` subclass among them
([Deviation 118](r:class.file.deviation.118)), and the items of the array that
answers are written in order, each as its string value ([the string
rule](r:class.index.string)); a position with no item is left out, and an
answer that is not an array raises `SYNTAX 98.913`. An `Array`, a `List` and a
`Queue` are ordered collections. With `Replace` an empty collection empties
the file, and with `Append` it leaves it as it was. No argument raises
`SYNTAX 93.901`, one with only the name `93.901` too, and a fourth argument
`93.902`.

```rexx
f = 'spec-file-writelines.tmp'
.file~writeLines(f, .array~of('one', 'two'))
.file~writeLines(f, .list~of('three'), 'Append')
.file~writeLines(f, .queue~of('four'), 'a')
say c2x(.file~readChars(f))
.file~writeLines(f, .array~of('a' || '0a'x || 'b', 1, 2.50), 'Replace')
say c2x(.file~readChars(f))
.file~writeLines(f, .array~of('x', , 'z'), 'r')
say c2x(.file~readChars(f))
.file~writeLines(f, .array~new, 'Append')
say c2x(.file~readChars(f))
.file~writeLines(f, .array~new, 'Replace')
say .file~new(f)~length
.file~writeLines(f, .array~of('x', .nil, .object~new), 'R')
say .file~readChars(f)
.file~writeLines(.file~new(f), .array~of('k'), 'R')
say c2x(.file~readChars(f))
.file~writeLines(f, .mk~new, 'R')
say c2x(.file~readChars(f))
call try '.file~writeLines(f, "text")'
call try '.file~writeLines(f, .nil)'
call try '.file~writeLines(f, .set~of("s"))'
call try '.file~writeLines(f, .bag~of("s"))'
call try '.file~writeLines(f, .table~new)'
call try '.file~writeLines(f, .stem~new)'
call try '.file~writeLines(f, .mutablebuffer~new("abc"))'
call try '.file~writeLines(.nil, "text", "Q")'
call try '.file~writeLines(.nil, .array~of("a"), "Q")'
call try '.file~writeLines(.nil, .array~of("a"))'
call try '.file~writeLines(f, .array~of("a"), "")'
call try '.file~writeLines(f, .array~of("a"), .nil)'
call try '.file~writeLines(f, .bad~new)'
call try '.file~writeLines'
call try '.file~writeLines(f)'
call try '.file~writeLines(f, .array~of("a"), "A", 1)'
.environment~log = .array~new
call try '.file~writeLines(f, .spy~new)'
say .environment~log~toString(,' ')
junk = .file~new(f)~delete
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class mk subclass array
::method makeArray
  return .array~of('MK1', 'MK2')
::class bad inherit orderedcollection
::method makeArray
  return 'not an array'
::class spy
::method isA
  .environment~log~append('isA' arg(1)~id)
  return 0
```

```output
6F6E650A74776F0A74687265650A666F75720A
610A620A310A322E35300A
780A7A0A
780A7A0A
0
x
The NIL object
an Object

6B0A
4D4B310A4D4B320A
.file~writeLines(f, "text") => 88.914
.file~writeLines(f, .nil) => 88.914
.file~writeLines(f, .set~of("s")) => 88.914
.file~writeLines(f, .bag~of("s")) => 88.914
.file~writeLines(f, .table~new) => 88.914
.file~writeLines(f, .stem~new) => 88.914
.file~writeLines(f, .mutablebuffer~new("abc")) => 88.914
.file~writeLines(.nil, "text", "Q") => 88.914
.file~writeLines(.nil, .array~of("a"), "Q") => 88.916
.file~writeLines(.nil, .array~of("a")) => 93.938
.file~writeLines(f, .array~of("a"), "") => 88.916
.file~writeLines(f, .array~of("a"), .nil) => 97.1
.file~writeLines(f, .bad~new) => 98.913
.file~writeLines => 93.901
.file~writeLines(f) => 93.901
.file~writeLines(f, .array~of("a"), "A", 1) => 93.902
.file~writeLines(f, .spy~new) => 88.914
isA OrderedCollection
```

## Deviations

r[class.file.deviation.34]
In a path that holds `..`, interrexx removes the component before each one, as
[the absolute rule](r:class.file.absolute) says, however that component was
written. ooRexx removes no component after a doubled slash and two after one
that starts with a period, so the absolute path, the name and the parent it
answers can name another place than the path says ([Deviation
34](r:bif.qualify.deviation.34)).

```rexx
say .file~new('/a/.b/../c')~absolutePath .file~new('/a/.b/../c')~name
say .file~new('/a/.b/../c')~parent .file~new('/a//../b')~absolutePath
```

```output
/a/c c
/a /b
```

r[class.file.deviation.109]
An instance of a subclass of `String` that is given as a path, a directory, a
mode or any other string argument of `File` is read by its own characters (a
`mode` is first sent `strip`, as [the writeChars rule](r:class.file.writechars)
says, and its answer is read so), and `request`, `makeString` and `string` are
not sent to it ([Deviation 109](r:class.string.deviation.109)). ooRexx sends
such an instance `request` with `STRING`, so an override can change what is
read.

```rexx
.environment~sent = 0
p = .digits~new('p/q')
say .file~new(p)~path .file~new('a', p)~path
.file~writeChars('spec-file-dev109.tmp', .digits~new('abc'), 'Replace')
say .file~readChars(.digits~new('spec-file-dev109.tmp'))
junk = .file~new('spec-file-dev109.tmp')~delete
say 'sends' .environment~sent
::class digits subclass string
::method makestring
  .environment~sent = .environment~sent + 1
  return 'zzz'
::method request
  .environment~sent = .environment~sent + 1
  return 'zzz'
```

```output defect=#140
p/q p/q/a
abc
sends 0
```

r[class.file.deviation.118]
An instance of a subclass of `List` that is given to `writeLines` is sent
`request` with `ARRAY`, as [the writeLines rule](r:class.file.writelines) says
of any collection, so an override of `makeArray` or `request` decides what is
written ([Deviation 118](../deviations.md)). ooRexx sends such an instance
neither message and writes the items it holds, so the example prints `one two`
there.

```rexx
f = 'spec-file-dev118.tmp'
.file~writeLines(f, .mine~of('one', 'two'), 'Replace')
say .file~readLines(f)~toString(,' ')
junk = .file~new(f)~delete
::class mine subclass list
::method makeArray
  return .array~of('three')
```

```output
three
```
