quiet_ = .output~destination(.stream~new('/dev/null'))
p = .properties~new
p~put('v1', 'a')
p['b'] = 7
p~setProperty('c', .made~new)
p~put(.array~of(1, 2), 'd')
do i over p~allIndexes~sort
  say i p[i]~class~id '['p[i]~changeStr('0a'x, '/')']'
end
call try 'p["k"] = .nil'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class made
::method makestring
  return 'made'
::class plain
