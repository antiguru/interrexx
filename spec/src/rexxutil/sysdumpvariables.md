---
element: "SysDumpVariables"
kind: function
syntax: "SysDumpVariables([filename])"
rexxref: "Rexx Utilities (RexxUtil) > SysDumpVariables"
---

# SysDumpVariables

SysDumpVariables writes the variables of the code that calls it, one line for
each, to the standard output or to the end of a file. It answers `0` when it
has done so.

> **Note.** The order in which the lines of a dump come is not specified, so
> the examples of this page read the lines back and print them in the order of
> their bytes, with a routine called `show`. A line of a dump that is read
> back as text ends in `<CR>` or `<LF>` in the examples where the line end
> matters, and `show` writes those as the five characters.

## Arguments

r[util.sysdumpvariables.args]
`filename` is optional. The call follows
[the argument rule](r:util.index.args): a second argument raises
`SYNTAX 88.922`, and an argument that is an object other than a string is read
by its string value or raises `SYNTAX 88.909` when it has none, as `.nil`
does. An omitted `filename`, also one that is omitted with a comma, means the
standard output. The null string is a name, not an omitted argument. The name
of a file is read as [the chapter page](index.md) says for every name.

```rexx
call try "SysDumpVariables('c3zqdva.txt', 'x')"
call try "SysDumpVariables(.nil)"
call try "SysDumpVariables(.mutablebuffer~new('c3zqdva.txt'))"
say SysFileExists('c3zqdva.txt')
call SysFileDelete 'c3zqdva.txt'
call SysMkDir 'c3zqdvb'
call try "SysDumpVariables('c3zqdvb/./f')"
say SysFileExists('c3zqdvb/f')
call SysFileDelete 'c3zqdvb/f'
call SysRmDir 'c3zqdvb'
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
88.922
88.909
0
1
0
1
```

## What is written

r[util.sysdumpvariables.line]
Each variable that has a value in the scope of the call is written as the
text `Name=`, the name of the variable in upper case, the text `, Value='`,
the string value of the variable exactly as it is, without any change to the
quote marks, line ends or other characters it holds, and the text `'`, and
then a line end. A simple or compound variable that has been dropped is not
written. A stem is written as [the stem rule](r:util.sysdumpvariables.stem)
says, even when it has been dropped. Nothing is written when there is no
variable, and a file is then made and is empty. The order of the lines is not
specified.

```rexx
call p
exit
p: procedure
  b = 2; a = 'one'; Mixed = 'mix'; c = ''
  say SysDumpVariables('c3zqdv1.txt')
  call show 'c3zqdv1.txt'
  say '--'
  drop b sigl
  call SysDumpVariables 'c3zqdv1.txt'
  call show 'c3zqdv1.txt'
  call SysFileDelete 'c3zqdv1.txt'
  return
show: procedure
  parse arg file
  n = 0
  do while lines(file) > 0
    n = n + 1
    l.n = changestr('0d'x, linein(file), '<CR>')
  end
  call stream file, 'c', 'close'
  l.0 = n
  call SysStemSort 'l.'
  do i = 1 to n; say l.i; end
  return
```

```output
0
Name=A, Value='one'
Name=B, Value='2'
Name=C, Value=''
Name=MIXED, Value='mix'
--
Name=A, Value='one'
Name=A, Value='one'
Name=B, Value='2'
Name=C, Value=''
Name=C, Value=''
Name=MIXED, Value='mix'
Name=MIXED, Value='mix'
```

r[util.sysdumpvariables.file]
With a `filename`, the lines are added to the end of the file, which is made
when it does not exist, and the file is closed when the call ends, so that a
later call adds more lines after them. The call answers `0`. A file that
cannot be opened for adding raises `SYNTAX 40.1`: a name that is a directory,
a name under something that is not a directory, and the null string. A name of
a symbolic link that points to nothing makes its target.

```rexx
call p
exit
p: procedure
  say SysDumpVariables('c3zqdv2.txt') SysFileExists('c3zqdv2.txt')
  say chars('c3zqdv2.txt')
  call stream 'c3zqdv2.txt', 'c', 'close'
  call SysFileDelete 'c3zqdv2.txt'
  call SysMkDir 'c3zqdv2d'
  call lineout 'c3zqdv2f', 'x'
  call lineout 'c3zqdv2f'
  call try "SysDumpVariables('c3zqdv2d')"
  call try "SysDumpVariables('c3zqdv2f/x')"
  call try "SysDumpVariables('')"
  address system 'ln -s c3zqdv2n c3zqdv2l'
  call try "SysDumpVariables('c3zqdv2l')"
  say SysFileExists('c3zqdv2n')
  call SysFileDelete 'c3zqdv2l'
  call SysFileDelete 'c3zqdv2n'
  call SysFileDelete 'c3zqdv2f'
  call SysRmDir 'c3zqdv2d'
  return
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
0 1
0
40.1
40.1
40.1
0
1
```

r[util.sysdumpvariables.stdout]
Without a `filename`, the lines are written to the standard output of the
process, whatever the stream `.output` is bound to: a redirection of `.output`
does not catch them, and they come in order with the text that the program
writes with SAY before and after the call.

```rexx
call p
exit
p: procedure
  v = 'val'
  say 'before'
  .local~old = .output~destination(.sink~new)
  say 'caught'
  call SysDumpVariables
  x = .output~destination
  say 'log:' .sink~log
  return
::class sink
::attribute log class
::method init class
  expose log
  log = ''
::method lineout
  .sink~log = .sink~log arg(1)
  return 0
::method say
  .sink~log = .sink~log arg(1)
  return 0
::method charout
  .sink~log = .sink~log arg(1)
  return 0
```

```output
before
Name=V, Value='val'
log:  caught
```

## Which variables

r[util.sysdumpvariables.scope]
The variables are those of the code that makes the call: in a procedure its
own variables and the ones that it has exposed, not the variables of its
caller; in a method the variables that the method exposes and its own,
together with `SELF` and `SUPER`; in the main program its variables. A
compound variable that a procedure exposes by itself, as `expose s.2` does, is
not written, and only its stem has a line. `RC`, `RESULT` and `SIGL` are
variables like the others, and are written when the program or a call has set
them.

```rexx
g = 'gg'; h = 'hh'; s.1 = 'one'; s.2 = 'two'
call p
say '--'
call SysFileDelete 'c3zqdv4.txt'
call q
say '--'; call r; say '--'
.thing~new~m
exit
p: procedure expose g
  loc = 'l'
  call SysDumpVariables 'c3zqdv4.txt'
  call show 'c3zqdv4.txt'
  return
q: procedure
  call q2
  call SysDumpVariables 'c3zqdv4.txt'
  call show 'c3zqdv4.txt'
  return
q2: return 'ret'
r: procedure expose s.2
  call SysDumpVariables 'c3zqdv4.txt'
  call show 'c3zqdv4.txt'
  return
::class thing
::method init
  expose iv
  iv = 'ivalue'
::method m
  expose iv
  local = 'x'
  call SysDumpVariables 'c3zqdv4.txt'
  call show 'c3zqdv4.txt'
::routine show
  parse arg file
  n = 0
  do while lines(file) > 0
    n = n + 1
    l.n = linein(file)
  end
  call stream file, 'c', 'close'
  call SysFileDelete file
  l.0 = n
  call SysStemSort 'l.'
  do i = 1 to n; say l.i; end
```

```output
Name=G, Value='gg'
Name=LOC, Value='l'
--
Name=RESULT, Value='ret'
Name=SIGL, Value='15'
--
Name=S., Value='S.'
--
Name=IV, Value='ivalue'
Name=LOCAL, Value='x'
Name=SELF, Value='a THING'
Name=SUPER, Value='The Object class'
```

r[util.sysdumpvariables.stem]
Every stem that the code has used is written with a line for the stem itself.
Using a stem means assigning, reading or dropping the stem or one of its
tails, whether in a clause or through a function such as VALUE, VAR or one of
the stem functions; naming one of its tails in a `SYMBOL` call; or calling a
procedure that exposes the stem or one of its tails. Naming the stem itself in
a `SYMBOL` call, as `symbol('u.')` does, is not a use. The line is written
whether or not the stem has a value of its own or any tail now. Its name is the name of the stem
with its period, and its value is the default value of the stem, or the name
of the stem when no default was assigned. A variable with a tail is written
with the name of the stem in upper case, a period and the tail as it is after
its symbols were replaced by their values, so that a tail that was written
with a variable that holds `key` is written as `key`, and a tail that was
written as a symbol is in upper case. A line of such a variable, with a tail,
ends with a carriage return before its line end, which the line of any other
variable does not have.

```rexx
call p
say '--'
call q
say '--'
call r
say '--'
call s
exit
p: procedure
  s.1 = 'x'; s.abc = 'q'; t. = 'dflt'; k = 'key'; v.k = 'kv'
  call SysDumpVariables 'c3zqdv5.txt'
  call show 'c3zqdv5.txt'
  return
q: procedure
  x = u.1 symbol('w.2')
  call SysDumpVariables 'c3zqdv5.txt'
  call show 'c3zqdv5.txt'
  return
r: procedure
  drop u.
  call SysDumpVariables 'c3zqdv5.txt'
  call show 'c3zqdv5.txt'
  return
s: procedure
  x = symbol('u.') value('v.1')
  call SysDumpVariables 'c3zqdv5.txt'
  call show 'c3zqdv5.txt'
  return
show: procedure
  parse arg file
  text = charin(file, 1, chars(file))
  call stream file, 'c', 'close'
  call SysFileDelete file
  n = 0
  do while text \== ''
    parse var text line '0a'x text
    n = n + 1
    l.n = changestr('0d'x, line, '<CR>')
  end
  l.0 = n
  call SysStemSort 'l.'
  do i = 1 to n; say l.i; end
  return
```

```output
Name=K, Value='key'
Name=S., Value='S.'
Name=S.1, Value='x'<CR>
Name=S.ABC, Value='q'<CR>
Name=T., Value='dflt'
Name=V., Value='V.'
Name=V.key, Value='kv'<CR>
--
Name=U., Value='U.'
Name=W., Value='W.'
Name=X, Value='U.1 LIT'
--
Name=U., Value='U.'
--
Name=V., Value='V.'
Name=X, Value='LIT V.1'
```

r[util.sysdumpvariables.value]
The value of a variable that holds a string is the string. A value that is
another object is written as its string value, found
[as the conventions say](r:conv.string.object) for a string argument, so that
`.nil` is written as `The NIL object` and an object with a `MAKESTRING`
method is written as its answer.

```rexx
call p
exit
p: procedure
  q = "it's" || '0a'x || 'two' || '09'x || 'x'
  call SysDumpVariables 'c3zqdv6.txt'
  text = charin('c3zqdv6.txt', 1, chars('c3zqdv6.txt'))
  say changestr('09'x, changestr('0a'x, text, '<LF>'), '<TAB>')
  call stream 'c3zqdv6.txt', 'c', 'close'
  call SysFileDelete 'c3zqdv6.txt'
  drop q text result
  a = .withstring~new; b = .withmake~new; c = .withboth~new; d = .nil
  e = .object~new
  call SysDumpVariables 'c3zqdv6.txt'
  call show 'c3zqdv6.txt'
  return
show: procedure
  parse arg file
  n = 0
  do while lines(file) > 0
    n = n + 1
    l.n = linein(file)
  end
  call stream file, 'c', 'close'
  call SysFileDelete file
  l.0 = n
  call SysStemSort 'l.'
  do i = 1 to n; say l.i; end
  return
::class withstring
::method string
  return 'S'
::class withmake
::method makestring
  return 'M'
::class withboth
::method string
  return 'S'
::method makestring
  return 'M'
```

```output
Name=Q, Value='it's<LF>two<TAB>x'<LF>
Name=A, Value='S'
Name=B, Value='M'
Name=C, Value='M'
Name=D, Value='The NIL object'
Name=E, Value='an Object'
```
