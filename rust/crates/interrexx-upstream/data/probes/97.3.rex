quiet_ = .output~destination(.stream~new('/dev/null'))
a = .target~new('A')
m = .monitor~new(a)
say m~hello(1, 2)
say m~hello(, 2)
say m~hello(1, , 3, )
say m~hello
say m~Hello(1) m~'hello'(1)
say m~unknown('hello', .array~of(7, 8))
call try "m~pk"
t = .monitor~new(.catcher~new)
say t~whatsoever(4, 5) t~Mixed(6)
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=>' x
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class target
::method init
  expose name
  use arg name
::method hello
  expose name
  return name 'hello' arg() arg(1, 'E') arg(2, 'E') arg(3, 'E')
::method fail
  raise syntax 40.900 array('boom')
::method priv private
  return 'private'
::method pk package
  return 'package'
::method classside class
  return 'class side'
::method silent
  return
::class catcher
::method unknown
  use arg name, args
  return 'caught' name args~items
