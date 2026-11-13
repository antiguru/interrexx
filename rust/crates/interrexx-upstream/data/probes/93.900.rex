quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name d
x = .k~subclass('X', , .sp~new)
d: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
::class k
::class sp
::method supplier
  raise syntax 93.900 array('boom')
