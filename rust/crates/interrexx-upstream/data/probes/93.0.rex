quiet_ = .output~destination(.stream~new('/dev/null'))
f = 'spec-stream-openmore.tmp'
call sysfiledelete f
call charout f, 'abcdef'
call charout f
call try "stream(f, 'c', 'open 1')"
call sysfiledelete f
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say left(arg(1), 42) '=> [' || r || ']'
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
