quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name raised_
/* `MapCollection~of` with an argument that is not an array: 88.923, whose
   first substitution is `.context~name` -- the message name the method was
   invoked under, `OF`. So this witnesses `RexxContext~name` from inside a
   Rexx-coded library method as much as it witnesses `of`:
   an engine whose `~name` answered the declaring spelling, or the program's
   path, prints a different first word.

   `map_collection_of_pair.rex` is the 88.924 half. Each raise ends its own
   program, which is why they are two files. */

say 'A1 [' || .Directory~of()~items || ']'
d = .Directory~of('k1')
say 'A2 [not reached]'

exit
raised_: c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end
