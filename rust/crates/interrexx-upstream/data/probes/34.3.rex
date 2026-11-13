quiet_ = .output~destination(.stream~new('/dev/null'))
n = 0
do while n < 3
  n = n + 1
  say 'w' n
end
do while 0
  say 'never'
end
do i = 1 to 5 while i < 3
  say 'wi' i
end
say 'i' i
do e over .array~of(1, 2, 3) while e < 3
  say 'w' e
end
say 'e' e
do 2 while 1, 1
  say 'list'
end
do while 0, 2
  say 'never'
end
call try 'do while 2; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
