quiet_ = .output~destination(.stream~new('/dev/null'))
say random(,, 0) random(,, -0) random(,, '+0')
say random(,, 1000000000) random() random()
say random(,, 999999999999999999) random() random()
signal on syntax name a
say random(,, -1)
a: say condition('o')~code
signal on syntax name b
say random(5, 2, -1)
b: say condition('o')~code
signal on syntax name c
say random(-1000000000, 0, -1)
c: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
signal on syntax name d
say random('x', 2, -1)
d: say condition('o')~code
signal on syntax name e
say random(5, 2, 'x')
e: say condition('o')~code
signal on syntax name f
say random(,, 1000000000000000000)
f: say condition('o')~code
