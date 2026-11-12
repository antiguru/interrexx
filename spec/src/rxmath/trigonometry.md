---
element: "RxCalcSin"
kind: chapter
syntax: "RxCalcSin(angle [, precision [, units]]); RxCalcCos(angle [, precision [, units]]); RxCalcTan(angle [, precision [, units]]); RxCalcCotan(angle [, precision [, units]]); RxCalcArcSin(x [, precision [, units]]); RxCalcArcCos(x [, precision [, units]]); RxCalcArcTan(x [, precision [, units]])"
rexxref: "Rexx Mathematical Functions Library (rxmath) > Functions"
---

# The trigonometric functions

`RxCalcSin`, `RxCalcCos`, `RxCalcTan` and `RxCalcCotan` answer a function of an
angle, and `RxCalcArcSin`, `RxCalcArcCos` and `RxCalcArcTan` answer an angle.
An angle is in degrees unless its function is asked for another unit. What
every function of [the rxmath library](index.md) shares, its availability, its
arguments and the form of its results, holds here too.

## Units

r[rxmath.trigonometry.args]
Each of the seven functions takes a number as its first argument, an optional
[precision](r:rxmath.index.precision) as its second and an optional unit as its
third, and no more: a fourth argument raises `SYNTAX 88.922`. The unit is a
string of which only the first character counts, in either case: `D` for
degrees, which is the default, `R` for radians and `G` for grades, a circle
being 360 degrees, 2 pi radians and 400 grades. Any other first character
raises `SYNTAX 88.916`, and so does the null string. The first argument is read
first, then the precision, then the unit, and the first of them that is wrong
raises its error.

```rexx
say RxCalcSin(30) RxCalcSin(30, , 'D') RxCalcSin(30, , 'd') RxCalcSin(30, , 'Degrees')
say RxCalcSin(0.5, , 'R') RxCalcSin(0.5, , 'radians') RxCalcSin(0.5, , 'Grad')
say RxCalcSin(33.3333333333333, , 'G') RxCalcSin(100, , 'g')
call try "RxCalcSin(30, , 'x')"
call try "RxCalcSin(30, , '')"
call try "RxCalcSin(30, , ' D')"
call try "RxCalcTan(30, 5, 'q')"
call try "RxCalcCotan(30, , 1)"
call try "RxCalcArcTan(1, , 'z')"
call try "RxCalcArcSin(1, 3, 'z')"
call try "RxCalcSin('x', 0, 'q')"
call try "RxCalcSin(1, 0, 'q')"
call try "RxCalcSin(1, 2, 'D', 3)"
call try "RxCalcArcCos(1, 2, 'D', 3)"
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
0.5 0.5 0.5 0.5
0.479425539 0.479425539 0.00785390089
0.5 1
88.916
88.916
88.916
88.916
88.916
88.916
88.916
88.921
88.905
88.922
88.922
```

## Angles

r[rxmath.trigonometry.angle]
`RxCalcSin(angle [, precision [, units]])`, `RxCalcCos`, `RxCalcTan` and
`RxCalcCotan` answer the sine, cosine, tangent and cotangent of `angle`, the
cotangent being the reciprocal of the tangent. An angle in radians is used as it
is, so that the sine of `180` radians is not `0`. An angle in degrees or grades
is first reduced to the circle, and a multiple of a quarter circle, `90` degrees
or `100` grades, answers exactly: `0`, `1` or `-1` for a sine or a cosine, and
for a tangent or a cotangent `0`, `1` or `-1` or an infinity where the function
is not defined at the angle. The sign of that infinity is the one the example
shows. A function of any other angle is that of its angle in radians, to the
precision.

```rexx
say RxCalcSin(0) RxCalcSin(90) RxCalcSin(180) RxCalcSin(270) RxCalcSin(360) RxCalcSin(450)
say RxCalcSin(-90) RxCalcSin(-180) RxCalcSin(720) RxCalcSin(30) RxCalcSin(60)
say RxCalcCos(0) RxCalcCos(90) RxCalcCos(180) RxCalcCos(270) RxCalcCos(360) RxCalcCos(-90)
say RxCalcCos(-180) RxCalcCos(540) RxCalcCos(30) RxCalcCos(60)
say RxCalcTan(0) RxCalcTan(45) RxCalcTan(90) RxCalcTan(135) RxCalcTan(180) RxCalcTan(270)
say RxCalcTan(-45) RxCalcTan(-90) RxCalcTan(360) RxCalcTan(30) RxCalcTan(60)
say RxCalcCotan(0) RxCalcCotan(45) RxCalcCotan(90) RxCalcCotan(180) RxCalcCotan(270)
say RxCalcCotan(-45) RxCalcCotan(-90) RxCalcCotan(30) RxCalcCotan(60)
say RxCalcSin(0, , 'G') RxCalcSin(100, , 'G') RxCalcSin(200, , 'G') RxCalcSin(300, , 'G') RxCalcSin(400, , 'G')
say RxCalcCos(100, , 'G') RxCalcCos(200, , 'G') RxCalcTan(50, , 'G') RxCalcTan(100, , 'G')
say RxCalcSin(180, , 'R') RxCalcCos(0, , 'R') RxCalcCos(180, , 'R') RxCalcSin(1, 5, 'R')
say RxCalcSin('+infinity') RxCalcCos('nan') RxCalcTan('-infinity', , 'R')
::requires 'rxmath' library
```

```output
0 1 0 -1 0 1
-1 0 0 0.5 0.866025404
1 0 -1 0 1 0
-1 -1 0.866025404 0.5
0 1 +infinity -1 0 -infinity
-1 -infinity 0 0.577350269 1.73205081
+infinity 1 0 -infinity 0
-1 0 1.73205081 0.577350269
0 1 0 -1 0
0 -1 1 +infinity
-0.801152636 1 -0.598460069 0.84147
nan nan nan
```

## Inverse functions

r[rxmath.trigonometry.inverse]
`RxCalcArcSin(x [, precision [, units]])`, `RxCalcArcCos` and `RxCalcArcTan`
answer the angle whose sine, cosine or tangent is `x`, in the unit that the
third argument names, degrees where it is omitted. The angle is that of the
principal value: from `-90` to `90` degrees for the arc sine and the arc
tangent, and from `0` to `180` degrees for the arc cosine, and the same angles
in the other units. The arc sine and the arc cosine of a number outside `-1` to
`1` are `nan`, and the arc tangent of an infinity is a quarter circle with the
sign of the infinity.

```rexx
say RxCalcArcSin(0) RxCalcArcSin(1) RxCalcArcSin(-1) RxCalcArcSin(0.5) RxCalcArcSin(1.5) RxCalcArcSin('nan')
say RxCalcArcSin(1, , 'R') RxCalcArcSin(-1, , 'r') RxCalcArcSin(1, , 'G') RxCalcArcSin(0.5, , 'g')
say RxCalcArcCos(1) RxCalcArcCos(0) RxCalcArcCos(-1) RxCalcArcCos(0.5) RxCalcArcCos(2)
say RxCalcArcCos(0, , 'R') RxCalcArcCos(-1, , 'R') RxCalcArcCos(0, , 'G') RxCalcArcCos(-1, , 'G')
say RxCalcArcTan(0) RxCalcArcTan(1) RxCalcArcTan(-1) RxCalcArcTan('+infinity') RxCalcArcTan('-infinity')
say RxCalcArcTan(1, , 'R') RxCalcArcTan(1, , 'G') RxCalcArcTan(-1, , 'G') RxCalcArcTan('+infinity', , 'R')
say RxCalcArcTan(1000000) RxCalcArcTan(0.5, 3) RxCalcArcTan(0.5, 16) RxCalcArcSin(0.5, 16, 'R')
::requires 'rxmath' library
```

```output
0 90 -90 30 nan nan
1.57079633 -1.57079633 100 33.3333333
0 90 180 60 nan
1.57079633 3.14159265 100 200
0 45 -45 90 -90
0.785398163 50 -50 1.57079633
89.9999427 26.6 26.56505117707799 0.5235987755982989
```
