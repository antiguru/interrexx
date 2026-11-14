quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name s
x = 1 + 'a'
s: say condition('E') condition('O')~code
signal on syntax name t
x = substr('abc', 0)
t: say condition('E') condition('O')~code
signal on novalue name n
y = unset + 1
n: say '['condition('E')']'
signal on syntax name z
call rz
z: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
exit
rz: raise syntax 40 return
