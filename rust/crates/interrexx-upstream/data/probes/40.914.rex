quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name a
say value('x', , 'SYSTEM')
a: say condition('o')~code
signal on syntax name b
say value('x y', , 'ENV')
b: say condition('o')~code
signal on syntax name c
say value('x', , 'ENVIRONMENT ')
c: say condition('o')~code
signal on syntax name d
say value('x', , .nil)
d: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
say value('x', , 'Environment') '/' value('x', , '')
