quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name a
say date('00'x, , , '-')
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
signal on syntax name b
say date(, '20070922', '00'x, , '-')
b: say condition('o')~code
