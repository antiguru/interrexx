quiet_ = .output~destination(.stream~new('/dev/null'))
s.0 = 2
s.1 = 'a'
s.2 = 'b'
say s.0 s.1
call try "SysStemDelete('s.', '1.0000000000000000001')"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
