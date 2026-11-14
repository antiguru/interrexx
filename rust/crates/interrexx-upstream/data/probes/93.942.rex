quiet_ = .output~destination(.stream~new('/dev/null'))
say '['format(9.4, 1, 0)']'
signal on syntax name a
say format(9.6, 1, 0)
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
say '['format(9.6, 2, 0)']'
