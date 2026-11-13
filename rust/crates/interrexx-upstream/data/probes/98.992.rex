quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name e
r = .routine~new('r', 'expose a')
r~call
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
