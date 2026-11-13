quiet_ = .output~destination(.stream~new('/dev/null'))
b = .bag~new
b~put('a')
b~put('a', 'a')
b['b'] = 'b'
b[] = 'a'
b~put(.nil)
b~put('d', )
say b~items b~items('a') b~items('d') b~items(.nil)
call try 'b~put(1, 2)'
say b~items
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
