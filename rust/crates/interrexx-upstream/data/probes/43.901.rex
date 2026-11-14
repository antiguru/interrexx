quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name raised_
/* Package~new of a file whose ::REQUIRES does not parse, untrapped. */
p = .package~new('mid.cls')
say 'not reached'

exit
raised_: c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end
