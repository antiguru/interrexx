quiet_ = .output~destination(.stream~new('/dev/null'))
p = .properties~new
do v over .array~of('0', '12', '-5', '+7', '007', '1.0', '1e2', '3.', ' 8 ')
  p~setWhole('k', v)
  call charout , '['p['k']'] '
end
say
do v over .array~of('1.5', 'abc', '1000000000', '99999999999', '1e9', '')
  call try 'p~setWhole("k", v)', '['v']'
end
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(2) '=> ok'
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class answers
::method init
  expose r
  use arg r
::method datatype
  expose r
  say '  datatype' arg(1)
  return r
