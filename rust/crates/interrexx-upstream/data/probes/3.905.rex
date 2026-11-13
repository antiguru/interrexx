/* probe 3.905 */
signal on syntax name raised
x = .Routine~new('r', .Array~of('#!/usr/bin/env rexx', '/**/@REXX@', 'AAA!'))
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
