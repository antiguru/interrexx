quiet_ = .output~destination(.stream~new('/dev/null'))
o = .k~new
signal on syntax name refused
say o~outer
exit
refused: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
::class k
::method outer
  m = self~start('inner')
  return m~result
::method inner
  return 'inner'
