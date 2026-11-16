/* Which and how many stored indexes and items a search compares, beyond those it must, are unspecified: r[class.relation.deviation.33] */
r = .relation~new
r~put('first', .key~new('A'))
r~put('second', .key~new('B'))
k = .key~new('A')
say 'has' r~hasItem('first',k)
say 'remove' r~removeItem('first',k)
exit
::class key
::attribute label
::method init
  expose label
  use arg label
::method hashCode
  return 'same hash'
::method '=='
  expose label
  use arg other
  return label == other~label
