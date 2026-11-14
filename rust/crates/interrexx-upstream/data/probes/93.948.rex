quiet_ = .output~destination(.stream~new('/dev/null'))
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 330)
t = .timespan~fromSeconds(90)
say d + t
say d + .timespan~fromSeconds(-90)
say (d + t)~offset (+d)~offset (+d == d)
say +d d~'+'
say d + .timespan~fromDays(0.5)
call try 'd + 5'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
