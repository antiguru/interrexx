/* Arithmetic at NUMERIC DIGITS 1000 on operands of tens to a thousand
   digits (issue #58): a modular multiply-reduce loop over 77-digit values,
   whose remainders have short quotients and a long DIGITS, then a series and
   a Newton iteration whose quotients fill all 1000 digits, and powers. */
numeric digits 1000
p = 2**255 - 19
x = 2**250 + 12345
y = 2**251 + 67890
n = 20000
do i = 1 to n
  z = (x*y)//p; x = (z+i)//p; y = (y-z)//p
end
say length(x) x

/* e by its series: each term a long quotient by a small divisor. */
do r = 1 to 5
  e = 1; t = 1
  do k = 1 until t < 1E-1000
    t = t / k
    e = e + t
  end
end
say k left(e, 50)

/* The square root of 2 by Newton: long quotients by long divisors. */
do r = 1 to 5
  s = 1.4
  do 12
    s = (s + 2 / s) / 2
  end
end
say left(s, 50) length(s)

/* Powers by repeated squaring at the full working precision. */
do r = 1 to 20
  w = 1.0001 ** (1000 + r) * 3 ** -(r + 600)
end
say format(w, 1, 40)
