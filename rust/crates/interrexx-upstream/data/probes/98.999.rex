/* probe 98.999 */
signal on syntax name raised
address system 'cat' with input stream 'nofile_zz.txt'
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
