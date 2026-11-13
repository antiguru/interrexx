/* probe 35.931 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('parse arg +()'))
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
