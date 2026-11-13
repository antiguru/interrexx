/* The order a stem answers its tails in, with tails dropped and set again
   in between: a dropped tail keeps its place, so setting it again does not
   move it, and it reads as unset rather than as the stem's default. */
t. = 'dflt'
do x over 'Q MANGO LONGKEYNAME ZEBRA APPLE'~makeArray(' ')
  t.x = x
end
drop t.ZEBRA
say 'dropped:' t.ZEBRA t.~items
say 'order:  ' t.~allIndexes~makeString('L', ',')
t.ZEBRA = 'again'
t.KIWI = 'k'
say 'again:  ' t.~allIndexes~makeString('L', ',')
say 'items:  ' t.~allItems~makeString('L', ',')

/* Supplier and DO OVER, with a drop between inserts. */
u. = 'dflt'
u.A = 1; u.B = 2; u.C = 3
drop u.B
u.D = 4
s = u.~supplier
line = ''
do while s~available
  line = line s~index'='s~item
  s~next
end
say 'supplier:' line
line = ''
do i over u.
  line = line i
end
say 'do over: ' line
u.B = 'back'
say 'back:    ' u.~allIndexes~makeString('L', ',')

/* The same tails written with t.k = t.k + 1, whose write reuses what its
   read found: a tail seen first through the default, a dropped one, and a
   held one, each written in place. */
c. = 0
do k over 'X Y X Z Y X'~makeArray(' ')
  c.k = c.k + 1
end
say 'counts:  ' c.X c.Y c.Z c.~allIndexes~makeString('L', ',')
drop c.Y
k = 'Y'
c.k = c.k || '!'
say 'revived: ' c.Y c.~allIndexes~makeString('L', ',')
c.~empty
c.k = c.k + 1
say 'emptied: ' c.Y c.~allIndexes~makeString('L', ',')

/* Many tails dropped and set again, past the few a stem keeps inline. */
w. = ''
do i = 1 to 12
  w.i = i
end
do i = 2 to 12 by 3
  drop w.i
end
do i = 12 to 1 by -1
  w.i = w.i || '.'
end
say 'many:    ' w.~allIndexes~makeString('L', ',')
say 'values:  ' w.~allItems~makeString('L', ',')
