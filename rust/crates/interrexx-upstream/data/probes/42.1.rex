quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name a
say 1e500000000 ** 2
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
signal on syntax name b
say 1e-500000000 ** 2
b: say condition('o')~code
signal on syntax name c
say 1e500000000 ** -2
c: say condition('o')~code
signal on syntax name d
say 99 ** 600000000
d: say condition('o')~code
signal on syntax name e
say 11 ** -999999999
e: say condition('o')~code
signal on syntax name f
say 9.99999999e999999998 ** -1
f: say condition('o')~code
say (1e300000000 ** 3) (1e-300000000 ** 3) (1e999999999 ** 1),
    (1e999999999 ** -1)
