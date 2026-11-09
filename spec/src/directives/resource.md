---
element: "::RESOURCE"
kind: directive
syntax: "::RESOURCE name [END delimiter]"
rexxref: "Directives > ::RESOURCE"
---

# ::RESOURCE

::RESOURCE puts lines of text into a package. The lines are not code: the
interpreter does not read them as clauses, and a program reads them back as
data by the name of the resource. A resource is the way to carry a template,
a table or a message text inside the file of a program. This page specifies
the directive and where the data ends, the lines as they are kept, and the
way that a program reaches them. The methods of the collection that holds the
resources belong to the classes chapter, and the examples use only those that
are needed to read an item.

## The directive

r[dir.resource.create]
The directive clause names the resource, and the lines that follow it are its
data, up to a line that starts with the end delimiter. The name is a symbol or
a string. The delimiter follows the word END, in any case, and is a symbol or
a string too. Without END the delimiter is `::END`. A delimiter that is a
symbol is taken in upper case, and one that is a string as written, so that
`end done` stands for the text `DONE` and `end "done"` for `done`. The match
is case sensitive and is made at the start of the line: a line ends the data
when its first characters are the delimiter, wherever the line goes on after
them, and a line that has the delimiter after a blank, or in another case, is
data. The delimiter line is not part of the data, and the clause that follows
it is the first clause after the resource. A semicolon after the directive
ends the directive clause, and what follows it on that line is dropped and
never becomes a clause. A comment may follow the directive.

```rexx
a = .resources~tune
say a~items
do line over a
  say '['line']'
end
say .resources~dflt~items .resources~sym~items .resources~str~items -
  .resources~semi~items
::resource tune end "="
Hold the line
=====
::resource dflt
line one
 ::END indented is data
::end lowercase is data
::END text after the delimiter is ignored
::resource sym end STOPHERE
a
Stophere is data
STOPHEREX is the delimiter line
::resource str end "end of it" -- a comment
x
end of it
::resource semi end "e"; ignored text
body
e
```

```output
1
[Hold the line]
3 2 1 1
```

r[dir.resource.name]
The name that a program reads a resource by is the name in upper case, whether
it was written as a symbol or as a string, so `::resource "Mixed"` is read as
`MIXED`. Names are not otherwise changed: a name that is a string may hold
blanks. A package holds one resource of a name, and a second ::RESOURCE of a
name that is the same in upper case, whatever its spelling, raises `SYNTAX
99.942`. A resource may have the same name as a class, a method or a routine.
A directive whose name is missing, or is neither a symbol nor a string,
raises `19.920`. The errors are found when the program is read.

```rexx
say .resources~allindexes~sort~makestring('l', ',')
say .resources['MIXED']~items .resources['A B']~items
say .resources['Mixed']
exit
::resource "Mixed" end "x"
one
x
::resource "a b"
::END
::resource 12
::END
::resource box
::END
::class box
::routine box
  return 1
```

```output
12,A B,BOX,MIXED
1 0
The NIL object
```

```rexx
call try '::resource a', '::END', '::resource a', '::END'
call try '::resource a', '::END', '::resource "A"', '::END'
call try '::resource a', '::END', '::resource b', '::END'
call try '::resource'
call try '::resource +', '::END'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-resource-name', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::resource a / ::END / ::resource a / ::END => 99.942
::resource a / ::END / ::resource "A" / ::END => 99.942
::resource a / ::END / ::resource b / ::END => ok
::resource => 19.920
::resource + / ::END => 19.920
```

r[dir.resource.delimiter]
A word after the name that is not END raises `SYNTAX 25.926`, and so does
LIBRARY. END with no delimiter after it, or with a delimiter that is neither a
symbol nor a string, such as a number sign or a parenthesis, raises `19.921`.
A token after the delimiter, other than a semicolon, a comment or the end of
the line, raises `21.914`. A string delimiter that is empty is
a prefix of every line and so ends the data at the first line: the resource is
empty. The data must end: when no line starts with the delimiter, the
directive raises `SYNTAX 99.943`, so that a delimiter written as a symbol
cannot end the data with a line in lower case. All of these faults are found
when the program is read.

```rexx
call try '::resource a bogus', 'x', '::END'
call try '::resource a library', 'x', '::END'
call try '::resource a end', 'x'
call try '::resource a end +', 'x'
call try '::resource a end ("q")', 'x', 'q'
call try '::resource a end "q" junk', 'x', 'q'
call try '::resource a end "q" ,', 'x', 'q'
call try '::resource a end "q"', 'x'
call try '::resource a end q', 'x', 'q'
call try '::resource a end q', 'x', 'Q'
call try '::resource a', 'x'
call try '::resource a', 'x', '::end'
call try '::resource a end ""', 'x', '::class k'
call try '::resource a end 5', 'x', '5 is a number'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-resource-delim', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::resource a bogus / x / ::END => 25.926
::resource a library / x / ::END => 25.926
::resource a end / x => 19.921
::resource a end + / x => 19.921
::resource a end ("q") / x / q => 19.921
::resource a end "q" junk / x / q => 21.914
::resource a end "q" , / x / q => 21.914
::resource a end "q" / x => 99.943
::resource a end q / x / q => 99.943
::resource a end q / x / Q => ok
::resource a / x => 99.943
::resource a / x / ::end => 99.943
::resource a end "" / x / ::class k => ok
::resource a end 5 / x / 5 is a number => ok
```

r[dir.resource.section]
A resource has no section of code of its own. The clauses after the delimiter
line that are not null clauses belong to no directive, and one raises `SYNTAX
99.916`, as [the section rule](r:dir.index.section) says. A line of the data
that starts with two colons is data, and so is every other line: the data is
not checked in any way, so that it may hold unbalanced quotes, comment starts
and directives. The next directive may follow the delimiter line at once. A
resource inside a class section does not end the class: the methods that
follow it belong to the class as before.

```rexx
call try '::resource a', '::END', 'say 1'
call try '::resource a', '::END', '', '-- a comment', '::class k'
call try '::resource a', ' say "unbalanced', '::method m', '::END'
call try '::class k', '::resource a', 'x', '::END', '::method m', ' return 1'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-resource-sec', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::resource a / ::END / say 1 => 99.916
::resource a / ::END /  / -- a comment / ::class k => ok
::resource a /  say "unbalanced / ::method m / ::END => ok
::class k / ::resource a / x / ::END / ::method m /  return 1 => ok
```

```rexx
say .k~new~m()
::class k
::resource a
x
::END
::method m
  return 'method of k'
```

```output
method of k
```

## The data

r[dir.resource.lines]
The data of a resource is the list of its lines, in order, each as written in
the file: leading and trailing blanks, tabs and empty lines are kept, and the
line end is not part of a line. A line end of one carriage return and a line
feed ends the line like a line feed alone. A resource with no line between
the directive and the delimiter has no items, and neither has one whose
delimiter is the empty string, since the first line is then the delimiter
line. The lines are not read for
comments, strings or symbols, so that a line holds exactly its characters.

```rexx
do name over 'BLANK', 'TAIL', 'EMPTY', 'GONE', 'MIXED'
  a = .resources[name]
  say name a~items
  do i = 1 to a~items
    say '  ['a[i]']' a[i]~length
  end
end
::resource blank
 
  

::END
::resource tail
x  
y	
::END
::resource empty
::END
::resource gone end ""
this line is the delimiter line
::resource mixed
/* not a comment */ "not a string" -- not a comment
::END
```

```output
BLANK 3
  [ ] 1
  [  ] 2
  [] 0
TAIL 2
  [x  ] 3
  [y	] 2
EMPTY 0
GONE 0
MIXED 1
  [/* not a comment */ "not a string" -- not a comment] 51
```

```rexx
f = 'spec-dir-resource-crlf.tmp'
call sysfiledelete f
call charout f, '::routine crlfitems public' || '0A'x
call charout f, '  a = .resources~crlf' || '0A'x
call charout f, '  return a~items a[1]~length a[2]~length' || '0A'x
call charout f, '::resource crlf' || '0D0A'x
call charout f, 'abc' || '0D0A'x
call charout f, 'de' || '0A'x
call charout f, '::END' || '0D0A'x
call charout f
src = .array~of('say crlfitems()', '::requires "'f'"')
p = .package~new('spec-dir-resource-crlf', src)
call sysfiledelete f
```

```output
2 3 2
```

## Reaching the data

r[dir.resource.access]
The environment symbol `.RESOURCES` holds the resources of the package whose
code reads it. It is a collection indexed by the resource names in upper case,
and each item is an array of the lines of that resource. It is there from the
start of the prolog, and in every routine and method of the package. A package
with no ::RESOURCE has no `.RESOURCES`, and the symbol has [its own text as
value](r:lex.symbols.environment). The index is looked up exactly, so that
`.resources['COBALT']` finds the resource named `cobalt` and
`.resources['cobalt']` finds nothing. The collection is each package's own:
the resources of a required package are not in the `.RESOURCES` of the
package that requires it, and the other way round, so that a routine of the
required package reads its own. The arrays are shared by all code of the
package: an item that one routine changes, or an array that it extends, is
changed for every later read, in any routine or method of the package.

```rexx
say 'none:' .resources
say .box~new~m
say r()
f = 'spec-dir-resource-lib.tmp'
call sysfiledelete f
call lineout f, '::routine libres public'
call lineout f, '  return .resources~items .resources~allindexes'
call lineout f, '::resource inlib'
call lineout f, 'x'
call lineout f, '::END'
call lineout f
src = .array~of('say "lib:" libres()', 'say "main:" .resources~items', -
  'say "second:" later()', '::requires "'f'"', -
  '::resource inmain', 'y', '::END', '::resource second', '::END', -
  '::routine later public', '  return .resources~items')
p = .package~new('spec-dir-resource-main', src)
call sysfiledelete f
::class box
::method m
  return .resources
::routine r
  return .resources
```

```output
none: .RESOURCES
.RESOURCES
.RESOURCES
lib: 1 INLIB
main: 2
second: 2
```

```rexx
.resources~tags[1] = 'changed'
call show 'prolog'
.resources~tags~append('third')
call show 'prolog'
say .box~new~m
exit
show: say arg(1) .resources~tags~items .resources~tags[1]; return
::class box
::method m
  return 'method' .resources~tags~items .resources~tags[1]
::resource tags
first
second
::END
```

```output
prolog 2 changed
prolog 3 changed
method 3 changed
```
