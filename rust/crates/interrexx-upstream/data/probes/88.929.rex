/* probe 88.929 */
signal on syntax name raised
x = 1
call r >x
exit
r:
 use arg >s.
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
