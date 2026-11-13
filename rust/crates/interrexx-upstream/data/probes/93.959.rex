/* probe 93.959 */
signal on syntax name raised
a = .array~new
a[200000000000000000] = 1
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
