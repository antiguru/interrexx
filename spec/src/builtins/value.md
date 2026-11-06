---
element: VALUE
kind: builtin
syntax: "VALUE(name [, newvalue [, selector]])"
ansi: "9.8.6"
rexxref: "Functions > Built-in Functions > VALUE"
---

# VALUE

VALUE reads a variable whose name the program has as a string, and when it is
given a new value, assigns that value to the variable. It answers the value the
variable had before the call. Its third argument chooses where the variable
is: among the variables of the running code, among the entries of the
`.environment` directory, or among the environment variables of the operating
system.

## Arguments

r[bif.value.args]
`name` and `selector` are [strings](r:conv.string.null) and `newvalue` can be
any object. The argument count is checked by the [builtin call
rules](r:bif.index.call.count): one argument is required and three is the
most. VALUE does not run as a String method: sending the message `VALUE` to a
string raises `SYNTAX 97.1`.

```rexx
x = 'a'
say value('x') value('x', ) value('x', , )
signal on syntax name a
say value()
a: say condition('o')~code
signal on syntax name b
say value(, 1)
b: say condition('o')~code
signal on syntax name c
say value('x', 1, , 2)
c: say condition('o')~code
signal on syntax name d
say 'x'~value
d: say condition('o')~code
```

```output
a a a
40.3
40.5
40.4
97.1
```

r[bif.value.selector]
`selector` chooses the variables `name` is looked up in. When it is omitted,
they are the [variables of the running code](r:bif.value.local). When it is
the null string, they are the entries of the
[`.environment` directory](r:bif.value.directory). When it is `ENVIRONMENT`, in
any combination of cases and with nothing around it, they are the
[environment variables](r:bif.value.os) of the operating system. Any other
`selector` raises `SYNTAX 40.914`, and it is checked before `name` is, so a
call with an invalid selector raises it also when `name` is invalid.

```rexx
signal on syntax name a
say value('x', , 'SYSTEM')
a: say condition('o')~code
signal on syntax name b
say value('x y', , 'ENV')
b: say condition('o')~code
signal on syntax name c
say value('x', , 'ENVIRONMENT ')
c: say condition('o')~code
signal on syntax name d
say value('x', , .nil)
d: say condition('o')~code
say value('x', , 'Environment') '/' value('x', , '')
```

```output
40.914
40.914
40.914
40.914
 / .X
```

## Variables of the running code

r[bif.value.local]
When `selector` is omitted, VALUE works on the variables that the routine or
method running it can see, as [SYMBOL](r:bif.symbol.scope) does: a routine
with PROCEDURE sees none of its caller's variables except the ones it
EXPOSEs, and a method sees an object variable once it exposes it. VALUE does
not expose a variable: what it reads or assigns is not an object variable
unless the method exposed it.

```rexx
g = 'caller'
call plain
call hidden
say g
o = .o~new
say o~read o~exposed
exit
plain:
say value('g') value('g', 'changed') g
return
hidden: procedure
say value('g') value('g', 'private') g
return
::class o
::attribute at
::method read
return value('at') value('at', 'local') at
::method exposed
expose at
at = 'object'
return value('at') value('at', 'changed') at
```

```output
caller caller changed
G G private
changed
AT AT local object object changed
```

r[bif.value.name]
`name` must be a symbol: a string that [SYMBOL](r:bif.symbol.result) does not
call `BAD`. Any other, among them the null string, a string with a blank
anywhere in it, one with a character that is not a symbol character and one of
more than 250 characters, raises `SYNTAX 40.26`. It is read in uppercase, so
it names the same variable in either case, and the name of a compound variable
has the parts of its tail replaced by the values of the variables they name,
as the [compound rule of SYMBOL](r:bif.symbol.compound) says. An object other
than a string is replaced by a string, as [a string
argument](r:conv.string.object) says.

```rexx
idx = 3
box.3 = 'three'
ab = 'plain'
say value('AB') value('ab') value('box.idx') value('BOX.IDXX') value('box.3')
signal on syntax name a
say value('')
a: say condition('o')~code
signal on syntax name b
say value('a b')
b: say condition('o')~code
signal on syntax name c
say value('a-b')
c: say condition('o')~code
signal on syntax name d
say value(copies('a', 251))
d: say condition('o')~code
signal on syntax name e
say value(.nil)
e: say condition('o')~code
say length(value(copies('a', 250)))
```

```output
plain plain three BOX.IDXX three
40.26
40.26
40.26
40.26
40.26
250
```

r[bif.value.get]
The result is the value of the variable named by `name`, as the program sees
it when it uses the symbol. A variable that holds no value, or has been
dropped, answers the name as the program would then see it: in uppercase, with
the parts of a compound name's tail that name a variable holding a value
replaced by that value and the others left as they are. VALUE never raises the
NOVALUE condition, so a trap for it does not see the call. A symbol that is a
constant symbol, a number or one that starts with a period, is answered as it
is, in uppercase, except that a name that
stands for an object of the environment, such as `.nil`, answers that object.
The value is the object the variable holds, not a copy of it, and a variable
that holds a non-string object answers it, so the call can be used with
`~`.

```rexx
signal on novalue name nv
a = 'x'
o = .array~of(1, 2)
t = 'tail'
say value('a') value('b') value('t.u') value('t.t') value('3') value('1e5') value('.zz')
say value('o')~class~id value('o')~items (value('o') == o)
say value('.nil')~class~id value('.nil') (value('.nil') == .nil)
drop a
say value('a')
y = n
nv: say condition('c') condition('d')
```

```output
x B T.U T.tail 3 1E5 .ZZ
Array 2 1
Object The NIL object 1
A
NOVALUE N
```

r[bif.value.set]
When `newvalue` is given, even as the null string, the variable named by
`name` is given it as its value, after the result has been found: the result is
the value from before. An [omitted](r:conv.arg.omitted) `newvalue` is the
only case where nothing is assigned. Any object can be the new value and is
stored as it is, with no change by NUMERIC DIGITS. The name of a constant
symbol, a number or a symbol that starts with a period, cannot be assigned
to, and the call raises `SYNTAX 40.26`.

```rexx
numeric digits 3
say value('v', 'first') value('v', 'second') value('v', ) value('v')
say value('v', '') '[' || v || ']' length(v)
say value('n', 123456) n value('n', 1.50) n
o = .array~of(7)
say value('p', o) (p == o) (value('p') == o)
signal on syntax name a
say value('3', 'x')
a: say condition('o')~code
signal on syntax name b
say value('.nil', 'x')
b: say condition('o')~code
signal on syntax name c
say value('1.a', 'x')
c: say condition('o')~code
```

```output
V first second second
second [] 0
N 123456 123456 1.50
P 1 1
40.26
40.26
40.26
```

r[bif.value.stem]
A `name` that ends at its first period is a stem. Read, it answers the stem
object, which the program uses as it does the stem. Given a `newvalue`, it
gives every compound variable of the stem that value, those assigned before
included, as the assignment `stem. = value` does. Both are references to the
stem, as for [SYMBOL](r:bif.symbol.stem).

```rexx
s.1 = 'own'
say value('s.', 'dflt') s.1 s.2 value('s.2') value('s.zz')
x = value('s.')
say x~class~id x[1] x[2] (x == s.)
say value('fresh.')~class~id symbol('fresh.')
```

```output
S. dflt dflt dflt dflt
Stem dflt dflt 1
Stem VAR
```

## Entries of the `.environment` directory

r[bif.value.directory]
When `selector` is the null string, VALUE works on the entries of the
`.environment` directory, the one that the symbol `.name` finds its object
in. The entry is the one whose index is `name` in uppercase, with no check
that it is a symbol, no replacement in a compound name and no other change, so
the name of a compound variable is the index of one entry. The result is the
object in the entry, and for a `name` with no entry it is the string made of a
period and the name in uppercase. A `newvalue` is stored in the entry as it
is, and the null string and `.nil` are values like any other. Because the
directory is the one that the symbol `.name` reads, the program sees an
assigned object as `.name`.

```rexx
k = 'tail'
say value('t14a', , '') value('T14A', 'one', '') value('t14a', 'two', '')
say value('T14A', , '') .t14a .environment~at('T14A')
say value('t14b.k', 'x', '') .environment~at('T14B.K') value('T14B.TAIL', , '')
say value('t14c', .array~of(1), '')~class~id value('t14c', .nil, '')~class~id,
    value('t14c', , '') '['value('t14d', '', '')']' '['value('t14d', , '')']'
say value('.t14e', 'dot', '') value('t14e', , '') value('.t14e', , '')
```

```output
.T14A .T14A one
two two two
.T14B.K x .T14B.TAIL
String Array The NIL object [.T14D] []
..T14E .T14E dot
```

## Environment variables

r[bif.value.os]
When `selector` is `ENVIRONMENT`, VALUE works on the environment variables of
the operating system. `name` is not checked as a symbol and is not changed, so
case matters and a name with blanks is a name like any other. The result is the
value of the variable, and the null string when there is none, so a variable
that is not set and one that is set to the null string are not told apart. A
`newvalue` that is not omitted is the value the variable has afterwards, as the
string it is replaced by, and `.nil` removes the variable. A character
`'00'x` ends the name as well as the value, so what follows it is lost. A call
whose name is the null string assigns nothing and answers the null string. This
book does not specify a name that contains an equals sign.

```rexx
e = 'ENVIRONMENT'
say '['value('T14_V', , e)']' '['value('T14_V', 'v', e)']' '['value('T14_V', 'w', e)']',
    '['value('T14_V', , e)']'
say '['value('t14_v', , e)']' '['value('T14_V', .nil, e)']' '['value('T14_V', , e)']'
say '['value('T14_V', 12, e)']' '['value('T14_V', '', e)']' '['value('T14_V', , e)']'
say '['value('T14 V', 'blank', 'environment')']' '['value('T14 V', , e)']',
    '['value('T14 V', .nil, e)']'
say '['value('T14_V', 'ab' || '00'x || 'cd', e)']' '['value('T14_V', .nil, e)']'
say '['value('T14_V', , e)']' '['value('T14_V', .nil, e)']'
```

```output
[] [] [v] [w]
[] [w] []
[] [12] []
[] [blank] [blank]
[] [ab]
[] []
```

r[bif.value.os.command]
A command that the program starts sees the environment variables as VALUE has
left them: a variable it gave a value has that value, one that it removed with
`.nil` is not there, and one it set to the null string is there with the null
string as its value.

```rexx
e = 'ENVIRONMENT'
call value 'T14F_PROBE', 'from rexx', e
address system 'printf "[%s]\n" "$T14F_PROBE"'
call value 'T14F_PROBE', .nil, e
address system 'printf "[%s]\n" "$T14F_PROBE"'
call value 'T14F_PROBE', '', e
address system 'printf "[%s]\n" "${T14F_PROBE+set}"'
call value 'T14F_PROBE', .nil, e
```

```output
[from rexx]
[]
[set]
```

> **Note.** The reference manual says that an environment variable set by VALUE
> is not kept after the program ends.
>
> The ANSI draft (9.8.6) raises `SYNTAX 40.37` for an external pool that does
> not exist and `SYNTAX 40.36` for a name the pool rejects, and checks `name`
> as a symbol only for the variables of the running code. ooRexx raises
> `SYNTAX 40.914` for a selector that is not valid, as the [selector
> rule](r:bif.value.selector) says, and this book follows it.
