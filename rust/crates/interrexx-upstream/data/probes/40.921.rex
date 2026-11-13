quiet_ = .output~destination(.stream~new('/dev/null'))
call try "SysStemSort('s.', , , 9, 8)", ""
exit
try:
  parse arg ex, setup
  s.0 = 6; s.1 = 'f'; s.2 = 'e'; s.3 = 'd'; s.4 = 'c'; s.5 = 'b'; s.6 = 'a'
  interpret setup
  signal on syntax name e
  interpret 'r = ' ex
  say r
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
