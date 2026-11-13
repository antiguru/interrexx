quiet_ = .output~destination(.stream~new('/dev/null'))
s = .supplier~new(.array~of('p', 'q'), .array~of(1, 2))
s~next
say s~item
s~next
say s~available
call try 's~next'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
