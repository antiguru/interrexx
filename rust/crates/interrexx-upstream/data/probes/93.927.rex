quiet_ = .output~destination(.stream~new('/dev/null'))
say d2x(65) d2x(0) d2x(255) d2x(256) d2x(65535) d2x(1234567) d2x('0065')
signal on syntax name a
say d2x(-1)
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
signal on syntax name b
say d2x(-65)
b: say condition('o')~code
