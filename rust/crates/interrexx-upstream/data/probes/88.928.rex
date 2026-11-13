quiet_ = .output~destination(.stream~new('/dev/null'))
v = 'orig'
call s <v
say v
call try 'plain'
call taken <v
exit
s:
  use arg <x
  say x
  x = 'changed'
  return
try:
  signal on syntax name e
  use arg <alias
  say 'ok'
  return
taken:
  signal on syntax name e
  has = 'a value'
  use arg <has
  say 'ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
