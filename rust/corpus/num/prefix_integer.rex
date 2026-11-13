/* Prefix + and - on integers at the edges of DIGITS and of the range an
   integer can be held in without a decimal: +a is 0 + a and -a is 0 - a,
   rounded to DIGITS under the current FORM, for an operand spelled plainly,
   held as an integer, or spelled with blanks, a sign, an exponent or
   leading zeros. */

vals = '0 1 -1 7 12345 -12345 999 1000 -999 -1000 99999 999999999 1000000000 -999999999 -1000000000 99999999999 999999999999999999 1000000000000000000 -999999999999999999 4611686018427387903 4611686018427387904 -4611686018427387904 -4611686018427387905 9223372036854775807'
digs = '1 2 3 4 9 10 18 19 20'
do j = 1 to words(digs)
  d = word(digs, j)
  out = 'd='d':'
  do i = 1 to words(vals)
    out = out pre(word(vals, i), d, 'SCIENTIFIC')
  end
  say out
end
out = 'eng:'
do i = 1 to words(vals)
  out = out pre(word(vals, i), 3, 'ENGINEERING')
end
say out
specials = "' 12 '|'+5'|'1e2'|'-0'|'0'|'00'|'012'|' -3'|'1.0'|'-0.0'|'.5'|'5.'|'0e0'|'1E+2'|'-'"
do while specials \= ''
  parse var specials s '|' specials
  interpret 'v =' s
  say 'special [' || v || '] ->' pre(v, 9, 'SCIENTIFIC') pre(v, 2, 'SCIENTIFIC')
end
v = 12345; say +v -v +(-v) -(+v) --v
v = 'abc'
signal on syntax
say +v
syntax: say 'syntax' condition('o')~code
signal on syntax name s2
v = ''
say -v
s2: say 'syntax' condition('o')~code
numeric digits 5
signal on lostdigits
v = 123456
say +v
lostdigits: say 'lostdigits' condition('d')
exit
pre: procedure
  use arg v, d, f
  signal on syntax name bad
  numeric digits d
  numeric form value f
  w = v + 0 /* a tagged one where it fits */
  return '[' || (+v) || ',' || (-v) || ',' || (+w) || ',' || (-w) || ']'
bad: return '[err' condition('o')~code || ']'
