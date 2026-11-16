/* Which and how many stored indexes and items a search compares, beyond those it must, are unspecified: r[class.relation.deviation.33] */
r = .relation~new
r~put('tail', 'key')
r~put('head', 'key')
q = .wanted~new(r)
answer = r~hasItem(q)
say 'answer-either' (answer == 0 | answer == 1) 'items' r~items
say 'compared' (q~calls > 0)
exit
::class wanted
::attribute calls
::method init
  expose table calls
  use arg table
  calls = 0
::method '=='
  expose table calls
  use arg held
  calls += 1
  if calls = 1 then do
    table~empty
    table~put('new-tail', 'key')
    table~put('new-head', 'key')
    return 0
  end
  return held == 'new-tail'
