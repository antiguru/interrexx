/* Comparisons against the null string and a blank, on either side of every
   comparison operator, with operands made of blanks, tabs and the other
   whitespace and control bytes the string rule does not strip. Each line
   prints the results with the literal written out and through a variable,
   which reach the comparison by different routes. */
values = .array~of('', ' ', '09'x, '200920'x, 'a', ' a ', '09'x || 'a' || '09'x, ,
  '0a'x, '0d'x, '0b'x, '0c'x, '00'x, 'a0'x, '200a20'x, '0', ' 0 ', '1e0', 'abc ')
empty = ''
blank = ' '
do v over values
  x = v
  line = left(c2x(x), 8)
  line = line '|' (x = '') (x \= '') (x <> '') (x >< '') (x == '') (x \== '') ,
    (x > '') (x < '') (x >= '') (x <= '') (x \> '') (x \< '') ,
    (x >> '') (x << '') (x >>= '') (x <<= '') (x \>> '') (x \<< '')
  line = line '|' ('' = x) ('' \= x) ('' <> x) ('' >< x) ('' == x) ('' \== x) ,
    ('' > x) ('' < x) ('' >= x) ('' <= x) ('' >> x) ('' << x)
  line = line '|' (x = empty) (x \= empty) (x == empty) (x \== empty) (empty = x) (empty \== x)
  line = line '|' (x = ' ') (x \= ' ') (x == ' ') (x \== ' ') (x > ' ') (x < ' ') ,
    (' ' = x) (' ' \= x) (' ' == x) (' ' \== x) (' ' >> x) (' ' << x)
  line = line '|' (x = blank) (x \== blank) (blank = x) (blank \= x)
  say line
end

/* A null string that is not a literal: built, and taken from a longer one. */
built = substr('abc', 4)
cut = left('', 0)
say (built = '') (built == '') ('' = built) (cut \= '') (cut \== '') (built = cut)

/* Operands that are not plain strings keep their own dispatch. */
say (.nil = '') (.nil \= '') ('' = .nil) ('' \== .nil)
s = .shout~new('')
say (s = '') (s \= '') (s == '') ('' = s)
t = .shout~new(' ')
say (t = '') (t == '')
exit

::class shout subclass string
::method '='
  say 'shout' arg(1)~c2x
  return 0
