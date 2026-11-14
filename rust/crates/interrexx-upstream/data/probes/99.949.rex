/* probe 99.949 */
signal on syntax name raised
.environment~foo = 1
p = .Package~new('x', .array~of('::class c subclass foo'))
x = p~classes
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
