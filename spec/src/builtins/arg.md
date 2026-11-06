---
element: ARG
kind: builtin
syntax: "ARG([n [, option]])"
ansi: "9.5.2"
rexxref: "Functions > Built-in Functions > ARG (Argument)"
---

# ARG

ARG reports on the arguments of the routine, method or program that is
running: how many there were, what one of them was, whether one was passed at
all, or all of them from a given one on.

## Arguments

r[bif.arg.args]
ARG takes at most two arguments, `n` and `option`, and both are optional. The
argument count is checked by the [builtin call rules](r:bif.index.call.count):
none is required and two is the most, so a call with three raises
`SYNTAX 40.4`. ARG does not run as a String method: sending the message `ARG`
to a string raises `SYNTAX 97.1`.

```rexx
call show 'a'
exit
show:
say arg() arg(,) arg(1) arg(1, ) arg(1, 'E')
signal on syntax name a
say arg(1, 'E', 1)
a: say condition('o')~code
signal on syntax name b
say 'a'~arg
b: say condition('o')~code
```

```output
1 1 a a 1
40.4
97.1
```

r[bif.arg.n.check]
`n` is a whole number that passes the
[whole-number rules](r:conv.whole.form) and whose value is at least 1. A
value of 0 or less raises `SYNTAX 40.14`; `-0` is 0. A string that is not a
whole number raises `SYNTAX 40.12`, the null string included, and so does a
number whose magnitude is [too large](r:conv.whole.digits). The caller's
NUMERIC DIGITS plays no part. These are not the range errors of the
[String-method builtins](r:conv.position.check).

```rexx
numeric digits 1
call show 'a', 'b'
exit
show:
say arg(2) arg(' 2 ') arg('2.0') arg(1e0)
signal on syntax name a
say arg(0)
a: say condition('o')~code
signal on syntax name b
say arg('-0')
b: say condition('o')~code
signal on syntax name c
say arg('')
c: say condition('o')~code
signal on syntax name d
say arg(1.5)
d: say condition('o')~code
signal on syntax name e
say arg(1e30)
e: say condition('o')~code
```

```output
b b b a
40.14
40.14
40.12
40.12
40.12
```

r[bif.arg.option.check]
`option` is a string; its first character, in either case, selects what ARG
answers, and every character after the first is ignored. The letters are
`A`, `E`, `N` and `O`. Any other first character, and the null string,
raises `SYNTAX 40.904`. When
`option` is given and `n` is omitted, the call raises `SYNTAX 40.5`. `n` is
checked before `option`.

```rexx
call show 'a'
exit
show:
say arg(1, 'E') arg(1, 'exists') arg(1, 'OMITTED') arg(1, 'n') arg(1, 'Array')~items
signal on syntax name a
say arg(1, 'x')
a: say condition('o')~code
signal on syntax name b
say arg(1, '')
b: say condition('o')~code
signal on syntax name c
say arg(1, ' e')
c: say condition('o')~code
signal on syntax name d
say arg(, 'e')
d: say condition('o')~code
signal on syntax name e
say arg(0, 'x')
e: say condition('o')~code
signal on syntax name f
say arg(1, 1)
f: say condition('o')~code
```

```output
1 1 0 a 1
40.904
40.904
40.904
40.5
40.14
40.904
```

## The number of arguments

r[bif.arg.count]
Without `n`, ARG answers the number of arguments of the call that started
the running routine, method or program: the position of the last argument
that was [not omitted](r:conv.arg.omitted). Omitted places at the end of the
list add nothing, and a call with only omitted places has no arguments. An
omitted argument before a later one counts, and so does an argument that is
the null string. The answer is [written as a count](r:bif.index.count.answer),
and whether it is a new object is
[not specified](r:bif.index.count.answer.deviation.4).

```rexx
call show
call show 'a', 'b'
call show 'a', , 'c'
call show , 'b'
call show ''
x = show('a', , )
y = show(, )
exit
show:
say arg()
return 0
```

```output
0
2
3
2
1
1
0
```

## One argument

r[bif.arg.n]
With `n` and no `option`, ARG answers argument `n` itself, the object the
caller passed and not a string made from it. When argument `n` was omitted, or
`n` is greater than the number of arguments, ARG answers the null string.

```rexx
call show 'one', , .nil, .array~of(1, 2)
exit
show:
say '['arg(1)']' '['arg(2)']' '['arg(3)']' arg(4)~class arg(4)~items
say '['arg(5)']' '['arg(999999)']' '['arg(999999999999999999)']'
```

```output
[one] [] [The NIL object] The Array class 2
[] [] []
```

r[bif.arg.option.normal]
With the option `N`, ARG answers what it does for `n` alone.

```rexx
call show 'x', , 'z'
exit
show:
say '['arg(1, 'N')']' '['arg(2, 'N')']' '['arg(3, 'N')']' '['arg(4, 'N')']'
```

```output
[x] [] [z] []
```

## Omitted and null arguments

r[bif.arg.option.exists]
With the option `E`, ARG answers `1` when argument `n` was passed, and `0`
when it was omitted or `n` is greater than the number of arguments. An
argument whose expression is the null string was passed, so the answer for it
is `1`.

```rexx
call show 'a', , '', , 'e'
exit
show:
say arg(1, 'E') arg(2, 'E') arg(3, 'E') arg(4, 'E') arg(5, 'E') arg(6, 'E')
```

```output
1 0 1 0 1 0
```

r[bif.arg.option.omitted]
With the option `O`, ARG answers the opposite of `E`: `0` when argument `n`
was passed, `1` when it was omitted or `n` is greater than the number of
arguments.

```rexx
call show 'a', , '', , 'e'
call show
exit
show:
say arg(1, 'O') arg(2, 'O') arg(3, 'O') arg(4, 'O') arg(5, 'O') arg(6, 'O')
return
```

```output
0 1 0 1 0 1
1 1 1 1 1 1
```

r[bif.arg.null-versus-omitted]
Only the options `E` and `O` tell a null-string argument from an omitted
one: `ARG(n)` and the option `N` answer the null string for both.

```rexx
call show '', 'b'
call show , 'b'
exit
show:
say '['arg(1)']' arg(1, 'E') arg(1, 'O') (arg(1) == '')
return
```

```output
[] 1 0 1
[] 0 1 1
```

## All arguments from one on

r[bif.arg.option.array]
With the option `A`, ARG answers an array of the arguments from argument `n`
on. Index 1 holds argument `n`, index 2 argument `n + 1`, and so on, up to the
last argument, so the array has as many places as there are arguments from
`n` on. An argument that was omitted leaves its index without an item. The
array has no places when `n` is greater than the number of arguments.

```rexx
call show 'a', , 'c', , 'e'
exit
show:
do n = 1 to 6
  a = arg(n, 'A')
  s = ''
  do i = 1 to a~size
    if a~hasIndex(i) then s = s i':'a[i]
    else s = s i':-'
  end
  say 'n='n 'size' a~size 'items' a~items '|'s
end
```

```output
n=1 size 5 items 3 | 1:a 2:- 3:c 4:- 5:e
n=2 size 4 items 2 | 1:- 2:c 3:- 4:e
n=3 size 3 items 2 | 1:c 2:- 3:e
n=4 size 2 items 1 | 1:- 2:e
n=5 size 1 items 1 | 1:e
n=6 size 0 items 0 |
```

## Which arguments

r[bif.arg.scope]
ARG answers for the routine, method or program whose code is running: for an
internal or external routine, the arguments of the call that started it; for
a method, the arguments of the message; for a program, those it was started
with. When a routine returns, the caller's arguments are what ARG reports
again. A routine does not see its caller's arguments, whether or not it is
declared with PROCEDURE. INTERPRET runs in the routine that holds it, so ARG
there reports that routine's arguments.

```rexx
say 'program' arg() '['arg(1)']'
call outer 'o1', 'o2'
exit
outer:
say 'outer' arg() arg(1)
call inner 'i1'
call plain 'p1'
say 'outer again' arg() arg(1) arg(2)
interpret 'say "interpreted" arg() arg(2)'
return
inner: procedure
say 'inner' arg() arg(1) '['arg(2)']'
return
plain:
say 'plain' arg() arg(1) '['arg(2)']'
return
```

```output
program 0 []
outer 2 o1
inner 1 i1 []
plain 1 p1 []
outer again 2 o1 o2
interpreted 2 o2
```

r[bif.arg.method]
In a method, ARG reports the arguments of the message that ran it, counted
as for a [routine](r:bif.arg.count): omitted places at the end add nothing,
and an omitted place before a later argument counts. A routine that the
method calls has its own arguments. A `::ROUTINE` directive gives a routine
that is called like any other, and a routine and a method with the same
arguments see the same ARG answers.

```rexx
o = .box~new
say o~m('p', , 'q')
say o~m()
say o~m(,)
say rt('p', , 'q')
say rt()
::class box
::method m
return arg() arg(1, 'E') arg(2, 'E') arg(3, 'E') '['arg(1)']' '['arg(2)']' '['arg(3)']'
::routine rt
return arg() arg(1, 'E') arg(2, 'E') arg(3, 'E') '['arg(1)']' '['arg(2)']' '['arg(3)']'
```

```output
3 1 0 1 [p] [] [q]
0 0 0 0 [] [] []
0 0 0 0 [] [] []
3 1 0 1 [p] [] [q]
0 0 0 0 [] [] []
```

r[bif.arg.large]
The answer for the count is exact however many arguments there are, and
however small the NUMERIC DIGITS setting of the code that asks.

```rexx
src = .array~of('numeric digits 3', 'return arg() arg(1200, "E") arg(1200)')
r = .routine~new('big', src)
a = .array~new
do i = 1 to 1200; a~append(i); end
say r~callWith(a)
```

```output
1200 1 1200
```

> **Note.** The ANSI draft (9.5.2) defines ARG over the arguments of the
> current level, with the options `E`, `N` and `O`. ooRexx adds the option
> `A`, and interrexx follows ooRexx.
