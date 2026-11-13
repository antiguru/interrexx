quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name raised_
say 'before'
if 0 then leave
say 'unreached leave is no error'
leave

exit
raised_: c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end
