quiet_ = .output~destination(.stream~new('/dev/null'))
o = .lazy~new
call try "o~cancel"
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class lazy subclass alarm
::method init
  nop
