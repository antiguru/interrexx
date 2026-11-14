quiet_ = .output~destination(.stream~new('/dev/null'))
call try "SysFork(1)"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
