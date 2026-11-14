quiet_ = .output~destination(.stream~new('/dev/null'))
d = .datetime~new(2017, 6, 5, 9, 8, 7, 12345, 0)
say d~europeanDate d~normalDate d~orderedDate,
    d~standardDate d~usaDate d~languageDate
say d~europeanDate('-') d~normalDate('-'),
    d~orderedDate('.') d~standardDate('-')
say d~usaDate('') d~europeanDate(' ') d~normalDate('') d~standardDate('')
say .datetime~new(5, 1, 1)~europeanDate .datetime~new(2100, 1, 1)~europeanDate
say .datetime~new(5, 1, 1)~normalDate .datetime~new(5, 1, 1)~languageDate
call try 'd~usaDate("//")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
