quiet_ = .output~destination(.stream~new('/dev/null'))
q = .queue~of('a', 'b', 'c')
do i over .array~of(4, 5, 20, 50, 1000)
  call try 'q~put("x", 'i')'
end
say q~allItems~makeString('L', ',')
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
