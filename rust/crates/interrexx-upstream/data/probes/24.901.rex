quiet_ = .output~destination(.stream~new('/dev/null'))
call try '5'
exit
try: procedure
  signal on syntax name e
  trace value arg(1)
  say arg(1) 'set'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
