quiet_ = .output~destination(.stream~new('/dev/null'))
say '['substr('abc', 2, 0)']' '['substr('abc', 2, '-0')']'
signal on syntax name a
say substr('abc', 1, -1)
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
