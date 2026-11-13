quiet_ = .output~destination(.stream~new('/dev/null'))
s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.4 = 'd'
signal on syntax name e
call SysStemDelete 's.', 1
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
