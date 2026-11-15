/* Which tails a stem answers, with tails dropped and set again in between:
   a dropped tail is not listed, it reads as unset rather than as the stem's
   default, and one set again is listed. The order of the answers is not
   compared (Deviation 1), so each is sorted; the order itself is in
   tests/stem_order_cases. */
t. = 'dflt'
do x over 'Q MANGO LONGKEYNAME ZEBRA APPLE'~makeArray(' ')
  t.x = x
end
drop t.ZEBRA
say 'dropped:' t.ZEBRA t.~items
say 'order:  ' t.~allIndexes~sort~makeString('L', ',')
t.ZEBRA = 'again'
t.KIWI = 'k'
say 'again:  ' t.~allIndexes~sort~makeString('L', ',')
say 'items:  ' t.~allItems~sort~makeString('L', ',')

/* Supplier and DO OVER, with a drop between inserts. */
u. = 'dflt'
u.A = 1; u.B = 2; u.C = 3
drop u.B
u.D = 4
s = u.~supplier
pairs = .array~new
do while s~available
  pairs~append(s~index'='s~item)
  s~next
end
say 'supplier:' pairs~sort~makeString('L', ' ')
seen = .array~new
do i over u.
  seen~append(i)
end
say 'do over: ' seen~sort~makeString('L', ' ')
u.B = 'back'
say 'back:    ' u.~allIndexes~sort~makeString('L', ',')

/* The same tails written with t.k = t.k + 1, whose write reuses what its
   read found: a tail seen first through the default, a dropped one, and a
   held one, each written in place. */
c. = 0
do k over 'X Y X Z Y X'~makeArray(' ')
  c.k = c.k + 1
end
say 'counts:  ' c.X c.Y c.Z c.~allIndexes~sort~makeString('L', ',')
drop c.Y
k = 'Y'
c.k = c.k || '!'
say 'revived: ' c.Y c.~allIndexes~sort~makeString('L', ',')
c.~empty
c.k = c.k + 1
say 'emptied: ' c.Y c.~allIndexes~sort~makeString('L', ',')

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
say 'many:    ' w.~allIndexes~sort~makeString('L', ',')
say 'values:  ' w.~allItems~sort~makeString('L', ',')
