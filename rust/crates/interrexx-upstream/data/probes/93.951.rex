quiet_ = .output~destination(.stream~new('/dev/null'))
n = .note~new
call try ".alarm~new('abc', n)"
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=> ok'
  x~cancel
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class note inherit AlarmNotification
::method triggered
  nop
