quiet_ = .output~destination(.stream~new('/dev/null'))
say b2x('10 0101 1100') b2x('0001' || '09'x || '1111')
signal on syntax name a
say b2x('12')
a: say condition('o')~code
signal on syntax name b
say b2x('1111 ')
b: say condition('o')~code
signal on syntax name c
say b2x('1 11')
c: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
