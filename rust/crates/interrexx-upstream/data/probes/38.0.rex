quiet_ = .output~destination(.stream~new('/dev/null'))
re = .RegularExpression~new('xy')
signal on syntax name bad
re~position(1)
bad:
say condition('o')~traceback~firstItem~word(1)
signal on syntax name bad2
.RegularExpression~new('[')
bad2:
do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
::requires "rxregexp.cls"
