quiet_ = .output~destination(.stream~new('/dev/null'))
say 'TWFu'~decodeBase64 'TWE='~decodeBase64 'TQ=='~decodeBase64
say '['||''~decodeBase64']' 'TWFuTQ=='~decodeBase64 '/+8='~decodeBase64~c2x
say 'TR=='~decodeBase64~c2x 'TWF='~decodeBase64~c2x
say 'abc'~encodeBase64~decodeBase64
call try "'TWE'~decodeBase64"
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
