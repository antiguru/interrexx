quiet_ = .output~destination(.stream~new('/dev/null'))
r = random(0, 999999999)
say r >= 0 & r <= 999999999
r = random(-999999999, 0)
say r >= -999999999 & r <= 0
signal on syntax name a
say random(1000000000)
a: say condition('o')~code
signal on syntax name b
say random(-1, 999999999)
b: say condition('o')~code
signal on syntax name c
say random(5, 2)
c: say condition('o')~code
signal on syntax name d
say random(-1)
d: say condition('o')~code
signal on syntax name e
say random(-1000000000, 0)
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
