quiet_ = .output~destination(.stream~new('/dev/null'))
call show .array~of('0', '1', .true, .false, '2')
call show .array~of('1.0', '01', ' 1', 'yes', '', .nil)
exit
show:
  do v over arg(1)
    say '['v~string']' t(v)
  end
  return
t: procedure
  use arg v
  signal on syntax name bad
  .validate~logical('flag', v)
  return 'ok'
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
