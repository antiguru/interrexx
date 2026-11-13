quiet_ = .output~destination(.stream~new('/dev/null'))
parse value 'abcdefghij' with u 03 v 4.0 w 1e1 y
say u'|'v'|'w'|'y
n = ' 3 '
parse value 'abcdefghij' with u =(n) v
say u'|'v
parse value 'abcdefghij' with u + 3 v - (n) w
say u'|'v'|'w
call try -2, 'abcdefghij'
numeric digits 3
call lit 'u 999 v'
call lit 'u 1000 v'
call lit 'u 0999 v'
call lit 'u 0.1e4 v'
call lit 'u +3a v'
call lit 'u 1e3 v'
call lit 'u +3.5 v'
call lit 'u >1000 v'
numeric digits 12
call lit 'u 1e9 v'
exit
try:
  signal on syntax name e
  p = arg(1)
  parse value arg(2) with u =(p) v
  say p 'ok' u'|'v
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
lit:
  signal on syntax name e2
  interpret 'parse value "abcdefghij" with' arg(1)
  say arg(1) 'ok'
  return
e2: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
