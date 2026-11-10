---
element: "RexxInfo"
kind: class
rexxref: "Builtin Classes > Utility Classes > RexxInfo Class; Builtin Classes > Utility Classes > RexxInfo Class > architecture; Builtin Classes > Utility Classes > RexxInfo Class > caseSensitiveFiles; Builtin Classes > Utility Classes > RexxInfo Class > date; Builtin Classes > Utility Classes > RexxInfo Class > debug; Builtin Classes > Utility Classes > RexxInfo Class > digits; Builtin Classes > Utility Classes > RexxInfo Class > directorySeparator; Builtin Classes > Utility Classes > RexxInfo Class > endofline; Builtin Classes > Utility Classes > RexxInfo Class > executable; Builtin Classes > Utility Classes > RexxInfo Class > form; Builtin Classes > Utility Classes > RexxInfo Class > fuzz; Builtin Classes > Utility Classes > RexxInfo Class > internalDigits; Builtin Classes > Utility Classes > RexxInfo Class > internalMaxNumber; Builtin Classes > Utility Classes > RexxInfo Class > internalMinNumber; Builtin Classes > Utility Classes > RexxInfo Class > languageLevel; Builtin Classes > Utility Classes > RexxInfo Class > libraryPath; Builtin Classes > Utility Classes > RexxInfo Class > majorVersion; Builtin Classes > Utility Classes > RexxInfo Class > maxArraySize; Builtin Classes > Utility Classes > RexxInfo Class > maxExponent; Builtin Classes > Utility Classes > RexxInfo Class > maxPathLength; Builtin Classes > Utility Classes > RexxInfo Class > minExponent; Builtin Classes > Utility Classes > RexxInfo Class > modification; Builtin Classes > Utility Classes > RexxInfo Class > name; Builtin Classes > Utility Classes > RexxInfo Class > package; Builtin Classes > Utility Classes > RexxInfo Class > pathSeparator; Builtin Classes > Utility Classes > RexxInfo Class > platform; Builtin Classes > Utility Classes > RexxInfo Class > release; Builtin Classes > Utility Classes > RexxInfo Class > revision; Builtin Classes > Utility Classes > RexxInfo Class > version"
deviations: [63]
---

# RexxInfo

`.REXXINFO` is an object that describes the interpreter that runs the program
and the system it runs on: its version, its numeric limits and defaults, and
the separators and paths of the host. Its class is `RexxInfo`, which a program
reaches only as `.rexxinfo~class`, since the symbol `.REXXINFO` names the
object and not the class.

Many answers depend on the build or the host, and the book does not specify
their values. For each such method the rules give the form of the answer, and
where the answer must agree with another part of the language, that
relation. A program reads the values when it needs them; it must not assume
the values of another build.

## The object

r[class.rexxinfo.creation]
`.REXXINFO` answers one object, the same every time, whose class `RexxInfo`
has `Object` as its only superclass and no mixin class ([the hierarchy
rule](r:class.index.hierarchy)). The instance side of the class defines
`architecture`, `caseSensitiveFiles`, `copy`, `date`, `debug`, `digits`,
`directorySeparator`, `endofline`, `executable`, `form`, `fuzz`,
`internalDigits`, `internalMaxNumber`, `internalMinNumber`, `languageLevel`,
`libraryPath`, `majorVersion`, `maxArraySize`, `maxExponent`,
`maxPathLength`, `minExponent`, `modification`, `name`, `package`,
`pathSeparator`, `platform`, `release`, `revision` and `version`, and no other
method. No method takes an argument, and one sent with an argument raises
`SYNTAX 93.902`, `copy` included. `copy` without an argument raises
`SYNTAX 93.970`, and [Deviation 63](r:class.rexxinfo.deviation.63) states the
one route on which it differs.

```rexx
r = .rexxinfo
say (r == .rexxinfo) r~class~id r~class~superclass~id
say r~class~superclasses~items r~class~subclasses~items
s = r~class~methods(.nil)
names = .array~new
do while s~available
  names~append(s~index)
  s~next
end
say names~items
names = names~sort
do i = 1 to names~items by 5
  say names~section(i, min(5, names~items - i + 1))~makeString('l', ' ')
end
call try 'r~copy'
call try 'r~copy(1)'
call try 'r~name(1)'
call try 'r~platform("x", "y")'
call try 'r~version(, 2)'
exit
try:
  signal on syntax name bad
  interpret 'rr =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 RexxInfo Object
1 0
29
ARCHITECTURE CASESENSITIVEFILES COPY DATE DEBUG
DIGITS DIRECTORYSEPARATOR ENDOFLINE EXECUTABLE FORM
FUZZ INTERNALDIGITS INTERNALMAXNUMBER INTERNALMINNUMBER LANGUAGELEVEL
LIBRARYPATH MAJORVERSION MAXARRAYSIZE MAXEXPONENT MAXPATHLENGTH
MINEXPONENT MODIFICATION NAME PACKAGE PATHSEPARATOR
PLATFORM RELEASE REVISION VERSION
r~copy => 93.970
r~copy(1) => 93.902
r~name(1) => 93.902
r~platform("x", "y") => 93.902
r~version(, 2) => 93.902
```

## Numeric settings and limits

r[class.rexxinfo.numeric]
`digits`, `form` and `fuzz` answer the defaults of the language, `9`,
`SCIENTIFIC` and `0`, whatever the NUMERIC settings of the running code are
and also when a `::OPTIONS` directive gives the package other defaults, as
[the DIGITS rule](r:bif.digits.default) states for the code. `maxExponent`
answers `999999999` and `minExponent` `-999999999`, the bounds of [the
exponent range](r:bif.index.num.limit).

```rexx
numeric digits 20; numeric form engineering; numeric fuzz 3
say .rexxinfo~digits .rexxinfo~form .rexxinfo~fuzz
say digits() form() fuzz() .context~digits .context~form .context~fuzz
say .rexxinfo~maxExponent .rexxinfo~minExponent
say .o~new~show
exit
::options digits 12 form engineering fuzz 2
::class o
::method show
  return digits() form() fuzz() .rexxinfo~digits .rexxinfo~form .rexxinfo~fuzz
```

```output
9 SCIENTIFIC 0
20 ENGINEERING 3 20 ENGINEERING 3
999999999 -999999999
12 ENGINEERING 2 9 SCIENTIFIC 0
```

r[class.rexxinfo.limits]
`internalDigits`, `internalMaxNumber`, `internalMinNumber`, `maxArraySize`
and `maxPathLength` answer whole numbers whose values depend on the build
and the host and are not specified. They agree with one another: the largest
internal whole number, `internalMaxNumber`, is made of `internalDigits`
nines, and `internalMinNumber` is the same digits with a minus sign. A
[whole-number argument](r:conv.whole.digits) may have the value of
`internalMaxNumber`. `maxArraySize` and `maxPathLength` are positive.

```rexx
r = .rexxinfo
say r~internalDigits~class~id datatype(r~internalDigits, 'W')
say (r~internalMaxNumber == copies('9', r~internalDigits))
say (r~internalMinNumber == '-'r~internalMaxNumber)
say datatype(r~maxArraySize, 'N') (r~maxArraySize > 0)
say datatype(r~maxPathLength, 'W') (r~maxPathLength > 0)
say substr('abc', r~internalMaxNumber)'|'
```

```output
String 1
1
1
1 1
1 1
|
```

## The interpreter

r[class.rexxinfo.version]
`name`, `version`, `majorVersion`, `release`, `modification`,
`languageLevel`, `date`, `platform` and `revision` describe the interpreter.
Their values are not specified; the book gives their forms and what each
says of the others. `version` is `majorVersion`, `release` and
`modification` as three whole numbers joined by points. `name` is the string
that [PARSE VERSION](r:instr.parse.version) reads, so that `languageLevel` is
its second word and `date` its last three, written as a day, a three-letter
month and a year; `platform` is a string without blanks, and `revision`
a string whose form is not specified either.

```rexx
r = .rexxinfo
parse version id level d m y
say (r~name == id level d m y) (r~languageLevel == level) (r~date == d m y)
say r~date~words datatype(d, 'W') length(m) length(y)
say (r~version == r~majorVersion'.'r~release'.'r~modification)
say datatype(r~majorVersion, 'W') datatype(r~release, 'W'),
    datatype(r~modification, 'W') datatype(r~languageLevel, 'N')
say r~platform~words (r~platform == r~platform~strip)
say r~revision~class~id
```

```output
1 1 1
3 1 3 4
1
1 1 1 1
1 1
String
```

## The host

r[class.rexxinfo.system]
`architecture` answers the number of bits of the interpreter's addressing,
`32` or `64`; `caseSensitiveFiles` and `debug` answer `1` or `0`, and
`caseSensitiveFiles` is what `isCaseSensitive` of the [`File`](file.md) class
answers. `directorySeparator` and `pathSeparator` answer the strings that
the `separator` and `pathSeparator` of [`File`](file.md) answer, one
character each, and `endofline` the line end of the host, either `'0A'x` or
`'0D0A'x`. `executable` and `libraryPath` answer a `File` for the
interpreter's program and for the directory of its libraries; their paths
are not specified.

```rexx
r = .rexxinfo
say (r~architecture == 32 | r~architecture == 64)
c = r~caseSensitiveFiles
say c~class~id (c == 0 | c == 1) (r~debug == 0 | r~debug == 1)
say (c == .file~isCaseSensitive)
say (r~directorySeparator == .file~separator),
    (r~pathSeparator == .file~pathSeparator),
    r~directorySeparator~length r~pathSeparator~length
say (r~endofline == '0A'x) | (r~endofline == '0D0A'x)
f = r~executable
l = r~libraryPath
say f~class~id l~class~id f~isFile l~isDirectory (f == r~executable)
```

```output
1
String 1 1
1
1 1 1 1
1
File File 1 1 1
```

r[class.rexxinfo.package]
`package` answers the [`Package`](package.md) that holds the library classes
that the interpreter itself defines, the one whose name is `REXX`. Its
`findClass` finds `RexxInfo` and answers `.REXXINFO` itself, but the class is
not among its public ones.

```rexx
p = .rexxinfo~package
say p~class~id p~name
say (p~findClass('RexxInfo') == .rexxinfo) p~findPublicClass('RexxInfo')~isNil
say (p == .context~package)
```

```output
Package REXX
1 1
0
```

## Deviations

r[class.rexxinfo.deviation.63]
A `RexxInfo` is never copied: a `copy` sent with a scope, as
`.rexxinfo~copy:.object` is, raises `SYNTAX 93.970` as every other route does
([the creation rule](r:class.rexxinfo.creation)). ooRexx copies the object in
that case.

```rexx
call try '.rexxinfo~copy:.object'
call try '.rexxinfo~copy'
call try '.rexxinfo~send(.array~of("copy", .object))'
exit
try:
  signal on syntax name bad
  interpret 'rr =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
.rexxinfo~copy:.object => 93.970
.rexxinfo~copy => 93.970
.rexxinfo~send(.array~of("copy", .object)) => 93.970
```

> **Note.** [Deviation 96](../deviations.md) changes the banners of interrexx,
> `rexx -v` and the version information of the C API, but not the answers of
> `RexxInfo` or the string of `PARSE VERSION`, which are the ones the rules
> above give.
