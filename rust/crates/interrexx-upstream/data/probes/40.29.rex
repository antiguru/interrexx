quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name a
say time('E', '13:05:09')
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
signal on syntax name b
say time('R', 'junk', 'Q')
b: say condition('o')~code
signal on syntax name c
say time('r', 5, 'H')
c: say condition('o')~code
