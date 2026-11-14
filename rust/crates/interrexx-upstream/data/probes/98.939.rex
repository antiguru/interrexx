quiet_ = .output~destination(.stream~new('/dev/null'))
do i = 1 to 2
  call ('R'i)
end
exit
::routine r1
  signal on syntax name own
  raise syntax 40.3 additional (.nil)
  say 'not reached'
own: say 'caught' condition('o')~code
  return
::routine r2
  signal on syntax name own
  raise syntax 40.3 additional (.object~new)
  say 'not reached'
own: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
