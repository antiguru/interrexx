quiet_ = .output~destination(.stream~new('/dev/null'))
say x2d('3b9ac9ff') x2d(copies('0', 30) || 'ff') x2d('ffffffffff', 2)
signal on syntax name a
say x2d('3b9aca00')
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
signal on syntax name b
say x2d('8000000000', 10)
b: say condition('o')~code
numeric digits 20
say x2d('ffffffffffffffff') x2d('8000000000000000', 16)
