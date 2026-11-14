quiet_ = .output~destination(.stream~new('/dev/null'))
say max(3, 8, 5) 3~max(8, 5) '3'~max('8', '5') (3)~max
signal on syntax name a
say max()
a: say condition('o')~code
signal on syntax name b
say '3'~max(8, 'five')
b: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
signal on syntax name c
say 'three'~max(8)
c: say condition('o')~code
