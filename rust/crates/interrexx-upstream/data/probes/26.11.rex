quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name a
say 5e999999999 * 20
a: say condition('o')~code
signal on syntax name b
say 1e-999999999 / 10
b: say condition('o')~code
signal on syntax name c
say 1e999999999 % 7
c: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
