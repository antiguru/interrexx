quiet_ = .output~destination(.stream~new('/dev/null'))
o = .box~new
say o~~note~~note~class~id
signal on syntax name e
say o~note~class~id
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
::class box
::method note
  say 'note'
