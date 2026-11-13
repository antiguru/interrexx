quiet_ = .output~destination(.stream~new('/dev/null'))
img = .array~of('#!/usr/bin/env rexx', '/**/@REXX@', 'c2F5ICdkZWNvZGVkJw==')
signal on syntax name bad
m = .method~new('n', img)
say 'made'
exit
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
