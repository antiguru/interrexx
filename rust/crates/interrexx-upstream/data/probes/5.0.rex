quiet_ = .output~destination(.stream~new('/dev/null'))
/* Growing an Array to a length the allocator refuses raises 5.0, the
   condition, on every path that extends one: insert past the end, put and
   []= past the end, and a new Array of that size. */

call try ".Array~new(999999999999)"
a = .Array~of('p', 'q', 'r')
say 'still' a~size a~items a~toString('L', ' ')
exit

try:
  parse arg expression
  signal on syntax name failed
  interpret expression
  say '  ' expression 'raised nothing'
  return
failed:
  do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
