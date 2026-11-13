quiet_ = .output~destination(.stream~new('/dev/null'))
t = .t~new
call try '.message~new(t, .array~of("hello"))'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r~class~id
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class t
::method hello
  return 'hello'
