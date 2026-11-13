quiet_ = .output~destination(.stream~new('/dev/null'))
call keep
c = kept
names = 'args condition digits executable form fuzz interpreter invocation',
  'line name package rs stackFrames thread variables'
do m over names~makearray(' ')
  call try c, m
end
call alive .context
exit
keep: procedure expose kept
  kept = .context
  return
alive: procedure
  say 'main still answers' arg(1)~line
  return
try: procedure
  use arg o, m
  signal on syntax name bad
  r = o~send(m)
  say m 'answers'
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
