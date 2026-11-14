quiet_ = .output~destination(.stream~new('/dev/null'))
call try '.method~new("n", .array~of(.onlystr~new))'
ssrc = .ss~new("a = 1" || "0A"x || "return a")
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class onlystr
::method string
  return 'onlystr'
::class ss subclass string
