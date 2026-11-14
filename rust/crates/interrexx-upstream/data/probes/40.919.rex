quiet_ = .output~destination(.stream~new('/dev/null'))
call try "SysStemDelete('s-.')", ""
exit
try:
  parse arg ex, setup
  s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
  interpret setup
  signal on syntax name e
  interpret 'r = ' ex
  say r
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
