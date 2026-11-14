quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name raised_
/* `~uninherit` of a mixin the class does not inherit. The class
   below inherits `.M1` and not `.M2`, so the refusal names the one that is
   missing rather than the one that is there. */
.K~inherit(.M1)
say .K~superClasses
.K~uninherit(.M2)

exit
raised_: c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end
::class M1 mixinclass Object
::class M2 mixinclass Object
::class K
