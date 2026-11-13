quiet_ = .output~destination(.stream~new('/dev/null'))
call try 'select; when 1, 5 then nop; end'
exit
f: say 'f('arg(1)')'; return arg(1)
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
