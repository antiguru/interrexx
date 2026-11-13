quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name raised_
/* A `GUARD ... WHEN` expression that is exposed but not exactly `0` or `1`
   is 34.902, GUARD's own sub-number rather than IF's 34.1 or WHEN's 34.2.
   The value is checked before anything would wait on it, so the raise is
   the oracle's answer whatever a scheduler does with a false one.
   */

say .K~m

exit
raised_: c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end
::class K

::method m class
  expose v
  v = 'x'
  guard on when v
  return 'ran'
