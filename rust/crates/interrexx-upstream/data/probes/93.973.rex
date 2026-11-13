quiet_ = .output~destination(.stream~new('/dev/null'))
d = .datetime~new(2017, 6, 30, 5, 6, 7, 8, 0)
call try 'd < .nil'
.local~log = ''
say 'sent:' .local~log
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class sub subclass datetime
::class probe
::method isA
  .local~log = .local~log 'ISA('arg(1)~id')'
  return 0
::method utcDate
  .local~log = .local~log 'UTCDATE'
  return 0
