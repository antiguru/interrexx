/* Which entry the search acts on is unspecified when its own == changes the collection: r[class.relation.deviation.33] */
r = .relation~new
k = .key~new(r)
r~put('old',k)
q = .key~new(r)
q~arm
a = r~allAt(q)
say 'calls' q~hashes
say 'answer-either' (a~items == 0 | a[1] == 'new') 'current' r~at(q) 'items' r~items
exit
::class key
::attribute hashes
::method init
  expose table armed hashes
  use arg table
  hashes = 0
  armed = 0
::method arm
  expose armed
  armed = 1
::method hashCode
  expose hashes
  hashes += 1
  return 'key'
::method '=='
  expose table armed
  if armed then do
    armed = 0
    table~empty
    table~put('new', .key~new(table))
  end
  return 1
