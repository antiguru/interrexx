/* A MAKESTRING answer with a string value is the source line, an array's
 * items joined; an Object or a .nil answer has none and is 93.961. */
m = .method~new('m', .bs~new)
say m~source~items '['m~source[1]']'
signal on syntax name s1
m = .method~new('m', .bo~new)
say 'no'
s1: say condition('o')~code
signal on syntax name s2
m = .method~new('m', .bn~new)
say 'no'
s2: say condition('o')~code
exit
::class bs
::method makestring
  return .array~of('return 1')
::class bo
::method makestring
  return .object~new
::class bn
::method makestring
  return .nil
