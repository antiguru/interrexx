/* probe 34.906 */
signal on syntax name raised
do with index i over .z~new; end
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
::class z
::method supplier
  return .w~new
::class w subclass supplier
::method init
  self~init:super(.array~of('a'), .array~of(1))
::method available
  return 'x'
