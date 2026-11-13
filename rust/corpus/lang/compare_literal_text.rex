/* Equality against a literal that is not a number compares as strings, in
   expressions, in conditions and in a DO WHILE header, with every operand
   kind: inline and heap strings, numbers, .nil, objects, stems, references. */
long = 'a string longer than a handle carries inline'
say (' ok ' = 'ok') ('ok' = ' ok ') (' ok ' == 'ok') ('ok' \== ' ok')
say ('' = '   ') ('   ' = '') ('' == '   ') ('' \= ' ') ('' <> ' ') ('' >< ' ')
say (long = 'ok') (long \= 'ok') (long == long'') (long = ' 'long' ')
n = 5; say (n = 'ok') (n \= 'ok') (n = '') (n == 'x')
h = 12345678901234567890; say (h = 'ok') (h \= 'ok')
f = 1.50; say (f = '1.5x') (f \== 'x')
say (.nil = '') (.nil \= '') ('' = .nil) ('' \= .nil) ('' == .nil)
a = .array~new; say (a = '') (a \= '') ('' = a) ('' \== a)
s. = 'ok'; say (s. = 'ok') ('ok' = s.) (s. == ' ok')
t. = ''; say (t.x = '') (t.x == '')
o = .ok~new; say (o = 'ok') ('ok' = o) (o \= 'xy')
v = 'ok'; r = >v; say (r = 'ok') ('ok' == r)
x = ' abc '; i = 0
do while x \= ''
  parse var x w x; i = i + 1
end
say i
if 'yes' == 'yes' then say 'strict true'
if 'yes' = 'yes ' then say 'stripped true'
numeric digits 3
say (1234567 = 'ok') ('ok' \= 1.23456789)
signal on novalue
say (undefined = 'UNDEFINED')
say (zz = '')
exit
novalue: say 'NOVALUE' condition('D'); exit

::class ok
::method '='
  say 'ok = called with' arg(1)
  return 1
::method '\='
  return 0
::method makestring
  return 'ok'
