/* probe 46.0 */
signal on syntax name raised
raise syntax 46
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
