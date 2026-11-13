/* The hash collections' class tests (string, identity or equality keys; an
   index-only or multi-value PUT) for classes made and dropped at run time,
   so a class identity freed by one can be minted for the next with another
   base, and for a class whose superclass list INHERIT and UNINHERIT change. */
bases = .array~of(.Directory, .IdentityTable, .Table, .Set, .Bag, .Relation, .StringTable)
do round = 1 to 3
  do b = 1 to bases~items
    k = (b + round) // bases~items + 1
    c = bases[k]~subclass('Made' || round || k)
    call probe c~new, bases[k]~id
    drop c
  end
  call gc
end
c = .Table~subclass('Mixed')
call probe c~new, 'Table'
c~inherit(.Marker)
call probe c~new, 'Table with a mixin'
c~uninherit(.Marker)
call probe c~new, 'Table again'
exit

gc: procedure
  /* Garbage so a collection runs and the dropped classes can go. */
  do i = 1 to 2000
    a = .array~new(50)
  end
  return

probe: procedure
  use arg h, base
  /* Two equal values that are not the same object, and a string index under
     two spellings. */
  k1 = .Same~new
  k2 = .Same~new
  if h~isA(.Set) | h~isA(.Bag) then do
    h~put(k1)
    h~put(k2)
    h~put(k1)
  end
  else do
    call putPairs h, k1, k2
    h~put(4, 'ab')
  end
  say left(base, 20) h~class~id 'items' h~items 'hasIndex' h~hasIndex('AB') h~hasIndex('ab')
  return

putPairs: procedure
  use arg h, k1, k2
  signal on syntax name refused
  h~put(1, k1)
  h~put(2, k2)
  h~put(3, k1)
  return
refused:
  say '  refused:' condition('o')~code
  return

::class Marker mixinclass Object

::class Same
::method '=='
  return .true
::method hashCode
  return 'x'
