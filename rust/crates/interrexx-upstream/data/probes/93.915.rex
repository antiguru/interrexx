quiet_ = .output~destination(.stream~new('/dev/null'))
x = ' pq '~strip('t')
say '['strip('   mn   ')']' '[' || x || ']'
signal on syntax name a
say strip('ab', 'Q')
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
signal on syntax name b
say strip('ab', , 'a', 1)
b: say condition('o')~code
signal on syntax name c
say strip(, 'L')
c: say condition('o')~code
signal on syntax name d
say strip()
d: say condition('o')~code
