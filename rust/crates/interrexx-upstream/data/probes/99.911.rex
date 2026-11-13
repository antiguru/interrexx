quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name raised_
/* GUARD is legal only in a method invocation. A ::ROUTINE is not one, so this
   is 99.911 -- and the traceback carries the routine's own clause and then the
   CALL that reached it, which is what makes this the wider of the two shapes
   the check has. */

call sub

exit
raised_: c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end
::routine sub
  guard on
  return
