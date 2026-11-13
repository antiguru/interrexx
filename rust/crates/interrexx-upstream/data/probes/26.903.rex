quiet_ = .output~destination(.stream~new('/dev/null'))
a = .array~of('b', 'A', 'a', '10', '9')
say (a~sort == a) a~allItems~makeString('L', ',')
a = .array~of(3, 1, 2)
a~sortWith(.cmp~new)
say a~allItems~makeString('L', ',')
a = .array~of('x', 'y', 'z')~~stableSortWith(.firstletter~new)
say a~allItems~makeString('L', ',')
b = .array~new(5)
b[1] = 'q'; b[2] = 'p'
b~stableSort
say b~size b~items b[1] b[2]
c = .array~new(2, 2)
c[1, 1] = 'd'; c[2, 1] = 'c'; c[1, 2] = 'b'; c[2, 2] = 'a'
c~sort
say c[1, 1] c[2, 1] c[1, 2] c[2, 2]
call try '.array~of(2, 1)~sortWith(.badcmp~new)'
say .array~of(5)~sortWith(.nil)~items
say .array~of(, 5)~sortWith(.nil)~items
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class cmp
::method compare
  use arg x, y
  return y - x
::class firstletter
::method compare
  return 0
::class badcmp
::method compare
  return 'x'
::class badlike
::method compareTo
  return 'x'
