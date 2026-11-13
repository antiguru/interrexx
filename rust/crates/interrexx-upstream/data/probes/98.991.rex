quiet_ = .output~destination(.stream~new('/dev/null'))
x = .k~new
y = .k~new
call try 'x~other(y)'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::routine p
  .k~new~setMethod('z', 'return 1')
::class k
::method self
  self~setMethod('z', 'return 1')
::method other
  use arg o
  o~setMethod('z', 'return 1')
::method scoped
  self~setMethod('z', 'return 1', arg(1))
::method cset class
  use arg o
  o~setMethod('z', 'return 1')
::class sub subclass k
::class other
