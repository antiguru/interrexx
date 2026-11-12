/* A Queue's size is its ITEM count -- `size` is bound to Array's `items` --
   and its insertion index is bounded by the last
   item where an Array's extends to meet it.

   An index past the last item is 93.966 (r[class.queue.put]).  The oracle
   answers 93.918 for an index far enough above the last item
   (Deviation 121, r[class.queue.deviation.121]), so only indexes the oracle
   also answers 93.966 for are probed here. */

q = .Queue~new
q~queue('a')
q~queue('b')
say 'before' q~items q~size
q~empty
say 'after empty' q~items q~size q~isEmpty
q~queue('z')
say 'and queueing starts again at the front' q~items q~size q~peek q~pull

r = .Queue~new
r~queue('a')
r~queue('b')
say 'insert at the last item is allowed' r~insert('k', 2) 'now' r~allItems~makeString('L', ',')

call bounded r, 'INSERT'
call bounded r, 'PUT'
say 'both refused'

/* Past the last item, inside the oracle's allocated extent. */
e = .Queue~new
e~queue('a')
call bounded2 e, 16, 'inside the default extent'
f = .Queue~new(50)
f~queue('a')
call bounded2 f, 50, 'inside a requested extent'
say 'both refused'

s = .Queue~new
s~queue('a')
s~queue('b')
s~put('n', 4)
say 'unreachable'
exit

bounded2: procedure
  use arg target, at, what
  signal on syntax name oops2
  target~put('Z', at)
  say what 'accepted'
  return
oops2:
  say what 'raised' rc
  return

bounded: procedure
  use arg target, name
  signal on syntax name oops
  v = target~send(name, 'z', 9)
  say name 'did not refuse'
  return
oops:
  say name 'past the last item raised' rc condition('C')
  return
