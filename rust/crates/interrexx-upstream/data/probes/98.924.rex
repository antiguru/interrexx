quiet_ = .output~destination(.stream~new('/dev/null'))
address system 'cat' with input using ('one') output stem o.
say o.0 o.1
l = .list~of('x', 'y')
address system 'cat' with input using (l) output stem o.
say o.0 o.1 o.2
address system 'cat' with input using (.set~of('s')) output stem o.
say o.0 o.1
mb = .mutablebuffer~new('mb')
address system 'cat' with input using (mb) output stem o.
say o.0 o.1
address system 'cat' with input using (.mak~new) output stem o.
say o.0 o.1 o.2
signal on syntax name e
address system 'cat' with input using (.nil)
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
::class mak
::method makearray
  return .array~of('m1', 'm2')
