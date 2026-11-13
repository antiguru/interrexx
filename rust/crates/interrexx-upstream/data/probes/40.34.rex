quiet_ = .output~destination(.stream~new('/dev/null'))
last = sourceline()
say sourceline(last)
signal on syntax name a
say sourceline(last + 1)
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
signal on syntax name b
say sourceline(999999999999999999)
b: say condition('o')~code
