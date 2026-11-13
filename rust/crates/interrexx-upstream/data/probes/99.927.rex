quiet_ = .output~destination(.stream~new('/dev/null'))
m = .k~mixinClass('Mix')
say m~id m~queryMixinClass m~superclass~id m~baseClass~id m~class~id
say .k~subclasses~items m~isAbstract
c = .k~subclass('Heir')
c~inherit(m)
say c~superclasses~items c~superclasses[2]~id c~new~isA(m) c~new~isA(.k)
m2 = m~mixinClass('Mix2')
say m2~superclass~id m2~baseClass~id
mm = .k~mixinClass('WithMeta', .meta)
say mm~class~id mm~queryMixinClass
call try '.k~mixinClass("X", 1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r~id
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class k
::class meta subclass class
