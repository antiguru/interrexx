/* probe 40.12 */
signal on syntax name raised
x = substr('abc', 1.5)
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
