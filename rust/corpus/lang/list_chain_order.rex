/* A List as a chain of entries addressed by handle: removal and insertion
   anywhere relink neighbours and never renumber, a freed handle is reused
   last-freed-first, and the walks forward and backward agree after every
   kind of change.  removeItem walks the live chain, so an `==` that removes
   entries while it is compared changes which entries are compared next. */

l = .list~new
do i = 1 to 6; h.i = l~append('v'i); end
call show l, 'init'
say 'remove 3' l~remove(h.3) 'remove 1' l~remove(h.1) 'remove 1 again' l~remove(h.1)
call show l, 'after removes'
a = l~append('n1'); b = l~insert('n2', .nil); c = l~insert('n3', h.4); d = l~insert('n4')
say 'new handles' a b c d
call show l, 'after inserts'
say 'next' l~next(h.4) 'prev' l~previous(h.4) 'nextlast' l~next(l~last) 'prevfirst' l~previous(l~first)
say 'next bad' l~next(99) l~previous(h.3)
signal on syntax name e0
say l~next('abc')
e0: say 'next abc' condition('o')~code
l~put('v5b', h.5)
pe = .list~of(a1,a2,a3,a4); pe~put(, 1)
say 'pe slot' pe~allItems[1] pe~allItems~size
say 'pe' pe~at(1) pe~hasIndex(1) pe~items pe~remove(1) pe~items pe~next(1) pe~next(0) pe~previous(2) pe~allIndexes~toString(,',')
say 'at nonempty' l~at(h.5) 'hasIndex' l~hasIndex(h.5) 'items' l~items 'remove empty' l~remove(h.5) l~items
say 'next of empty' l~next(h.5)
say 'next into empty' l~next(c) l~previous(h.6)
call show l, 'after put empty'
say 'removeItem v4' l~removeItem('v4') 'removeItem zz' l~removeItem('zz')
call show l, 'after removeItem'
s = l~section(l~first, 3); call show s, 'section 3'
s = l~section(l~next(l~first)); call show s, 'section rest'
signal on syntax name e1
s = l~section(99, 1)
e1: say 'section bad' condition('o')~code
say 'index' l~index('n1') l~index('zz') l~hasItem('n2') l~hasItem('q')
cp = l~copy; cp~append('copyonly'); cp~remove(cp~first)
call show l, 'orig after copy change'; call show cp, 'copy'
l2 = .list~of(1,2,3); call show l2, 'of'
l2~empty; say 'empty items' l2~items l2~isEmpty l2~first l2~last l2~firstItem l2~append('z') l2~append('y')
q = .list~new; do i = 1 to 5; q~append(i); end
do i = 1 to 5; q~remove(q~first); end
say 'emptied' q~items q~first q~append('a') q~append('b') q~append('c') q~insert('d', .nil)
call show q, 'reuse'
w = .list~new; x1 = w~insert('x'); x2 = w~insert('y', x1); x3 = w~insert('z', .nil)
call show w, 'insert mix'
sup = w~supplier; do while sup~available; say ' sup' sup~index sup~item; sup~next; end
say w~makeArray~toString(,',') w~allIndexes~toString(,',')
signal on syntax name e2
w~put('q', 77)
e2: say 'put bad' condition('o')~code
signal on syntax name e3
w~insert('q', 77)
e3: say 'insert bad' condition('o')~code
say w~remove(77) w~at(77) w~hasIndex('1.0') w~at(' 1')
signal on syntax name e5
say w~at(-1)
e5: say 'at -1' condition('o')~code
say 'firstItem empty entry'; z = .list~new; z~insert(); z~append('b'); say z~firstItem z~first z~items
signal on syntax name e4
say z~at(1.5)
e4: say 'at 1.5' condition('o')~code

cl = .list~new
k = .key~new(cl)
do i = 1 to 5; cl~append(.thing~new(i)); end
say 'removeItem under a mutating ==' cl~removeItem(k) cl~items
call show cl, 'after mutating =='
exit 0

::class thing
::method init; expose n; use arg n
::method n; expose n; return n
::method string; expose n; return 'T'n
::class key
::method init; expose l; use arg l
::method '=='
  expose l
  use arg other
  say '  compare' other
  if other~n = 2 then do; l~remove(l~first); l~remove(l~next(l~first)); end
  return other~n = 4
::routine show
  use arg l, tag
  out = tag':' l~items '['
  i = l~first
  do while i \= .nil; out = out i'='l~at(i); i = l~next(i); end
  say out ']' 'last' l~last 'rev:' rev(l)
  return
::routine rev
  use arg l
  out = ''; i = l~last
  do while i \= .nil; out = out i; i = l~previous(i); end
  return out
