---
element: "rxmath"
kind: chapter
rexxref: "Rexx Mathematical Functions Library (rxmath) > Functions"
---

# The rxmath library

The library `rxmath` provides the mathematical functions of a program that works
on binary floating-point numbers: square root, exponential, logarithm, power,
the hyperbolic functions, [the trigonometric functions](trigonometry.md) and
their inverses. This page holds what every function of the library shares: how
the functions are made available, how their arguments are read, and how a result
is written. [The trigonometric page](trigonometry.md) holds the functions of an
angle.

Every function reads its number arguments as 64-bit binary floating-point
numbers, makes one call of the function of that name on them, and writes the
answer as a number with a given count of significant digits. A result is as
exact as a 64-bit binary floating-point number is, and no more.

The examples call the functions after one of them has been made available, as
[the availability rule](r:rxmath.index.available) says.

## Availability

r[rxmath.index.available]
The functions of the library are not provided from the start of a program. They
are made available by [RXFUNCADD](r:bif.rxfuncadd.library) of the name of any
one of them with the library name `rxmath`, or by a
[`::requires`](r:dir.requires.library) directive with the `LIBRARY` keyword,
and from then on every function of the library can be called, as `name(arguments)`
or by CALL, with its name in any case. A name is not a method of `String`:
sending it to a string as a message raises `SYNTAX 97.1`. A call of a function
that the library has not made available raises `SYNTAX 43.1`. The functions
are `MathLoadFuncs`, `MathDropFuncs`, `RxCalcPi`, `RxCalcSqrt`, `RxCalcExp`,
`RxCalcLog`, `RxCalcLog10`, `RxCalcSinH`, `RxCalcCosH`, `RxCalcTanH`,
`RxCalcPower`, `RxCalcSin`, `RxCalcCos`, `RxCalcTan`, `RxCalcCotan`,
`RxCalcArcSin`, `RxCalcArcCos` and `RxCalcArcTan`. RXFUNCDROP of one of them
removes nothing: the library stays available until the program ends.

```rexx
say rxfuncquery('RxCalcSqrt') rxfuncquery('RxCalcPi')
say rxfuncadd('RxCalcPi', 'rxmath')
say rxfuncquery('RxCalcSqrt') rxfuncquery('RxCalcPi') rxfuncquery('MathLoadFuncs')
say RxCalcSqrt(25) RXCALCSQRT(25) 'rxcalcsqrt'(25)
call RxCalcSqrt 16
say result
say rxfuncdrop('RxCalcPi') rxfuncquery('RxCalcPi')
signal on syntax name a
say 'x'~RxCalcSqrt
a: say condition('o')~code
```

```output
1 1
0
0 0 0
5 5 5
4
1 0
97.1
```

```rexx
say RxCalcSqrt(81)
::requires 'rxmath' library
```

```output
9
```

```rexx
signal on syntax name a
say RxCalcFoo(1)
a: say condition('o')~code
::requires 'rxmath' library
```

```output
43.1
```

## Loading and dropping

r[rxmath.index.load]
`MathLoadFuncs` and `MathDropFuncs` exist for programs that were written for a
system where the library had to be loaded and dropped. Each answers the null
string and changes nothing: the functions are available without a call of
either, and after any number of calls. `MathLoadFuncs` called with an argument,
of any value, writes a banner on standard output before it answers. The banner
names the library and is not specified ([Deviation 96](../deviations.md#96-version-and-library-banners)),
and `MathLoadFuncs` called without one writes nothing. `MathLoadFuncs` takes at
most one argument and `MathDropFuncs` none: more raise `SYNTAX 88.922`.

```rexx
call lineout 'c1zqm1.rex', "say '['MathLoadFuncs('x')']'"
call lineout 'c1zqm1.rex', "say '['MathLoadFuncs()']'"
call lineout 'c1zqm1.rex', "say '['MathDropFuncs()']'"
call lineout 'c1zqm1.rex', "::requires 'rxmath' library"
call lineout 'c1zqm1.rex'
address system 'rexx c1zqm1.rex' with output stem out.
say pos('rxmath', out.1) > 0
do i = out.0 - 1 to out.0
  say out.i
end
call sysfiledelete 'c1zqm1.rex'
```

```output
1
[]
[]
```

```rexx
call try "MathLoadFuncs(1, 2)"
call try "MathDropFuncs(1)"
call try "MathDropFuncs(,)"
exit
try:
  signal on syntax name e
  interpret 'say length(' arg(1) ')'
  return
e: say condition('o')~code
  return
::requires 'rxmath' library
```

```output
88.922
88.922
0
```

## Arguments

r[rxmath.index.args]
The functions follow [the native call rule](r:bif.index.call.native), as the
[RexxUtil](../rexxutil/index.md) functions do. A required argument that is not
given, or is omitted, raises `SYNTAX 88.901`. An argument past the last one of
the function raises `SYNTAX 88.922`, and the count is checked before the
arguments are read. An omitted place at the end of the call adds nothing to
the count, so `RxCalcSqrt(4,,)` is a call with one argument. An argument that is
an object other than a string is replaced by the string that its `request`
with the argument `STRING` answers, and an object that has none raises `SYNTAX
88.909`.

```rexx
call try "RxCalcSqrt()"
call try "RxCalcSqrt(,)"
call try "RxCalcSqrt(, 5)"
call try "RxCalcSqrt(4, 2, 3)"
call try "RxCalcSqrt(4,,)"
call try "RxCalcSqrt(4, , 3)"
call try "RxCalcPower(2)"
call try "RxCalcPower(, 3)"
call try "RxCalcPower(2, 3, 4, 5)"
call try "RxCalcPi(1, 2)"
call try "RxCalcSin()"
call try "RxCalcSin(1, 2, 'D', 3)"
call try "RxCalcSqrt(.mutablebuffer~new('49'))"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::requires 'rxmath' library
```

```output
88.901
88.901
88.901
88.922
2
88.922
88.901
88.901
88.922
88.922
88.901
88.922
7
```

r[rxmath.index.number]
An argument that the page of a function calls a number is read as a 64-bit
binary floating-point number. A [Rexx number](r:num.formats.operand) is read at the
precision of that format whatever `NUMERIC DIGITS` is in force: blanks before
and after it, a sign and an exponent are allowed, and a number too large for the
format is the infinity of its sign. Besides numbers, the three strings `nan`,
`+infinity` and `-infinity`, in lower case and exactly so, are read as the
values they name. Any other string, the null string included, raises `SYNTAX
88.921`, and so does an object whose string value is none of these.

```rexx
call try "RxCalcSqrt(' 4 ')"
call try "RxCalcSqrt('+16')"
call try "RxCalcSqrt('4e2')"
call try "RxCalcSqrt(1e400)"
call try "RxCalcSqrt('nan')"
call try "RxCalcSqrt('+infinity')"
call try "RxCalcSqrt('-infinity')"
call try "RxCalcSqrt('x')"
call try "RxCalcSqrt('')"
call try "RxCalcSqrt('infinity')"
call try "RxCalcSqrt('NaN')"
call try "RxCalcSqrt('Infinity')"
call try "RxCalcSqrt('0x10')"
call try "RxCalcSqrt(.nil)"
call try "RxCalcPower(2, 'x')"
call try "RxCalcPower('x', 2)"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::requires 'rxmath' library
```

```output
2
4
20
+infinity
nan
+infinity
nan
88.921
88.921
88.921
88.921
88.921
88.921
88.921
88.921
88.921
```

## Precision

r[rxmath.index.precision]
The last numeric argument of most functions is the precision: the number of
significant digits of the answer. It is optional. An argument given is a whole
number that is above `0`, and written as any [number](r:num.formats.operand) that has a
whole value is allowed, as `5.0`, `' 5 '` and `5E0` are; anything else raises
`SYNTAX 88.905`, which a number that has a fraction, a number below `1` and a
string that is no number all do. A precision above `16` is `16`: no answer has
more digits than a 64-bit binary floating-point number has. An omitted
precision is the value of `NUMERIC DIGITS` of the program that calls the
function, again at most `16`. The precision does not change what is calculated,
only how many digits of the result are kept, rounded.

```rexx
say RxCalcSqrt(2) RxCalcSqrt(2, 1) RxCalcSqrt(2, 5) RxCalcSqrt(2, 16) RxCalcSqrt(2, 17) RxCalcSqrt(2, 1000)
say RxCalcSqrt(2, 5.0) RxCalcSqrt(2, ' 5 ') RxCalcSqrt(2, '5E0') RxCalcSqrt(2, '+5')
numeric digits 3
say RxCalcSqrt(2) RxCalcSqrt(2, 5) RxCalcPi()
numeric digits 50
say RxCalcSqrt(2) RxCalcPi()
numeric digits 9
call try "RxCalcSqrt(4, 0)"
call try "RxCalcSqrt(4, -1)"
call try "RxCalcSqrt(4, 1.5)"
call try "RxCalcSqrt(4, 'x')"
call try "RxCalcSqrt(4, '')"
call try "RxCalcPi(0)"
call try "RxCalcPi('x')"
call try "RxCalcPower(2, 3, 0)"
call try "RxCalcSin(30, 0)"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::requires 'rxmath' library
```

```output
1.41421356 1 1.4142 1.414213562373095 1.414213562373095 1.414213562373095
1.4142 1.4142 1.4142 1.4142
1.41 1.4142 3.14
1.414213562373095 3.141592653589793
88.905
88.905
88.905
88.905
88.905
88.905
88.905
88.905
88.905
```

## Results

r[rxmath.index.result]
The answer of a function is the result of its calculation, rounded to the
precision and written as [the result of arithmetic](r:num.formats.result) is
written at a `NUMERIC DIGITS` of that precision, so that it takes the form
with an exponent where those rules do. A result that no number has is one of the three
strings `nan`, `+infinity` and `-infinity`. The domain of a function is that of
the function of the same name for 64-bit binary floating-point numbers: a
function that is not defined for its argument answers `nan`, and an overflow
answers an infinity. The answer is a string, not a number that carries a
precision of its own, and a program reads it as it reads any string of digits.

```rexx
say RxCalcSqrt(2) RxCalcExp(1) RxCalcExp(1, 3) RxCalcExp(0)
say RxCalcExp(100) RxCalcExp(100, 16) RxCalcExp(-100)
say RxCalcPower(10, 15) RxCalcPower(10, 16) RxCalcPower(10, -5) RxCalcPower(10, -4)
say RxCalcSqrt(-4) RxCalcLog(-1) RxCalcLog(0) RxCalcExp(1000) RxCalcExp(-1000)
say RxCalcSqrt('nan') RxCalcExp('+infinity') RxCalcExp('-infinity') RxCalcLog('+infinity')
say datatype(RxCalcSqrt(2), 'N') datatype(RxCalcSqrt(-1), 'N') datatype(RxCalcExp(1000), 'N')
::requires 'rxmath' library
```

```output
1.41421356 2.71828183 2.72 1
2.68811714E+43 2.688117141816136E+43 3.72007598E-44
1E+15 1E+16 0.00001 0.0001
nan nan -infinity +infinity 0
nan +infinity 0 +infinity
1 0 0
```

## RxCalcPi

r[rxmath.index.pi]
`RxCalcPi([precision])` answers the 64-bit binary floating-point number nearest
to the ratio of the circumference of a circle to its diameter, at the
precision, `RxCalcPi(15)` is `3.14159265358979` and `RxCalcPi(16)` is
`3.141592653589793`. It takes at most one argument, the precision.

```rexx
say RxCalcPi() RxCalcPi(1) RxCalcPi(3) RxCalcPi(15) RxCalcPi(16) RxCalcPi(40)
::requires 'rxmath' library
```

```output
3.14159265 3 3.14 3.14159265358979 3.141592653589793 3.141592653589793
```

## Square root, exponential, logarithms and power

r[rxmath.index.unary]
`RxCalcSqrt(x [, precision])` answers the square root of `x`, `RxCalcExp(x [,
precision])` answers `e` raised to the power `x`, `RxCalcLog(x [, precision])`
answers the natural logarithm of `x`, and `RxCalcLog10(x [, precision])` the
logarithm of `x` to the base 10. The square root of a negative number is `nan`:
the answer is not the square root of its absolute value. The logarithm of `0`
is `-infinity` and the logarithm of a negative number is `nan`.

```rexx
say RxCalcSqrt(16) RxCalcSqrt(2) RxCalcSqrt(0) RxCalcSqrt(0.0001) RxCalcSqrt(-4)
say RxCalcExp(0) RxCalcExp(1) RxCalcExp(-1) RxCalcExp(2, 4)
say RxCalcLog(1) RxCalcLog(10) RxCalcLog(0.5) RxCalcLog(0) RxCalcLog(-1)
say RxCalcLog10(1) RxCalcLog10(1000) RxCalcLog10(2) RxCalcLog10(0.001) RxCalcLog10(0)
::requires 'rxmath' library
```

```output
4 1.41421356 0 0.01 nan
1 2.71828183 0.367879441 7.389
0 2.30258509 -0.693147181 -infinity nan
0 3 0.301029996 -3 -infinity
```

r[rxmath.index.power]
`RxCalcPower(x, y [, precision])` answers `x` raised to the power `y`, where
both are numbers and `y` need not be whole. `0` to the power `0` is `1`. A
negative `x` with a `y` that is not whole is `nan`, and `0` to a negative power
is `+infinity`. It takes at most three arguments and requires two.

```rexx
say RxCalcPower(2, 10) RxCalcPower(2, 0.5) RxCalcPower(2, -1) RxCalcPower(10, 2, 1)
say RxCalcPower(0, 0) RxCalcPower(0, 2) RxCalcPower(0, -1) RxCalcPower(-8, 1/3)
say RxCalcPower(-2, 3) RxCalcPower(-2, 2) RxCalcPower(1.5, 2) RxCalcPower(4, 0.5)
::requires 'rxmath' library
```

```output
1024 1.41421356 0.5 1E+2
1 0 +infinity nan
-8 4 2.25 2
```

## The hyperbolic functions

r[rxmath.index.hyperbolic]
`RxCalcSinH(x [, precision])`, `RxCalcCosH(x [, precision])` and
`RxCalcTanH(x [, precision])` answer the hyperbolic sine, cosine and tangent of
`x`. The sine and cosine of a large argument overflow to an infinity of the
sign of the result, and the tangent of a large argument is `1` or `-1`.

```rexx
say RxCalcSinH(0) RxCalcSinH(1) RxCalcSinH(-1) RxCalcSinH(1000) RxCalcSinH(-1000)
say RxCalcCosH(0) RxCalcCosH(1) RxCalcCosH(-1) RxCalcCosH(1000) RxCalcCosH(-1000)
say RxCalcTanH(0) RxCalcTanH(1) RxCalcTanH(-1) RxCalcTanH(1000) RxCalcTanH(-1000)
::requires 'rxmath' library
```

```output
0 1.17520119 -1.17520119 +infinity -infinity
1 1.54308063 1.54308063 +infinity +infinity
0 0.761594156 -0.761594156 1 -1
```
