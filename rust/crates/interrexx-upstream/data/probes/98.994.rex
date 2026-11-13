quiet_ = .output~destination(.stream~new('/dev/null'))
a = .array~of('x', 'y', 'z')
do with index ix item it over a
  say ix it
end
do with item it index ix over a for 2
  say ix it
end
do with index ix over a
  say 'index' ix
end
do with index i item i over a
  say 'same' i
end
do with over a
  say 'bare'
end
b = .array~of(1, 2)
do with item it over b
  say 'it' it
  if it = 1 then b~append(3)
end
say b~items
call try 'do with item it over .badsup~new; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class badsup
::method supplier
  return 42
::class onlysup
::method supplier
  return .array~of(1)~supplier
