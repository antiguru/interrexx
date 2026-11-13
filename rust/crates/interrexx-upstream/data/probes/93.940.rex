quiet_ = .output~destination(.stream~new('/dev/null'))
say 9~modulo(4) (-7)~modulo(4) 0~modulo(5) '1e2'~modulo(7) 12~modulo(' 5 ')
say '1.0'~modulo(5) 9~modulo(5.0) '11.0'~modulo(5) (9 // 4)
call try "10.5~modulo(3)"
numeric digits 3
say '1000'~modulo(7) '1.0000'~modulo(7)
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
