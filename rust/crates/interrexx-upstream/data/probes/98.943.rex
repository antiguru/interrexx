quiet_ = .output~destination(.stream~new('/dev/null'))
  '::class c subclass string inherit ms'
  '::class c subclass string inherit ms'
call try '::class ms mixinclass array', -
  '::class c subclass string inherit ms'
  '::class c subclass string inherit m2'
  '::class c subclass string inherit m2'
  '::class c inherit m2'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-class-mix2', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
