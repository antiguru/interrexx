/* probe 93.964 */
signal on syntax name raised
raise syntax 93.964 array ('m', 'd')
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
