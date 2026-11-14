quiet_ = .output~destination(.stream~new('/dev/null'))
say x2d(10) x2d(0010)
signal on syntax name a
say x2d(1.5)
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
signal on syntax name b
say x2d(1e+3)
b: say condition('o')~code
