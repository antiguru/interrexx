quiet_ = .output~destination(.stream~new('/dev/null'))
say (4)~max(2,,)
signal on syntax name a
say (1)~max(,3)
a: say condition('o')~code
signal on syntax name b
say (1)~max(2,,3)
b: say condition('o')~code
signal on syntax name c
say ('x')~max(,3)
c: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
