/* Which entry the search acts on is unspecified when its own == changes the collection: r[class.table.deviation.33] */
t = .table~new
t~put('victim', .key~new(1))
w = .wanted~new(t)
removed = t~removeItem(w)
say 'consistent' ((removed == 'extra' & t~items == 0) | (removed == .nil & t~items == 1)) 'victim-still-present' t~hasIndex(.key~new(1)) 'key-2-left' (t~hasIndex(.key~new(2)) == (removed == .nil))
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
  use arg candidate
  if armed then do
    armed = 0
    table~remove(.key~new(1))
    table~put('extra', .key~new(2))
  end
  return candidate == 'victim'
