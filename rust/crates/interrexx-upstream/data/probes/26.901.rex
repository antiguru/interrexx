quiet_ = .output~destination(.stream~new('/dev/null'))
p = .properties~new
do v over .array~of('0', '12', '-5', '+7', '007', '1.0', '1e2', '3.', ' 8 ')
  p~setProperty('k', v)
  say '['v']' '['p~getWhole('k')']'
end
do v over .array~of('1.5', 'abc', '1000000000', '99999999999', '0.5', '')
  p~setProperty('k', v)
  call try 'p~getWhole("k")', '['v']'
end
p~setProperty('w', '12')
say p~getWhole('w', 'junk') p~getWhole('zz', 5) p~getWhole('zz', '2.0')
numeric digits 20
numeric digits 3
say p~getWhole('zz', '123456')
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(2) '=>' r
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
