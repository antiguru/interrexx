quiet_ = .output~destination(.stream~new('/dev/null'))
say space(' a b  c ', 0) space(' a b  c ', 0, '*')
signal on syntax name a
say space(' a b ', 0, '**')
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
