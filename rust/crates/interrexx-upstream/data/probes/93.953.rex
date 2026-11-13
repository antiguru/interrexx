quiet_ = .output~destination(.stream~new('/dev/null'))
p = .package~new('named', 'return 1')
say p~class~id p~name
call try '.package~new("n", .nil, 5)'
say .mine~new('n', 'nop', , 'a', 'b')~class~id
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class mine subclass package
::method init
  say 'init' arg() arg(1) arg(2)
::class onlystr
::method string
  return 'onlystr'
