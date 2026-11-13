quiet_ = .output~destination(.stream~new('/dev/null'))
a = .array~new(2, 2)
a[2, 1] = 'x'
say a[2, 1] a[.array~of(2, 1)] a~at('2', 1.0)
call try 'a~at(1)'
say .array~new~at(0, 0) .array~new~at(5, 5, 5)
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class pair subclass array
