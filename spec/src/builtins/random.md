---
element: RANDOM
kind: builtin
syntax: "RANDOM([min] [, [max] [, seed]])"
ansi: "9.8.3"
rexxref: "Functions > Built-in Functions > RANDOM"
---

# RANDOM

RANDOM answers a quasi-random whole number from a range. A call can start a
new sequence of numbers from a seed, and a sequence started with a seed is
the same on every run of a program. A sequence started without a seed is not:
the numbers of such a sequence are not specified, so the examples of this page
check what they can say about them, their range and their form, and show
values only for seeded sequences.

## Arguments

r[bif.random.args]
RANDOM takes up to three arguments and requires none: `min`, `max` and
`seed`, each a [whole-number argument](r:conv.whole.form), and each also
accepted when written as a number of the form but not as an integer (`3.0`,
`3e0`, ` 3 `), as long as [its value](r:conv.whole.digits) is a whole
number. The argument count is checked by the [builtin call
rules](r:bif.index.call.count): a call with more than three arguments
raises `SYNTAX 40.4`, and a call with only omitted places counts as a call
with no arguments. A `min`, `max` or `seed` that is not a whole number, the
null string included, raises `SYNTAX 40.12`. RANDOM does not run as a String
method: sending the message `RANDOM` to a string raises `SYNTAX 97.1`.

```rexx
say random(3, 3) random(' 3 ', '3.0') random(3e0, 3e0) random(,0) random(,0,)
signal on syntax name a
say random(1, 2, 3, 4)
a: say condition('o')~code
signal on syntax name b
say random('x')
b: say condition('o')~code
signal on syntax name c
say random(1.5)
c: say condition('o')~code
signal on syntax name d
say random('')
d: say condition('o')~code
signal on syntax name e
say random(, 'x')
e: say condition('o')~code
signal on syntax name f
say random(, , 1.5)
f: say condition('o')~code
signal on syntax name g
say 'a'~random
g: say condition('o')~code
```

```output
3 3 3 0 0
40.4
40.12
40.12
40.12
40.12
40.12
97.1
```

r[bif.random.defaults]
The arguments of a call mean different things according to the
[count of the call](r:bif.index.call.count). When the count is 1, the one
argument is `max`, and `min` is 0. When the count is 2 or 3, the arguments
are `min`, `max` and `seed` in that order; an omitted `min` is 0 and an
omitted `max` is 999. A call with no arguments takes the range 0 to 999.

```rexx
say within('random()', 0, 999) within('random(7)', 0, 7),
    within('random(7, )', 0, 7) within('random(,7)', 0, 7),
    within('random(5, 7)', 5, 7) within('random(-3, -1)', -3, -1),
    within('random(995, , 4)', 995, 999)
say reaches('random(7)', 0, 7) reaches('random(,7)', 0, 7),
    reaches('random(5, 7)', 5, 7) reaches('random(-3, -1)', -3, -1)
exit
within: procedure
parse arg expr, lo, hi
do 300
  interpret 'r =' expr
  if r < lo | r > hi then return 0
end
return 1
reaches: procedure
parse arg expr, lo, hi
do 300
  interpret 'r =' expr
  seen.r = 1
end
return seen.lo & seen.hi
```

```output
1 1 1 1 1 1 1
1 1 1 1
```

> **Note.** The example draws 300 numbers from each call. That both ends of a
> narrow range come out is what 300 draws reach with all but certainty, not
> something a call must do. The call with a seed is run for one draw only,
> since each call with a seed starts the sequence over.

## The result

r[bif.random.result]
RANDOM answers a whole number from `min` to `max`, both included. The
number is written without a sign when it is not negative, with a `-` when it
is, and with no leading zeros, point or exponent; when `min` and `max` are
equal, the number is that value. RANDOM writes every result in full at every
NUMERIC DIGITS setting.

```rexx
bad = 0
do 500
  r = random(-3, 2)
  if r < -3 | r > 2 | \datatype(r, 'W') | r \== r + 0 then bad = bad + 1
  seen.r = 1
end
a = -3; b = -2; c = -1
say bad seen.a seen.b seen.c seen.0 seen.1 seen.2
say random(-4, -4) random(0, 0) random(999999999, 999999999),
    random(1000000000, 1000000000)
numeric digits 3
say random('-123456', '-123456') random(123456, 123456)
numeric digits 1
say random('-55', '-55') random(55, 55)
```

```output
0 1 1 1 1 1 1
-4 0 999999999 1000000000
-123456 123456
-55 55
```

> **Note.** A negative number written as a literal is an expression, whose
> prefix minus rounds the number to NUMERIC DIGITS before RANDOM is called: at
> `numeric digits 3`, the argument `-123456` is `-1.23E+5`, and RANDOM
> answers -123000 for it. The example passes the bounds as quoted strings,
> which the call receives as they are.

r[bif.random.range-limit]
The range from `min` to `max` can span at most 999999999: a call with `max`
more than 999999999 above `min` raises `SYNTAX 40.32`. A call with `max`
below `min` raises `SYNTAX 40.33`, and so does a call with one argument
that is negative, since that argument is `max`. The range checks come after
the [whole-number checks](r:conv.whole.form) and after the
[check of the seed](r:bif.random.seed.check).

```rexx
r = random(0, 999999999)
say r >= 0 & r <= 999999999
r = random(-999999999, 0)
say r >= -999999999 & r <= 0
signal on syntax name a
say random(1000000000)
a: say condition('o')~code
signal on syntax name b
say random(-1, 999999999)
b: say condition('o')~code
signal on syntax name c
say random(5, 2)
c: say condition('o')~code
signal on syntax name d
say random(-1)
d: say condition('o')~code
signal on syntax name e
say random(-1000000000, 0)
e: say condition('o')~code
```

```output
1
1
40.32
40.32
40.33
40.33
40.32
```

## Seeds

r[bif.random.seed]
A call with a `seed` starts a new sequence from it and answers the first
number of that sequence, taken from the range of the call. A call without a
`seed` answers the next number of the current sequence. The n-th number of a
sequence depends on the seed, on n and on the range of the n-th call, and on
nothing else: the ranges of the calls before it play no part. A program that
seeds the sequence therefore prints the same numbers on every run, and a
second call with the same seed starts the sequence again.

```rexx
say random(,, 5) random() random() random() random()
say random(,, 5) random() random() random() random()
say random(,, 0) random() random() random()
say random(,, 999999999) random() random()
say random(11, 14, 7) random(11, 14) random(11, 14)
say random(-7, 4, 7) random(-7, 4) random(-7, 4)
say random(10, , 7) random(10) random(10)
say random(0, 1, 5) random() random()
say random(0, 999999999, 5) random() random()
```

```output
458 880 443 106 246
458 880 443 106 246
579 728 246 192
365 926 53
14 12 14
0 -6 0
695 10 0
0 880 443
807179458 880 443
```

r[bif.random.seed.check]
A `seed` must not be negative: a negative seed raises `SYNTAX 40.13`. This
check comes after the checks of the form of all three arguments, and before
those of the range of the numbers. A seed of 0 is a seed like any other, and
`-0` is 0. A seed is any [whole-number argument](r:conv.whole.digits)
that is not negative, so it is below 10\*\*18, and the sequence is the same
for it on every run. A seed of 10\*\*18 or more raises `SYNTAX 40.12`.

```rexx
say random(,, 0) random(,, -0) random(,, '+0')
say random(,, 1000000000) random() random()
say random(,, 999999999999999999) random() random()
signal on syntax name a
say random(,, -1)
a: say condition('o')~code
signal on syntax name b
say random(5, 2, -1)
b: say condition('o')~code
signal on syntax name c
say random(-1000000000, 0, -1)
c: say condition('o')~code
signal on syntax name d
say random('x', 2, -1)
d: say condition('o')~code
signal on syntax name e
say random(5, 2, 'x')
e: say condition('o')~code
signal on syntax name f
say random(,, 1000000000000000000)
f: say condition('o')~code
```

```output
579 579 579
608 162 861
768 891 817
40.13
40.13
40.13
40.12
40.12
40.12
```

> **Note.** The reference manual gives a smaller upper limit for a seed. This
> book follows the engines, which accept every seed that is a whole-number
> argument and give the same sequence for it.

r[bif.random.scope]
A sequence belongs to the code that runs it. A program, its internal routines
(the ones that a label starts), whether called with CALL or as
functions, and the code it runs with INTERPRET all draw from one sequence: a
seed given in any of them applies to all of them. Each run of a method, each
run of a routine declared with `::ROUTINE`, and each message sent with START
has a sequence of its own that begins unseeded. Its draws and its seeds
leave the caller's sequence as it was.

```rexx
x = random(,, 5)
say random() inside() proc()
say 'interpret:' random()
interpret 'say random()'
junk = .o~new~draw + draw()
junk = .o~new~start('draw')~result
say 'after draws elsewhere:' random()
say 'a method that seeds:' .o~new~seeded
say 'a routine that seeds:' routine_seeded()
say 'main continues:' random()
exit
inside: return random()
proc: procedure
return random()
::class o
::method seeded
x = random(,, 5)
return x random() random()
::method draw
return random()
::routine routine_seeded
x = random(,, 5)
return x random() random()
::routine draw
return random()
```

```output
880 443 106
interpret: 246
452
after draws elsewhere: 954
a method that seeds: 458 880 443
a routine that seeds: 458 880 443
main continues: 670
```

r[bif.random.scope.reply]
A method that continues after a REPLY instruction goes on with the sequence
that the method had when it replied: a seed it gave before the REPLY
applies to the numbers it draws after it. The sequence of the caller, which
carries on at the same time, is not touched by the continuation.

```rexx
x = random(,, 5)
say random()
o = .o~new
say o~go
say o~rest
say random()
::class o
::attribute done
::method go
expose done
x = random(,, 5)
reply random()
done = random() random()
::method rest
expose done
return done
```

```output
880
880
443 106
443
```
