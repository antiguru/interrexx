quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name raised_
/* A LEAVE naming no enclosing loop raises 28.3 without a debug pause. */
trace ?a
do i = 1 to 2
  leave zz
end

exit
raised_: c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end
