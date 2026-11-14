quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name e
src = .array~of('call tt 0', -
  '::routine tt external "registered rxmath RxCalcTan"')
p = .package~new('a3zqregdev', src)
say 'loaded'
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
