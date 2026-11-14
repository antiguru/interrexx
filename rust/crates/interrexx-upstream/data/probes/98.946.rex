quiet_ = .output~destination(.stream~new('/dev/null'))
m = .monitor~new(.target~new)
signal on syntax name bad
say m~unknown('HELLO', .ans~new)
exit
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
::class target
::method hello
  return 'hello' arg()
::class ans
::method request
  return .queue~of('x')
