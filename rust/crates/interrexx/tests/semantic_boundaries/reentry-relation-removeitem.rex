/* Which entry the search acts on is unspecified when its own == changes the collection: r[class.relation.deviation.33] */
t = .relation~new
t~put('victim', .key~new(1))
w = .wanted~new(t)
removed = t~removeItem(w, .key~new(1))
left = t~allItems[1]
say 'removed-one-of-two' (removed == 'victim' | removed == 'extra') 'items' t~items 'other-left' (left \== removed & (left == 'victim' | left == 'extra'))
exit
::class key
::attribute label
::method init
  expose label
  use arg label
::method hashCode
  return 'key'
::method '=='
  expose label
  use arg other
  return label == other~label
::class wanted
::method init
  expose table armed
  use arg table
  armed = 1
::method '=='
  expose table armed
  if armed then do
    armed = 0
    table~put('extra', .key~new(2))
  end
  return 1
