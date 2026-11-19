/* Which and how many stored indexes and items a search compares, beyond those it must, are unspecified: r[class.bag.deviation.33] */
b = .bag~new
b~put(.key~new(b, 'old'))
q = .key~new(b, 'query')
q~arm
answer = b~hasItem(q)
say 'answer-either' (answer == 0 | answer == 1) 'items' b~items
say 'calls' q~hashes
exit
::class key
::attribute label
::attribute hashes
::method init
  expose bag label armed hashes
  use arg bag, label
  armed = 0
  hashes = 0
::method arm
  expose armed
  armed = 1
::method hashCode
  expose hashes
  hashes += 1
  return 'key'
::method '=='
  expose bag armed label
  use arg held
  if armed then do
    armed = 0
    bag~empty
    bag~put(.key~new(bag, 'new'))
  end
  return 1
