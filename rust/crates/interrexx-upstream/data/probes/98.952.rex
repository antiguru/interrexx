quiet_ = .output~destination(.stream~new('/dev/null'))
f = 'spec-dir-requires-bad.tmp'
g = 'spec-dir-requires-deep.tmp'
tx = 'spec-dir-requires-text.tmp'
c1 = 'spec-dir-requires-cyc1.tmp'
c2 = 'spec-dir-requires-cyc2.tmp'
call sysfiledelete f
call sysfiledelete g
call sysfiledelete tx
call sysfiledelete c1
call sysfiledelete c2
call lineout f, 'say "bad prolog"'
call lineout f, 'x = 1 / 0'
call lineout f
call lineout tx, '::frob'
call lineout tx
call lineout g, '::requires "spec-dir-requires-nosuchfile-xyz.rex"'
call lineout g
call lineout c1, 'say "cyc1 prolog"'
call lineout c1, '::requires "'c2'"'
call lineout c1
call lineout c2, 'say "cyc2 prolog"'
call lineout c2, '::requires "'c1'"'
call lineout c2
call try 'say "main prolog"', '::requires "'c1'"'
call sysfiledelete f
call sysfiledelete g
call sysfiledelete tx
call sysfiledelete c1
call sysfiledelete c2
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-requires-fail', src)
  say 'ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
