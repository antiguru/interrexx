quiet_ = .output~destination(.stream~new('/dev/null'))
call try "sysfiletree('', 'f.', '', 'zz', 'qq')"
exit
try: procedure
  parse arg ex
  signal on syntax name bad
  interpret 'r = ' ex
  say r
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
return
