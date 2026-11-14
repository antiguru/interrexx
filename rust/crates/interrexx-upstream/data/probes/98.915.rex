quiet_ = .output~destination(.stream~new('/dev/null'))
/* A message a start was sent to cannot be sent or started again. */
o = .t~new
m = .message~new(o, 'n')
m~start
say m~result
signal on syntax name s1
m~send
s1: say 'send' condition('O')~code
signal on syntax name s2
m~start('x')
s2: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
m2 = o~start('n')
signal on syntax name s3
m2~sendWith(o, .array~new)
s3: say 'sendWith' condition('O')~code
say m2~result
::class t
::method n unguarded
  return 'n'
