/* Arithmetic on operands of hundreds of digits, at DIGITS settings from 9
   to 1000: every operator on pseudo-random operands, including ones longer
   than DIGITS (truncated, not rounded), negative operands of % and //,
   powers with large and negative exponents, FUZZ comparisons, FORMAT, TRUNC
   and a long decimal string converted to a number and back.

   The operands come from a fixed-seed generator, so the output is the same
   on every run. Arguments raise the number of cases per setting and change
   the seed, for a broader run by hand; the corpus runs it without them. */
parse arg rounds seed
if rounds = '' then rounds = 3
if seed = '' then seed = 20261112

settings = '9 10 18 19 20 21 27 40 64 100 250 1000'
do s = 1 to words(settings)
  d = word(settings, s)
  numeric digits d
  say '== DIGITS' d
  do r = 1 to rounds
    a = operand(d)
    b = operand(d)
    say 'a' a
    say 'b' b
    ops = '+ - * / % //'
    do o = 1 to words(ops)
      op = word(ops, o)
      say op apply(op, a, b) apply(op, b, a) apply(op, a, -b) apply(op, -a, b)
    end
    /* An exact product divided back, and a remainder of a multiple. */
    p = apply('*', a, b)
    say 'p/b' apply('/', p, b) apply('%', p, b) apply('//', p, b)
    say 'p+r//b' apply('//', apply('+', p, 7), b) apply('//', apply('-', 0, p), b)
    /* Divisors of powers of 2 and 5, whose quotients come out even some
       way past the dividend's own digits, word-sized and wider. */
    say 'even' apply('/', a, 1024) apply('/', a, 78125) apply('/', a, 95367431640625) ,
      apply('/', a, 1180591620717411303424) apply('/', a, '0.0128') apply('//', a, '0.0128')
    /* Powers: whole, large and negative exponents. */
    k = random1(2000)
    say '**' apply('**', a, 2) apply('**', 1.0001, k) apply('**', a, -1) ,
      apply('**', '-1.000000001', k) apply('**', 2, k)
    /* Comparisons at FUZZ, on values that differ in a low digit. */
    c = apply('+', a, '1E' || (length(strip(a, 'L', '-')) - d - 2))
    do f = 0 to 2
      numeric fuzz f
      say 'fuzz' f (a = c) (a < c) (a >= c) (a == c) (a \= b)
    end
    numeric fuzz 0
    say 'format' apply('f', a) apply('f53', a) apply('f003', a) apply('f0200', b)
    say 'trunc' apply('t', a) apply('t3', b) apply('t7', apply('/', a, 3))
    say 'abs/sign' abs(b) sign(b) max(a, b) min(a, b)
  end
  /* A long decimal string converted to a number and back. */
  long = '1' || copies('0123456789', 103) || '.5'
  say 'conv' (long + 0) (long * 1) (-long) (long - long) length(long + 0)
  /* A product whose digit columns sum past 65535 before carrying. */
  nines = copies(9, d + 1)
  square = nines * nines
  say 'nines' square apply('//', square, 7) apply('/', square, nines)
  /* At the edges of the exponent range, where a rounding carry decides. */
  nines = '9.' || copies(9, d)
  say 'range' apply('*', '1E+999999999', 10) apply('/', '1E-999999999', 10) ,
    apply('*', nines || 'E+999999999', 1) apply('*', nines || 'E+999999998', 1) ,
    apply('%', nines || 'E+999999999', '0.3') apply('//', nines || 'E+999999999', 7) ,
    apply('**', '1' || copies(0, d % 2) || '1', 999999) apply('**', 10, -999999999)
end
exit

/* One operation, or the error it raises as E<code>. */
apply: procedure
  parse arg op, x, y
  signal on syntax name failed
  select
    when op = '+' then return x + y
    when op = '-' then return x - y
    when op = '*' then return x * y
    when op = '/' then return x / y
    when op = '%' then return x % y
    when op = '//' then return x // y
    when op = '**' then return x ** y
    when op = 'f' then return format(x)
    when op = 'f53' then return format(x, 5, 3)
    when op = 'f003' then return format(x, , , 3)
    when op = 'f0200' then return format(x, , 2, , 0)
    when op = 't' then return trunc(x)
    when op = 't3' then return trunc(x, 3)
    when op = 't7' then return trunc(x, 7)
  end
failed:
  return 'E' || condition('O')~code

/* A pseudo-random operand of up to twice the DIGITS setting: sign, point
   and exponent each sometimes, runs of 9s and 0s sometimes. */
operand: procedure expose seed
  parse arg d
  len = 1 + random1(2 * d + 3)
  style = random1(4)
  text = ''
  do while length(text) < len
    select
      when style = 1 then text = text || '999999999'
      when style = 2 then text = text || '100000000'
      otherwise text = text || right(random1(1000000000) - 1, 9, '0')
    end
  end
  text = strip(left(text, len), 'L', '0')
  if text = '' then text = '7'
  if random1(3) = 1 then do
    point = random1(length(text))
    text = left(text, point) || '.' || substr(text, point + 1) || '0'
  end
  if random1(4) = 1 then text = text || 'E' || (random1(41) - 21)
  if random1(2) = 1 then text = '-' || text
  return text

/* 1..n from a linear congruential generator, computed at its own precision. */
random1: procedure expose seed
  parse arg n
  numeric digits 30
  seed = (seed * 1103515245 + 12345) // 2147483648
  return 1 + (seed * n) % 2147483648
