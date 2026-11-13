quiet_ = .output~destination(.stream~new('/dev/null'))
o = .leaf~new
name = 'mid'
call try 'say o~hello:.other'
exit
try:
  signal on syntax name e
  interpret arg(1)
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class mid
::method hello
  return 'mid hello'
::class leaf subclass mid
::class other
::method hello
  return 'other hello'
