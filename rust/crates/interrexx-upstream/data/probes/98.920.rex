/* probe 98.920 */
signal on syntax name raised
address system 'true' with output replace stream '/nonexistent_dir_zz/f.txt'
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
