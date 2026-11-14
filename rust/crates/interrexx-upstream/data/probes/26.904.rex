/* probe 26.904 */
signal on syntax name raised
in.0 = 'x'
address system 'true' with input stem in.
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
