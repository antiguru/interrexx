quiet_ = .output~destination(.stream~new('/dev/null'))
call lineout , 'a'
call lineout 'STDOUT:', 'b'
say lineout(, 'c')
signal on syntax name a
call lineout , 'd', 5
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
