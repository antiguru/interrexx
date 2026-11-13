quiet_ = .output~destination(.stream~new('/dev/null'))
call try "x = stream('nofile_zz', 'c', 'bogus')"
exit
try:
  signal on syntax name e
  interpret arg(1)
  say left(arg(1), 44) '=> none'
  return
e: label = arg(1)
  if arg(2, 'E') then label = arg(2)
  do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
