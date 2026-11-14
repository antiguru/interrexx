quiet_ = .output~destination(.stream~new('/dev/null'))
vals = '5 -5 0 -0 0.00 5.5 1e3 1e-3 +7 abc 1E 2e+'
meths = 'number positiveNumber nonNegativeNumber'
do v over vals~makearray(' ')
  s = v~left(5)
  do m over meths~makearray(' ')
    s = s t(m, v)~left(7)
  end
  say s~strip('T')
end
say t('number', ' + 2 ') t('number', '2 E1') t('number', ''),
    t('number', .object~new)
signal on syntax name done
.validate~positiveNumber('my argument', '-3')
done: say condition('o')~code condition('o')~additional~makeString('l', ' ; ')
exit
t: procedure
  use arg m, v
  signal on syntax name bad
  .validate~send(m, 'N', v)
  return 'ok'
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
