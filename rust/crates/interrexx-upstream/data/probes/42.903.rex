quiet_ = .output~destination(.stream~new('/dev/null'))
call try '0 ** -1'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
