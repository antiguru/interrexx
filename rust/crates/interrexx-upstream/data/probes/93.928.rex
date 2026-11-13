quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name a
say d2x('65.5')
a: say condition('o')~code
signal on syntax name b
say d2x(65, '1.5')
b: say condition('o')~code
signal on syntax name c
say d2x(65, -1)
c: say condition('o')~code
signal on syntax name d
say d2x('abc', -1)
d: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
