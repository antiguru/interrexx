quiet_ = .output~destination(.stream~new('/dev/null'))
call try '::class a inherit m public'
exit
try:
  signal on syntax name e
  p = .package~new('spec-dir-class-opt', .array~of(arg(1), -
    '::class m mixinclass object'))
  say arg(1) '=> ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
