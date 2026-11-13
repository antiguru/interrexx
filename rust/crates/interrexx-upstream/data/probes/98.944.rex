quiet_ = .output~destination(.stream~new('/dev/null'))
call try '::class c inherit m1 m1'
exit
try:
  signal on syntax name e
  src = .array~of('::class m1 mixinclass object', -
    '::class m2 mixinclass object', '::class plain', arg(1))
  p = .package~new('spec-dir-class-mix', src)
  say arg(1) '=> ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
