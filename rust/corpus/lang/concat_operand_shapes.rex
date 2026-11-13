/* Concatenation operands of every shape a value comes in: a string held in
   the handle, a longer string, a whole number, a number with a fraction or
   an exponent, and an object. Each join is printed with its length, around
   the sizes where a result stops fitting the handle and stops fitting its
   object. */
short = 'abc'; seven = 'abcdefg'; eight = 'abcdefgh'
mid = copies('m', 40); long = copies('L', 60)
say '['short || seven']' length(short || seven)
say '['short seven']' length(short seven)
null = ''
say '['seven || null']' length(seven || null)
say '['null || seven']' length(null || seven)
say '['seven null']' length(seven null)
say '['short || 'defg' || ']' length(short || 'defg')
say '['short 'def' || ']' length(short 'def')
say '['seven || eight']' length(seven || eight)
/* the joins on either side of the object's own width */
do n = 50 to 56
  j = copies('x', n - 4) || 'abcd'
  b = copies('y', n - 5) 'abcd'
  say n length(j) length(b) right(j, 6) right(b, 6)
end
say length(mid || mid) length(mid mid) length(long || short) length(short long)
/* whole numbers, as each side and both */
i = 7; neg = -12345; big = 999999999; zero = 0
say i || i; say i neg; say neg || big; say zero || ''; say '' zero
say 'n' || neg || 'n'; say big || big || big; say i || short || i
k = 3 * 4; say k || k; say k mid || k
/* numbers that are not whole, and arithmetic results */
f = 1.5; e = 1E3; d = 2 / 3; p = 0.1 + 0.2
say f || f; say e e; say d || '|'; say p || p; say f mid
say 1.0 || 'x'; say 1.0'' || 2; say '' || 1.50; say 12.30 short
/* a chain whose later operands are of each shape */
c = short || seven || i || neg || f || mid || ' ' || long
say c length(c)
c = short seven i neg f
say c length(c)
c = 'a' || i
c = c || i || i || i || i || i || i || i
say c length(c)
c = seven || 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23
say c length(c)
c = mid || neg neg neg neg
say c length(c)
/* objects, read through their string values */
a = .array~of(1, 2)
say left(a || 'x', 8) left('x' a, 8)
st. = 'stem'
say 'x' || st. st. || 'y'
q = .strobj~new
say 'pre-' || q || '-post' q 'x'
exit

::class strobj
::method string
  return 'from-string'
