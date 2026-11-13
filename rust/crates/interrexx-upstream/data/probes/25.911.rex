/* probe 25.911 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('raise error 1 return 1 exit 2'))
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
