quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name s
x = .a~new~hidden
say 'not reached'
s: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
say 'before'
x = .a~new~hidden
say 'not reached'
::class a
::method hidden private
  return 'ran'
