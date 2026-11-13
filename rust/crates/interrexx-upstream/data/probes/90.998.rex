quiet_ = .output~destination(.stream~new('/dev/null'))
call try '::attribute a external "library rexxutil sysfiledelete"'
exit
try:
  signal on syntax name e
  src = .array~of('::class k')
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-attribute-ext', src)
  say arg(1) '=> ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
