quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name d
x = .k~subclass('X', , .array~of('return 1', .object~new))
d: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
::class k
