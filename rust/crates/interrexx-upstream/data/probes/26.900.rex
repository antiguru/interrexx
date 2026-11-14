/* probe 26.900 */
signal on syntax name raised
raise syntax 26.900 additional ('x')
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
