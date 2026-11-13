quiet_ = .output~destination(.stream~new('/dev/null'))
say RxCalcSin(30) RxCalcSin(30, , 'D') RxCalcSin(30, , 'd') RxCalcSin(30, , 'Degrees')
say RxCalcSin(0.5, , 'R') RxCalcSin(0.5, , 'radians') RxCalcSin(0.5, , 'Grad')
say RxCalcSin(33.3333333333333, , 'G') RxCalcSin(100, , 'g')
call try "RxCalcSin(30, , '')"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::requires 'rxmath' library
