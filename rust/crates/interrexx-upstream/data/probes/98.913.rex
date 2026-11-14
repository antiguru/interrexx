quiet_ = .output~destination(.stream~new('/dev/null'))
d = .directory~new
call try "d~unknown('Q', .ans~new)"
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class ans
::method request
  return .queue~of('x')
