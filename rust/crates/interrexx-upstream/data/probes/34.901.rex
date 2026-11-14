quiet_ = .output~destination(.stream~new('/dev/null'))
x = 5
call try 'x &= 2'
say x
exit
try:
  signal on syntax name e
  interpret arg(1)
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
