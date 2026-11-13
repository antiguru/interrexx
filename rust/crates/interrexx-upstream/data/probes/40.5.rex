quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name a
say substr(, 1)
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
signal on syntax name b
say wordpos('a', , 1)
b: say condition('o')~code
