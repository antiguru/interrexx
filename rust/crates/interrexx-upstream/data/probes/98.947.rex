quiet_ = .output~destination(.stream~new('/dev/null'))
o = .box~new
say o~deep
call try 'main'
say 'inside =>' o~inside
exit
try:
  what = arg(1)
  signal on syntax name e
  if what = 'main' then forward to ('abc') message 'length'
  if what = 'internal' then do; call lab; return; end
  if what = 'routine' then call r
  say what '=> ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
lab:
  forward to ('abc') message 'length'
::routine r
  forward to ('abc') message 'length'
::class box
::method deep
  do 1
    if 1 then forward to ('abcd') message 'length'
  end
  return 'never'
::method inside
  signal on syntax name h
  interpret "forward to ('abc') message 'length'"
  return 'no'
h: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
