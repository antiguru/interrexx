quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name a
say c2d('ffffffffffff'x, 'x')
a: say condition('o')~code
signal on syntax name b
say c2d('ffffffffffff'x, -1)
b: say condition('o')~code
signal on syntax name c
say c2d('ffffffffffff'x)
c: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
