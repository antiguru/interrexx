quiet_ = .output~destination(.stream~new('/dev/null'))
numeric digits 1
call show 'a', 'b'
exit
show:
say arg(2) arg(' 2 ') arg('2.0') arg(1e0)
signal on syntax name a
say arg(0)
a: say condition('o')~code
signal on syntax name b
say arg('-0')
b: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
signal on syntax name c
say arg('')
c: say condition('o')~code
signal on syntax name d
say arg(1.5)
d: say condition('o')~code
signal on syntax name e
say arg(1e30)
e: say condition('o')~code
