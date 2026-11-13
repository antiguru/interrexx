quiet_ = .output~destination(.stream~new('/dev/null'))
say format('1e15', , , 2) format('1e-25', , , 3) format('1e150', , , 3),
    format('1e15', , , 2, 16) format('5', , , 3, 6)
signal on syntax name a
say format('1e150', , , 2)
a: say condition('o')~code
signal on syntax name b
say format('1e-25', , , 1)
b: say condition('o')~code
signal on syntax name c
say format('1e999999999', , , 3)
c: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
say format('1e999999999', , , 9)
