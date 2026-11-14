quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name raised_
/* `MapCollection~of` with an array that is not a two-item pair: 88.924, the
   other `.context~name` substitution `map_collection_of_index.rex` describes. */

say 'A1 [' || .Directory~of(.array~of('k1', 'v1'))['k1'] || ']'
d = .Directory~of(.array~of('k1'))
say 'A2 [not reached]'

exit
raised_: c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end
