quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name s
l = copies('a', 250)
drop (l)
say 'ok 250'
l = copies('a', 251)
drop (l)
say 'ok 251'
exit
s: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
