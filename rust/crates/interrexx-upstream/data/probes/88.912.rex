quiet_ = .output~destination(.stream~new('/dev/null'))
t = 'abcdefghij'
call put "b[3] = 'XY'"
call put "b[3, 1] = 'XY'"
call put "b[3, 4] = 'X'"
call put "b[3, 0] = 'X'"
call put "b[3, 99] = 'X'"
call put "b[3] = ''"
call put "b[11] = 'X'"
call put "b[14] = 'X'"
call put "b[2, 3] = 5"
call put "b[0] = 'X'"
call put "b[-2] = 'X'"
call put "b['x'] = 'X'"
call put "b[2, -1] = 'X'"
call put "b[2, 'x'] = 'X'"
call put "b[, 1] = 'X'"
call put "b[2] = .nil"
call put "b~'[]='('X')"
call put "b~'[]='"
call put "b~'[]='('X', 2, 1, 1)"
call put "b~'[]='('Q', 2)"
b = .mutablebuffer~new(t)
r = b~'[]='('Q', 2)
say r~class~id (r~identityHash == b~identityHash) b~string
exit
put:
  signal on syntax name bad
  b = .mutablebuffer~new(t)
  interpret arg(1)
  say arg(1) '=> <' || b~string || '>'
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
