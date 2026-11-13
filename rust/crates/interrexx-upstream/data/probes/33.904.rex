quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name s
raise syntax (t('abc')) description (t('text')) additional (t('extra')),
      return (t('value'))
s: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
exit
t: say 'evaluate' arg(1)
  return arg(1)
