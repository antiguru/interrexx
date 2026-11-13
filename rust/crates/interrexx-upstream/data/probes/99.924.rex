quiet_ = .output~destination(.stream~new('/dev/null'))
call try 'say "first"; reply 1'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say 'ok' arg(1)
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
