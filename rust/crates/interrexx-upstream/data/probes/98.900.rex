quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name e1
y = .object~subclass('Y', .meta)
say 'not reached'
e1: say 'subclass' condition('o')~code
signal on syntax name e2
z = .object~mixinclass('Z', .meta)
say 'not reached'
e2: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
::class meta subclass class
::method new class
  return .nil
