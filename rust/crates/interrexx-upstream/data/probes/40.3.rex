quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name s
say 'start'
call ie
say 'not reached'
exit
ie: raise syntax 40.3 exit
s: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
