quiet_ = .output~destination(.stream~new('/dev/null'))
say '['changestr('a', '', 'x')']' '['changestr('a', '', 'x', 3)']'
signal on syntax name a
say changestr('a', '', 'x', -1)
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
