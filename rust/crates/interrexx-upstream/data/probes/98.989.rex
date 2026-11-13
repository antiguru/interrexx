quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name raised_
/* provide.xml `abscla`: an abstract class provides definitions but is not
   intended to create instances, and trying to create one is an error. */
say 'declared' .ab~id .ab~class~id
say 'instance' .ab~new

exit
raised_: c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end
::class ab abstract
