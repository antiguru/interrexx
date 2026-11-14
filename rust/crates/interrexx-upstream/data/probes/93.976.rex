quiet_ = .output~destination(.stream~new('/dev/null'))
say x2b('4a 4B') x2b('4' || '09'x || '4a')
signal on syntax name a
say x2b('4g')
a: say condition('o')~code
signal on syntax name b
say x2b('4a ')
b: say condition('o')~code
signal on syntax name c
say x2b('4 a')
c: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
