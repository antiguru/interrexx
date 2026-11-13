quiet_ = .output~destination(.stream~new('/dev/null'))
a = .array~of('a', 'b', , )
say a~append('c') a~size
a~remove(3)
say a~append('d') a~allItems~makeString('L', ',')
b = .array~new(5)
say b~append('x') b~size b~dimension
c = .array~new
say c~append(.nil) c~dimension c~items
call try '.array~new(2, 2)~append(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
