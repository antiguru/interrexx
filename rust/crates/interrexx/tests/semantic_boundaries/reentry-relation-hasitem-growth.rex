/* Which and how many stored indexes and items a search compares, beyond those it must, are unspecified: r[class.relation.deviation.33] */
r = .relation~new
k = .key~new(r)
r~put('old',k)
q = .key~new(r)
q~arm
answer = r~hasItem('old', q)
say 'calls' q~hashes
say 'answer' answer 'items' r~items
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
    do i = 1 to 40
      table~put('extra', .key~new(table))
    end
  end
  return 1
