quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name raised_
/* REPLY is legal only in a method invocation, and 99.919 is GUARD's own
   99.911 one number over. A ::ROUTINE is not a method invocation whatever it
   was called from. */

call sub

exit
raised_: c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end
::routine sub
  reply 5
  return
